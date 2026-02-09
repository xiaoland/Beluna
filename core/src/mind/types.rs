use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

pub type GoalId = String;
pub type IntentId = String;
pub type CycleId = u64;
pub type CheckpointToken = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GoalLevel {
    High,
    Mid,
    Low,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GoalStatus {
    Proposed,
    Active,
    Paused,
    Cancelled,
    Completed,
    Merged,
}

impl GoalStatus {
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Cancelled | Self::Completed | Self::Merged)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Goal {
    pub id: GoalId,
    pub title: String,
    pub level: GoalLevel,
    #[serde(default)]
    pub parent_goal_id: Option<GoalId>,
    pub priority: u8,
    pub created_cycle: CycleId,
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoalRecord {
    pub goal: Goal,
    pub status: GoalStatus,
    #[serde(default)]
    pub merged_into: Option<GoalId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SafePoint {
    pub preemptable: bool,
    #[serde(default)]
    pub checkpoint_token: Option<CheckpointToken>,
    pub rationale: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreemptionDisposition {
    Pause,
    Cancel,
    Continue,
    Merge,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreemptionDecision {
    pub disposition: PreemptionDisposition,
    pub rationale: String,
    pub safe_point: SafePoint,
    #[serde(default)]
    pub merge_goal_id: Option<GoalId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvaluationCriterion {
    GoalAlignment,
    SubsystemReliability,
    SignalFaithfulness,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvaluationVerdict {
    Pass,
    Borderline,
    Fail,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Judgment {
    pub criterion: EvaluationCriterion,
    pub verdict: EvaluationVerdict,
    pub confidence: f32,
    pub rationale: String,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
}

impl Judgment {
    pub fn clamped_confidence(&self) -> f32 {
        clamp_confidence(self.confidence)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvaluationReport {
    #[serde(default)]
    pub goal_id: Option<GoalId>,
    pub judgments: Vec<Judgment>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntentKind {
    Delegate,
    Evaluate,
    Observe,
    ProposeEvolution,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionIntent {
    pub intent_id: IntentId,
    #[serde(default)]
    pub goal_id: Option<GoalId>,
    pub kind: IntentKind,
    pub description: String,
    #[serde(default)]
    pub checkpoint_token: Option<CheckpointToken>,
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DelegationResult {
    pub intent_id: IntentId,
    pub helper_id: String,
    pub payload: serde_json::Value,
    pub confidence: f32,
}

impl DelegationResult {
    pub fn clamped_confidence(&self) -> f32 {
        clamp_confidence(self.confidence)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ConflictCase {
    HelperOutputSameIntent {
        intent_id: IntentId,
        candidates: Vec<DelegationResult>,
    },
    EvaluatorVerdictSameCriterion {
        criterion: EvaluationCriterion,
        candidates: Vec<Judgment>,
    },
    MergeCompatibility {
        active_goal_id: GoalId,
        incoming_goal_id: GoalId,
        compatible: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ConflictResolution {
    SelectedHelperResult {
        intent_id: IntentId,
        helper_id: String,
    },
    SelectedJudgment {
        criterion: EvaluationCriterion,
        verdict: EvaluationVerdict,
    },
    MergeAllowed {
        merged_goal_id: GoalId,
    },
    MergeRejected,
    NoConflict,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MemoryDirective {
    Remember { key: String, summary: String },
    Forget { key: String, rationale: String },
    KeepTransient,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EvolutionTarget {
    Model { id: String },
    MemoryStructure { id: String },
    PerceptionPipeline { id: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvolutionAction {
    Replace,
    Retrain,
    Reconfigure,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChangeProposal {
    pub target: EvolutionTarget,
    pub action: EvolutionAction,
    pub rationale: String,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    pub confidence: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EvolutionDecision {
    NoChange { rationale: String },
    ChangeProposal(ChangeProposal),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MindCommand {
    ProposeGoal(Goal),
    ObserveSignal {
        signal_id: String,
        #[serde(default)]
        fidelity_hint: Option<f32>,
        payload: serde_json::Value,
    },
    SubmitDelegationResult(DelegationResult),
    EvaluateNow,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignalObservation {
    pub signal_id: String,
    #[serde(default)]
    pub fidelity_hint: Option<f32>,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MindEvent {
    GoalActivated {
        goal_id: GoalId,
    },
    GoalPaused {
        goal_id: GoalId,
    },
    GoalCancelled {
        goal_id: GoalId,
    },
    GoalMerged {
        from_goal_id: GoalId,
        into_goal_id: GoalId,
    },
    PreemptionDecided {
        disposition: PreemptionDisposition,
    },
    EvaluationCompleted,
    ConflictResolved,
    MemoryPolicyApplied,
    EvolutionDecided,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MindDecision {
    Preemption(PreemptionDecision),
    DelegationPlan(Vec<ActionIntent>),
    Evaluation(EvaluationReport),
    Conflict(ConflictResolution),
    MemoryPolicy(MemoryDirective),
    Evolution(EvolutionDecision),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MindCycleOutput {
    pub cycle_id: CycleId,
    pub events: Vec<MindEvent>,
    pub decisions: Vec<MindDecision>,
}

pub fn clamp_confidence(value: f32) -> f32 {
    value.clamp(0.0, 1.0)
}

pub fn merged_goal_id(a: &GoalId, b: &GoalId) -> GoalId {
    if a <= b {
        format!("merge:{a}+{b}")
    } else {
        format!("merge:{b}+{a}")
    }
}
