use jsonschema::JSONSchema;
use sha2::{Digest, Sha256};

use crate::{
    admission::types::{IntentAttempt, RequestedResources},
    cortex::{
        error::{CortexError, internal_error},
        ports::{AttemptClampPort, AttemptClampRequest},
        types::{
            AttemptDraft, ClampResult, ClampViolation, ClampViolationCode, ReactionId, SenseId,
        },
    },
};

#[derive(Debug, Default, Clone, Copy)]
pub struct DeterministicAttemptClamp;

impl AttemptClampPort for DeterministicAttemptClamp {
    fn clamp(&self, req: AttemptClampRequest) -> Result<ClampResult, CortexError> {
        let mut drafts = req.drafts.clone();
        drafts.sort_by(|lhs, rhs| {
            lhs.endpoint_id
                .cmp(&rhs.endpoint_id)
                .then_with(|| lhs.capability_id.cmp(&rhs.capability_id))
                .then_with(|| lhs.capability_instance_id.cmp(&rhs.capability_instance_id))
                .then_with(|| {
                    canonicalize_json(&lhs.payload_draft)
                        .to_string()
                        .cmp(&canonicalize_json(&rhs.payload_draft).to_string())
                })
                .then_with(|| lhs.intent_span.cmp(&rhs.intent_span))
        });

        let known_sense_ids: std::collections::BTreeSet<String> = req
            .sense_window
            .iter()
            .map(|s| s.sense_id.clone())
            .collect();

        let mut attempts = Vec::new();
        let mut violations = Vec::new();

        for draft in drafts {
            let Some(attempt) = self.validate_and_build_attempt(
                req.reaction_id,
                attempts.len() as u16,
                &draft,
                &req,
                &known_sense_ids,
                &mut violations,
            )?
            else {
                continue;
            };

            attempts.push(attempt);
        }

        attempts.sort_by(|lhs, rhs| lhs.attempt_id.cmp(&rhs.attempt_id));
        attempts.truncate(req.limits.max_attempts);

        let based_on = stable_union_based_on(&attempts);
        let attention_tags = stable_attention_tags(&req.drafts);

        Ok(ClampResult {
            attempts,
            based_on,
            attention_tags,
            violations,
            original_drafts: req.drafts,
        })
    }
}

impl DeterministicAttemptClamp {
    #[allow(clippy::too_many_arguments)]
    fn validate_and_build_attempt(
        &self,
        reaction_id: ReactionId,
        planner_slot: u16,
        draft: &AttemptDraft,
        req: &AttemptClampRequest,
        known_sense_ids: &std::collections::BTreeSet<String>,
        violations: &mut Vec<ClampViolation>,
    ) -> Result<Option<IntentAttempt>, CortexError> {
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

        let Some(profile) = req.capability_catalog.resolve(&draft.endpoint_id) else {
            violations.push(ClampViolation {
                code: ClampViolationCode::UnknownEndpointId,
                message: format!("unknown endpoint_id '{}'", draft.endpoint_id),
            });
            return Ok(None);
        };

        if profile.allowed_capability_ids.is_empty()
            || !profile
                .allowed_capability_ids
                .iter()
                .any(|id| id == &draft.capability_id)
        {
            violations.push(ClampViolation {
                code: ClampViolationCode::UnsupportedCapabilityId,
                message: format!(
                    "unsupported capability_id '{}' for endpoint_id '{}'",
                    draft.capability_id, draft.endpoint_id
                ),
            });
            return Ok(None);
        }

        let payload = canonicalize_json(&draft.payload_draft);
        let payload_bytes = serde_json::to_vec(&payload)
            .map_err(|err| internal_error(format!("payload serialization error: {err}")))?
            .len();
        let payload_cap = req.limits.max_payload_bytes.min(profile.max_payload_bytes);
        if payload_bytes > payload_cap {
            violations.push(ClampViolation {
                code: ClampViolationCode::PayloadTooLarge,
                message: format!("payload size {} exceeds {}", payload_bytes, payload_cap),
            });
            return Ok(None);
        }

        let compiled = match JSONSchema::compile(&profile.payload_schema) {
            Ok(compiled) => compiled,
            Err(err) => {
                violations.push(ClampViolation {
                    code: ClampViolationCode::PayloadSchemaViolation,
                    message: format!("invalid schema for '{}': {}", profile.endpoint_id, err),
                });
                return Ok(None);
            }
        };

        if compiled.validate(&payload).is_err() {
            violations.push(ClampViolation {
                code: ClampViolationCode::PayloadSchemaViolation,
                message: format!(
                    "payload does not conform to schema for '{}'",
                    profile.endpoint_id
                ),
            });
            return Ok(None);
        }

        let resources = clamp_resources(&draft.requested_resources);
        let based_on = stable_dedupe_sense_ids(&draft.based_on);
        let cost_attribution_id = derive_cost_attribution_id(
            reaction_id,
            &draft.endpoint_id,
            &draft.capability_id,
            &based_on,
            planner_slot,
        );
        let attempt_id = derive_attempt_id(
            reaction_id,
            &based_on,
            &draft.endpoint_id,
            &draft.capability_id,
            &payload,
            &resources,
            &cost_attribution_id,
        );

        let commitment_id = draft
            .commitment_hint
            .clone()
            .unwrap_or_else(|| format!("com:reaction:{}", reaction_id));
        let goal_id = draft
            .goal_hint
            .clone()
            .unwrap_or_else(|| format!("goal:{}", short_hash(&draft.intent_span)));

        Ok(Some(IntentAttempt {
            attempt_id,
            cycle_id: reaction_id,
            commitment_id,
            goal_id,
            planner_slot,
            based_on,
            endpoint_id: draft.endpoint_id.clone(),
            capability_id: draft.capability_id.clone(),
            capability_instance_id: draft.capability_instance_id.clone(),
            normalized_payload: payload,
            requested_resources: resources,
            cost_attribution_id,
        }))
    }
}

pub fn derive_attempt_id(
    reaction_id: u64,
    based_on: &[SenseId],
    endpoint_id: &str,
    capability_id: &str,
    normalized_payload: &serde_json::Value,
    requested_resources: &RequestedResources,
    cost_attribution_id: &str,
) -> String {
    let canonical = canonicalize_json(&serde_json::json!({
        "reaction_id": reaction_id,
        "based_on": based_on,
        "endpoint_id": endpoint_id,
        "capability_id": capability_id,
        "normalized_payload": normalized_payload,
        "requested_resources": {
            "survival_micro": requested_resources.survival_micro,
            "time_ms": requested_resources.time_ms,
            "io_units": requested_resources.io_units,
            "token_units": requested_resources.token_units,
        },
        "cost_attribution_id": cost_attribution_id,
    }));

    let mut hasher = Sha256::new();
    hasher.update(canonical.to_string().as_bytes());
    let digest = hasher.finalize();
    let hex = format!("{:x}", digest);
    format!("att:{}", &hex[..24])
}

pub fn derive_cost_attribution_id(
    reaction_id: u64,
    endpoint_id: &str,
    capability_id: &str,
    based_on: &[SenseId],
    planner_slot: u16,
) -> String {
    let canonical = canonicalize_json(&serde_json::json!({
        "reaction_id": reaction_id,
        "endpoint_id": endpoint_id,
        "capability_id": capability_id,
        "based_on": based_on,
        "planner_slot": planner_slot,
    }));

    let mut hasher = Sha256::new();
    hasher.update(canonical.to_string().as_bytes());
    let digest = hasher.finalize();
    let hex = format!("{:x}", digest);
    format!("cat:{}", &hex[..24])
}

fn short_hash(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let digest = hasher.finalize();
    let hex = format!("{:x}", digest);
    hex[..12].to_string()
}

fn clamp_resources(resources: &RequestedResources) -> RequestedResources {
    RequestedResources {
        survival_micro: resources.survival_micro.max(0),
        time_ms: resources.time_ms,
        io_units: resources.io_units,
        token_units: resources.token_units,
    }
}

fn stable_dedupe_sense_ids(ids: &[SenseId]) -> Vec<SenseId> {
    let mut seen = std::collections::BTreeSet::new();
    for id in ids {
        seen.insert(id.clone());
    }
    seen.into_iter().collect()
}

fn stable_union_based_on(attempts: &[IntentAttempt]) -> Vec<SenseId> {
    let mut union = std::collections::BTreeSet::new();
    for attempt in attempts {
        for sense_id in &attempt.based_on {
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
            let mut sorted = serde_json::Map::new();
            for key in keys {
                if let Some(item) = map.get(&key) {
                    sorted.insert(key, canonicalize_json(item));
                }
            }
            serde_json::Value::Object(sorted)
        }
        serde_json::Value::Array(items) => {
            serde_json::Value::Array(items.iter().map(canonicalize_json).collect())
        }
        primitive => primitive.clone(),
    }
}
