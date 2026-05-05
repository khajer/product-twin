use axum::{routing::get, Router};
use dotenvy::dotenv;
use log::{debug, info};

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("Starting server...");

    let app = Router::new().route("/", get(hello_world));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    info!("Server running on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}

async fn hello_world() -> &'static str {
    debug!("hello_world handler called");
    "Hello, World!"
}
