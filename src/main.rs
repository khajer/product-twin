use axum::{
    extract::Multipart,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Extension, Json, Router,
};
use std::path::Path;
use std::sync::Arc;
use axum_extra::extract::cookie::{Cookie, CookieJar};
use dotenvy::dotenv;
use log::{debug, info, warn};
use neo4rs::Graph;
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

#[derive(Serialize)]
struct UploadedFile {
    filename: String,
    size: u64,
}

#[derive(Deserialize)]
struct ImportRequest {
    filename: String,
}

#[derive(Serialize)]
struct ImportResult {
    imported: usize,
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

    let neo4j_uri = env::var("NEO4J_URI").unwrap_or_else(|_| "bolt://localhost:7687".into());
    let neo4j_user = env::var("NEO4J_USER").unwrap_or_else(|_| "neo4j".into());
    let neo4j_pass = env::var("NEO4J_PASSWORD").unwrap_or_else(|_| "password123".into());
    let graph = Graph::new(&neo4j_uri, &neo4j_user, &neo4j_pass)
        .await
        .expect("failed to connect to Neo4j");
    let graph = Arc::new(graph);
    info!("Connected to Neo4j at {neo4j_uri}");

    let api_routes = Router::new()
        .route("/health", get(health_check))
        .route("/me", get(me))
        .route("/login", post(login_submit))
        .route("/logout", post(logout))
        .route("/upload", post(upload_file))
        .route("/uploads", get(list_uploads))
        .route("/import", post(import_file))
        .fallback(api_not_found);

    let serve_dir = ServeDir::new("frontend/dist")
        .fallback(ServeFile::new("frontend/dist/index.html"));

    let app = Router::new()
        .nest("/api", api_routes)
        .fallback_service(serve_dir)
        .layer(Extension(sqlite))
        .layer(Extension(graph));

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

fn is_authenticated(jar: &CookieJar) -> bool {
    matches!(jar.get(SESSION_COOKIE), Some(c) if c.value() == SESSION_VALUE)
}

async fn upload_file(jar: CookieJar, mut multipart: Multipart) -> impl IntoResponse {
    if !is_authenticated(&jar) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse { error: "unauthorized".into() }),
        )
            .into_response();
    }

    tokio::fs::create_dir_all("data/uploads").await.ok();

    while let Ok(Some(field)) = multipart.next_field().await {
        let raw_name = field.file_name().unwrap_or("upload").to_string();
        // ponytail: basename-only to prevent path traversal
        let filename = Path::new(&raw_name)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("upload")
            .to_string();

        let ext = Path::new(&filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        if ext != "csv" && ext != "txt" {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(ErrorResponse { error: "only .csv and .txt files are allowed".into() }),
            )
                .into_response();
        }

        let data = match field.bytes().await {
            Ok(b) => b,
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse { error: e.to_string() }),
                )
                    .into_response()
            }
        };
        let size = data.len() as u64;

        if let Err(e) = tokio::fs::write(format!("data/uploads/{filename}"), &data).await {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: e.to_string() }),
            )
                .into_response();
        }

        return Json(UploadedFile { filename, size }).into_response();
    }

    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse { error: "no file provided".into() }),
    )
        .into_response()
}

async fn list_uploads(jar: CookieJar) -> impl IntoResponse {
    if !is_authenticated(&jar) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse { error: "unauthorized".into() }),
        )
            .into_response();
    }

    let mut files: Vec<UploadedFile> = Vec::new();
    if let Ok(mut dir) = tokio::fs::read_dir("data/uploads").await {
        while let Ok(Some(entry)) = dir.next_entry().await {
            let size = entry.metadata().await.map(|m| m.len()).unwrap_or(0);
            let filename = entry.file_name().to_string_lossy().to_string();
            files.push(UploadedFile { filename, size });
        }
    }
    files.sort_by(|a, b| a.filename.cmp(&b.filename));
    Json(files).into_response()
}

async fn import_file(
    jar: CookieJar,
    Extension(graph): Extension<Arc<Graph>>,
    Json(req): Json<ImportRequest>,
) -> impl IntoResponse {
    if !is_authenticated(&jar) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse { error: "unauthorized".into() }),
        )
            .into_response();
    }

    // basename-only to prevent path traversal
    let filename = Path::new(&req.filename)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();
    if filename.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: "invalid filename".into() })).into_response();
    }

    let path = format!("data/uploads/{filename}");
    let content = match tokio::fs::read_to_string(&path).await {
        Ok(c) => c,
        Err(_) => return (StatusCode::NOT_FOUND, Json(ErrorResponse { error: "file not found".into() })).into_response(),
    };

    let mut rdr = csv::Reader::from_reader(content.as_bytes());
    let headers: Vec<String> = match rdr.headers() {
        Ok(h) => h.iter()
            .map(|s| s.chars().filter(|c| c.is_alphanumeric() || *c == '_').collect())
            .collect(),
        Err(e) => return (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e.to_string() })).into_response(),
    };

    if headers.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: "no headers found".into() })).into_response();
    }

    let props = headers.iter().map(|h| format!("{h}: ${h}")).collect::<Vec<_>>().join(", ");
    let query_str = format!("CREATE (n:Entity {{{props}}})", props = props);

    let mut imported = 0usize;
    for result in rdr.records() {
        let record = match result {
            Ok(r) => r,
            Err(_) => continue,
        };
        let mut q = neo4rs::query(&query_str);
        for (h, v) in headers.iter().zip(record.iter()) {
            q = q.param(h.as_str(), v);
        }
        if graph.run(q).await.is_ok() {
            imported += 1;
        }
    }

    Json(ImportResult { imported }).into_response()
}
