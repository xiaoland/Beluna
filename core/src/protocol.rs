use serde::Deserialize;

use crate::cortex::{
    AdmissionOutcomeSignal, CapabilityCatalog, EndpointSnapshot, IntentContext, ReactionLimits,
    SenseDelta,
};

#[derive(Debug, Clone, PartialEq)]
pub enum ClientMessage {
    Exit,
    Sense(SenseDelta),
    EnvSnapshot(EndpointSnapshot),
    AdmissionFeedback(AdmissionOutcomeSignal),
    CapabilityCatalogUpdate(CapabilityCatalog),
    CortexLimitsUpdate(ReactionLimits),
    IntentContextUpdate(IntentContext),
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum WireMessage {
    Exit {},
    Sense {
        sense_id: String,
        source: String,
        payload: serde_json::Value,
    },
    EnvSnapshot {
        endpoint_key: String,
        blob: serde_json::Value,
        #[serde(default)]
        truncated: bool,
    },
    AdmissionFeedback {
        attempt_id: String,
        code: String,
    },
    CapabilityCatalogUpdate {
        catalog: CapabilityCatalog,
    },
    CortexLimitsUpdate {
        limits: ReactionLimits,
    },
    IntentContextUpdate {
        context: IntentContext,
    },
}

pub fn parse_client_message(line: &str) -> Result<ClientMessage, serde_json::Error> {
    let wire: WireMessage = serde_json::from_str(line)?;
    let message = match wire {
        WireMessage::Exit {} => ClientMessage::Exit,
        WireMessage::Sense {
            sense_id,
            source,
            payload,
        } => ClientMessage::Sense(SenseDelta {
            sense_id,
            source,
            payload,
        }),
        WireMessage::EnvSnapshot {
            endpoint_key,
            blob,
            truncated,
        } => {
            let blob_bytes = serde_json::to_vec(&blob).map(|bytes| bytes.len()).unwrap_or(0);
            ClientMessage::EnvSnapshot(EndpointSnapshot {
                endpoint_key,
                blob,
                truncated,
                blob_bytes,
            })
        }
        WireMessage::AdmissionFeedback { attempt_id, code } => {
            ClientMessage::AdmissionFeedback(AdmissionOutcomeSignal { attempt_id, code })
        }
        WireMessage::CapabilityCatalogUpdate { catalog } => {
            ClientMessage::CapabilityCatalogUpdate(catalog)
        }
        WireMessage::CortexLimitsUpdate { limits } => ClientMessage::CortexLimitsUpdate(limits),
        WireMessage::IntentContextUpdate { context } => ClientMessage::IntentContextUpdate(context),
    };
    Ok(message)
}

#[cfg(test)]
mod tests {
    use super::{ClientMessage, parse_client_message};

    #[test]
    fn accepts_exact_exit_message() {
        let parsed = parse_client_message(r#"{"type":"exit"}"#).expect("exit message should parse");
        assert_eq!(parsed, ClientMessage::Exit);
    }

    #[test]
    fn accepts_sense_message() {
        let parsed = parse_client_message(
            r#"{"type":"sense","sense_id":"s1","source":"sensor","payload":{"v":1}}"#,
        )
        .expect("sense message should parse");
        assert!(matches!(parsed, ClientMessage::Sense(_)));
    }

    #[test]
    fn rejects_plain_string_message() {
        assert!(parse_client_message(r#""exit""#).is_err());
    }

    #[test]
    fn rejects_unknown_message_type() {
        assert!(parse_client_message(r#"{"type":"ping"}"#).is_err());
    }

    #[test]
    fn rejects_unknown_fields() {
        assert!(parse_client_message(r#"{"type":"exit","extra":"value"}"#).is_err());
    }
}
