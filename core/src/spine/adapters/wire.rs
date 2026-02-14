use serde::{Deserialize, Serialize};

use crate::{
    runtime_types::{CapabilityDropPatch, CapabilityPatch, SenseDatum},
    spine::types::{ActDispatchRequest, EndpointCapabilityDescriptor, RouteKey},
};

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BodyEgressMessage {
    Act { request: ActDispatchRequest },
}

#[derive(Debug, Clone, PartialEq)]
pub enum InboundBodyMessage {
    Sense(SenseDatum),
    NewCapabilities(CapabilityPatch),
    DropCapabilities(CapabilityDropPatch),
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
    NewCapabilities {
        entries: Vec<EndpointCapabilityDescriptor>,
    },
    DropCapabilities {
        routes: Vec<RouteKey>,
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
            InboundBodyMessage::Sense(SenseDatum {
                sense_id,
                source,
                payload,
            })
        }
        WireMessage::NewCapabilities { entries } => {
            InboundBodyMessage::NewCapabilities(CapabilityPatch { entries })
        }
        WireMessage::DropCapabilities { routes } => {
            InboundBodyMessage::DropCapabilities(CapabilityDropPatch { routes })
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

    let Some(act_id) = object.get("act_id") else {
        return Ok(());
    };

    if act_id
        .as_str()
        .map(|value| value.trim().is_empty())
        .unwrap_or(true)
    {
        return Err(invalid_correlated_sense_error(
            "act_id must be a non-empty string",
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
    use crate::{
        runtime_types::{Act, RequestedResources},
        spine::{
            types::ActDispatchRequest,
            adapters::wire::{
                BodyEgressMessage, InboundBodyMessage, encode_body_egress_message,
                parse_body_ingress_message,
            },
        },
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
            r#"{"type":"sense","sense_id":"s2","source":"body","payload":{"kind":"present_message_result","act_id":"act:1","capability_instance_id":"chat.1","endpoint_id":"macos-app.01","capability_id":"present.message"}}"#,
        )
        .expect("correlated sense should parse");
        assert!(matches!(parsed, InboundBodyMessage::Sense(_)));
    }

    #[test]
    fn rejects_correlated_sense_message_missing_echo_fields() {
        assert!(
            parse_body_ingress_message(
                r#"{"type":"sense","sense_id":"s3","source":"body","payload":{"kind":"present_message_result","act_id":"act:1","endpoint_id":"macos-app.01","capability_id":"present.message"}}"#
            )
            .is_err()
        );
    }

    #[test]
    fn rejects_removed_admission_feedback_message() {
        assert!(
            parse_body_ingress_message(
                r#"{"type":"admission_feedback","attempt_id":"att:1","code":"admitted"}"#
            )
            .is_err()
        );
    }

    #[test]
    fn act_encoding_contains_act_and_target_fields() {
        let encoded = encode_body_egress_message(&BodyEgressMessage::Act {
            request: ActDispatchRequest {
                cycle_id: 1,
                seq_no: 1,
                act: Act {
                    act_id: "act:1".to_string(),
                    based_on: vec!["s1".to_string()],
                    endpoint_id: "macos-app.01".to_string(),
                    capability_id: "present.message".to_string(),
                    capability_instance_id: "chat.instance".to_string(),
                    normalized_payload: serde_json::json!({"ok": true}),
                    requested_resources: RequestedResources::default(),
                },
                reserve_entry_id: "res:1".to_string(),
                cost_attribution_id: "cat:1".to_string(),
            },
        })
        .expect("act should encode");

        assert!(encoded.contains("\"type\":\"act\""));
        assert!(encoded.contains("\"act_id\":\"act:1\""));
        assert!(encoded.contains("\"capability_id\":\"present.message\""));
    }
}
