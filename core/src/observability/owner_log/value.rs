use std::collections::HashMap;

use opentelemetry::{Key, logs::AnyValue};
use serde_json::{Number, Value};

pub(crate) fn json_to_any(value: Value) -> AnyValue {
    match value {
        Value::Null => AnyValue::String("null".into()),
        Value::Bool(value) => AnyValue::Boolean(value),
        Value::Number(value) => number_to_any(value),
        Value::String(value) => AnyValue::String(value.into()),
        Value::Array(values) => {
            AnyValue::ListAny(Box::new(values.into_iter().map(json_to_any).collect()))
        }
        Value::Object(values) => AnyValue::Map(Box::new(
            values
                .into_iter()
                .map(|(key, value)| (Key::new(key), json_to_any(value)))
                .collect::<HashMap<_, _>>(),
        )),
    }
}

fn number_to_any(value: Number) -> AnyValue {
    if let Some(value) = value.as_i64() {
        return AnyValue::Int(value);
    }
    if let Some(value) = value.as_u64() {
        if let Ok(value) = i64::try_from(value) {
            return AnyValue::Int(value);
        }
    }
    if let Some(value) = value.as_f64() {
        return AnyValue::Double(value);
    }
    AnyValue::String(value.to_string().into())
}

#[cfg(test)]
mod tests {
    use opentelemetry::logs::AnyValue;
    use serde_json::json;

    use super::json_to_any;

    #[test]
    fn converts_nested_json_to_any_value() {
        let value = json_to_any(json!({
            "summary": "ok",
            "count": 2,
            "nested": { "flag": true },
            "items": ["a", "b"]
        }));

        let AnyValue::Map(values) = value else {
            panic!("expected map");
        };
        assert_eq!(
            values.get("summary").cloned(),
            Some(AnyValue::String("ok".into()))
        );
        assert_eq!(values.get("count").cloned(), Some(AnyValue::Int(2)));
        assert!(matches!(values.get("nested"), Some(AnyValue::Map(_))));
        assert!(matches!(values.get("items"), Some(AnyValue::ListAny(_))));
    }
}
