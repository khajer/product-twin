use axum::{
    extract::Query,
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
    Extension, Form, Json, Router,
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use dotenvy::dotenv;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

mod db;

const SESSION_COOKIE: &str = "session";
const SESSION_VALUE: &str = "authenticated";

const LOGIN_HTML: &str = include_str!("./static/login.html");
const LANDING_HTML: &str = include_str!("./static/landing.html");

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    timestamp: u64,
    service: String,
    version: String,
}

#[derive(Deserialize)]
struct LoginForm {
    username: String,
    password: String,
}

#[derive(Deserialize)]
struct LoginQuery {
    error: Option<String>,
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

    let app = Router::new()
        .route("/", get(hello_world))
        .route("/health", get(health_check))
        .route("/login", get(login_page).post(login_submit))
        .route("/logout", post(logout))
        .route("/landing", get(landing_page))
        .layer(Extension(sqlite));

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

async fn login_page(Query(q): Query<LoginQuery>) -> Html<String> {
    let error_banner = if q.error.is_some() {
        r#"<p class="error">Invalid username or password</p>"#
    } else {
        ""
    };
    Html(LOGIN_HTML.replace("{{error_banner}}", error_banner))
}

async fn login_submit(jar: CookieJar, Form(form): Form<LoginForm>) -> impl IntoResponse {
    let expected_user = env::var("APP_USERNAME").unwrap_or_default();
    let expected_pass = env::var("APP_PASSWORD").unwrap_or_default();

    if expected_user.is_empty() || expected_pass.is_empty() {
        warn!("APP_USERNAME or APP_PASSWORD is not set; rejecting login");
        return Redirect::to("/login?error=1").into_response();
    }

    if form.username == expected_user && form.password == expected_pass {
        let cookie = Cookie::build((SESSION_COOKIE, SESSION_VALUE))
            .path("/")
            .http_only(true)
            .build();
        (jar.add(cookie), Redirect::to("/landing")).into_response()
    } else {
        Redirect::to("/login?error=1").into_response()
    }
}

async fn logout(jar: CookieJar) -> impl IntoResponse {
    (
        jar.remove(Cookie::from(SESSION_COOKIE)),
        Redirect::to("/login"),
    )
}

async fn landing_page(jar: CookieJar) -> Result<Html<&'static str>, Redirect> {
    match jar.get(SESSION_COOKIE) {
        Some(c) if c.value() == SESSION_VALUE => Ok(Html(LANDING_HTML)),
        _ => Err(Redirect::to("/login")),
    }
}
