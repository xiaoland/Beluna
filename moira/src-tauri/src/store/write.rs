use std::collections::BTreeSet;

use duckdb::params;

use super::MoiraStore;

#[derive(Debug, Clone)]
pub struct NormalizedEvent {
    pub raw_event_id: String,
    pub received_at: String,
    pub observed_at: String,
    pub severity_text: String,
    pub severity_number: i32,
    pub target: Option<String>,
    pub family: Option<String>,
    pub subsystem: Option<String>,
    pub run_id: Option<String>,
    pub tick: Option<i64>,
    pub message_text: Option<String>,
    pub body_json: String,
    pub attributes_json: String,
    pub resource_json: String,
    pub scope_json: String,
}

#[derive(Debug, Clone)]
pub struct IngestOutcome {
    pub touched_run_ids: Vec<String>,
    pub last_batch_at: String,
}

#[derive(Debug, Clone)]
pub struct IngestCounts {
    pub raw_event_count: u64,
    pub run_count: u64,
    pub tick_count: u64,
}

impl MoiraStore {
    pub async fn ingest_events(&self, events: Vec<NormalizedEvent>) -> Result<IngestOutcome, String> {
        if events.is_empty() {
            return Ok(IngestOutcome {
                touched_run_ids: Vec::new(),
                last_batch_at: String::new(),
            });
        }

        let last_batch_at = events
            .last()
            .map(|event| event.received_at.clone())
            .unwrap_or_default();
        let touched_run_ids = events
            .iter()
            .filter_map(|event| event.run_id.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();

        let conn = self.conn.lock().await;
        conn.execute_batch("BEGIN TRANSACTION;")
            .map_err(|err| format!("failed to begin Moira ingest transaction: {err}"))?;

        let result = (|| -> Result<(), String> {
            let mut insert = conn
                .prepare(
                    r#"
                    INSERT INTO raw_events (
                      raw_event_id, received_at, observed_at, severity_text, severity_number,
                      target, family, subsystem, run_id, tick, message_text, body_json,
                      attributes_json, resource_json, scope_json
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                    "#,
                )
                .map_err(|err| format!("failed to prepare raw event insert: {err}"))?;

            for event in &events {
                insert
                    .execute(params![
                        event.raw_event_id,
                        event.received_at,
                        event.observed_at,
                        event.severity_text,
                        event.severity_number,
                        event.target,
                        event.family,
                        event.subsystem,
                        event.run_id,
                        event.tick,
                        event.message_text,
                        event.body_json,
                        event.attributes_json,
                        event.resource_json,
                        event.scope_json
                    ])
                    .map_err(|err| format!("failed to insert raw event: {err}"))?;
            }

            for run_id in &touched_run_ids {
                refresh_run_projection(&conn, run_id)?;
                refresh_tick_projection(&conn, run_id)?;
            }

            Ok(())
        })();

        match result {
            Ok(()) => conn
                .execute_batch("COMMIT;")
                .map_err(|err| format!("failed to commit Moira ingest transaction: {err}"))?,
            Err(err) => {
                let _ = conn.execute_batch("ROLLBACK;");
                return Err(err);
            }
        }

        Ok(IngestOutcome {
            touched_run_ids,
            last_batch_at,
        })
    }

    pub async fn counts(&self) -> Result<IngestCounts, String> {
        let conn = self.conn.lock().await;
        Ok(IngestCounts {
            raw_event_count: scalar_count(&conn, "SELECT COUNT(*) FROM raw_events")?,
            run_count: scalar_count(&conn, "SELECT COUNT(*) FROM runs")?,
            tick_count: scalar_count(&conn, "SELECT COUNT(*) FROM ticks")?,
        })
    }
}

fn refresh_run_projection(conn: &duckdb::Connection, run_id: &str) -> Result<(), String> {
    conn.execute("DELETE FROM runs WHERE run_id = ?", params![run_id])
        .map_err(|err| format!("failed to clear run projection: {err}"))?;
    conn.execute(
        r#"
        INSERT INTO runs (
          run_id, first_seen_at, last_seen_at, event_count, warning_count, error_count, latest_tick
        )
        SELECT
          run_id,
          MIN(observed_at),
          MAX(observed_at),
          COUNT(*),
          SUM(CASE WHEN LOWER(COALESCE(severity_text, '')) = 'warn' THEN 1 ELSE 0 END),
          SUM(CASE WHEN LOWER(COALESCE(severity_text, '')) = 'error' THEN 1 ELSE 0 END),
          MAX(tick)
        FROM raw_events
        WHERE run_id = ?
        GROUP BY run_id
        "#,
        params![run_id],
    )
    .map_err(|err| format!("failed to refresh run projection: {err}"))?;
    Ok(())
}

fn refresh_tick_projection(conn: &duckdb::Connection, run_id: &str) -> Result<(), String> {
    conn.execute("DELETE FROM ticks WHERE run_id = ?", params![run_id])
        .map_err(|err| format!("failed to clear tick projection: {err}"))?;
    conn.execute(
        r#"
        INSERT INTO ticks (
          run_id, tick, first_seen_at, last_seen_at, event_count, warning_count, error_count
        )
        SELECT
          run_id,
          tick,
          MIN(observed_at),
          MAX(observed_at),
          COUNT(*),
          SUM(CASE WHEN LOWER(COALESCE(severity_text, '')) = 'warn' THEN 1 ELSE 0 END),
          SUM(CASE WHEN LOWER(COALESCE(severity_text, '')) = 'error' THEN 1 ELSE 0 END)
        FROM raw_events
        WHERE run_id = ? AND tick IS NOT NULL
        GROUP BY run_id, tick
        "#,
        params![run_id],
    )
    .map_err(|err| format!("failed to refresh tick projection: {err}"))?;
    Ok(())
}

fn scalar_count(conn: &duckdb::Connection, sql: &str) -> Result<u64, String> {
    conn.query_row(sql, [], |row| row.get::<_, i64>(0))
        .map(|value| value.max(0) as u64)
        .map_err(|err| format!("failed to read Moira count: {err}"))
}
