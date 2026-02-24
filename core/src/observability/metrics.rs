use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use metrics::{Unit, counter, describe_counter, describe_gauge, gauge};
use metrics_exporter_prometheus::{BuildError, PrometheusBuilder};

pub const CORTEX_CYCLE_ID_METRIC: &str = "beluna_cortex_cycle_id";
pub const CORTEX_INPUT_IR_ACT_DESCRIPTOR_CATALOG_COUNT_METRIC: &str =
    "beluna_cortex_input_ir_act_descriptor_catalog_count";
pub const CHAT_TASK_LATENCY_MS_METRIC: &str = "beluna_chat_task_latency_ms";
pub const CHAT_TASK_FAILURES_TOTAL_METRIC: &str = "beluna_chat_task_failures_total";
pub const CHAT_TASK_RETRIES_TOTAL_METRIC: &str = "beluna_chat_task_retries_total";
pub const CHAT_THREAD_TURNS_TOTAL_METRIC: &str = "beluna_chat_thread_turns_total";
pub const CHAT_THREAD_TOOL_CALLS_TOTAL_METRIC: &str = "beluna_chat_thread_tool_calls_total";
pub const CHAT_THREAD_TOKENS_IN_TOTAL_METRIC: &str = "beluna_chat_thread_tokens_in_total";
pub const CHAT_THREAD_TOKENS_OUT_TOTAL_METRIC: &str = "beluna_chat_thread_tokens_out_total";
pub const CHAT_THREAD_FAILURES_TOTAL_METRIC: &str = "beluna_chat_thread_failures_total";
pub const CHAT_THREAD_LAST_TURN_LATENCY_MS_METRIC: &str = "beluna_chat_thread_last_turn_latency_ms";

const DEFAULT_METRICS_PORT: u16 = 9464;

#[derive(Debug, Clone, Copy)]
pub struct MetricsRuntime {
    pub listen_addr: SocketAddr,
}

impl MetricsRuntime {
    pub fn default_listen_addr() -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), DEFAULT_METRICS_PORT)
    }
}

pub fn start_prometheus_exporter(listen_addr: SocketAddr) -> Result<MetricsRuntime, BuildError> {
    describe_gauge!(
        CORTEX_CYCLE_ID_METRIC,
        Unit::Count,
        "Latest cortex cycle id processed by stem."
    );
    describe_gauge!(
        CORTEX_INPUT_IR_ACT_DESCRIPTOR_CATALOG_COUNT_METRIC,
        Unit::Count,
        "Count of act descriptors included in cortex input IR catalog."
    );
    describe_gauge!(
        CHAT_TASK_LATENCY_MS_METRIC,
        Unit::Milliseconds,
        "Latest latency observed for chat tasks by task type/backend/model."
    );
    describe_counter!(
        CHAT_TASK_FAILURES_TOTAL_METRIC,
        Unit::Count,
        "Total failures observed for chat task executions."
    );
    describe_counter!(
        CHAT_TASK_RETRIES_TOTAL_METRIC,
        Unit::Count,
        "Total retries issued for chat backend requests."
    );
    describe_counter!(
        CHAT_THREAD_TURNS_TOTAL_METRIC,
        Unit::Count,
        "Total completed turns per chat thread."
    );
    describe_counter!(
        CHAT_THREAD_TOOL_CALLS_TOTAL_METRIC,
        Unit::Count,
        "Total tool calls proposed by the model per chat thread."
    );
    describe_counter!(
        CHAT_THREAD_TOKENS_IN_TOTAL_METRIC,
        Unit::Count,
        "Accumulated input tokens per chat thread."
    );
    describe_counter!(
        CHAT_THREAD_TOKENS_OUT_TOTAL_METRIC,
        Unit::Count,
        "Accumulated output tokens per chat thread."
    );
    describe_counter!(
        CHAT_THREAD_FAILURES_TOTAL_METRIC,
        Unit::Count,
        "Accumulated failed turns per chat thread and error kind."
    );
    describe_gauge!(
        CHAT_THREAD_LAST_TURN_LATENCY_MS_METRIC,
        Unit::Milliseconds,
        "Latency of the last observed terminal turn per chat thread."
    );

    PrometheusBuilder::new()
        .with_http_listener(listen_addr)
        .install()?;

    Ok(MetricsRuntime { listen_addr })
}

pub fn record_cortex_cycle_id(cycle_id: u64) {
    gauge!(CORTEX_CYCLE_ID_METRIC).set(cycle_id as f64);
}

pub fn record_cortex_input_ir_act_descriptor_catalog_count(count: usize) {
    gauge!(CORTEX_INPUT_IR_ACT_DESCRIPTOR_CATALOG_COUNT_METRIC).set(count as f64);
}

pub fn record_chat_task_latency_ms(task_type: &str, backend: &str, model: &str, latency_ms: u64) {
    gauge!(
        CHAT_TASK_LATENCY_MS_METRIC,
        "task_type" => task_type.to_string(),
        "backend" => backend.to_string(),
        "model" => model.to_string()
    )
    .set(latency_ms as f64);
}

pub fn increment_chat_task_failures_total(task_type: &str, error_kind: &str) {
    counter!(
        CHAT_TASK_FAILURES_TOTAL_METRIC,
        "task_type" => task_type.to_string(),
        "error_kind" => error_kind.to_string()
    )
    .increment(1);
}

pub fn increment_chat_task_retries_total(backend: &str, model: &str, retry_count: u64) {
    if retry_count == 0 {
        return;
    }
    counter!(
        CHAT_TASK_RETRIES_TOTAL_METRIC,
        "backend" => backend.to_string(),
        "model" => model.to_string()
    )
    .increment(retry_count);
}

pub fn increment_chat_thread_turns_total(session_id: &str, thread_id: &str) {
    counter!(
        CHAT_THREAD_TURNS_TOTAL_METRIC,
        "session_id" => session_id.to_string(),
        "thread_id" => thread_id.to_string()
    )
    .increment(1);
}

pub fn add_chat_thread_tool_calls_total(
    session_id: &str,
    thread_id: &str,
    tool_name: &str,
    value: u64,
) {
    if value == 0 {
        return;
    }
    counter!(
        CHAT_THREAD_TOOL_CALLS_TOTAL_METRIC,
        "session_id" => session_id.to_string(),
        "thread_id" => thread_id.to_string(),
        "tool_name" => tool_name.to_string()
    )
    .increment(value);
}

pub fn add_chat_thread_tokens_in_total(session_id: &str, thread_id: &str, value: u64) {
    if value == 0 {
        return;
    }
    counter!(
        CHAT_THREAD_TOKENS_IN_TOTAL_METRIC,
        "session_id" => session_id.to_string(),
        "thread_id" => thread_id.to_string()
    )
    .increment(value);
}

pub fn add_chat_thread_tokens_out_total(session_id: &str, thread_id: &str, value: u64) {
    if value == 0 {
        return;
    }
    counter!(
        CHAT_THREAD_TOKENS_OUT_TOTAL_METRIC,
        "session_id" => session_id.to_string(),
        "thread_id" => thread_id.to_string()
    )
    .increment(value);
}

pub fn increment_chat_thread_failures_total(session_id: &str, thread_id: &str, error_kind: &str) {
    counter!(
        CHAT_THREAD_FAILURES_TOTAL_METRIC,
        "session_id" => session_id.to_string(),
        "thread_id" => thread_id.to_string(),
        "error_kind" => error_kind.to_string()
    )
    .increment(1);
}

pub fn set_chat_thread_last_turn_latency_ms(session_id: &str, thread_id: &str, latency_ms: u64) {
    gauge!(
        CHAT_THREAD_LAST_TURN_LATENCY_MS_METRIC,
        "session_id" => session_id.to_string(),
        "thread_id" => thread_id.to_string()
    )
    .set(latency_ms as f64);
}
