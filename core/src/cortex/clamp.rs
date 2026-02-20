use jsonschema::JSONSchema;
use uuid::Uuid;

use crate::{
    cortex::{
        error::{CortexError, internal_error},
        types::{AttemptDraft, ClampResult, ClampViolation, ClampViolationCode},
    },
    types::{Act, NeuralSignalDescriptorCatalog, NeuralSignalType, SenseId},
};

#[derive(Debug, Clone)]
pub struct AttemptClampRequest {
    pub cycle_id: u64,
    pub drafts: Vec<AttemptDraft>,
    pub neural_signal_descriptor_catalog: NeuralSignalDescriptorCatalog,
    pub known_sense_ids: Vec<SenseId>,
    pub limits: crate::cortex::ReactionLimits,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct DeterministicAttemptClamp;

impl DeterministicAttemptClamp {
    pub fn clamp(&self, req: AttemptClampRequest) -> Result<ClampResult, CortexError> {
        let mut drafts = req.drafts.clone();
        drafts.sort_by(|lhs, rhs| {
            lhs.endpoint_id
                .cmp(&rhs.endpoint_id)
                .then_with(|| {
                    lhs.neural_signal_descriptor_id
                        .cmp(&rhs.neural_signal_descriptor_id)
                })
                .then_with(|| {
                    canonicalize_json(&lhs.payload_draft)
                        .to_string()
                        .cmp(&canonicalize_json(&rhs.payload_draft).to_string())
                })
                .then_with(|| lhs.intent_span.cmp(&rhs.intent_span))
        });

        let known_sense_ids: std::collections::BTreeSet<String> =
            req.known_sense_ids.iter().cloned().collect();

        let mut acts = Vec::new();
        let mut violations = Vec::new();

        for draft in drafts {
            let Some(act) = self.validate_and_build_act(
                req.cycle_id,
                &draft,
                &req,
                &known_sense_ids,
                &mut violations,
            )?
            else {
                continue;
            };

            acts.push(act);
        }

        acts.sort_by(|lhs, rhs| lhs.act_id.cmp(&rhs.act_id));
        acts.truncate(req.limits.max_attempts);

        let based_on = stable_union_based_on_from_drafts(&req.drafts);
        let attention_tags = stable_attention_tags(&req.drafts);

        Ok(ClampResult {
            acts,
            based_on,
            attention_tags,
            violations,
            original_drafts: req.drafts,
        })
    }

    fn validate_and_build_act(
        &self,
        cycle_id: u64,
        draft: &AttemptDraft,
        req: &AttemptClampRequest,
        known_sense_ids: &std::collections::BTreeSet<String>,
        violations: &mut Vec<ClampViolation>,
    ) -> Result<Option<Act>, CortexError> {
        if draft.intent_span.trim().is_empty() {
            violations.push(ClampViolation {
                code: ClampViolationCode::MissingIntentSpan,
                message: "intent_span is required".to_string(),
            });
            return Ok(None);
        }

        if draft.based_on.is_empty() {
            violations.push(ClampViolation {
                code: ClampViolationCode::MissingBasedOn,
                message: "based_on cannot be empty".to_string(),
            });
            return Ok(None);
        }

        if draft
            .based_on
            .iter()
            .any(|sense_id| !known_sense_ids.contains(sense_id))
        {
            violations.push(ClampViolation {
                code: ClampViolationCode::UnknownSenseId,
                message: "based_on references unknown sense_id".to_string(),
            });
            return Ok(None);
        }

        let endpoint_known = req
            .neural_signal_descriptor_catalog
            .entries
            .iter()
            .any(|entry| {
                entry.r#type == NeuralSignalType::Act && entry.endpoint_id == draft.endpoint_id
            });
        if !endpoint_known {
            violations.push(ClampViolation {
                code: ClampViolationCode::UnknownEndpointId,
                message: format!("unknown endpoint_id '{}'", draft.endpoint_id),
            });
            return Ok(None);
        }

        let Some(descriptor) = req
            .neural_signal_descriptor_catalog
            .entries
            .iter()
            .find(|entry| {
                entry.r#type == NeuralSignalType::Act
                    && entry.endpoint_id == draft.endpoint_id
                    && entry.neural_signal_descriptor_id == draft.neural_signal_descriptor_id
            })
        else {
            violations.push(ClampViolation {
                code: ClampViolationCode::UnsupportedNeuralSignalDescriptorId,
                message: format!(
                    "unsupported neural_signal_descriptor_id '{}' for endpoint_id '{}'",
                    draft.neural_signal_descriptor_id, draft.endpoint_id
                ),
            });
            return Ok(None);
        };

        let payload = canonicalize_json(&draft.payload_draft);
        let payload_bytes = serde_json::to_vec(&payload)
            .map_err(|err| internal_error(format!("payload serialization error: {err}")))?
            .len();
        let payload_cap = req.limits.max_payload_bytes;
        if payload_bytes > payload_cap {
            violations.push(ClampViolation {
                code: ClampViolationCode::PayloadTooLarge,
                message: format!("payload size {} exceeds {}", payload_bytes, payload_cap),
            });
            return Ok(None);
        }

        let compiled = match JSONSchema::compile(&descriptor.payload_schema) {
            Ok(compiled) => compiled,
            Err(err) => {
                violations.push(ClampViolation {
                    code: ClampViolationCode::PayloadSchemaViolation,
                    message: format!("invalid schema for '{}': {}", descriptor.endpoint_id, err),
                });
                return Ok(None);
            }
        };

        if compiled.validate(&payload).is_err() {
            violations.push(ClampViolation {
                code: ClampViolationCode::PayloadSchemaViolation,
                message: format!(
                    "payload does not conform to schema for '{}'",
                    descriptor.endpoint_id
                ),
            });
            return Ok(None);
        }

        let based_on = stable_dedupe_sense_ids(&draft.based_on);
        let act_id = derive_act_id(
            cycle_id,
            &based_on,
            &draft.endpoint_id,
            &draft.neural_signal_descriptor_id,
            &payload,
        );

        Ok(Some(Act {
            act_id,
            endpoint_id: draft.endpoint_id.clone(),
            neural_signal_descriptor_id: draft.neural_signal_descriptor_id.clone(),
            payload,
        }))
    }
}

pub fn derive_act_id(
    cycle_id: u64,
    based_on: &[SenseId],
    endpoint_id: &str,
    neural_signal_descriptor_id: &str,
    payload: &serde_json::Value,
) -> String {
    let _ = (
        cycle_id,
        based_on,
        endpoint_id,
        neural_signal_descriptor_id,
        payload,
    );
    Uuid::now_v7().to_string()
}

fn stable_dedupe_sense_ids(ids: &[SenseId]) -> Vec<SenseId> {
    let mut seen = std::collections::BTreeSet::new();
    for id in ids {
        seen.insert(id.clone());
    }
    seen.into_iter().collect()
}

fn stable_union_based_on_from_drafts(drafts: &[AttemptDraft]) -> Vec<SenseId> {
    let mut union = std::collections::BTreeSet::new();
    for draft in drafts {
        for sense_id in &draft.based_on {
            union.insert(sense_id.clone());
        }
    }
    union.into_iter().collect()
}

fn stable_attention_tags(drafts: &[AttemptDraft]) -> Vec<String> {
    let mut tags = std::collections::BTreeSet::new();
    for draft in drafts {
        for tag in &draft.attention_tags {
            tags.insert(tag.clone());
        }
    }
    tags.into_iter().collect()
}

fn canonicalize_json(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let mut keys = map.keys().cloned().collect::<Vec<_>>();
            keys.sort();
            let mut normalized = serde_json::Map::new();
            for key in keys {
                if let Some(inner) = map.get(&key) {
                    normalized.insert(key, canonicalize_json(inner));
                }
            }
            serde_json::Value::Object(normalized)
        }
        serde_json::Value::Array(items) => {
            serde_json::Value::Array(items.iter().map(canonicalize_json).collect())
        }
        _ => value.clone(),
    }
}
