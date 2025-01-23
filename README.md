# Valo-Sens - Valorant Sensitivity Analysis Tool

A high-performance web application for analyzing and converting Valorant professional player sensitivity configurations. Built with Rust's Axum framework and SQLite.

## Key Features

- Professional player database with daily automated updates
- Sensitivity search functionality with name filtering
- eDPI (effective DPI) similarity matching algorithm
- Cross-DPI sensitivity conversion calculator
- Accessible web interface with semantic HTML
- Efficient database design using Rusqlite

## Technical Implementation

- **Backend**: Rust 1.82.0 with Axum web framework
- **Data Storage**: SQLite with Rusqlite (SQLite bindings)
- **Web Scraping**: Reqwest HTTP client + Scraper HTML parser
- **Templating**: Askama for server-side HTML rendering
- **Concurrency**: Tokio runtime with async/await
- **Logging**: Tracing framework with environment filters

## Installation

### Docker Deployment

```bash
docker build -t valo-sens .
docker run -p 8001:8001 valo-sens
```
### Local Development

1. Install Rust toolchain via [rustup](https://rustup.rs/)
2. Build release binary:
```bash
cargo build --release
```
3. Run service:
```bash
./target/release/valo-sens
```

## Application Structure
```plaintext
/
├── src/            # Rust source code
├── assets/         # Static assets (CSS, icons)
├── templates/      # Askama HTML templates
└── Dockerfile      # Production deployment configuration

```

