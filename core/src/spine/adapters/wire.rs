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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BodyEndpointResultOutcome {
    Applied {
        actual_cost_micro: i64,
        reference_id: String,
    },
    Rejected {
        reason_code: String,
        reference_id: String,
    },
    Deferred {
        reason_code: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BodyEndpointResultMessage {
    pub request_id: String,
    pub outcome: BodyEndpointResultOutcome,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BodyEgressMessage {
    BodyEndpointInvoke {
        request_id: String,
        action: AdmittedAction,
    },
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
    BodyEndpointResult(BodyEndpointResultMessage),
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
        affordance_key: String,
        capability_handle: String,
    },
    BodyEndpointResult {
        request_id: String,
        outcome: BodyEndpointResultOutcome,
    },
}

pub fn parse_body_ingress_message(line: &str) -> Result<InboundBodyMessage, serde_json::Error> {
    let wire: WireMessage = serde_json::from_str(line)?;
    let message = match wire {
        WireMessage::Sense {
            sense_id,
            source,
            payload,
        } => InboundBodyMessage::Sense(SenseDelta {
            sense_id,
            source,
            payload,
        }),
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
            affordance_key,
            capability_handle,
        } => InboundBodyMessage::BodyEndpointUnregister {
            route: RouteKey {
                affordance_key,
                capability_handle,
            },
        },
        WireMessage::BodyEndpointResult {
            request_id,
            outcome,
        } => InboundBodyMessage::BodyEndpointResult(BodyEndpointResultMessage {
            request_id,
            outcome,
        }),
    };
    Ok(message)
}

pub fn encode_body_egress_message(
    message: &BodyEgressMessage,
) -> Result<String, serde_json::Error> {
    let encoded = serde_json::to_string(message)?;
    Ok(format!("{encoded}\n"))
}

#[cfg(test)]
mod tests {
    use super::{InboundBodyMessage, parse_body_ingress_message};

    #[test]
    fn accepts_sense_message() {
        let parsed = parse_body_ingress_message(
            r#"{"type":"sense","sense_id":"s1","source":"sensor","payload":{"v":1}}"#,
        )
        .expect("sense message should parse");
        assert!(matches!(parsed, InboundBodyMessage::Sense(_)));
    }

    #[test]
    fn accepts_body_endpoint_register_message() {
        let parsed = parse_body_ingress_message(
            r#"{
                "type":"body_endpoint_register",
                "endpoint_id":"ep:test",
                "descriptor":{
                    "route":{"affordance_key":"chat.reply.emit","capability_handle":"cap.apple"},
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
    fn accepts_body_endpoint_result_message() {
        let parsed = parse_body_ingress_message(
            r#"{
                "type":"body_endpoint_result",
                "request_id":"req:1",
                "outcome":{"type":"applied","actual_cost_micro":1,"reference_id":"ref:1"}
            }"#,
        )
        .expect("body endpoint result should parse");
        assert!(matches!(parsed, InboundBodyMessage::BodyEndpointResult(_)));
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
                r#"{"type":"endpoint_unregister","affordance_key":"chat.reply.emit","capability_handle":"cap.apple"}"#,
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
