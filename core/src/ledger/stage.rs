use sha2::{Digest, Sha256};

use crate::{
    continuity::error::{ContinuityError, invariant_violation, ledger_conflict},
    ledger::{
        SurvivalLedger,
        types::{PolicyVersionTuple, ReservationState},
    },
    runtime_types::{Act, DispatchDecision, PhysicalLedgerSnapshot},
    spine::types::SpineEvent,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LedgerDispatchTicket {
    pub reserve_entry_id: String,
    pub cost_attribution_id: String,
    pub reserved_survival_micro: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DispatchContext {
    pub cycle_id: u64,
    pub act_seq_no: u64,
}

#[derive(Debug, Clone)]
pub struct LedgerStage {
    ledger: SurvivalLedger,
    reservation_ttl_cycles: u64,
    policy_versions: PolicyVersionTuple,
}

impl LedgerStage {
    pub fn new(initial_survival_micro: i64) -> Self {
        Self {
            ledger: SurvivalLedger::new(initial_survival_micro),
            reservation_ttl_cycles: 8,
            policy_versions: PolicyVersionTuple {
                affordance_registry_version: "v2".to_string(),
                cost_policy_version: "v2".to_string(),
                admission_ruleset_version: "removed".to_string(),
            },
        }
    }

    pub fn with_ledger(ledger: SurvivalLedger) -> Self {
        Self {
            ledger,
            reservation_ttl_cycles: 8,
            policy_versions: PolicyVersionTuple {
                affordance_registry_version: "v2".to_string(),
                cost_policy_version: "v2".to_string(),
                admission_ruleset_version: "removed".to_string(),
            },
        }
    }

    pub fn pre_dispatch(
        &mut self,
        act: &Act,
        ctx: &DispatchContext,
    ) -> Result<(DispatchDecision, Option<LedgerDispatchTicket>), ContinuityError> {
        let reserve_survival_micro = act.requested_resources.survival_micro.max(0);
        if self.ledger.available_survival_micro() < reserve_survival_micro {
            return Ok((DispatchDecision::Break, None));
        }

        let cost_attribution_id = derive_cost_attribution_id(ctx.cycle_id, &act.act_id);
        let reserve_entry_id = self.ledger.reserve(
            ctx.cycle_id,
            reserve_survival_micro,
            self.reservation_ttl_cycles,
            cost_attribution_id.clone(),
            format!("reserve:{}", act.act_id),
            self.policy_versions.clone(),
        )?;

        Ok((
            DispatchDecision::Continue,
            Some(LedgerDispatchTicket {
                reserve_entry_id,
                cost_attribution_id,
                reserved_survival_micro: reserve_survival_micro,
            }),
        ))
    }

    pub fn settle_from_spine(
        &mut self,
        ticket: &LedgerDispatchTicket,
        event: &SpineEvent,
        ctx: &DispatchContext,
    ) -> Result<(), ContinuityError> {
        if event.reserve_entry_id() != ticket.reserve_entry_id {
            return Err(invariant_violation(format!(
                "settlement reserve mismatch: expected={}, got={}",
                ticket.reserve_entry_id,
                event.reserve_entry_id()
            )));
        }

        if event.cost_attribution_id() != ticket.cost_attribution_id {
            return Err(invariant_violation(format!(
                "settlement cost attribution mismatch: expected={}, got={}",
                ticket.cost_attribution_id,
                event.cost_attribution_id()
            )));
        }

        match event {
            SpineEvent::ActApplied {
                reference_id,
                actual_cost_micro,
                act_id,
                ..
            } => self.ledger.settle_reservation(
                ctx.cycle_id,
                &ticket.reserve_entry_id,
                reference_id,
                *actual_cost_micro,
                Some(act_id.clone()),
                self.policy_versions.clone(),
            )?,
            SpineEvent::ActRejected {
                reference_id,
                act_id,
                ..
            }
            | SpineEvent::ActDeferred {
                reference_id,
                act_id,
                ..
            } => self.ledger.refund_reservation(
                ctx.cycle_id,
                &ticket.reserve_entry_id,
                reference_id,
                Some(act_id.clone()),
                self.policy_versions.clone(),
            )?,
        }

        Ok(())
    }

    pub fn physical_snapshot(&self) -> PhysicalLedgerSnapshot {
        PhysicalLedgerSnapshot {
            available_survival_micro: self.ledger.available_survival_micro(),
            open_reservation_count: self
                .ledger
                .reservations
                .values()
                .filter(|reservation| reservation.state == ReservationState::Open)
                .count(),
        }
    }

    pub fn ledger(&self) -> &SurvivalLedger {
        &self.ledger
    }

    pub fn ledger_mut(&mut self) -> &mut SurvivalLedger {
        &mut self.ledger
    }

    pub fn expire_open_reservations(&mut self, cycle_id: u64) -> Result<Vec<String>, ContinuityError> {
        self.ledger.expire_open_reservations(
            cycle_id,
            "expiry",
            self.policy_versions.clone(),
        )
    }

    pub fn ensure_reservation_open(&self, reserve_entry_id: &str) -> Result<(), ContinuityError> {
        let Some(record) = self.ledger.reservations.get(reserve_entry_id) else {
            return Err(ledger_conflict(format!(
                "reservation '{}' does not exist",
                reserve_entry_id
            )));
        };
        if record.state != ReservationState::Open {
            return Err(ledger_conflict(format!(
                "reservation '{}' is not open",
                reserve_entry_id
            )));
        }
        Ok(())
    }
}

fn derive_cost_attribution_id(cycle_id: u64, act_id: &str) -> String {
    let canonical = serde_json::json!({
        "cycle_id": cycle_id,
        "act_id": act_id,
    });
    let mut hasher = Sha256::new();
    hasher.update(canonical.to_string().as_bytes());
    let digest = hasher.finalize();
    let hex = format!("{:x}", digest);
    format!("cat:{}", &hex[..24])
}
