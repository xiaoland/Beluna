use std::collections::BTreeSet;

use duckdb::params;

use super::LachesisStore;

#[derive(Debug, Clone)]
pub struct NormalizedEvent {
    pub raw_event_id: String,
    pub received_at: String,
    pub observed_at: String,
    pub severity_text: String,
    pub severity_number: i32,
    pub record_kind: String,
    pub scope_name: Option<String>,
    pub event_name: Option<String>,
    pub trace_id_hex: Option<String>,
    pub span_id_hex: Option<String>,
    pub trace_flags: Option<u32>,
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

impl LachesisStore {
    pub async fn ingest_events(
        &self,
        events: Vec<NormalizedEvent>,
    ) -> Result<IngestOutcome, String> {
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
        let direct_run_ids = events
            .iter()
            .filter_map(|event| event.run_id.clone())
            .collect::<BTreeSet<_>>();
        let touched_trace_ids = events
            .iter()
            .filter_map(|event| event.trace_id_hex.clone())
            .collect::<BTreeSet<_>>();
        let mut touched_run_ids = direct_run_ids;

        let conn = self.conn.lock().await;
        conn.execute_batch("BEGIN TRANSACTION;")
            .map_err(|err| format!("failed to begin Lachesis ingest transaction: {err}"))?;

        let result = (|| -> Result<(), String> {
            let mut insert = conn
                .prepare(
                    r#"
                    INSERT INTO raw_events (
                      raw_event_id, received_at, observed_at, severity_text, severity_number,
                      record_kind, scope_name, event_name, trace_id_hex, span_id_hex, trace_flags,
                      target, family, subsystem, run_id, tick, message_text, body_json,
                      attributes_json, resource_json, scope_json
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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
                        event.record_kind,
                        event.scope_name,
                        event.event_name,
                        event.trace_id_hex,
                        event.span_id_hex,
                        event.trace_flags,
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

            for trace_id in &touched_trace_ids {
                collect_run_ids_for_trace(&conn, trace_id, &mut touched_run_ids)?;
            }

            for run_id in &touched_run_ids {
                refresh_tick_projection(&conn, run_id)?;
                refresh_run_projection(&conn, run_id)?;
            }

            Ok(())
        })();

        match result {
            Ok(()) => conn
                .execute_batch("COMMIT;")
                .map_err(|err| format!("failed to commit Lachesis ingest transaction: {err}"))?,
            Err(err) => {
                let _ = conn.execute_batch("ROLLBACK;");
                return Err(err);
            }
        }

        Ok(IngestOutcome {
            touched_run_ids: touched_run_ids.into_iter().collect(),
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
        WITH run_events AS (
          SELECT run_id, raw_event_id, observed_at, severity_text, tick
          FROM raw_events
          WHERE run_id = ?
          UNION
          SELECT ticks.run_id, raw_events.raw_event_id, raw_events.observed_at, raw_events.severity_text, ticks.tick
          FROM raw_events
          JOIN ticks ON ticks.trace_id_hex IS NOT NULL
                    AND ticks.trace_id_hex = raw_events.trace_id_hex
          WHERE ticks.run_id = ?
        )
        SELECT
          run_id,
          MIN(observed_at),
          MAX(observed_at),
          COUNT(*),
          SUM(CASE WHEN LOWER(COALESCE(severity_text, '')) = 'warn' THEN 1 ELSE 0 END),
          SUM(CASE WHEN LOWER(COALESCE(severity_text, '')) = 'error' THEN 1 ELSE 0 END),
          MAX(tick)
        FROM run_events
        GROUP BY run_id
        "#,
        params![run_id, run_id],
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
          run_id, tick, trace_id_hex, first_seen_at, last_seen_at, event_count, warning_count,
          error_count
        )
        SELECT
          anchor.run_id,
          anchor.tick,
          anchor.trace_id_hex,
          MIN(COALESCE(event.observed_at, anchor.observed_at)),
          MAX(COALESCE(event.observed_at, anchor.observed_at)),
          COUNT(event.raw_event_id),
          SUM(CASE WHEN LOWER(COALESCE(event.severity_text, '')) = 'warn' THEN 1 ELSE 0 END),
          SUM(CASE WHEN LOWER(COALESCE(event.severity_text, '')) = 'error' THEN 1 ELSE 0 END)
        FROM raw_events anchor
        LEFT JOIN raw_events event ON event.trace_id_hex = anchor.trace_id_hex
        WHERE anchor.run_id = ?
          AND anchor.tick IS NOT NULL
          AND anchor.trace_id_hex IS NOT NULL
          AND anchor.scope_name = 'beluna.core.stem.tick'
          AND anchor.event_name = 'granted'
        GROUP BY anchor.run_id, anchor.tick, anchor.trace_id_hex
        "#,
        params![run_id],
    )
    .map_err(|err| format!("failed to refresh native tick projection: {err}"))?;

    conn.execute(
        r#"
        INSERT INTO ticks (
          run_id, tick, trace_id_hex, first_seen_at, last_seen_at, event_count, warning_count,
          error_count
        )
        SELECT
          run_id,
          tick,
          MAX(trace_id_hex),
          MIN(observed_at),
          MAX(observed_at),
          COUNT(*),
          SUM(CASE WHEN LOWER(COALESCE(severity_text, '')) = 'warn' THEN 1 ELSE 0 END),
          SUM(CASE WHEN LOWER(COALESCE(severity_text, '')) = 'error' THEN 1 ELSE 0 END)
        FROM raw_events
        WHERE run_id = ? AND tick IS NOT NULL
          AND NOT EXISTS (
            SELECT 1
            FROM ticks
            WHERE ticks.run_id = raw_events.run_id
              AND ticks.tick = raw_events.tick
          )
        GROUP BY run_id, tick
        "#,
        params![run_id],
    )
    .map_err(|err| format!("failed to refresh tick projection: {err}"))?;
    Ok(())
}

fn collect_run_ids_for_trace(
    conn: &duckdb::Connection,
    trace_id: &str,
    out: &mut BTreeSet<String>,
) -> Result<(), String> {
    let mut stmt = conn
        .prepare("SELECT DISTINCT run_id FROM ticks WHERE trace_id_hex = ?")
        .map_err(|err| format!("failed to prepare trace run lookup: {err}"))?;
    let rows = stmt
        .query_map(params![trace_id], |row| row.get::<_, String>(0))
        .map_err(|err| format!("failed to query trace run lookup: {err}"))?;
    for row in rows {
        out.insert(row.map_err(|err| format!("failed to read trace run lookup: {err}"))?);
    }
    Ok(())
}

fn scalar_count(conn: &duckdb::Connection, sql: &str) -> Result<u64, String> {
    conn.query_row(sql, [], |row| row.get::<_, i64>(0))
        .map(|value| value.max(0) as u64)
        .map_err(|err| format!("failed to read Lachesis count: {err}"))
}
