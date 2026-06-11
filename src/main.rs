use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Extension, Json, Router,
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use dotenvy::dotenv;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};
use tower_http::services::{ServeDir, ServeFile};

mod db;

const SESSION_COOKIE: &str = "session";
const SESSION_VALUE: &str = "authenticated";

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    timestamp: u64,
    service: String,
    version: String,
}

#[derive(Serialize)]
struct MeResponse {
    authenticated: bool,
}

#[derive(Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("Starting server...");

    let db_path = env::var("SQLITE_PATH").unwrap_or_else(|_| "data/product-twin.db".into());
    let sqlite = db::connect(&db_path).expect("failed to open SQLite database");
    info!("SQLite database opened at {db_path}");

    let api_routes = Router::new()
        .route("/health", get(health_check))
        .route("/me", get(me))
        .route("/login", post(login_submit))
        .route("/logout", post(logout))
        .fallback(api_not_found);

    let serve_dir = ServeDir::new("frontend/dist")
        .fallback(ServeFile::new("frontend/dist/index.html"));

    let app = Router::new()
        .nest("/api", api_routes)
        .fallback_service(serve_dir)
        .layer(Extension(sqlite));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    info!("Server running on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}

async fn api_not_found() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            error: "not found".to_string(),
        }),
    )
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

async fn me(jar: CookieJar) -> Json<MeResponse> {
    debug!("me handler called");
    let authenticated = matches!(jar.get(SESSION_COOKIE), Some(c) if c.value() == SESSION_VALUE);
    Json(MeResponse { authenticated })
}

async fn login_submit(jar: CookieJar, Json(form): Json<LoginRequest>) -> impl IntoResponse {
    let expected_user = env::var("APP_USERNAME").unwrap_or_default();
    let expected_pass = env::var("APP_PASSWORD").unwrap_or_default();

    if expected_user.is_empty() || expected_pass.is_empty() {
        warn!("APP_USERNAME or APP_PASSWORD is not set; rejecting login");
        return (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "invalid credentials".to_string(),
            }),
        )
            .into_response();
    }

    if form.username == expected_user && form.password == expected_pass {
        let cookie = Cookie::build((SESSION_COOKIE, SESSION_VALUE))
            .path("/")
            .http_only(true)
            .build();
        (
            jar.add(cookie),
            Json(MeResponse { authenticated: true }),
        )
            .into_response()
    } else {
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "invalid credentials".to_string(),
            }),
        )
            .into_response()
    }
}

async fn logout(jar: CookieJar) -> impl IntoResponse {
    let removal = Cookie::build(SESSION_COOKIE).path("/").build();
    (
        jar.remove(removal),
        Json(MeResponse {
            authenticated: false,
        }),
    )
}
