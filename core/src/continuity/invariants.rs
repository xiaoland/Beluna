use crate::{
    continuity::{
        error::{ContinuityError, invariant_violation},
        state::ContinuityState,
    },
    ledger::types::ReservationState,
};

pub fn assert_settlement_consistency(state: &ContinuityState) -> Result<(), ContinuityError> {
    for (reserve_entry_id, reservation) in &state.ledger.reservations {
        match reservation.state {
            ReservationState::Open => {
                if state.cycle_id > reservation.expires_at_cycle {
                    return Err(invariant_violation(format!(
                        "reservation '{}' leaked past expiry cycle {} at current cycle {}",
                        reserve_entry_id, reservation.expires_at_cycle, state.cycle_id
                    )));
                }
            }
            ReservationState::Settled | ReservationState::Refunded | ReservationState::Expired => {
                if reservation.terminal_reference_id.is_none() {
                    return Err(invariant_violation(format!(
                        "reservation '{}' is terminal without terminal_reference_id",
                        reserve_entry_id
                    )));
                }
            }
        }
    }

    Ok(())
}
