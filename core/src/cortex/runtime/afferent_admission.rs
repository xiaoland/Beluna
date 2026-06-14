use std::collections::{BTreeMap, VecDeque};

use async_trait::async_trait;
use regex::Regex;
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, mpsc};

use crate::{
    stem::{
        AfferentMiddleware, AfferentMiddlewareDecision, AfferentPathwayError,
        PathwayMiddlewareDecision, SenseConsumerHandle,
    },
    types::{Sense, build_fq_neural_signal_id},
};

pub type RuleRevision = u64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleControlErrorKind {
    InvalidInput,
    Internal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleControlError {
    pub kind: RuleControlErrorKind,
    pub message: String,
}

impl RuleControlError {
    fn invalid_input(message: impl Into<String>) -> Self {
        Self {
            kind: RuleControlErrorKind::InvalidInput,
            message: message.into(),
        }
    }

    fn internal(message: impl Into<String>) -> Self {
        Self {
            kind: RuleControlErrorKind::Internal,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for RuleControlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for RuleControlError {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeferralRuleAddInput {
    pub rule_id: String,
    #[serde(default)]
    pub min_weight: Option<f64>,
    #[serde(default)]
    pub fq_sense_id_pattern: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeferralRuleSnapshot {
    pub rule_id: String,
    pub min_weight: Option<f64>,
    pub fq_sense_id_pattern: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeferralRuleSetSnapshot {
    pub revision: RuleRevision,
    pub rules: Vec<DeferralRuleSnapshot>,
}

#[async_trait]
pub trait AfferentRuleControlPort: Send + Sync {
    async fn add_rule(&self, input: DeferralRuleAddInput)
    -> Result<RuleRevision, RuleControlError>;

    async fn remove_rule(&self, rule_id: String) -> Result<RuleRevision, RuleControlError>;

    async fn replace_ruleset(
        &self,
        rules: Vec<DeferralRuleAddInput>,
    ) -> Result<RuleRevision, RuleControlError>;

    async fn snapshot_rules(&self) -> DeferralRuleSetSnapshot;
}

#[derive(Debug, Clone)]
struct DeferralRuleRuntime {
    rule_id: String,
    min_weight: Option<f64>,
    fq_sense_id_pattern: Option<String>,
    fq_sense_id_regex: Option<Regex>,
}

impl DeferralRuleRuntime {
    fn from_input(input: DeferralRuleAddInput) -> Result<Self, RuleControlError> {
        let rule_id = input.rule_id.trim().to_string();
        if rule_id.is_empty() {
            return Err(RuleControlError::invalid_input("rule_id cannot be empty"));
        }
        if input.min_weight.is_none() && input.fq_sense_id_pattern.is_none() {
            return Err(RuleControlError::invalid_input(
                "at least one selector is required: min_weight or fq_sense_id_pattern",
            ));
        }
        if let Some(min_weight) = input.min_weight
            && !(0.0..=1.0).contains(&min_weight)
        {
            return Err(RuleControlError::invalid_input(
                "min_weight must be within [0, 1]",
            ));
        }

        let fq_sense_id_regex = input
            .fq_sense_id_pattern
            .as_ref()
            .map(|pattern| {
                Regex::new(pattern).map_err(|err| {
                    RuleControlError::invalid_input(format!(
                        "invalid fq_sense_id_pattern regex: {err}"
                    ))
                })
            })
            .transpose()?;

        Ok(Self {
            rule_id,
            min_weight: input.min_weight,
            fq_sense_id_pattern: input.fq_sense_id_pattern,
            fq_sense_id_regex,
        })
    }

    fn snapshot(&self) -> DeferralRuleSnapshot {
        DeferralRuleSnapshot {
            rule_id: self.rule_id.clone(),
            min_weight: self.min_weight,
            fq_sense_id_pattern: self.fq_sense_id_pattern.clone(),
        }
    }

    fn matches(&self, sense: &Sense) -> bool {
        if let Some(min_weight) = self.min_weight
            && !(sense.weight < min_weight)
        {
            return false;
        }
        if let Some(regex) = &self.fq_sense_id_regex {
            let fq_sense_id =
                build_fq_neural_signal_id(&sense.endpoint_id, &sense.neural_signal_descriptor_id);
            if !regex.is_match(&fq_sense_id) {
                return false;
            }
        }
        true
    }
}

#[derive(Debug, Default)]
struct DeferralState {
    revision: RuleRevision,
    rules_by_id: BTreeMap<String, DeferralRuleRuntime>,
    deferred_fifo: VecDeque<Sense>,
}

pub struct CortexAfferentAdmission {
    state: Mutex<DeferralState>,
    egress_tx: mpsc::Sender<Sense>,
    max_deferred_senses: usize,
}

impl CortexAfferentAdmission {
    pub fn new(queue_capacity: usize, max_deferred_senses: usize) -> (Self, SenseConsumerHandle) {
        let (egress_tx, egress_rx) = mpsc::channel(queue_capacity.max(1));
        (
            Self {
                state: Mutex::new(DeferralState::default()),
                egress_tx,
                max_deferred_senses: max_deferred_senses.max(1),
            },
            SenseConsumerHandle::new(egress_rx),
        )
    }

    async fn admit_or_defer(&self, sense: Sense) -> Result<(), AfferentPathwayError> {
        let mut state = self.state.lock().await;
        if matching_rule_ids(&state, &sense).is_empty() {
            drop(state);
            self.egress_tx
                .send(sense)
                .await
                .map_err(|_| AfferentPathwayError {
                    kind: crate::stem::AfferentPathwayErrorKind::QueueClosed,
                    message: "cortex afferent admission queue is closed".to_string(),
                })?;
            return Ok(());
        }

        state.deferred_fifo.push_back(sense);
        while state.deferred_fifo.len() > self.max_deferred_senses {
            state.deferred_fifo.pop_front();
        }
        Ok(())
    }

    async fn release_unblocked_senses(&self) -> Result<(), RuleControlError> {
        let mut state = self.state.lock().await;
        let mut still_deferred = VecDeque::new();
        let mut released = Vec::new();
        while let Some(sense) = state.deferred_fifo.pop_front() {
            if matching_rule_ids(&state, &sense).is_empty() {
                released.push(sense);
            } else {
                still_deferred.push_back(sense);
            }
        }
        state.deferred_fifo = still_deferred;
        drop(state);

        let mut not_released = VecDeque::new();
        for sense in released {
            match self.egress_tx.try_send(sense) {
                Ok(()) => {}
                Err(mpsc::error::TrySendError::Full(sense)) => {
                    not_released.push_back(sense);
                }
                Err(mpsc::error::TrySendError::Closed(_sense)) => {
                    return Err(RuleControlError::internal(
                        "cortex admission queue is closed",
                    ));
                }
            }
        }

        if !not_released.is_empty() {
            let mut state = self.state.lock().await;
            while let Some(sense) = not_released.pop_back() {
                state.deferred_fifo.push_front(sense);
            }
        }
        Ok(())
    }
}

#[async_trait]
impl AfferentMiddleware for CortexAfferentAdmission {
    async fn handle_sense(
        &self,
        sense: &Sense,
    ) -> Result<AfferentMiddlewareDecision, AfferentPathwayError> {
        self.admit_or_defer(sense.clone()).await?;
        Ok(PathwayMiddlewareDecision::Accepted(()))
    }
}

#[async_trait]
impl AfferentRuleControlPort for CortexAfferentAdmission {
    async fn add_rule(
        &self,
        input: DeferralRuleAddInput,
    ) -> Result<RuleRevision, RuleControlError> {
        let runtime_rule = DeferralRuleRuntime::from_input(input)?;
        let mut state = self.state.lock().await;
        let rule_id = runtime_rule.rule_id.clone();
        if state.rules_by_id.contains_key(rule_id.as_str()) {
            return Err(RuleControlError::invalid_input(format!(
                "rule_id '{rule_id}' already exists"
            )));
        }
        state.rules_by_id.insert(rule_id, runtime_rule);
        state.revision = state.revision.saturating_add(1);
        Ok(state.revision)
    }

    async fn remove_rule(&self, rule_id: String) -> Result<RuleRevision, RuleControlError> {
        let mut state = self.state.lock().await;
        let removed = state.rules_by_id.remove(rule_id.as_str()).is_some();
        if removed {
            state.revision = state.revision.saturating_add(1);
        }
        let revision = state.revision;
        drop(state);
        self.release_unblocked_senses().await?;
        Ok(revision)
    }

    async fn replace_ruleset(
        &self,
        rules: Vec<DeferralRuleAddInput>,
    ) -> Result<RuleRevision, RuleControlError> {
        let mut replacement = BTreeMap::new();
        for input in rules {
            let runtime_rule = DeferralRuleRuntime::from_input(input)?;
            let rule_id = runtime_rule.rule_id.clone();
            if replacement.insert(rule_id.clone(), runtime_rule).is_some() {
                return Err(RuleControlError::invalid_input(format!(
                    "duplicate rule_id '{rule_id}'"
                )));
            }
        }
        let mut state = self.state.lock().await;
        state.rules_by_id = replacement;
        state.revision = state.revision.saturating_add(1);
        let revision = state.revision;
        drop(state);
        self.release_unblocked_senses().await?;
        Ok(revision)
    }

    async fn snapshot_rules(&self) -> DeferralRuleSetSnapshot {
        let state = self.state.lock().await;
        DeferralRuleSetSnapshot {
            revision: state.revision,
            rules: state
                .rules_by_id
                .values()
                .map(|rule| rule.snapshot())
                .collect(),
        }
    }
}

fn matching_rule_ids(state: &DeferralState, sense: &Sense) -> Vec<String> {
    state
        .rules_by_id
        .values()
        .filter(|rule| rule.matches(sense))
        .map(|rule| rule.rule_id.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{Duration, timeout};

    fn test_sense(id: &str, weight: f64) -> Sense {
        Sense {
            sense_instance_id: id.to_string(),
            endpoint_id: "endpoint".to_string(),
            neural_signal_descriptor_id: "sense.test".to_string(),
            payload: "{}".to_string(),
            weight,
            act_instance_id: None,
        }
    }

    #[tokio::test]
    async fn deferral_is_owned_by_cortex_admission() {
        let (admission, mut consumer) = CortexAfferentAdmission::new(4, 4);
        admission
            .add_rule(DeferralRuleAddInput {
                rule_id: "low-weight".to_string(),
                min_weight: Some(0.5),
                fq_sense_id_pattern: None,
            })
            .await
            .expect("rule should be accepted");

        let decision = admission
            .handle_sense(&test_sense("sense-1", 0.25))
            .await
            .expect("admission should handle sense");

        assert_eq!(decision, PathwayMiddlewareDecision::Accepted(()));
        assert!(
            timeout(Duration::from_millis(20), consumer.recv())
                .await
                .is_err(),
            "deferred sense should not reach Cortex consumer yet"
        );

        admission
            .remove_rule("low-weight".to_string())
            .await
            .expect("rule removal should release deferred sense");
        let released = timeout(Duration::from_millis(100), consumer.recv())
            .await
            .expect("released sense should arrive")
            .expect("consumer should stay open");
        assert_eq!(released.sense_instance_id, "sense-1");
    }
}
