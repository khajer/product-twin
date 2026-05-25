use rusqlite::Connection;
use std::path::Path;
use std::sync::{Arc, Mutex};

pub type SqliteDb = Arc<Mutex<Connection>>;

pub fn connect(path: &str) -> rusqlite::Result<SqliteDb> {
    if let Some(parent) = Path::new(path).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).expect("failed to create database directory");
        }
    }

    let conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    Ok(Arc::new(Mutex::new(conn)))
}
