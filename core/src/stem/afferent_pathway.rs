use std::{
    collections::{BTreeMap, VecDeque},
    fmt,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use async_trait::async_trait;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::{Mutex, broadcast, mpsc, oneshot};

use crate::{
    observability::runtime as observability_runtime,
    types::{Sense, build_fq_neural_signal_id},
};

pub type SenseIngressHandle = SenseAfferentPathway;
pub type AfferentControlHandle = SenseAfferentPathway;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AfferentPathwayErrorKind {
    Closed,
    QueueClosed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AfferentPathwayError {
    pub kind: AfferentPathwayErrorKind,
    pub message: String,
}

impl AfferentPathwayError {
    fn closed() -> Self {
        Self {
            kind: AfferentPathwayErrorKind::Closed,
            message: "sense afferent pathway gate is closed".to_string(),
        }
    }

    fn queue_closed() -> Self {
        Self {
            kind: AfferentPathwayErrorKind::QueueClosed,
            message: "sense afferent pathway queue receiver is closed".to_string(),
        }
    }
}

impl fmt::Display for AfferentPathwayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for AfferentPathwayError {}

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

impl fmt::Display for RuleControlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for RuleControlError {}

pub type RuleRevision = u64;

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

#[derive(Debug, Clone, PartialEq)]
pub enum AfferentSidecarEvent {
    RuleAdded {
        revision: RuleRevision,
        rule_id: String,
    },
    RuleRemoved {
        revision: RuleRevision,
        rule_id: String,
        removed: bool,
    },
    SenseDeferred {
        sense_instance_id: String,
        rule_ids: Vec<String>,
        deferred_len: usize,
    },
    SenseReleased {
        sense_instance_id: String,
        deferred_len: usize,
    },
    SenseEvicted {
        sense_instance_id: String,
        reason: String,
        deferred_len: usize,
    },
}

pub struct AfferentSidecarSubscription {
    rx: broadcast::Receiver<AfferentSidecarEvent>,
}

impl AfferentSidecarSubscription {
    pub async fn recv(&mut self) -> Result<AfferentSidecarEvent, broadcast::error::RecvError> {
        self.rx.recv().await
    }
}

pub trait AfferentSidecarPort {
    fn subscribe_sidecar(&self) -> AfferentSidecarSubscription;
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

pub struct SenseConsumerHandle {
    rx: mpsc::Receiver<Sense>,
}

impl SenseConsumerHandle {
    pub fn new(rx: mpsc::Receiver<Sense>) -> Self {
        Self { rx }
    }

    pub async fn recv(&mut self) -> Option<Sense> {
        self.rx.recv().await
    }

    pub fn try_recv(&mut self) -> Result<Sense, mpsc::error::TryRecvError> {
        self.rx.try_recv()
    }
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

        let regex = if let Some(pattern) = &input.fq_sense_id_pattern {
            Some(Regex::new(pattern).map_err(|err| {
                RuleControlError::invalid_input(format!("invalid fq_sense_id_pattern regex: {err}"))
            })?)
        } else {
            None
        };

        Ok(Self {
            rule_id,
            min_weight: input.min_weight,
            fq_sense_id_pattern: input.fq_sense_id_pattern,
            fq_sense_id_regex: regex,
        })
    }

    fn snapshot(&self) -> DeferralRuleSnapshot {
        DeferralRuleSnapshot {
            rule_id: self.rule_id.clone(),
            min_weight: self.min_weight,
            fq_sense_id_pattern: self.fq_sense_id_pattern.clone(),
        }
    }

    fn matches(&self, sense: &Sense, fq_sense_id: &str) -> bool {
        if let Some(min_weight) = self.min_weight
            && !(sense.weight < min_weight)
        {
            return false;
        }

        if let Some(regex) = &self.fq_sense_id_regex
            && !regex.is_match(fq_sense_id)
        {
            return false;
        }

        true
    }
}

#[derive(Debug)]
struct DeferredSenseEntry {
    sense: Sense,
}

#[derive(Debug, Default)]
struct DeferralState {
    revision: RuleRevision,
    rules_by_id: BTreeMap<String, DeferralRuleRuntime>,
    deferred_fifo: VecDeque<DeferredSenseEntry>,
}

enum RuleCommand {
    AddOne {
        input: DeferralRuleAddInput,
        reply_tx: oneshot::Sender<Result<RuleRevision, RuleControlError>>,
    },
    RemoveOne {
        rule_id: String,
        reply_tx: oneshot::Sender<Result<RuleRevision, RuleControlError>>,
    },
    ReplaceAll {
        rules: Vec<DeferralRuleAddInput>,
        reply_tx: oneshot::Sender<Result<RuleRevision, RuleControlError>>,
    },
    Snapshot {
        reply_tx: oneshot::Sender<DeferralRuleSetSnapshot>,
    },
}

#[derive(Clone)]
pub struct SenseAfferentPathway {
    gate_open: Arc<AtomicBool>,
    send_lock: Arc<Mutex<()>>,
    ingress_tx: mpsc::Sender<Sense>,
    command_tx: mpsc::Sender<RuleCommand>,
    sidecar_tx: broadcast::Sender<AfferentSidecarEvent>,
}

impl SenseAfferentPathway {
    pub fn new(
        queue_capacity: usize,
        max_deferring_nums: usize,
        sidecar_capacity: usize,
    ) -> (Self, mpsc::Receiver<Sense>) {
        let queue_capacity = queue_capacity.max(1);
        let max_deferring_nums = max_deferring_nums.max(1);
        let sidecar_capacity = sidecar_capacity.max(1);

        let (ingress_tx, ingress_rx) = mpsc::channel(queue_capacity);
        let (egress_tx, egress_rx) = mpsc::channel(queue_capacity);
        let (command_tx, command_rx) = mpsc::channel(queue_capacity);
        let (sidecar_tx, _) = broadcast::channel(sidecar_capacity);

        tokio::spawn(run_scheduler_loop(
            ingress_rx,
            egress_tx,
            command_rx,
            sidecar_tx.clone(),
            max_deferring_nums,
        ));

        (
            Self {
                gate_open: Arc::new(AtomicBool::new(true)),
                send_lock: Arc::new(Mutex::new(())),
                ingress_tx,
                command_tx,
                sidecar_tx,
            },
            egress_rx,
        )
    }

    pub fn new_handles(
        queue_capacity: usize,
        max_deferring_nums: usize,
        sidecar_capacity: usize,
    ) -> (
        SenseIngressHandle,
        SenseConsumerHandle,
        AfferentControlHandle,
    ) {
        let (ingress, rx) = Self::new(queue_capacity, max_deferring_nums, sidecar_capacity);
        let control = ingress.clone();
        (ingress, SenseConsumerHandle::new(rx), control)
    }

    pub fn is_open(&self) -> bool {
        self.gate_open.load(Ordering::Acquire)
    }

    pub async fn send(&self, sense: Sense) -> Result<(), AfferentPathwayError> {
        let _guard = self.send_lock.lock().await;
        if !self.gate_open.load(Ordering::Acquire) {
            return Err(AfferentPathwayError::closed());
        }
        let endpoint_id = sense.endpoint_id.clone();
        let descriptor_id = sense.neural_signal_descriptor_id.clone();
        let sense_id = sense.sense_instance_id.clone();
        let sense_payload = json!(sense.payload.clone());
        let sense_weight = sense.weight;
        self.ingress_tx
            .send(sense)
            .await
            .map_err(|_| AfferentPathwayError::queue_closed())?;
        observability_runtime::emit_stem_afferent(
            "enqueue",
            &descriptor_id,
            Some(&endpoint_id),
            Some(&sense_id),
            None,
            Some(sense_payload),
            Some(sense_weight),
            Some(json!({
                "queue_name": "afferent",
            })),
            None,
            None,
        );
        Ok(())
    }

    pub async fn close_gate(&self) {
        let _guard = self.send_lock.lock().await;
        self.gate_open.store(false, Ordering::Release);
    }
}

impl AfferentSidecarPort for SenseAfferentPathway {
    fn subscribe_sidecar(&self) -> AfferentSidecarSubscription {
        AfferentSidecarSubscription {
            rx: self.sidecar_tx.subscribe(),
        }
    }
}

#[async_trait]
impl AfferentRuleControlPort for SenseAfferentPathway {
    async fn add_rule(
        &self,
        input: DeferralRuleAddInput,
    ) -> Result<RuleRevision, RuleControlError> {
        let (reply_tx, reply_rx) = oneshot::channel();
        self.command_tx
            .send(RuleCommand::AddOne { input, reply_tx })
            .await
            .map_err(|_| RuleControlError::internal("rule command channel is closed"))?;

        reply_rx
            .await
            .map_err(|_| RuleControlError::internal("rule add response dropped"))?
    }

    async fn remove_rule(&self, rule_id: String) -> Result<RuleRevision, RuleControlError> {
        let (reply_tx, reply_rx) = oneshot::channel();
        self.command_tx
            .send(RuleCommand::RemoveOne { rule_id, reply_tx })
            .await
            .map_err(|_| RuleControlError::internal("rule command channel is closed"))?;

        reply_rx
            .await
            .map_err(|_| RuleControlError::internal("rule remove response dropped"))?
    }

    async fn replace_ruleset(
        &self,
        rules: Vec<DeferralRuleAddInput>,
    ) -> Result<RuleRevision, RuleControlError> {
        let (reply_tx, reply_rx) = oneshot::channel();
        self.command_tx
            .send(RuleCommand::ReplaceAll { rules, reply_tx })
            .await
            .map_err(|_| RuleControlError::internal("rule command channel is closed"))?;

        reply_rx
            .await
            .map_err(|_| RuleControlError::internal("rule replace response dropped"))?
    }

    async fn snapshot_rules(&self) -> DeferralRuleSetSnapshot {
        let (reply_tx, reply_rx) = oneshot::channel();
        if self
            .command_tx
            .send(RuleCommand::Snapshot { reply_tx })
            .await
            .is_err()
        {
            return DeferralRuleSetSnapshot {
                revision: 0,
                rules: Vec::new(),
            };
        }

        reply_rx.await.unwrap_or(DeferralRuleSetSnapshot {
            revision: 0,
            rules: Vec::new(),
        })
    }
}

async fn run_scheduler_loop(
    mut ingress_rx: mpsc::Receiver<Sense>,
    egress_tx: mpsc::Sender<Sense>,
    mut command_rx: mpsc::Receiver<RuleCommand>,
    sidecar_tx: broadcast::Sender<AfferentSidecarEvent>,
    max_deferring_nums: usize,
) {
    let mut state = DeferralState::default();

    loop {
        tokio::select! {
            maybe_sense = ingress_rx.recv() => {
                let Some(sense) = maybe_sense else {
                    break;
                };
                handle_ingress_sense(
                    &mut state,
                    sense,
                    &egress_tx,
                    &sidecar_tx,
                    max_deferring_nums,
                ).await;
            }
            maybe_cmd = command_rx.recv() => {
                let Some(cmd) = maybe_cmd else {
                    break;
                };
                handle_command(
                    &mut state,
                    cmd,
                    &egress_tx,
                    &sidecar_tx,
                ).await;
            }
        }
    }
}

async fn handle_ingress_sense(
    state: &mut DeferralState,
    sense: Sense,
    egress_tx: &mpsc::Sender<Sense>,
    sidecar_tx: &broadcast::Sender<AfferentSidecarEvent>,
    max_deferring_nums: usize,
) {
    let matched_rule_ids = matching_rule_ids(state, &sense);
    if matched_rule_ids.is_empty() {
        if egress_tx.send(sense).await.is_err() {
            tracing::warn!(target: "stem.afferent", "consumer_channel_closed_drop_sense");
        }
        return;
    }

    state.deferred_fifo.push_back(DeferredSenseEntry {
        sense: sense.clone(),
    });
    let sense_instance_id = sense.sense_instance_id.clone();
    emit_sidecar(
        sidecar_tx,
        AfferentSidecarEvent::SenseDeferred {
            sense_instance_id,
            rule_ids: matched_rule_ids.clone(),
            deferred_len: state.deferred_fifo.len(),
        },
    );
    observability_runtime::emit_stem_afferent(
        "defer",
        &sense.neural_signal_descriptor_id,
        Some(&sense.endpoint_id),
        Some(&sense.sense_instance_id),
        None,
        Some(json!(sense.payload)),
        Some(sense.weight),
        Some(json!({
            "queue_name": "afferent",
            "deferred_len": state.deferred_fifo.len(),
        })),
        Some(json!(matched_rule_ids)),
        None,
    );

    while state.deferred_fifo.len() > max_deferring_nums {
        if let Some(evicted) = state.deferred_fifo.pop_front() {
            let evicted_sense_id = evicted.sense.sense_instance_id.clone();
            tracing::warn!(
                target: "stem.afferent",
                sense_instance_id = %evicted_sense_id,
                deferred_len = state.deferred_fifo.len(),
                max_deferring_nums = max_deferring_nums,
                "deferred_fifo_overflow_evict_oldest"
            );
            emit_sidecar(
                sidecar_tx,
                AfferentSidecarEvent::SenseEvicted {
                    sense_instance_id: evicted_sense_id.clone(),
                    reason: "deferred_fifo_overflow".to_string(),
                    deferred_len: state.deferred_fifo.len(),
                },
            );
            observability_runtime::emit_stem_afferent(
                "drop",
                &evicted.sense.neural_signal_descriptor_id,
                Some(&evicted.sense.endpoint_id),
                Some(&evicted_sense_id),
                None,
                Some(json!(evicted.sense.payload)),
                Some(evicted.sense.weight),
                Some(json!({
                    "queue_name": "afferent",
                    "deferred_len": state.deferred_fifo.len(),
                })),
                Some(json!(matched_rule_ids.clone())),
                Some("deferred_fifo_overflow"),
            );
        }
    }
}

async fn handle_command(
    state: &mut DeferralState,
    cmd: RuleCommand,
    egress_tx: &mpsc::Sender<Sense>,
    sidecar_tx: &broadcast::Sender<AfferentSidecarEvent>,
) {
    match cmd {
        RuleCommand::AddOne { input, reply_tx } => {
            let result = DeferralRuleRuntime::from_input(input).and_then(|runtime_rule| {
                let rule_snapshot = runtime_rule.snapshot();
                let rule_id = runtime_rule.rule_id.clone();
                if state.rules_by_id.contains_key(rule_id.as_str()) {
                    return Err(RuleControlError::invalid_input(format!(
                        "rule_id '{rule_id}' already exists"
                    )));
                }
                state.rules_by_id.insert(rule_id.clone(), runtime_rule);
                state.revision = state.revision.saturating_add(1);
                emit_sidecar(
                    sidecar_tx,
                    AfferentSidecarEvent::RuleAdded {
                        revision: state.revision,
                        rule_id: rule_id.clone(),
                    },
                );
                observability_runtime::emit_stem_afferent_rule(
                    None,
                    "add",
                    state.revision,
                    &rule_id,
                    Some(json!({
                        "rule_id": rule_snapshot.rule_id,
                        "min_weight": rule_snapshot.min_weight,
                        "fq_sense_id_pattern": rule_snapshot.fq_sense_id_pattern,
                    })),
                    None,
                );
                Ok(state.revision)
            });
            let _ = reply_tx.send(result);
            release_unblocked_front_fifo(state, egress_tx, sidecar_tx).await;
        }
        RuleCommand::RemoveOne { rule_id, reply_tx } => {
            let rule_id = rule_id.trim().to_string();
            let result = if rule_id.is_empty() {
                Err(RuleControlError::invalid_input("rule_id cannot be empty"))
            } else {
                let removed_rule = state.rules_by_id.remove(rule_id.as_str());
                let removed = removed_rule.is_some();
                state.revision = state.revision.saturating_add(1);
                emit_sidecar(
                    sidecar_tx,
                    AfferentSidecarEvent::RuleRemoved {
                        revision: state.revision,
                        rule_id: rule_id.clone(),
                        removed,
                    },
                );
                observability_runtime::emit_stem_afferent_rule(
                    None,
                    "remove",
                    state.revision,
                    &rule_id,
                    removed_rule.map(|rule| json!(rule.snapshot())),
                    Some(removed),
                );
                Ok(state.revision)
            };
            let _ = reply_tx.send(result);
            release_unblocked_front_fifo(state, egress_tx, sidecar_tx).await;
        }
        RuleCommand::ReplaceAll { rules, reply_tx } => {
            let result = build_replacement_ruleset(rules).map(|replacement| {
                let previous_rule_ids = state.rules_by_id.keys().cloned().collect::<Vec<_>>();
                state.rules_by_id = replacement;
                state.revision = state.revision.saturating_add(1);
                for rule_id in previous_rule_ids {
                    emit_sidecar(
                        sidecar_tx,
                        AfferentSidecarEvent::RuleRemoved {
                            revision: state.revision,
                            rule_id,
                            removed: true,
                        },
                    );
                }
                for rule_id in state.rules_by_id.keys() {
                    emit_sidecar(
                        sidecar_tx,
                        AfferentSidecarEvent::RuleAdded {
                            revision: state.revision,
                            rule_id: rule_id.clone(),
                        },
                    );
                }
                observability_runtime::emit_stem_afferent_rule(
                    None,
                    "replace",
                    state.revision,
                    "ruleset",
                    Some(json!({
                        "rules": state.rules_by_id
                            .values()
                            .map(|rule| rule.snapshot())
                            .collect::<Vec<_>>(),
                    })),
                    None,
                );
                state.revision
            });
            let _ = reply_tx.send(result);
            release_unblocked_front_fifo(state, egress_tx, sidecar_tx).await;
        }
        RuleCommand::Snapshot { reply_tx } => {
            let mut rules = state
                .rules_by_id
                .values()
                .map(|rule| rule.snapshot())
                .collect::<Vec<_>>();
            rules.sort_by(|lhs, rhs| lhs.rule_id.cmp(&rhs.rule_id));
            let _ = reply_tx.send(DeferralRuleSetSnapshot {
                revision: state.revision,
                rules,
            });
        }
    }
}

fn build_replacement_ruleset(
    rules: Vec<DeferralRuleAddInput>,
) -> Result<BTreeMap<String, DeferralRuleRuntime>, RuleControlError> {
    let mut replacement = BTreeMap::new();
    for input in rules {
        let runtime_rule = DeferralRuleRuntime::from_input(input)?;
        let rule_id = runtime_rule.rule_id.clone();
        if replacement.contains_key(rule_id.as_str()) {
            return Err(RuleControlError::invalid_input(format!(
                "duplicate rule_id '{rule_id}'"
            )));
        }
        replacement.insert(rule_id, runtime_rule);
    }
    Ok(replacement)
}

async fn release_unblocked_front_fifo(
    state: &mut DeferralState,
    egress_tx: &mpsc::Sender<Sense>,
    sidecar_tx: &broadcast::Sender<AfferentSidecarEvent>,
) {
    loop {
        let Some(front) = state.deferred_fifo.front() else {
            break;
        };

        if !matching_rule_ids(state, &front.sense).is_empty() {
            break;
        }

        let Some(released) = state.deferred_fifo.pop_front() else {
            break;
        };

        let sense_instance_id = released.sense.sense_instance_id.clone();
        let endpoint_id = released.sense.endpoint_id.clone();
        let descriptor_id = released.sense.neural_signal_descriptor_id.clone();
        let payload = json!(released.sense.payload.clone());
        let weight = released.sense.weight;
        if egress_tx.send(released.sense.clone()).await.is_err() {
            tracing::warn!(target: "stem.afferent", "consumer_channel_closed_drop_released_sense");
            observability_runtime::emit_stem_afferent(
                "drop",
                &descriptor_id,
                Some(&endpoint_id),
                Some(&sense_instance_id),
                None,
                Some(payload.clone()),
                Some(weight),
                Some(json!({
                    "queue_name": "afferent",
                    "deferred_len": state.deferred_fifo.len(),
                })),
                Some(json!([])),
                Some("consumer_channel_closed"),
            );
            break;
        }

        emit_sidecar(
            sidecar_tx,
            AfferentSidecarEvent::SenseReleased {
                sense_instance_id: sense_instance_id.clone(),
                deferred_len: state.deferred_fifo.len(),
            },
        );
        observability_runtime::emit_stem_afferent(
            "release",
            &descriptor_id,
            Some(&endpoint_id),
            Some(&sense_instance_id),
            None,
            Some(payload),
            Some(weight),
            Some(json!({
                "queue_name": "afferent",
                "deferred_len": state.deferred_fifo.len(),
            })),
            Some(json!([])),
            None,
        );
    }
}

fn matching_rule_ids(state: &DeferralState, sense: &Sense) -> Vec<String> {
    if state.rules_by_id.is_empty() {
        return Vec::new();
    }

    let fq_sense_id =
        build_fq_neural_signal_id(&sense.endpoint_id, &sense.neural_signal_descriptor_id);
    state
        .rules_by_id
        .values()
        .filter(|rule| rule.matches(sense, &fq_sense_id))
        .map(|rule| rule.rule_id.clone())
        .collect()
}

fn emit_sidecar(sidecar_tx: &broadcast::Sender<AfferentSidecarEvent>, event: AfferentSidecarEvent) {
    if sidecar_tx.send(event).is_err() {
        tracing::debug!(target: "stem.afferent", "sidecar_event_dropped_no_subscribers");
    }
}
