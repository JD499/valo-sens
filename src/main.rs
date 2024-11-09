use axum::{
    extract::State,
    routing::get,
    Json, Router,
};
use rusqlite::{params, Connection, OptionalExtension, Result as SqliteResult};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use axum::routing::get_service;
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

#[derive(Debug, Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: T,
    message: Option<String>,
}

impl<T> ApiResponse<T> {
    fn success(data: T) -> Self {
        Self {
            success: true,
            data,
            message: None,
        }
    }

    fn error(message: &str, data: T) -> Self {
        Self {
            success: false,
            data,
            message: Some(message.to_string()),
        }
    }
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

    fn should_update(&self) -> SqliteResult<bool> {
        let conn = Connection::open(&self.db_path)?;
        let result: Option<i64> = conn.query_row(
            "SELECT timestamp FROM last_update WHERE id = 1",
            [],
            |row| row.get(0),
        ).optional()?;

        match result {
            None => Ok(true),
            Some(timestamp) => {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64;
                Ok((now - timestamp) >= 24 * 60 * 60)
            }
        }
    }

    async fn update_players(&self) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let response = client
            .get("https://prosettings.net/lists/valorant/")
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
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
        let mut stmt = conn.prepare(
            "SELECT name, team, sens, dpi, edpi FROM players ORDER BY name",
        )?;

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
}

async fn get_players(
    State(state): State<Arc<RwLock<AppState>>>,
) -> Json<ApiResponse<Vec<Player>>> {
    let state = state.read().await;


    match state.should_update() {
        Ok(true) => {
            if let Err(e) = state.update_players().await {
                eprintln!("Error updating players: {}", e);
            }
        }
        Err(e) => eprintln!("Error checking update status: {}", e),
        _ => {}
    }


    match state.get_all_players() {
        Ok(players) => Json(ApiResponse::success(players)),
        Err(e) => Json(ApiResponse::error(&e.to_string(), Vec::new())),
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

    tracing::info!("Database initialized successfully");

    let app = Router::new()
        .route("/api/players", get(get_players))
        .nest_service("/assets", get_service(ServeDir::new("assets")))
        .route("/", get(serve_static))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8001));
    tracing::info!("Server starting on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn serve_static() -> axum::response::Response {
    axum::response::Response::builder()
        .header("content-type", "text/html")
        .body(include_str!("../assets/index.html").into())
        .unwrap()
}