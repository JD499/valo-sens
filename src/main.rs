use askama::Template;
use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse},
    routing::{get, get_service, post},
    Form, Router,
};
use rusqlite::{params, Connection, Result as SqliteResult};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::sync::RwLock;
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Player {
    name: String,
    team: String,
    sens: f64,
    dpi: i32,
    edpi: f64,
}

#[derive(Template)]
#[template(path = "search.html")]
struct SearchTemplate {
    players: Vec<Player>,
    query: Option<String>,
    page: String,
}

#[derive(Template)]
#[template(path = "similar.html")]
struct SimilarTemplate {
    players: Vec<Player>,
    page: String,
}

#[derive(Deserialize, Template)]
#[template(path = "convert.html")]
struct ConvertTemplate {
    players: Vec<Player>,
    query: Option<String>,
    your_dpi: Option<i32>,
    your_sens: Option<f64>,
    target_dpi: Option<i32>,
    converted_sens: Option<f64>,
    page: String,
}

#[derive(Deserialize)]
struct ConvertQuery {
    q: Option<String>,
    your_dpi: Option<String>,
    your_sens: Option<String>,
    target_dpi: Option<String>,
    selected_pro: Option<String>,
}

#[derive(Deserialize)]
struct SearchQuery {
    q: Option<String>,
}

#[derive(Deserialize)]
struct SimilarForm {
    dpi: i32,
    sens: f64,
}

#[derive(Debug, Clone)]
struct AppState {
    db_path: String,
}

impl AppState {
    fn new(db_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let conn = Connection::open(db_path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS players (
                name TEXT PRIMARY KEY,
                team TEXT NOT NULL,
                sens REAL NOT NULL,
                dpi INTEGER NOT NULL,
                edpi REAL NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS last_update (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                timestamp INTEGER NOT NULL
            )",
            [],
        )?;

        Ok(AppState {
            db_path: db_path.to_string(),
        })
    }

    async fn update_players(&self) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let response = client
            .get("https://prosettings.net/lists/valorant/")
            .header(
                "User-Agent",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
            )
            .send()
            .await?
            .text()
            .await?;

        let document = scraper::Html::parse_document(&response);
        let table_selector = scraper::Selector::parse("table tbody tr").unwrap();
        let td_selector = scraper::Selector::parse("td").unwrap();

        let mut conn = Connection::open(&self.db_path)?;
        let tx = conn.transaction()?;

        tx.execute("DELETE FROM players", [])?;

        for row in document.select(&table_selector) {
            let cells: Vec<String> = row
                .select(&td_selector)
                .map(|cell| cell.text().collect::<String>().trim().to_string())
                .collect();

            if cells.len() >= 8 {
                let sens: f64 = cells[6].parse().unwrap_or(0.0);
                let dpi: i32 = cells[5].parse().unwrap_or(0);
                let edpi = sens * dpi as f64;

                if !cells[2].is_empty() && edpi > 0.0 {
                    tx.execute(
                        "INSERT OR REPLACE INTO players (name, team, sens, dpi, edpi) VALUES (?1, ?2, ?3, ?4, ?5)",
                        params![&cells[2], &cells[1], sens, dpi, edpi],
                    )?;
                }
            }
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        tx.execute(
            "INSERT OR REPLACE INTO last_update (id, timestamp) VALUES (1, ?1)",
            params![now],
        )?;

        tx.commit()?;

        Ok(())
    }

    fn get_all_players(&self) -> SqliteResult<Vec<Player>> {
        let conn = Connection::open(&self.db_path)?;
        let mut stmt =
            conn.prepare("SELECT name, team, sens, dpi, edpi FROM players ORDER BY name")?;

        let players = stmt.query_map([], |row| {
            Ok(Player {
                name: row.get(0)?,
                team: row.get(1)?,
                sens: row.get(2)?,
                dpi: row.get(3)?,
                edpi: row.get(4)?,
            })
        })?;

        players.collect()
    }

    fn find_similar_players(&self, target_edpi: f64) -> SqliteResult<Vec<Player>> {
        let conn = Connection::open(&self.db_path)?;
        let mut stmt = conn.prepare(
            "SELECT name, team, sens, dpi, edpi,
                    ABS(edpi - ?1) as diff
             FROM players 
             ORDER BY diff ASC 
             LIMIT 10",
        )?;

        let players = stmt.query_map([target_edpi], |row| {
            Ok(Player {
                name: row.get(0)?,
                team: row.get(1)?,
                sens: row.get(2)?,
                dpi: row.get(3)?,
                edpi: row.get(4)?,
            })
        })?;

        players.collect()
    }

    fn search_players(&self, query: &str) -> SqliteResult<Vec<Player>> {
        let conn = Connection::open(&self.db_path)?;

        let pattern = format!("{}%", query);

        let mut stmt = conn.prepare(
            "SELECT name, team, sens, dpi, edpi 
             FROM players 
             WHERE LOWER(name) LIKE LOWER(?1)
             ORDER BY name",
        )?;

        let players = stmt.query_map([pattern], |row| {
            Ok(Player {
                name: row.get(0)?,
                team: row.get(1)?,
                sens: row.get(2)?,
                dpi: row.get(3)?,
                edpi: row.get(4)?,
            })
        })?;

        players.collect()
    }
}

async fn search_page(
    State(state): State<Arc<RwLock<AppState>>>,
    Query(query): Query<SearchQuery>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let state = state.read().await;

    let players = if let Some(ref q) = query.q {
        state.search_players(q).unwrap_or_default()
    } else {
        state.get_all_players().unwrap_or_default()
    };

    let is_ajax = headers
        .get("X-Requested-With")
        .and_then(|value| value.to_str().ok())
        .map_or(false, |value| value == "XMLHttpRequest");

    if is_ajax {
        let template = SearchTemplate {
            players,
            query: query.q,
            page: "search".to_string(),
        };
        Html(template.render().unwrap_or_default())
    } else {
        let template = SearchTemplate {
            players,
            query: query.q,
            page: "search".to_string(),
        };
        Html(template.render().unwrap_or_default())
    }
}

async fn similar_page() -> impl IntoResponse {
    let template = SimilarTemplate {
        players: Vec::new(),
        page: "similar".to_string(),
    };
    Html(template.render().unwrap_or_default())
}

async fn similar_submit(
    State(state): State<Arc<RwLock<AppState>>>,
    Form(form): Form<SimilarForm>,
) -> impl IntoResponse {
    let state = state.read().await;
    let target_edpi = form.sens * form.dpi as f64;
    let players = state.find_similar_players(target_edpi).unwrap_or_default();

    let template = SimilarTemplate {
        players,
        page: "similar".to_string(),
    };
    Html(template.render().unwrap_or_default())
}

async fn convert_page(
    State(state): State<Arc<RwLock<AppState>>>,
    Query(query): Query<ConvertQuery>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let state = state.read().await;

    let players = if let Some(ref q) = query.q {
        state.search_players(q).unwrap_or_default()
    } else {
        state.get_all_players().unwrap_or_default()
    };

    let your_dpi = query.your_dpi.as_ref().and_then(|s| s.parse::<i32>().ok());
    let your_sens = query.your_sens.as_ref().and_then(|s| s.parse::<f64>().ok());
    let target_dpi = query
        .target_dpi
        .as_ref()
        .and_then(|s| s.parse::<i32>().ok());

    let converted_sens = match (your_dpi, your_sens) {
        (Some(dpi), Some(sens)) if dpi > 0 && sens > 0.0 => {
            let your_edpi = dpi as f64 * sens;

            if let Some(target) = target_dpi {
                if target > 0 {
                    Some((your_edpi / target as f64 * 1000.0).round() / 1000.0)
                } else {
                    None
                }
            } else if let Some(pro_name) = &query.selected_pro {
                if let Some(pro) = players.iter().find(|p| p.name == *pro_name) {
                    Some(((pro.edpi / dpi as f64 * 1000.0).round()) / 1000.0)
                } else {
                    None
                }
            } else {
                None
            }
        }
        _ => None,
    };

    let template = ConvertTemplate {
        players,
        query: query.q,
        your_dpi: your_dpi,
        your_sens: your_sens,
        target_dpi: target_dpi,
        converted_sens,
        page: "convert".to_string(),
    };

    let is_ajax = headers
        .get("X-Requested-With")
        .and_then(|value| value.to_str().ok())
        .map_or(false, |value| value == "XMLHttpRequest");

    if is_ajax {
        Html(template.render().unwrap_or_default())
    } else {
        Html(template.render().unwrap_or_default())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting server initialization...");

    let state = Arc::new(RwLock::new(
        AppState::new("data.db").expect("Failed to initialize database"),
    ));

    {
        let state = state.read().await;
        if let Err(e) = state.update_players().await {
            eprintln!("Error during initial database population: {}", e);
        }
    }

    {
        let state = state.clone();
        let mut interval = tokio::time::interval(Duration::from_secs(60 * 60 * 24));
        tokio::spawn(async move {
            loop {
                interval.tick().await;
                let state = state.read().await;
                match state.update_players().await {
                    Ok(_) => println!("Scheduled database update completed successfully"),
                    Err(e) => eprintln!("Scheduled database update failed: {}", e),
                }
            }
        });
    }

    let app = Router::new()
        .route("/", get(search_page))
        .route("/search", get(search_page))
        .route("/similar", get(similar_page))
        .route("/similar", post(similar_submit))
        .route("/convert", get(convert_page))
        .nest_service("/assets", get_service(ServeDir::new("assets")))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8001));
    tracing::info!("Server starting on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
