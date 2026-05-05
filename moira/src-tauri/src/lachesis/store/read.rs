use duckdb::{Row, params};
use serde_json::Value;

use super::LachesisStore;
use crate::lachesis::model::{EventRecord, RunSummary, TickDetail, TickSummary};

const CORTEX_HANDLED_FAMILIES_SQL: &str = r#"'cortex.primary', 'cortex.sense-helper', 'cortex.goal-forest-helper', 'cortex.acts-helper', 'cortex.goal-forest'"#;
const CORTEX_HANDLED_EVENTS_SQL: &str = r#"'primary.started', 'primary.finished'"#;

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
            .prepare(&format!(
                r#"
                SELECT run_id, tick, trace_id_hex, first_seen_at, last_seen_at, event_count,
                       warning_count, error_count,
                       EXISTS(
                           SELECT 1
                           FROM raw_events
                           WHERE (
                               ticks.trace_id_hex IS NOT NULL
                               AND raw_events.trace_id_hex = ticks.trace_id_hex
                               AND raw_events.scope_name = 'beluna.core.cortex'
                               AND raw_events.event_name IN ({CORTEX_HANDLED_EVENTS_SQL})
                             )
                             OR (
                               raw_events.run_id = ticks.run_id
                               AND raw_events.tick = ticks.tick
                               AND raw_events.family IN ({CORTEX_HANDLED_FAMILIES_SQL})
                             )
                       ) AS cortex_handled
                FROM ticks
                WHERE run_id = ?
                ORDER BY tick DESC
                LIMIT 400
                "#,
            ))
            .map_err(|err| format!("failed to prepare ticks query: {err}"))?;
        let rows = stmt
            .query_map(params![run_id], |row| {
                Ok(TickSummary {
                    run_id: row.get(0)?,
                    tick: row.get::<_, i64>(1)?.max(0) as u64,
                    trace_id: row.get(2)?,
                    first_seen_at: row.get(3)?,
                    last_seen_at: row.get(4)?,
                    event_count: row.get::<_, i64>(5)?.max(0) as u64,
                    warning_count: row.get::<_, i64>(6)?.max(0) as u64,
                    error_count: row.get::<_, i64>(7)?.max(0) as u64,
                    cortex_handled: row.get(8)?,
                })
            })
            .map_err(|err| format!("failed to query ticks: {err}"))?;
        collect_rows(rows)
    }

    pub async fn tick_detail(&self, run_id: &str, tick: u64) -> Result<TickDetail, String> {
        let summary = {
            let conn = self.conn.lock().await;
            let mut stmt = conn
                .prepare(&format!(
                    r#"
                    SELECT run_id, tick, trace_id_hex, first_seen_at, last_seen_at, event_count,
                           warning_count, error_count,
                           EXISTS(
                               SELECT 1
                               FROM raw_events
                               WHERE (
                                   ticks.trace_id_hex IS NOT NULL
                                   AND raw_events.trace_id_hex = ticks.trace_id_hex
                                   AND raw_events.scope_name = 'beluna.core.cortex'
                                   AND raw_events.event_name IN ({CORTEX_HANDLED_EVENTS_SQL})
                                 )
                                 OR (
                                   raw_events.run_id = ticks.run_id
                                   AND raw_events.tick = ticks.tick
                                   AND raw_events.family IN ({CORTEX_HANDLED_FAMILIES_SQL})
                                 )
                           ) AS cortex_handled
                    FROM ticks
                    WHERE run_id = ? AND tick = ?
                    LIMIT 1
                    "#,
                ))
                .map_err(|err| format!("failed to prepare tick summary query: {err}"))?;
            stmt.query_row(params![run_id, tick as i64], |row| {
                Ok(TickSummary {
                    run_id: row.get(0)?,
                    tick: row.get::<_, i64>(1)?.max(0) as u64,
                    trace_id: row.get(2)?,
                    first_seen_at: row.get(3)?,
                    last_seen_at: row.get(4)?,
                    event_count: row.get::<_, i64>(5)?.max(0) as u64,
                    warning_count: row.get::<_, i64>(6)?.max(0) as u64,
                    error_count: row.get::<_, i64>(7)?.max(0) as u64,
                    cortex_handled: row.get(8)?,
                })
            })
            .map_err(|err| format!("failed to query tick summary: {err}"))?
        };

        let conn = self.conn.lock().await;
        let raw = if let Some(trace_id) = summary.trace_id.as_deref() {
            let query = raw_event_select(
                "WHERE trace_id_hex = ? ORDER BY observed_at ASC, raw_event_id ASC",
            );
            let mut stmt = conn
                .prepare(&query)
                .map_err(|err| format!("failed to prepare native tick detail query: {err}"))?;
            let rows = stmt
                .query_map(params![trace_id], event_record_from_row)
                .map_err(|err| format!("failed to query native raw tick events: {err}"))?;
            collect_rows(rows)?
        } else {
            let query = raw_event_select(
                "WHERE run_id = ? AND tick = ? ORDER BY observed_at ASC, raw_event_id ASC",
            );
            let mut stmt = conn
                .prepare(&query)
                .map_err(|err| format!("failed to prepare tick detail query: {err}"))?;
            let rows = stmt
                .query_map(params![run_id, tick as i64], event_record_from_row)
                .map_err(|err| format!("failed to query raw tick events: {err}"))?;
            collect_rows(rows)?
        };

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
        .or_else(|| event.scope_name.as_deref().and_then(core_owner_from_scope))
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

fn raw_event_select(where_clause: &str) -> String {
    format!(
        r#"
        SELECT raw_event_id, received_at, observed_at, severity_text,
               COALESCE(
                 record_kind,
                 CASE
                   WHEN scope_name = 'observability.contract'
                     OR (family IS NOT NULL AND attributes_json LIKE '%"payload"%')
                   THEN 'legacy_contract'
                   WHEN scope_name LIKE 'beluna.core.%' AND event_name IS NOT NULL THEN 'native_owner'
                   ELSE 'ordinary_log'
                 END
               ) AS record_kind,
               scope_name, event_name, trace_id_hex, span_id_hex, trace_flags, run_id, tick,
               target, family, subsystem, message_text, attributes_json, body_json, resource_json,
               scope_json
        FROM raw_events
        {where_clause}
        "#
    )
}

fn event_record_from_row(row: &Row<'_>) -> duckdb::Result<EventRecord> {
    Ok(EventRecord {
        raw_event_id: row.get(0)?,
        received_at: row.get(1)?,
        observed_at: row.get(2)?,
        severity_text: row
            .get::<_, Option<String>>(3)?
            .unwrap_or_else(|| "INFO".to_string()),
        record_kind: row.get(4)?,
        scope_name: row.get(5)?,
        event_name: row.get(6)?,
        trace_id: row.get(7)?,
        span_id: row.get(8)?,
        trace_flags: row
            .get::<_, Option<i64>>(9)?
            .map(|value| value.max(0) as u32),
        run_id: row.get(10)?,
        tick: row
            .get::<_, Option<i64>>(11)?
            .map(|value| value.max(0) as u64),
        target: row.get(12)?,
        family: row.get(13)?,
        subsystem: row.get(14)?,
        message_text: row.get(15)?,
        attributes: parse_json_column(row.get::<_, String>(16)?),
        body: parse_json_column(row.get::<_, String>(17)?),
        resource: parse_json_column(row.get::<_, String>(18)?),
        scope: parse_json_column(row.get::<_, String>(19)?),
    })
}

fn core_owner_from_scope(scope_name: &str) -> Option<&str> {
    scope_name
        .strip_prefix("beluna.core.")
        .map(|owner| owner.split('.').next().unwrap_or(owner))
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
