use std::{cmp::Ordering, collections::BTreeMap};

use sha2::{Digest, Sha256};

use crate::{
    admission::{
        affordance::{AffordanceProfile, AffordanceRegistry, DegradationProfile},
        types::{
            AdmissionDisposition, AdmissionReport, AdmissionReportItem, AdmissionWhy,
            AffordabilitySnapshot, AttributionRecord, CostAttributionId, IntentAttempt,
            ReservationDelta,
        },
    },
    continuity::error::{ContinuityError, arithmetic_error, invalid_request},
    ledger::{
        SurvivalLedger,
        types::{CycleId, PolicyVersionTuple},
    },
    spine::types::{AdmittedAction, AdmittedActionBatch, CostVector},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DegradationPreference {
    LeastCapabilityLossFirst,
    CheapestFirst,
}

#[derive(Debug, Clone)]
pub struct AdmissionResolverConfig {
    pub reservation_ttl_cycles: u64,
    pub max_degradation_variants: usize,
    pub max_degradation_depth: u8,
    pub degradation_preference: DegradationPreference,
}

impl Default for AdmissionResolverConfig {
    fn default() -> Self {
        Self {
            reservation_ttl_cycles: 8,
            max_degradation_variants: 8,
            max_degradation_depth: 2,
            degradation_preference: DegradationPreference::LeastCapabilityLossFirst,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CostAdmissionPolicy {
    pub max_time_ms_per_action: u64,
    pub max_io_units_per_action: u64,
    pub max_token_units_per_action: u64,
    pub reserve_ratio_milli: u16,
    pub time_to_survival_micro: u64,
    pub io_to_survival_micro: u64,
    pub token_to_survival_micro: u64,
}

impl Default for CostAdmissionPolicy {
    fn default() -> Self {
        Self {
            max_time_ms_per_action: 5_000,
            max_io_units_per_action: 64,
            max_token_units_per_action: 16_384,
            reserve_ratio_milli: 1_000,
            time_to_survival_micro: 1,
            io_to_survival_micro: 40,
            token_to_survival_micro: 1,
        }
    }
}

impl CostAdmissionPolicy {
    fn reserve_amount_micro(&self, required_survival_micro: i64) -> Result<i64, ContinuityError> {
        let scaled = (required_survival_micro as i128)
            .checked_mul(self.reserve_ratio_milli as i128)
            .ok_or_else(|| arithmetic_error("reserve amount overflow"))?;
        let amount = scaled
            .checked_div(1_000)
            .ok_or_else(|| arithmetic_error("reserve amount division overflow"))?;

        i64::try_from(amount).map_err(|_| arithmetic_error("reserve amount conversion overflow"))
    }

    fn estimate_cost(
        &self,
        profile: &AffordanceProfile,
        attempt: &IntentAttempt,
        degradation: Option<&DegradationProfile>,
    ) -> Result<CostVector, ContinuityError> {
        let multiplier = degradation
            .map(|candidate| candidate.cost_multiplier_milli)
            .unwrap_or(1_000) as i128;

        let required_time_ms = profile
            .base_cost
            .time_ms
            .max(attempt.requested_resources.time_ms);
        let required_io_units = profile
            .base_cost
            .io_units
            .max(attempt.requested_resources.io_units);
        let required_token_units = profile
            .base_cost
            .token_units
            .max(attempt.requested_resources.token_units);

        let base_survival_micro = profile
            .base_cost
            .survival_micro
            .max(0)
            .checked_add(attempt.requested_resources.survival_micro.max(0))
            .ok_or_else(|| arithmetic_error("survival estimate overflow"))?;

        let conversion = (required_time_ms as i128)
            .checked_mul(self.time_to_survival_micro as i128)
            .and_then(|v| {
                v.checked_add((required_io_units as i128) * (self.io_to_survival_micro as i128))
            })
            .and_then(|v| {
                v.checked_add(
                    (required_token_units as i128) * (self.token_to_survival_micro as i128),
                )
            })
            .ok_or_else(|| arithmetic_error("survival conversion overflow"))?;

        let unscaled = (base_survival_micro as i128)
            .checked_add(conversion)
            .ok_or_else(|| arithmetic_error("survival estimate overflow"))?;

        let scaled = unscaled
            .checked_mul(multiplier)
            .and_then(|v| v.checked_div(1_000))
            .ok_or_else(|| arithmetic_error("scaled survival estimate overflow"))?;

        let survival_micro = i64::try_from(scaled)
            .map_err(|_| arithmetic_error("survival estimate conversion overflow"))?;

        Ok(CostVector {
            survival_micro,
            time_ms: required_time_ms,
            io_units: required_io_units,
            token_units: required_token_units,
        })
    }

    fn affordability_snapshot(
        &self,
        available_survival_micro: i64,
        estimated: &CostVector,
    ) -> AffordabilitySnapshot {
        AffordabilitySnapshot {
            available_survival_micro,
            required_survival_micro: estimated.survival_micro,
            required_time_ms: estimated.time_ms,
            required_io_units: estimated.io_units,
            required_token_units: estimated.token_units,
            max_time_ms: self.max_time_ms_per_action,
            max_io_units: self.max_io_units_per_action,
            max_token_units: self.max_token_units_per_action,
        }
    }
}

pub struct AdmissionContext<'a> {
    pub ledger: &'a mut SurvivalLedger,
    pub attribution_index: &'a mut BTreeMap<CostAttributionId, Vec<AttributionRecord>>,
    pub policy_versions: PolicyVersionTuple,
}

pub struct AdmissionResolver {
    registry: AffordanceRegistry,
    cost_policy: CostAdmissionPolicy,
    config: AdmissionResolverConfig,
}

impl AdmissionResolver {
    pub fn new(
        registry: AffordanceRegistry,
        cost_policy: CostAdmissionPolicy,
        config: AdmissionResolverConfig,
    ) -> Self {
        Self {
            registry,
            cost_policy,
            config,
        }
    }

    pub fn admit_attempts(
        &self,
        context: &mut AdmissionContext<'_>,
        cycle_id: CycleId,
        attempts: Vec<IntentAttempt>,
    ) -> Result<(AdmissionReport, AdmittedActionBatch), ContinuityError> {
        let mut report = AdmissionReport {
            cycle_id,
            outcomes: Vec::new(),
            total_reserved_survival_micro: 0,
        };

        let mut admitted_actions = Vec::new();

        let mut sorted_attempts = attempts;
        sorted_attempts.sort_by(|lhs, rhs| lhs.attempt_id.cmp(&rhs.attempt_id));

        for attempt in sorted_attempts {
            let Some(profile) = self.registry.resolve(&attempt.affordance_key) else {
                report.outcomes.push(AdmissionReportItem {
                    attempt_id: attempt.attempt_id,
                    disposition: AdmissionDisposition::DeniedHard {
                        code: "unknown_affordance".to_string(),
                    },
                    why: Some(AdmissionWhy::HardRule {
                        code: "unknown_affordance".to_string(),
                    }),
                    ledger_delta: None,
                    admitted_action_id: None,
                    degradation_profile_id: None,
                });
                continue;
            };

            if attempt.capability_handle != profile.capability_handle {
                report.outcomes.push(AdmissionReportItem {
                    attempt_id: attempt.attempt_id,
                    disposition: AdmissionDisposition::DeniedHard {
                        code: "unsupported_capability_handle".to_string(),
                    },
                    why: Some(AdmissionWhy::HardRule {
                        code: "unsupported_capability_handle".to_string(),
                    }),
                    ledger_delta: None,
                    admitted_action_id: None,
                    degradation_profile_id: None,
                });
                continue;
            }

            let payload_len = serde_json::to_vec(&attempt.normalized_payload)
                .map_err(|err| invalid_request(format!("payload serialization error: {err}")))?
                .len();
            if payload_len > profile.max_payload_bytes {
                report.outcomes.push(AdmissionReportItem {
                    attempt_id: attempt.attempt_id,
                    disposition: AdmissionDisposition::DeniedHard {
                        code: "payload_too_large".to_string(),
                    },
                    why: Some(AdmissionWhy::HardRule {
                        code: "payload_too_large".to_string(),
                    }),
                    ledger_delta: None,
                    admitted_action_id: None,
                    degradation_profile_id: None,
                });
                continue;
            }

            let estimated_cost = self.cost_policy.estimate_cost(profile, &attempt, None)?;
            let snapshot = self
                .cost_policy
                .affordability_snapshot(context.ledger.available_survival_micro(), &estimated_cost);

            if snapshot.within_runtime_limits() && snapshot.survival_affordable() {
                let admission = self.materialize_admitted_action(
                    context,
                    cycle_id,
                    &attempt,
                    profile,
                    estimated_cost,
                    false,
                    None,
                )?;
                report.total_reserved_survival_micro = report
                    .total_reserved_survival_micro
                    .checked_add(admission.1)
                    .ok_or_else(|| arithmetic_error("report reserved_survival overflow"))?;
                report.outcomes.push(admission.2);
                admitted_actions.push(admission.0);
                continue;
            }

            let degraded = self.find_degraded_candidate(context, profile, &attempt)?;
            if let Some((degradation_profile, degraded_cost)) = degraded {
                let admission = self.materialize_admitted_action(
                    context,
                    cycle_id,
                    &attempt,
                    profile,
                    degraded_cost,
                    true,
                    Some(degradation_profile),
                )?;
                report.total_reserved_survival_micro = report
                    .total_reserved_survival_micro
                    .checked_add(admission.1)
                    .ok_or_else(|| arithmetic_error("report reserved_survival overflow"))?;
                report.outcomes.push(admission.2);
                admitted_actions.push(admission.0);
                continue;
            }

            let economic_code = Self::economic_denial_code(&snapshot);
            report.outcomes.push(AdmissionReportItem {
                attempt_id: attempt.attempt_id,
                disposition: AdmissionDisposition::DeniedEconomic {
                    code: economic_code.clone(),
                },
                why: Some(AdmissionWhy::Economic {
                    code: economic_code,
                    available_survival_micro: snapshot.available_survival_micro,
                    required_survival_micro: snapshot.required_survival_micro,
                }),
                ledger_delta: None,
                admitted_action_id: None,
                degradation_profile_id: None,
            });
        }

        Ok((
            report,
            AdmittedActionBatch {
                cycle_id,
                actions: admitted_actions,
            },
        ))
    }

    fn materialize_admitted_action(
        &self,
        context: &mut AdmissionContext<'_>,
        cycle_id: CycleId,
        attempt: &IntentAttempt,
        profile: &AffordanceProfile,
        estimated_cost: CostVector,
        degraded: bool,
        degradation_profile: Option<&DegradationProfile>,
    ) -> Result<(AdmittedAction, i64, AdmissionReportItem), ContinuityError> {
        let reserve_survival_micro = self
            .cost_policy
            .reserve_amount_micro(estimated_cost.survival_micro.max(0))?;

        let reserve_entry_id = context.ledger.reserve(
            cycle_id,
            reserve_survival_micro,
            self.config.reservation_ttl_cycles,
            attempt.cost_attribution_id.clone(),
            format!("reserve:{}", attempt.attempt_id),
            context.policy_versions.clone(),
        )?;

        let action_id = derive_action_id(cycle_id, &attempt.attempt_id, &reserve_entry_id);
        context
            .ledger
            .attach_action_id(&reserve_entry_id, action_id.clone())?;

        context
            .attribution_index
            .entry(attempt.cost_attribution_id.clone())
            .or_default()
            .push(AttributionRecord {
                action_id: action_id.clone(),
                reserve_entry_id: reserve_entry_id.clone(),
                cycle_id,
            });

        let capability_handle = degradation_profile
            .and_then(|candidate| candidate.capability_handle_override.clone())
            .unwrap_or_else(|| profile.capability_handle.clone());

        let action = AdmittedAction {
            action_id: action_id.clone(),
            source_attempt_id: attempt.attempt_id.clone(),
            reserve_entry_id: reserve_entry_id.clone(),
            cost_attribution_id: attempt.cost_attribution_id.clone(),
            affordance_key: attempt.affordance_key.clone(),
            capability_handle,
            normalized_payload: attempt.normalized_payload.clone(),
            reserved_cost: CostVector {
                survival_micro: reserve_survival_micro,
                time_ms: estimated_cost.time_ms,
                io_units: estimated_cost.io_units,
                token_units: estimated_cost.token_units,
            },
            degraded,
            degradation_profile_id: degradation_profile
                .map(|candidate| candidate.profile_id.clone()),
            admission_cycle: cycle_id,
            metadata: Default::default(),
        };

        let report_item = AdmissionReportItem {
            attempt_id: attempt.attempt_id.clone(),
            disposition: AdmissionDisposition::Admitted { degraded },
            why: None,
            ledger_delta: Some(ReservationDelta {
                reserve_entry_id,
                reserved_survival_micro: reserve_survival_micro,
            }),
            admitted_action_id: Some(action_id),
            degradation_profile_id: degradation_profile
                .map(|candidate| candidate.profile_id.clone()),
        };

        Ok((action, reserve_survival_micro, report_item))
    }

    fn find_degraded_candidate<'a>(
        &'a self,
        context: &AdmissionContext<'_>,
        profile: &'a AffordanceProfile,
        attempt: &IntentAttempt,
    ) -> Result<Option<(&'a DegradationProfile, CostVector)>, ContinuityError> {
        let mut candidates: Vec<(usize, &DegradationProfile, CostVector)> = profile
            .degradation_profiles
            .iter()
            .enumerate()
            .filter(|(_, candidate)| candidate.depth <= self.config.max_degradation_depth)
            .map(|(index, candidate)| {
                self.cost_policy
                    .estimate_cost(profile, attempt, Some(candidate))
                    .map(|cost| (index, candidate, cost))
            })
            .collect::<Result<Vec<_>, _>>()?;

        candidates.sort_by(|lhs, rhs| {
            let (_, lhs_profile, lhs_cost) = lhs;
            let (_, rhs_profile, rhs_cost) = rhs;

            let ordering = match self.config.degradation_preference {
                DegradationPreference::LeastCapabilityLossFirst => (
                    lhs_profile.capability_loss_score,
                    lhs_cost.survival_micro,
                    lhs_profile.profile_id.as_str(),
                )
                    .cmp(&(
                        rhs_profile.capability_loss_score,
                        rhs_cost.survival_micro,
                        rhs_profile.profile_id.as_str(),
                    )),
                DegradationPreference::CheapestFirst => (
                    lhs_cost.survival_micro,
                    lhs_profile.capability_loss_score,
                    lhs_profile.profile_id.as_str(),
                )
                    .cmp(&(
                        rhs_cost.survival_micro,
                        rhs_profile.capability_loss_score,
                        rhs_profile.profile_id.as_str(),
                    )),
            };

            if ordering == Ordering::Equal {
                lhs.0.cmp(&rhs.0)
            } else {
                ordering
            }
        });

        for (_, candidate, estimated_cost) in candidates
            .into_iter()
            .take(self.config.max_degradation_variants)
        {
            let snapshot = self
                .cost_policy
                .affordability_snapshot(context.ledger.available_survival_micro(), &estimated_cost);
            if snapshot.within_runtime_limits() && snapshot.survival_affordable() {
                return Ok(Some((candidate, estimated_cost)));
            }
        }

        Ok(None)
    }

    fn economic_denial_code(snapshot: &AffordabilitySnapshot) -> String {
        if snapshot.required_time_ms > snapshot.max_time_ms {
            return "time_budget_exceeded".to_string();
        }
        if snapshot.required_io_units > snapshot.max_io_units {
            return "io_budget_exceeded".to_string();
        }
        if snapshot.required_token_units > snapshot.max_token_units {
            return "token_budget_exceeded".to_string();
        }
        "insufficient_survival_budget".to_string()
    }
}

pub fn derive_action_id(cycle_id: u64, source_attempt_id: &str, reserve_entry_id: &str) -> String {
    let canonical = serde_json::json!({
        "cycle_id": cycle_id,
        "source_attempt_id": source_attempt_id,
        "reserve_entry_id": reserve_entry_id,
    });

    let mut hasher = Sha256::new();
    hasher.update(canonicalize_json(&canonical).to_string().as_bytes());
    let digest = hasher.finalize();
    let hex = format!("{:x}", digest);
    format!("act:{}", &hex[..24])
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
