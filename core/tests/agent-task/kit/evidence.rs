use std::sync::{Arc, Mutex};

use serde_json::{Map, Value};

#[derive(Debug, Clone, Default)]
pub struct EvidenceJournal {
    events: Arc<Mutex<Vec<Value>>>,
}

impl EvidenceJournal {
    pub fn record(&self, stream: &str, fields: Value) {
        let mut event = match fields {
            Value::Object(map) => map,
            value => {
                let mut map = Map::new();
                map.insert("value".to_string(), value);
                map
            }
        };
        event.insert("stream".to_string(), Value::String(stream.to_string()));
        self.events
            .lock()
            .expect("lock poisoned")
            .push(Value::Object(event));
    }

    pub fn events(&self) -> Vec<Value> {
        self.events.lock().expect("lock poisoned").clone()
    }
}
