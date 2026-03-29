mod read;
mod write;

use std::{path::Path, sync::Arc};

use duckdb::Connection;
use tokio::sync::Mutex;

pub use write::NormalizedEvent;

pub struct MoiraStore {
    conn: Mutex<Connection>,
    db_path: String,
}

impl MoiraStore {
    pub async fn open(path: &Path) -> Result<Arc<Self>, String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|err| format!("failed to create Moira store directory: {err}"))?;
        }
        let conn = Connection::open(path)
            .map_err(|err| format!("failed to open Moira DuckDB store: {err}"))?;
        let store = Arc::new(Self {
            conn: Mutex::new(conn),
            db_path: path.display().to_string(),
        });
        store.init_schema().await?;
        Ok(store)
    }

    pub fn db_path(&self) -> String {
        self.db_path.clone()
    }

    async fn init_schema(&self) -> Result<(), String> {
        let conn = self.conn.lock().await;
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS raw_events (
              raw_event_id TEXT PRIMARY KEY,
              received_at TEXT NOT NULL,
              observed_at TEXT NOT NULL,
              severity_text TEXT,
              severity_number INTEGER,
              target TEXT,
              family TEXT,
              subsystem TEXT,
              run_id TEXT,
              tick BIGINT,
              message_text TEXT,
              body_json TEXT NOT NULL,
              attributes_json TEXT NOT NULL,
              resource_json TEXT NOT NULL,
              scope_json TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_raw_events_run_tick
            ON raw_events(run_id, tick, observed_at);

            CREATE TABLE IF NOT EXISTS runs (
              run_id TEXT PRIMARY KEY,
              first_seen_at TEXT NOT NULL,
              last_seen_at TEXT NOT NULL,
              event_count BIGINT NOT NULL,
              warning_count BIGINT NOT NULL,
              error_count BIGINT NOT NULL,
              latest_tick BIGINT
            );

            CREATE TABLE IF NOT EXISTS ticks (
              run_id TEXT NOT NULL,
              tick BIGINT NOT NULL,
              first_seen_at TEXT NOT NULL,
              last_seen_at TEXT NOT NULL,
              event_count BIGINT NOT NULL,
              warning_count BIGINT NOT NULL,
              error_count BIGINT NOT NULL,
              PRIMARY KEY(run_id, tick)
            );
            "#,
        )
        .map_err(|err| format!("failed to initialize Moira schema: {err}"))?;
        Ok(())
    }
}
