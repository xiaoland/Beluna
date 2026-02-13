use serde::{Deserialize, Serialize};

use crate::cortex::{
    AdmissionOutcomeSignal, CapabilityCatalog, EndpointSnapshot, IntentContext, ReactionLimits,
    SenseDelta,
};
use crate::spine::types::{AdmittedAction, EndpointCapabilityDescriptor, RouteKey};

#[derive(Debug, Clone, PartialEq)]
pub enum BodyIngressMessage {
    Sense(SenseDelta),
    EnvSnapshot(EndpointSnapshot),
    AdmissionFeedback(AdmissionOutcomeSignal),
    CapabilityCatalogUpdate(CapabilityCatalog),
    CortexLimitsUpdate(ReactionLimits),
    IntentContextUpdate(IntentContext),
    BodyEndpointRegister {
        body_endpoint_id: u64,
        endpoint_id: String,
        descriptor: EndpointCapabilityDescriptor,
    },
    BodyEndpointUnregister {
        body_endpoint_id: u64,
        route: RouteKey,
    },
    BodyEndpointDisconnected {
        body_endpoint_id: u64,
        routes: Vec<RouteKey>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BodyEgressMessage {
    Act { action: AdmittedAction },
}

#[derive(Debug, Clone, PartialEq)]
pub enum InboundBodyMessage {
    Sense(SenseDelta),
    EnvSnapshot(EndpointSnapshot),
    AdmissionFeedback(AdmissionOutcomeSignal),
    CapabilityCatalogUpdate(CapabilityCatalog),
    CortexLimitsUpdate(ReactionLimits),
    IntentContextUpdate(IntentContext),
    BodyEndpointRegister {
        endpoint_id: String,
        descriptor: EndpointCapabilityDescriptor,
    },
    BodyEndpointUnregister {
        route: RouteKey,
    },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum WireMessage {
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
    BodyEndpointRegister {
        endpoint_id: String,
        descriptor: EndpointCapabilityDescriptor,
    },
    BodyEndpointUnregister {
        endpoint_id: String,
        capability_id: String,
    },
}

pub fn parse_body_ingress_message(line: &str) -> Result<InboundBodyMessage, serde_json::Error> {
    let wire: WireMessage = serde_json::from_str(line)?;
    let message = match wire {
        WireMessage::Sense {
            sense_id,
            source,
            payload,
        } => {
            validate_correlated_sense_payload(&payload)?;
            InboundBodyMessage::Sense(SenseDelta {
                sense_id,
                source,
                payload,
            })
        }
        WireMessage::EnvSnapshot {
            endpoint_key,
            blob,
            truncated,
        } => {
            let blob_bytes = serde_json::to_vec(&blob)
                .map(|bytes| bytes.len())
                .unwrap_or(0);
            InboundBodyMessage::EnvSnapshot(EndpointSnapshot {
                endpoint_key,
                blob,
                truncated,
                blob_bytes,
            })
        }
        WireMessage::AdmissionFeedback { attempt_id, code } => {
            InboundBodyMessage::AdmissionFeedback(AdmissionOutcomeSignal { attempt_id, code })
        }
        WireMessage::CapabilityCatalogUpdate { catalog } => {
            InboundBodyMessage::CapabilityCatalogUpdate(catalog)
        }
        WireMessage::CortexLimitsUpdate { limits } => {
            InboundBodyMessage::CortexLimitsUpdate(limits)
        }
        WireMessage::IntentContextUpdate { context } => {
            InboundBodyMessage::IntentContextUpdate(context)
        }
        WireMessage::BodyEndpointRegister {
            endpoint_id,
            descriptor,
        } => InboundBodyMessage::BodyEndpointRegister {
            endpoint_id,
            descriptor,
        },
        WireMessage::BodyEndpointUnregister {
            endpoint_id,
            capability_id,
        } => InboundBodyMessage::BodyEndpointUnregister {
            route: RouteKey {
                endpoint_id,
                capability_id,
            },
        },
    };
    Ok(message)
}

fn validate_correlated_sense_payload(payload: &serde_json::Value) -> Result<(), serde_json::Error> {
    let Some(object) = payload.as_object() else {
        return Ok(());
    };

    let Some(neural_signal_id) = object.get("neural_signal_id") else {
        return Ok(());
    };

    if neural_signal_id
        .as_str()
        .map(|value| value.trim().is_empty())
        .unwrap_or(true)
    {
        return Err(invalid_correlated_sense_error(
            "neural_signal_id must be a non-empty string",
        ));
    }

    for field in ["capability_instance_id", "endpoint_id", "capability_id"] {
        let valid = object
            .get(field)
            .and_then(|value| value.as_str())
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false);
        if !valid {
            return Err(invalid_correlated_sense_error(&format!(
                "correlated sense missing required field '{}'",
                field
            )));
        }
    }

    Ok(())
}

fn invalid_correlated_sense_error(message: &str) -> serde_json::Error {
    serde_json::Error::io(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        message.to_string(),
    ))
}

pub fn encode_body_egress_message(
    message: &BodyEgressMessage,
) -> Result<String, serde_json::Error> {
    let encoded = serde_json::to_string(message)?;
    Ok(format!("{encoded}\n"))
}

#[cfg(test)]
mod tests {
    use crate::spine::types::{AdmittedAction, CostVector};

    use super::{
        BodyEgressMessage, InboundBodyMessage, encode_body_egress_message,
        parse_body_ingress_message,
    };

    #[test]
    fn accepts_sense_message() {
        let parsed = parse_body_ingress_message(
            r#"{"type":"sense","sense_id":"s1","source":"sensor","payload":{"v":1}}"#,
        )
        .expect("sense message should parse");
        assert!(matches!(parsed, InboundBodyMessage::Sense(_)));
    }

    #[test]
    fn accepts_correlated_sense_message_with_required_echo_fields() {
        let parsed = parse_body_ingress_message(
            r#"{"type":"sense","sense_id":"s2","source":"body","payload":{"kind":"present_message_result","neural_signal_id":"018f94da-9f92-7bc5-bc58-b5f01b0406f5","capability_instance_id":"chat.1","endpoint_id":"macos-app.01","capability_id":"present.message"}}"#,
        )
        .expect("correlated sense should parse");
        assert!(matches!(parsed, InboundBodyMessage::Sense(_)));
    }

    #[test]
    fn rejects_correlated_sense_message_missing_echo_fields() {
        assert!(
            parse_body_ingress_message(
                r#"{"type":"sense","sense_id":"s3","source":"body","payload":{"kind":"present_message_result","neural_signal_id":"018f94da-9f92-7bc5-bc58-b5f01b0406f5","endpoint_id":"macos-app.01","capability_id":"present.message"}}"#
            )
            .is_err()
        );
    }

    #[test]
    fn act_encoding_contains_neural_signal_and_target_fields() {
        let encoded = encode_body_egress_message(&BodyEgressMessage::Act {
            action: AdmittedAction {
                neural_signal_id: "018f94da-9f92-7bc5-bc58-b5f01b0406f5".to_string(),
                capability_instance_id: "chat.instance".to_string(),
                source_attempt_id: "att:1".to_string(),
                reserve_entry_id: "res:1".to_string(),
                cost_attribution_id: "cat:1".to_string(),
                endpoint_id: "macos-app.01".to_string(),
                capability_id: "present.message".to_string(),
                normalized_payload: serde_json::json!({"ok": true}),
                reserved_cost: CostVector::default(),
                degraded: false,
                degradation_profile_id: None,
                admission_cycle: 1,
                metadata: Default::default(),
            },
        })
        .expect("act should encode");

        let json: serde_json::Value =
            serde_json::from_str(encoded.trim()).expect("encoded act should be valid JSON");
        assert_eq!(json["type"], serde_json::json!("act"));
        assert_eq!(
            json["action"]["neural_signal_id"],
            serde_json::json!("018f94da-9f92-7bc5-bc58-b5f01b0406f5")
        );
        assert_eq!(
            json["action"]["capability_instance_id"],
            serde_json::json!("chat.instance")
        );
        assert_eq!(
            json["action"]["endpoint_id"],
            serde_json::json!("macos-app.01")
        );
        assert_eq!(
            json["action"]["capability_id"],
            serde_json::json!("present.message")
        );
    }

    #[test]
    fn accepts_body_endpoint_register_message() {
        let parsed = parse_body_ingress_message(
            r#"{
                "type":"body_endpoint_register",
                "endpoint_id":"ep:test",
                "descriptor":{
                    "route":{"endpoint_id":"macos-app.01","capability_id":"present.message"},
                    "payload_schema":{"type":"object"},
                    "max_payload_bytes":1024,
                    "default_cost":{"survival_micro":1,"time_ms":1,"io_units":1,"token_units":1},
                    "metadata":{}
                }
            }"#,
        )
        .expect("body endpoint register should parse");
        assert!(matches!(
            parsed,
            InboundBodyMessage::BodyEndpointRegister { .. }
        ));
    }

    #[test]
    fn rejects_legacy_endpoint_register_message() {
        assert!(
            parse_body_ingress_message(r#"{"type":"endpoint_register","endpoint_id":"ep:test"}"#)
                .is_err()
        );
    }

    #[test]
    fn rejects_legacy_endpoint_result_message() {
        assert!(
            parse_body_ingress_message(
                r#"{"type":"endpoint_result","request_id":"req:1","outcome":{"type":"applied","actual_cost_micro":1,"reference_id":"ref:1"}}"#
            )
            .is_err()
        );
    }

    #[test]
    fn rejects_legacy_endpoint_unregister_message() {
        assert!(
            parse_body_ingress_message(
                r#"{"type":"endpoint_unregister","endpoint_id":"macos-app.01","capability_id":"present.message"}"#,
            )
            .is_err()
        );
    }

    #[test]
    fn rejects_exit_message() {
        assert!(parse_body_ingress_message(r#"{"type":"exit"}"#).is_err());
    }

    #[test]
    fn rejects_plain_string_message() {
        assert!(parse_body_ingress_message(r#""exit""#).is_err());
    }

    #[test]
    fn rejects_unknown_message_type() {
        assert!(parse_body_ingress_message(r#"{"type":"ping"}"#).is_err());
    }

    #[test]
    fn rejects_unknown_fields() {
        assert!(parse_body_ingress_message(r#"{"type":"sense","extra":"value"}"#).is_err());
    }
}
