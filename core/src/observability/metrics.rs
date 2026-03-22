use std::sync::OnceLock;

use opentelemetry::{
    KeyValue, global,
    metrics::{Counter, Gauge, Meter},
};

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

struct MetricsInstruments {
    cortex_cycle_id: Gauge<f64>,
    cortex_input_ir_act_descriptor_catalog_count: Gauge<f64>,
    chat_task_latency_ms: Gauge<f64>,
    chat_task_failures_total: Counter<u64>,
    chat_task_retries_total: Counter<u64>,
    chat_thread_turns_total: Counter<u64>,
    chat_thread_tool_calls_total: Counter<u64>,
    chat_thread_tokens_in_total: Counter<u64>,
    chat_thread_tokens_out_total: Counter<u64>,
    chat_thread_failures_total: Counter<u64>,
    chat_thread_last_turn_latency_ms: Gauge<f64>,
}

static METRICS: OnceLock<MetricsInstruments> = OnceLock::new();

fn meter() -> Meter {
    global::meter("beluna.core")
}

fn instruments() -> &'static MetricsInstruments {
    METRICS.get_or_init(|| {
        let meter = meter();
        MetricsInstruments {
            cortex_cycle_id: meter
                .f64_gauge(CORTEX_CYCLE_ID_METRIC)
                .with_description("Latest cortex cycle id processed by stem.")
                .with_unit("count")
                .build(),
            cortex_input_ir_act_descriptor_catalog_count: meter
                .f64_gauge(CORTEX_INPUT_IR_ACT_DESCRIPTOR_CATALOG_COUNT_METRIC)
                .with_description(
                    "Latest act descriptor catalog count observed while building cortex input IR.",
                )
                .with_unit("count")
                .build(),
            chat_task_latency_ms: meter
                .f64_gauge(CHAT_TASK_LATENCY_MS_METRIC)
                .with_description(
                    "Latest latency observed for chat tasks by task type/backend/model.",
                )
                .with_unit("ms")
                .build(),
            chat_task_failures_total: meter
                .u64_counter(CHAT_TASK_FAILURES_TOTAL_METRIC)
                .with_description("Total failures observed for chat task executions.")
                .with_unit("count")
                .build(),
            chat_task_retries_total: meter
                .u64_counter(CHAT_TASK_RETRIES_TOTAL_METRIC)
                .with_description("Total retries issued for chat backend requests.")
                .with_unit("count")
                .build(),
            chat_thread_turns_total: meter
                .u64_counter(CHAT_THREAD_TURNS_TOTAL_METRIC)
                .with_description("Total completed turns per chat thread.")
                .with_unit("count")
                .build(),
            chat_thread_tool_calls_total: meter
                .u64_counter(CHAT_THREAD_TOOL_CALLS_TOTAL_METRIC)
                .with_description("Total tool calls proposed by the model per chat thread.")
                .with_unit("count")
                .build(),
            chat_thread_tokens_in_total: meter
                .u64_counter(CHAT_THREAD_TOKENS_IN_TOTAL_METRIC)
                .with_description("Accumulated input tokens per chat thread.")
                .with_unit("count")
                .build(),
            chat_thread_tokens_out_total: meter
                .u64_counter(CHAT_THREAD_TOKENS_OUT_TOTAL_METRIC)
                .with_description("Accumulated output tokens per chat thread.")
                .with_unit("count")
                .build(),
            chat_thread_failures_total: meter
                .u64_counter(CHAT_THREAD_FAILURES_TOTAL_METRIC)
                .with_description("Accumulated failed turns per chat thread and error kind.")
                .with_unit("count")
                .build(),
            chat_thread_last_turn_latency_ms: meter
                .f64_gauge(CHAT_THREAD_LAST_TURN_LATENCY_MS_METRIC)
                .with_description("Latency of the last observed terminal turn per chat thread.")
                .with_unit("ms")
                .build(),
        }
    })
}

pub fn record_cortex_cycle_id(cycle_id: u64) {
    instruments().cortex_cycle_id.record(cycle_id as f64, &[]);
}

pub fn record_cortex_input_ir_act_descriptor_catalog_count(catalog_count: usize) {
    instruments()
        .cortex_input_ir_act_descriptor_catalog_count
        .record(catalog_count as f64, &[]);
}

pub fn record_chat_task_latency_ms(task_type: &str, backend: &str, model: &str, latency_ms: u64) {
    instruments().chat_task_latency_ms.record(
        latency_ms as f64,
        &[
            KeyValue::new("task_type", task_type.to_string()),
            KeyValue::new("backend", backend.to_string()),
            KeyValue::new("model", model.to_string()),
        ],
    );
}

pub fn increment_chat_task_failures_total(task_type: &str, error_kind: &str) {
    instruments().chat_task_failures_total.add(
        1,
        &[
            KeyValue::new("task_type", task_type.to_string()),
            KeyValue::new("error_kind", error_kind.to_string()),
        ],
    );
}

pub fn increment_chat_task_retries_total(backend: &str, model: &str, retry_count: u64) {
    if retry_count == 0 {
        return;
    }
    instruments().chat_task_retries_total.add(
        retry_count,
        &[
            KeyValue::new("backend", backend.to_string()),
            KeyValue::new("model", model.to_string()),
        ],
    );
}

pub fn increment_chat_thread_turns_total(session_id: &str, thread_id: &str) {
    instruments().chat_thread_turns_total.add(
        1,
        &[
            KeyValue::new("session_id", session_id.to_string()),
            KeyValue::new("thread_id", thread_id.to_string()),
        ],
    );
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
    instruments().chat_thread_tool_calls_total.add(
        value,
        &[
            KeyValue::new("session_id", session_id.to_string()),
            KeyValue::new("thread_id", thread_id.to_string()),
            KeyValue::new("tool_name", tool_name.to_string()),
        ],
    );
}

pub fn add_chat_thread_tokens_in_total(session_id: &str, thread_id: &str, value: u64) {
    if value == 0 {
        return;
    }
    instruments().chat_thread_tokens_in_total.add(
        value,
        &[
            KeyValue::new("session_id", session_id.to_string()),
            KeyValue::new("thread_id", thread_id.to_string()),
        ],
    );
}

pub fn add_chat_thread_tokens_out_total(session_id: &str, thread_id: &str, value: u64) {
    if value == 0 {
        return;
    }
    instruments().chat_thread_tokens_out_total.add(
        value,
        &[
            KeyValue::new("session_id", session_id.to_string()),
            KeyValue::new("thread_id", thread_id.to_string()),
        ],
    );
}

pub fn increment_chat_thread_failures_total(session_id: &str, thread_id: &str, error_kind: &str) {
    instruments().chat_thread_failures_total.add(
        1,
        &[
            KeyValue::new("session_id", session_id.to_string()),
            KeyValue::new("thread_id", thread_id.to_string()),
            KeyValue::new("error_kind", error_kind.to_string()),
        ],
    );
}

pub fn set_chat_thread_last_turn_latency_ms(session_id: &str, thread_id: &str, latency_ms: u64) {
    instruments().chat_thread_last_turn_latency_ms.record(
        latency_ms as f64,
        &[
            KeyValue::new("session_id", session_id.to_string()),
            KeyValue::new("thread_id", thread_id.to_string()),
        ],
    );
}
