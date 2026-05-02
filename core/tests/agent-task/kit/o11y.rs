use std::{
    collections::BTreeMap,
    fmt,
    sync::{Arc, Mutex, OnceLock},
};

use anyhow::Result;
use serde_json::{Map, Value, json};
use tracing::{
    Event, Subscriber,
    field::{Field, Visit},
};
use tracing_subscriber::{Layer, layer::Context, layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
pub struct ContractEventCapture {
    inner: Arc<CaptureInner>,
    start_index: usize,
    installed: bool,
}

struct GlobalCapture {
    inner: Arc<CaptureInner>,
    installed: bool,
}

#[derive(Default)]
struct CaptureInner {
    events: Mutex<Vec<Value>>,
}

struct ContractEventLayer {
    inner: Arc<CaptureInner>,
}

#[derive(Default)]
struct FieldVisitor {
    fields: Map<String, Value>,
}

static GLOBAL_CAPTURE: OnceLock<GlobalCapture> = OnceLock::new();

impl ContractEventCapture {
    pub fn start() -> Self {
        let global = GLOBAL_CAPTURE.get_or_init(|| {
            let inner = Arc::new(CaptureInner::default());
            let layer = ContractEventLayer {
                inner: Arc::clone(&inner),
            };
            let installed = tracing_subscriber::registry()
                .with(layer)
                .try_init()
                .is_ok();
            GlobalCapture { inner, installed }
        });
        let start_index = global.inner.len();
        Self {
            inner: Arc::clone(&global.inner),
            start_index,
            installed: global.installed,
        }
    }

    pub fn installed(&self) -> bool {
        self.installed
    }

    pub fn events(&self) -> Vec<Value> {
        self.inner.events_since(self.start_index)
    }
}

impl CaptureInner {
    fn len(&self) -> usize {
        self.events.lock().expect("lock poisoned").len()
    }

    fn push(&self, event: Value) {
        self.events.lock().expect("lock poisoned").push(event);
    }

    fn events_since(&self, start_index: usize) -> Vec<Value> {
        self.events
            .lock()
            .expect("lock poisoned")
            .iter()
            .skip(start_index)
            .cloned()
            .collect()
    }
}

impl<S> Layer<S> for ContractEventLayer
where
    S: Subscriber,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let metadata = event.metadata();
        if metadata.target() != "observability.contract" {
            return;
        }

        let mut visitor = FieldVisitor::default();
        event.record(&mut visitor);
        let payload = visitor
            .fields
            .get("payload")
            .and_then(Value::as_str)
            .and_then(|value| serde_json::from_str::<Value>(value).ok());

        self.inner.push(json!({
            "target": metadata.target(),
            "level": metadata.level().to_string(),
            "name": metadata.name(),
            "family": visitor.fields.get("family").cloned(),
            "tick": visitor.fields.get("tick").cloned(),
            "request_id": visitor.fields.get("request_id").cloned(),
            "thread_id": visitor.fields.get("thread_id").cloned(),
            "turn_id": visitor.fields.get("turn_id").cloned(),
            "payload": payload,
            "fields": visitor.fields,
        }));
    }
}

impl Visit for FieldVisitor {
    fn record_bool(&mut self, field: &Field, value: bool) {
        self.insert(field, Value::Bool(value));
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.insert(field, Value::Number(value.into()));
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.insert(field, Value::Number(value.into()));
    }

    fn record_f64(&mut self, field: &Field, value: f64) {
        self.insert(field, json!(value));
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.insert(field, Value::String(value.to_string()));
    }

    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        self.insert(field, Value::String(format!("{value:?}")));
    }
}

impl FieldVisitor {
    fn insert(&mut self, field: &Field, value: Value) {
        self.fields.insert(field.name().to_string(), value);
    }
}

pub fn summarize_ai_gateway_events(events: &[Value]) -> Result<Value> {
    let mut family_counts = BTreeMap::<String, usize>::new();
    let mut request_kinds = BTreeMap::<String, usize>::new();
    let mut turn_statuses = BTreeMap::<String, usize>::new();
    let mut backends = BTreeMap::<String, usize>::new();
    let mut models = BTreeMap::<String, usize>::new();
    let mut turns = Vec::new();
    let mut errors = Vec::new();

    for event in events {
        let Some(payload) = event.get("payload") else {
            continue;
        };
        let Some(family) = payload.get("family").and_then(Value::as_str) else {
            continue;
        };
        if !family.starts_with("ai-gateway.") {
            continue;
        }

        *family_counts.entry(family.to_string()).or_default() += 1;
        match family {
            "ai-gateway.request" => {
                bump(payload, "kind", &mut request_kinds);
                bump(payload, "backend_id", &mut backends);
                bump(payload, "model", &mut models);
                if let Some(error) = payload.get("error") {
                    errors.push(json!({
                        "family": family,
                        "request_id": payload.get("request_id"),
                        "kind": payload.get("kind"),
                        "error": error,
                    }));
                }
            }
            "ai-gateway.chat.turn" => {
                bump(payload, "status", &mut turn_statuses);
                if let Some(error) = payload.get("error") {
                    errors.push(json!({
                        "family": family,
                        "request_id": payload.get("request_id"),
                        "turn_id": payload.get("turn_id"),
                        "status": payload.get("status"),
                        "error": error,
                    }));
                }
                turns.push(json!({
                    "tick": payload.get("tick"),
                    "thread_id": payload.get("thread_id"),
                    "turn_id": payload.get("turn_id"),
                    "request_id": payload.get("request_id"),
                    "status": payload.get("status"),
                    "finish_reason": payload.get("finish_reason"),
                    "usage": payload.get("usage"),
                    "backend_metadata": payload.get("backend_metadata"),
                    "message_count": payload
                        .get("messages_when_committed")
                        .and_then(Value::as_array)
                        .map(Vec::len),
                    "assistant_text": assistant_text(payload.get("messages_when_committed")),
                    "tool_calls": tool_calls(payload.get("messages_when_committed")),
                    "tool_results": tool_results(payload.get("messages_when_committed")),
                }));
            }
            _ => {}
        }
    }

    Ok(json!({
        "contract_event_count": events.len(),
        "ai_gateway_event_count": family_counts.values().sum::<usize>(),
        "family_counts": family_counts,
        "request_kinds": request_kinds,
        "turn_statuses": turn_statuses,
        "backends": backends,
        "models": models,
        "turns": turns,
        "errors": errors,
    }))
}

fn bump(payload: &Value, key: &str, counts: &mut BTreeMap<String, usize>) {
    if let Some(value) = payload.get(key).and_then(Value::as_str) {
        *counts.entry(value.to_string()).or_default() += 1;
    }
}

fn assistant_text(messages: Option<&Value>) -> Option<String> {
    let text = messages?
        .as_array()?
        .iter()
        .filter(|message| message.get("kind").and_then(Value::as_str) == Some("assistant"))
        .flat_map(|message| message.get("parts").and_then(Value::as_array))
        .flatten()
        .filter_map(|part| {
            part.get("text")
                .and_then(|text| text.get("text"))
                .and_then(Value::as_str)
        })
        .collect::<Vec<_>>()
        .join("\n");
    let trimmed = text.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn tool_calls(messages: Option<&Value>) -> Vec<Value> {
    messages
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|message| message.get("kind").and_then(Value::as_str) == Some("tool_call"))
        .map(|message| {
            let arguments_json = message
                .get("arguments_json")
                .and_then(Value::as_str)
                .and_then(|value| serde_json::from_str::<Value>(value).ok())
                .unwrap_or(Value::Null);
            json!({
                "name": message.get("name"),
                "call_id": message.get("call_id"),
                "arguments": arguments_json,
            })
        })
        .collect()
}

fn tool_results(messages: Option<&Value>) -> Vec<Value> {
    messages
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|message| message.get("kind").and_then(Value::as_str) == Some("tool_call_result"))
        .map(|message| {
            let payload = message.get("payload");
            json!({
                "name": message.get("name"),
                "call_id": message.get("call_id"),
                "ok": payload.and_then(|payload| payload.get("ok")),
                "error": payload.and_then(|payload| payload.get("error")),
                "dispatch_result": payload
                    .and_then(|payload| payload.get("data"))
                    .and_then(|data| data.get("dispatch_result")),
            })
        })
        .collect()
}
