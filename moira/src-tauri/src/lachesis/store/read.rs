use duckdb::params;
use serde_json::Value;

use super::LachesisStore;
use crate::lachesis::model::{EventRecord, RunSummary, TickDetail, TickSummary};

impl LachesisStore {
    pub async fn list_runs(&self) -> Result<Vec<RunSummary>, String> {
        let conn = self.conn.lock().await;
        let mut stmt = conn
            .prepare(
                r#"
                SELECT run_id, first_seen_at, last_seen_at, event_count,
                       warning_count, error_count, latest_tick
                FROM runs
                ORDER BY last_seen_at DESC
                LIMIT 50
                "#,
            )
            .map_err(|err| format!("failed to prepare runs query: {err}"))?;
        let rows = stmt
            .query_map([], |row| {
                Ok(RunSummary {
                    run_id: row.get(0)?,
                    first_seen_at: row.get(1)?,
                    last_seen_at: row.get(2)?,
                    event_count: row.get::<_, i64>(3)?.max(0) as u64,
                    warning_count: row.get::<_, i64>(4)?.max(0) as u64,
                    error_count: row.get::<_, i64>(5)?.max(0) as u64,
                    latest_tick: row
                        .get::<_, Option<i64>>(6)?
                        .map(|value| value.max(0) as u64),
                })
            })
            .map_err(|err| format!("failed to query runs: {err}"))?;
        collect_rows(rows)
    }

    pub async fn list_ticks(&self, run_id: &str) -> Result<Vec<TickSummary>, String> {
        let conn = self.conn.lock().await;
        let mut stmt = conn
            .prepare(
                r#"
                SELECT run_id, tick, first_seen_at, last_seen_at, event_count,
                       warning_count, error_count
                FROM ticks
                WHERE run_id = ?
                ORDER BY tick DESC
                LIMIT 400
                "#,
            )
            .map_err(|err| format!("failed to prepare ticks query: {err}"))?;
        let rows = stmt
            .query_map(params![run_id], |row| {
                Ok(TickSummary {
                    run_id: row.get(0)?,
                    tick: row.get::<_, i64>(1)?.max(0) as u64,
                    first_seen_at: row.get(2)?,
                    last_seen_at: row.get(3)?,
                    event_count: row.get::<_, i64>(4)?.max(0) as u64,
                    warning_count: row.get::<_, i64>(5)?.max(0) as u64,
                    error_count: row.get::<_, i64>(6)?.max(0) as u64,
                })
            })
            .map_err(|err| format!("failed to query ticks: {err}"))?;
        collect_rows(rows)
    }

    pub async fn tick_detail(&self, run_id: &str, tick: u64) -> Result<TickDetail, String> {
        let summary = {
            let conn = self.conn.lock().await;
            let mut stmt = conn
                .prepare(
                    r#"
                    SELECT run_id, tick, first_seen_at, last_seen_at, event_count,
                           warning_count, error_count
                    FROM ticks
                    WHERE run_id = ? AND tick = ?
                    LIMIT 1
                    "#,
                )
                .map_err(|err| format!("failed to prepare tick summary query: {err}"))?;
            stmt.query_row(params![run_id, tick as i64], |row| {
                Ok(TickSummary {
                    run_id: row.get(0)?,
                    tick: row.get::<_, i64>(1)?.max(0) as u64,
                    first_seen_at: row.get(2)?,
                    last_seen_at: row.get(3)?,
                    event_count: row.get::<_, i64>(4)?.max(0) as u64,
                    warning_count: row.get::<_, i64>(5)?.max(0) as u64,
                    error_count: row.get::<_, i64>(6)?.max(0) as u64,
                })
            })
            .map_err(|err| format!("failed to query tick summary: {err}"))?
        };

        let conn = self.conn.lock().await;
        let mut stmt = conn
            .prepare(
                r#"
                SELECT raw_event_id, received_at, observed_at, severity_text, run_id, tick,
                       target, family, subsystem, message_text, attributes_json, body_json,
                       resource_json, scope_json
                FROM raw_events
                WHERE run_id = ? AND tick = ?
                ORDER BY observed_at ASC, raw_event_id ASC
                "#,
            )
            .map_err(|err| format!("failed to prepare tick detail query: {err}"))?;
        let rows = stmt
            .query_map(params![run_id, tick as i64], |row| {
                Ok(EventRecord {
                    raw_event_id: row.get(0)?,
                    received_at: row.get(1)?,
                    observed_at: row.get(2)?,
                    severity_text: row
                        .get::<_, Option<String>>(3)?
                        .unwrap_or_else(|| "INFO".to_string()),
                    run_id: row.get(4)?,
                    tick: row
                        .get::<_, Option<i64>>(5)?
                        .map(|value| value.max(0) as u64),
                    target: row.get(6)?,
                    family: row.get(7)?,
                    subsystem: row.get(8)?,
                    message_text: row.get(9)?,
                    attributes: parse_json_column(row.get::<_, String>(10)?),
                    body: parse_json_column(row.get::<_, String>(11)?),
                    resource: parse_json_column(row.get::<_, String>(12)?),
                    scope: parse_json_column(row.get::<_, String>(13)?),
                })
            })
            .map_err(|err| format!("failed to query raw tick events: {err}"))?;
        let raw = collect_rows(rows)?;

        let mut cortex = Vec::new();
        let mut stem = Vec::new();
        let mut spine = Vec::new();

        for event in &raw {
            match infer_subsystem(event) {
                Some("cortex") => cortex.push(event.clone()),
                Some("stem") => stem.push(event.clone()),
                Some("spine") => spine.push(event.clone()),
                _ => {}
            }
        }

        Ok(TickDetail {
            summary,
            cortex,
            stem,
            spine,
            raw,
        })
    }
}

fn infer_subsystem(event: &EventRecord) -> Option<&str> {
    event
        .subsystem
        .as_deref()
        .or_else(|| {
            event
                .family
                .as_deref()
                .and_then(|family| family.split('.').next())
        })
        .or_else(|| {
            event
                .target
                .as_deref()
                .and_then(|target| target.split('.').next())
        })
}

fn parse_json_column(source: String) -> Value {
    serde_json::from_str(&source).unwrap_or(Value::Null)
}

fn collect_rows<T>(
    rows: duckdb::MappedRows<'_, impl FnMut(&duckdb::Row<'_>) -> duckdb::Result<T>>,
) -> Result<Vec<T>, String> {
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|err| format!("failed to collect Lachesis rows: {err}"))
}
