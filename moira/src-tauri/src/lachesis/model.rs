use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceiverStatus {
    pub endpoint: String,
    pub wake_state: String,
    pub db_path: String,
    pub last_batch_at: Option<String>,
    pub last_error: Option<String>,
    pub raw_event_count: u64,
    pub wake_count: u64,
    pub tick_count: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunSummary {
    pub run_id: String,
    pub first_seen_at: String,
    pub last_seen_at: String,
    pub event_count: u64,
    pub warning_count: u64,
    pub error_count: u64,
    pub latest_tick: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TickSummary {
    pub run_id: String,
    pub tick: u64,
    pub first_seen_at: String,
    pub last_seen_at: String,
    pub event_count: u64,
    pub warning_count: u64,
    pub error_count: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventRecord {
    pub raw_event_id: String,
    pub received_at: String,
    pub observed_at: String,
    pub severity_text: String,
    pub target: Option<String>,
    pub family: Option<String>,
    pub subsystem: Option<String>,
    pub run_id: Option<String>,
    pub tick: Option<u64>,
    pub message_text: Option<String>,
    pub attributes: Value,
    pub body: Value,
    pub resource: Value,
    pub scope: Value,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TickDetail {
    pub summary: TickSummary,
    pub cortex: Vec<EventRecord>,
    pub stem: Vec<EventRecord>,
    pub spine: Vec<EventRecord>,
    pub raw: Vec<EventRecord>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IngestPulse {
    pub touched_run_ids: Vec<String>,
    pub last_batch_at: String,
}
