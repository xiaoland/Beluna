use std::collections::BTreeMap;

use crate::{
    continuity::{
        error::{ContinuityError, arithmetic_error, invalid_request, ledger_conflict},
        types::ExternalDebitObservation,
    },
    ledger::types::{
        CycleId, LedgerEntry, LedgerEntryId, LedgerEntryKind, PolicyVersionTuple,
        ReservationRecord, ReservationState,
    },
};

#[derive(Debug, Clone)]
pub struct SurvivalLedger {
    balance_survival_micro: i64,
    next_sequence: u64,
    pub entries: Vec<LedgerEntry>,
    pub reservations: BTreeMap<String, ReservationRecord>,
}

impl SurvivalLedger {
    pub fn new(initial_survival_micro: i64) -> Self {
        Self {
            balance_survival_micro: initial_survival_micro,
            next_sequence: 0,
            entries: Vec::new(),
            reservations: BTreeMap::new(),
        }
    }

    pub fn balance_survival_micro(&self) -> i64 {
        self.balance_survival_micro
    }

    pub fn available_survival_micro(&self) -> i64 {
        self.balance_survival_micro
    }

    pub fn reserve(
        &mut self,
        cycle_id: CycleId,
        reserve_survival_micro: i64,
        ttl_cycles: u64,
        cost_attribution_id: String,
        reference_id: String,
        policy_versions: PolicyVersionTuple,
    ) -> Result<String, ContinuityError> {
        if reserve_survival_micro < 0 {
            return Err(invalid_request(
                "reserve_survival_micro must be non-negative",
            ));
        }

        if self.balance_survival_micro < reserve_survival_micro {
            return Err(ledger_conflict(format!(
                "insufficient survival budget for reservation: required={}, available={}",
                reserve_survival_micro, self.balance_survival_micro
            )));
        }

        self.balance_survival_micro = self
            .balance_survival_micro
            .checked_sub(reserve_survival_micro)
            .ok_or_else(|| arithmetic_error("survival budget underflow during reserve"))?;

        let reserve_entry_id = format!("resv:{}:{}", cycle_id, self.next_sequence + 1);
        let expires_at_cycle = cycle_id.saturating_add(ttl_cycles);
        let reservation = ReservationRecord {
            reserve_entry_id: reserve_entry_id.clone(),
            cost_attribution_id: cost_attribution_id.clone(),
            reserved_survival_micro: reserve_survival_micro,
            created_cycle: cycle_id,
            expires_at_cycle,
            state: ReservationState::Open,
            terminal_reference_id: None,
            terminal_cycle: None,
            action_id: None,
        };

        self.reservations
            .insert(reserve_entry_id.clone(), reservation);

        self.append_entry(
            cycle_id,
            LedgerEntryKind::Reserve {
                reserve_entry_id: reserve_entry_id.clone(),
            },
            -reserve_survival_micro,
            Some(cost_attribution_id),
            None,
            Some(reference_id),
            policy_versions,
        )?;

        Ok(reserve_entry_id)
    }

    pub fn attach_action_id(
        &mut self,
        reserve_entry_id: &str,
        action_id: String,
    ) -> Result<(), ContinuityError> {
        let reservation = self.reservations.get_mut(reserve_entry_id).ok_or_else(|| {
            invalid_request(format!("unknown reserve_entry_id '{}'", reserve_entry_id))
        })?;
        reservation.action_id = Some(action_id);
        Ok(())
    }

    pub fn settle_reservation(
        &mut self,
        cycle_id: CycleId,
        reserve_entry_id: &str,
        reference_id: &str,
        actual_cost_micro: i64,
        action_id: Option<String>,
        policy_versions: PolicyVersionTuple,
    ) -> Result<(), ContinuityError> {
        let (cost_attribution_id, current_action_id, reserved_survival_micro) = {
            let reservation = self.reservations.get_mut(reserve_entry_id).ok_or_else(|| {
                invalid_request(format!("unknown reserve_entry_id '{}'", reserve_entry_id))
            })?;

            match reservation.state {
                ReservationState::Settled
                    if reservation.terminal_reference_id.as_deref() == Some(reference_id) =>
                {
                    return Ok(());
                }
                ReservationState::Settled
                | ReservationState::Refunded
                | ReservationState::Expired => {
                    return Err(ledger_conflict(format!(
                        "reservation '{}' already terminal via reference {:?}",
                        reserve_entry_id, reservation.terminal_reference_id
                    )));
                }
                ReservationState::Open => {}
            }

            reservation.state = ReservationState::Settled;
            reservation.terminal_reference_id = Some(reference_id.to_string());
            reservation.terminal_cycle = Some(cycle_id);
            if reservation.action_id.is_none() {
                reservation.action_id = action_id.clone();
            }

            (
                reservation.cost_attribution_id.clone(),
                reservation.action_id.clone().or(action_id.clone()),
                reservation.reserved_survival_micro,
            )
        };

        let delta = (actual_cost_micro as i128) - (reserved_survival_micro as i128);
        if delta != 0 {
            if delta.is_positive() {
                self.balance_survival_micro = self
                    .balance_survival_micro
                    .checked_sub(delta as i64)
                    .ok_or_else(|| arithmetic_error("survival budget underflow during settle"))?;
            } else {
                self.balance_survival_micro = self
                    .balance_survival_micro
                    .checked_add((-delta) as i64)
                    .ok_or_else(|| arithmetic_error("survival budget overflow during settle"))?;
            }

            self.append_entry(
                cycle_id,
                LedgerEntryKind::Adjustment {
                    reserve_entry_id: reserve_entry_id.to_string(),
                },
                delta as i64,
                Some(cost_attribution_id.clone()),
                current_action_id.clone(),
                Some(reference_id.to_string()),
                policy_versions.clone(),
            )?;
        }

        self.append_entry(
            cycle_id,
            LedgerEntryKind::Settle {
                reserve_entry_id: reserve_entry_id.to_string(),
            },
            0,
            Some(cost_attribution_id),
            current_action_id,
            Some(reference_id.to_string()),
            policy_versions,
        )?;

        Ok(())
    }

    pub fn refund_reservation(
        &mut self,
        cycle_id: CycleId,
        reserve_entry_id: &str,
        reference_id: &str,
        action_id: Option<String>,
        policy_versions: PolicyVersionTuple,
    ) -> Result<(), ContinuityError> {
        let (cost_attribution_id, reserved_survival_micro, current_action_id) = {
            let reservation = self.reservations.get_mut(reserve_entry_id).ok_or_else(|| {
                invalid_request(format!("unknown reserve_entry_id '{}'", reserve_entry_id))
            })?;

            match reservation.state {
                ReservationState::Refunded
                    if reservation.terminal_reference_id.as_deref() == Some(reference_id) =>
                {
                    return Ok(());
                }
                ReservationState::Settled
                | ReservationState::Refunded
                | ReservationState::Expired => {
                    return Err(ledger_conflict(format!(
                        "reservation '{}' already terminal via reference {:?}",
                        reserve_entry_id, reservation.terminal_reference_id
                    )));
                }
                ReservationState::Open => {}
            }

            reservation.state = ReservationState::Refunded;
            reservation.terminal_reference_id = Some(reference_id.to_string());
            reservation.terminal_cycle = Some(cycle_id);
            if reservation.action_id.is_none() {
                reservation.action_id = action_id.clone();
            }

            (
                reservation.cost_attribution_id.clone(),
                reservation.reserved_survival_micro,
                reservation.action_id.clone().or(action_id),
            )
        };

        self.balance_survival_micro = self
            .balance_survival_micro
            .checked_add(reserved_survival_micro)
            .ok_or_else(|| arithmetic_error("survival budget overflow during refund"))?;

        self.append_entry(
            cycle_id,
            LedgerEntryKind::Refund {
                reserve_entry_id: reserve_entry_id.to_string(),
            },
            reserved_survival_micro,
            Some(cost_attribution_id),
            current_action_id,
            Some(reference_id.to_string()),
            policy_versions,
        )?;

        Ok(())
    }

    pub fn expire_open_reservations(
        &mut self,
        cycle_id: CycleId,
        reference_prefix: &str,
        policy_versions: PolicyVersionTuple,
    ) -> Result<Vec<String>, ContinuityError> {
        let to_expire: Vec<String> = self
            .reservations
            .iter()
            .filter(|(_, reservation)| {
                matches!(reservation.state, ReservationState::Open)
                    && cycle_id >= reservation.expires_at_cycle
            })
            .map(|(reserve_entry_id, _)| reserve_entry_id.clone())
            .collect();

        for reserve_entry_id in &to_expire {
            self.expire_reservation(
                cycle_id,
                reserve_entry_id,
                &format!("{}:{}", reference_prefix, reserve_entry_id),
                policy_versions.clone(),
            )?;
        }

        Ok(to_expire)
    }

    fn expire_reservation(
        &mut self,
        cycle_id: CycleId,
        reserve_entry_id: &str,
        reference_id: &str,
        policy_versions: PolicyVersionTuple,
    ) -> Result<(), ContinuityError> {
        let (cost_attribution_id, reserved_survival_micro, action_id) = {
            let reservation = self.reservations.get_mut(reserve_entry_id).ok_or_else(|| {
                invalid_request(format!("unknown reserve_entry_id '{}'", reserve_entry_id))
            })?;

            match reservation.state {
                ReservationState::Expired
                    if reservation.terminal_reference_id.as_deref() == Some(reference_id) =>
                {
                    return Ok(());
                }
                ReservationState::Settled
                | ReservationState::Refunded
                | ReservationState::Expired => {
                    return Err(ledger_conflict(format!(
                        "reservation '{}' already terminal via reference {:?}",
                        reserve_entry_id, reservation.terminal_reference_id
                    )));
                }
                ReservationState::Open => {}
            }

            reservation.state = ReservationState::Expired;
            reservation.terminal_reference_id = Some(reference_id.to_string());
            reservation.terminal_cycle = Some(cycle_id);

            (
                reservation.cost_attribution_id.clone(),
                reservation.reserved_survival_micro,
                reservation.action_id.clone(),
            )
        };

        self.balance_survival_micro = self
            .balance_survival_micro
            .checked_add(reserved_survival_micro)
            .ok_or_else(|| arithmetic_error("survival budget overflow during expiration"))?;

        self.append_entry(
            cycle_id,
            LedgerEntryKind::Expire {
                reserve_entry_id: reserve_entry_id.to_string(),
            },
            reserved_survival_micro,
            Some(cost_attribution_id),
            action_id,
            Some(reference_id.to_string()),
            policy_versions,
        )?;

        Ok(())
    }

    pub fn apply_external_debit(
        &mut self,
        cycle_id: CycleId,
        observation: &ExternalDebitObservation,
        policy_versions: PolicyVersionTuple,
    ) -> Result<LedgerEntryId, ContinuityError> {
        if observation.debit_survival_micro < 0 {
            return Err(invalid_request(
                "external debit amount must be non-negative for debit entries",
            ));
        }

        self.balance_survival_micro = self
            .balance_survival_micro
            .checked_sub(observation.debit_survival_micro)
            .ok_or_else(|| arithmetic_error("survival budget underflow during external debit"))?;

        self.append_entry(
            cycle_id,
            LedgerEntryKind::ExternalDebit {
                reference_id: observation.reference_id.clone(),
            },
            -observation.debit_survival_micro,
            Some(observation.cost_attribution_id.clone()),
            observation.action_id.clone(),
            Some(observation.reference_id.clone()),
            policy_versions,
        )
    }

    fn append_entry(
        &mut self,
        cycle_id: CycleId,
        kind: LedgerEntryKind,
        amount_survival_micro: i64,
        cost_attribution_id: Option<String>,
        action_id: Option<String>,
        reference_id: Option<String>,
        policy_versions: PolicyVersionTuple,
    ) -> Result<LedgerEntryId, ContinuityError> {
        self.next_sequence = self
            .next_sequence
            .checked_add(1)
            .ok_or_else(|| arithmetic_error("ledger sequence overflow"))?;

        let entry_id = format!("led:{:016}", self.next_sequence);
        self.entries.push(LedgerEntry {
            entry_id: entry_id.clone(),
            seq_no: self.next_sequence,
            cycle_id,
            kind,
            amount_survival_micro,
            cost_attribution_id,
            action_id,
            reference_id,
            policy_versions,
        });
        Ok(entry_id)
    }
}

impl Default for SurvivalLedger {
    fn default() -> Self {
        Self::new(1_000_000)
    }
}
