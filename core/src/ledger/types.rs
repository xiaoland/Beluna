use serde::{Deserialize, Serialize};

pub type CycleId = u64;
pub type LedgerEntryId = String;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyVersionTuple {
    pub affordance_registry_version: String,
    pub cost_policy_version: String,
    pub admission_ruleset_version: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReservationState {
    Open,
    Settled,
    Refunded,
    Expired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReservationRecord {
    pub reserve_entry_id: String,
    pub cost_attribution_id: String,
    pub reserved_survival_micro: i64,
    pub created_cycle: CycleId,
    pub expires_at_cycle: CycleId,
    pub state: ReservationState,
    #[serde(default)]
    pub terminal_reference_id: Option<String>,
    #[serde(default)]
    pub terminal_cycle: Option<CycleId>,
    #[serde(default)]
    pub action_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LedgerEntryKind {
    Reserve { reserve_entry_id: String },
    Adjustment { reserve_entry_id: String },
    Settle { reserve_entry_id: String },
    Refund { reserve_entry_id: String },
    Expire { reserve_entry_id: String },
    ExternalDebit { reference_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub entry_id: LedgerEntryId,
    pub seq_no: u64,
    pub cycle_id: CycleId,
    pub kind: LedgerEntryKind,
    pub amount_survival_micro: i64,
    #[serde(default)]
    pub cost_attribution_id: Option<String>,
    #[serde(default)]
    pub action_id: Option<String>,
    #[serde(default)]
    pub reference_id: Option<String>,
    pub policy_versions: PolicyVersionTuple,
}
