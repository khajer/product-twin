use axum::{
    extract::Query,
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
    Form, Json, Router,
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use dotenvy::dotenv;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

const SESSION_COOKIE: &str = "session";
const SESSION_VALUE: &str = "authenticated";

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

    let app = Router::new()
        .route("/", get(hello_world))
        .route("/health", get(health_check))
        .route("/login", get(login_page).post(login_submit))
        .route("/logout", post(logout))
        .route("/landing", get(landing_page));

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
    Html(format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>Login</title>
<style>
  body {{ font-family: system-ui, sans-serif; display: flex; align-items: center; justify-content: center; height: 100vh; margin: 0; background: #f5f5f5; }}
  form {{ background: #fff; padding: 2rem; border-radius: 8px; box-shadow: 0 2px 8px rgba(0,0,0,.1); width: 320px; }}
  h1 {{ margin: 0 0 1rem; font-size: 1.25rem; }}
  label {{ display: block; margin-top: .75rem; font-size: .875rem; }}
  input {{ width: 100%; padding: .5rem; margin-top: .25rem; border: 1px solid #ccc; border-radius: 4px; box-sizing: border-box; }}
  button {{ width: 100%; margin-top: 1rem; padding: .6rem; border: 0; border-radius: 4px; background: #2563eb; color: #fff; font-weight: 600; cursor: pointer; }}
  button:hover {{ background: #1d4ed8; }}
  .error {{ color: #b91c1c; font-size: .875rem; margin: 0 0 .5rem; }}
</style>
</head>
<body>
<form method="post" action="/login">
  <h1>Sign in</h1>
  {error_banner}
  <label>Username
    <input name="username" autocomplete="username" required autofocus>
  </label>
  <label>Password
    <input name="password" type="password" autocomplete="current-password" required>
  </label>
  <button type="submit">Login</button>
</form>
</body>
</html>"#
    ))
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
        Some(c) if c.value() == SESSION_VALUE => Ok(Html(
            r#"<!DOCTYPE html>
<html lang="en">
<head><meta charset="utf-8"><title>Landing</title></head>
<body></body>
</html>"#,
        )),
        _ => Err(Redirect::to("/login")),
    }
}
