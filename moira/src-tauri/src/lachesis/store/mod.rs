mod read;
mod write;

use std::{path::Path, sync::Arc};

use duckdb::Connection;
use tokio::sync::Mutex;

pub use write::NormalizedEvent;

pub struct LachesisStore {
    conn: Mutex<Connection>,
    db_path: String,
}

impl LachesisStore {
    pub async fn open(path: &Path) -> Result<Arc<Self>, String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|err| format!("failed to create Lachesis store directory: {err}"))?;
        }
        let conn = Connection::open(path)
            .map_err(|err| format!("failed to open Lachesis DuckDB store: {err}"))?;
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
              record_kind TEXT,
              scope_name TEXT,
              event_name TEXT,
              trace_id_hex TEXT,
              span_id_hex TEXT,
              trace_flags INTEGER,
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

            CREATE INDEX IF NOT EXISTS idx_raw_events_trace
            ON raw_events(trace_id_hex, observed_at);

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
              trace_id_hex TEXT,
              first_seen_at TEXT NOT NULL,
              last_seen_at TEXT NOT NULL,
              event_count BIGINT NOT NULL,
              warning_count BIGINT NOT NULL,
              error_count BIGINT NOT NULL,
              PRIMARY KEY(run_id, tick)
            );

            ALTER TABLE raw_events ADD COLUMN IF NOT EXISTS scope_name TEXT;
            ALTER TABLE raw_events ADD COLUMN IF NOT EXISTS record_kind TEXT;
            ALTER TABLE raw_events ADD COLUMN IF NOT EXISTS event_name TEXT;
            ALTER TABLE raw_events ADD COLUMN IF NOT EXISTS trace_id_hex TEXT;
            ALTER TABLE raw_events ADD COLUMN IF NOT EXISTS span_id_hex TEXT;
            ALTER TABLE raw_events ADD COLUMN IF NOT EXISTS trace_flags INTEGER;
            ALTER TABLE ticks ADD COLUMN IF NOT EXISTS trace_id_hex TEXT;

            CREATE INDEX IF NOT EXISTS idx_ticks_trace
            ON ticks(trace_id_hex);
            "#,
        )
        .map_err(|err| format!("failed to initialize Lachesis schema: {err}"))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use uuid::Uuid;

    use super::{LachesisStore, NormalizedEvent};

    #[tokio::test]
    async fn native_tick_anchor_projects_trace_backed_detail() {
        let path = std::env::temp_dir().join(format!("lachesis-native-{}.duckdb", Uuid::now_v7()));
        let store = LachesisStore::open(&path).await.expect("open store");

        store
            .ingest_events(vec![
                native_event(
                    "evt-tick",
                    "2026-05-05T00:00:00Z",
                    "beluna.core.stem",
                    "tick.granted",
                    "trace-1",
                    "span-tick",
                    Some("run-1"),
                    Some(1),
                    json!({
                        "summary": "Stem granted tick 1.",
                        "run_id": "run-1",
                        "tick": 1,
                        "tick_seq": 1,
                    }),
                ),
                native_event(
                    "evt-primary",
                    "2026-05-05T00:00:01Z",
                    "beluna.core.cortex",
                    "primary.started",
                    "trace-1",
                    "span-primary",
                    None,
                    None,
                    json!({
                        "summary": "Cortex primary phase started.",
                        "input": {},
                    }),
                ),
            ])
            .await
            .expect("ingest native events");

        let runs = store.list_runs().await.expect("list runs");
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].run_id, "run-1");
        assert_eq!(runs[0].event_count, 2);
        assert_eq!(runs[0].latest_tick, Some(1));

        let ticks = store.list_ticks("run-1").await.expect("list ticks");
        assert_eq!(ticks.len(), 1);
        assert_eq!(ticks[0].trace_id.as_deref(), Some("trace-1"));
        assert_eq!(ticks[0].event_count, 2);
        assert!(ticks[0].cortex_handled);

        let detail = store.tick_detail("run-1", 1).await.expect("tick detail");
        assert_eq!(detail.raw.len(), 2);
        assert_eq!(detail.raw[1].event_name.as_deref(), Some("primary.started"));

        drop(store);
        let _ = std::fs::remove_file(path);
    }

    fn native_event(
        raw_event_id: &str,
        observed_at: &str,
        scope_name: &str,
        event_name: &str,
        trace_id: &str,
        span_id: &str,
        run_id: Option<&str>,
        tick: Option<i64>,
        body: serde_json::Value,
    ) -> NormalizedEvent {
        NormalizedEvent {
            raw_event_id: raw_event_id.to_string(),
            received_at: observed_at.to_string(),
            observed_at: observed_at.to_string(),
            severity_text: "INFO".to_string(),
            severity_number: 9,
            scope_name: Some(scope_name.to_string()),
            record_kind: "native_owner".to_string(),
            event_name: Some(event_name.to_string()),
            trace_id_hex: Some(trace_id.to_string()),
            span_id_hex: Some(span_id.to_string()),
            trace_flags: Some(1),
            target: None,
            family: None,
            subsystem: scope_name
                .strip_prefix("beluna.core.")
                .and_then(|value| value.split('.').next())
                .map(str::to_string),
            run_id: run_id.map(str::to_string),
            tick,
            message_text: None,
            body_json: body.to_string(),
            attributes_json: "{}".to_string(),
            resource_json: r#"{"service.name":"beluna.core"}"#.to_string(),
            scope_json: json!({ "name": scope_name, "version": "", "attributes": {} }).to_string(),
        }
    }
}
