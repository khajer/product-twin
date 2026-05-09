use axum::{routing::get, Json, Router};
use dotenvy::dotenv;
use log::{debug, info};
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    timestamp: u64,
    service: String,
    version: String,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("Starting server...");

    let app = Router::new()
        .route("/", get(hello_world))
        .route("/health", get(health_check));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    info!("Server running on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}

async fn hello_world() -> &'static str {
    debug!("hello_world handler called");
    "Hello, World!"
}

async fn health_check() -> Json<HealthResponse> {
    debug!("health_check handler called");
    Json(HealthResponse {
        status: "ok".to_string(),
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        service: "product-twin".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}
