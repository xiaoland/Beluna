use beluna::{
    continuity::ExternalDebitObservation,
    ledger::{PolicyVersionTuple, ReservationState, SurvivalLedger},
};

fn versions() -> PolicyVersionTuple {
    PolicyVersionTuple {
        affordance_registry_version: "ar:v1".to_string(),
        cost_policy_version: "cp:v1".to_string(),
        admission_ruleset_version: "rs:v1".to_string(),
    }
}

#[test]
fn given_settlement_replay_with_same_reference_when_settle_then_idempotent_and_strict() {
    let mut ledger = SurvivalLedger::new(1_000);
    let reserve_entry_id = ledger
        .reserve(
            1,
            100,
            4,
            "cat:1".to_string(),
            "ref:reserve".to_string(),
            versions(),
        )
        .expect("reserve should succeed");

    ledger
        .settle_reservation(
            1,
            &reserve_entry_id,
            "ref:settle",
            100,
            Some("act:1".to_string()),
            versions(),
        )
        .expect("settle should succeed");

    ledger
        .settle_reservation(
            1,
            &reserve_entry_id,
            "ref:settle",
            100,
            Some("act:1".to_string()),
            versions(),
        )
        .expect("idempotent replay with same reference should be allowed");

    let err = ledger
        .refund_reservation(
            1,
            &reserve_entry_id,
            "ref:refund",
            Some("act:1".to_string()),
            versions(),
        )
        .expect_err("second terminal operation must fail");
    assert!(err.message.contains("already terminal"));
}

#[test]
fn given_open_reservation_when_cycle_reaches_expiry_then_it_expires_and_releases_budget() {
    let mut ledger = SurvivalLedger::new(1_000);
    let reserve_entry_id = ledger
        .reserve(
            1,
            120,
            2,
            "cat:1".to_string(),
            "ref:reserve".to_string(),
            versions(),
        )
        .expect("reserve should succeed");

    assert_eq!(ledger.balance_survival_micro(), 880);

    let none_expired = ledger
        .expire_open_reservations(2, "expiry", versions())
        .expect("expiration scan should succeed");
    assert!(none_expired.is_empty());

    let expired = ledger
        .expire_open_reservations(3, "expiry", versions())
        .expect("expiration scan should succeed");
    assert_eq!(expired, vec![reserve_entry_id.clone()]);

    let record = ledger
        .reservations
        .get(&reserve_entry_id)
        .expect("reservation exists");
    assert_eq!(record.state, ReservationState::Expired);
    assert_eq!(ledger.balance_survival_micro(), 1_000);
}

#[test]
fn given_external_debit_when_applied_then_balance_decreases() {
    let mut ledger = SurvivalLedger::new(500);
    ledger
        .apply_external_debit(
            2,
            &ExternalDebitObservation {
                reference_id: "gw:r1".to_string(),
                cost_attribution_id: "cat:1".to_string(),
                action_id: None,
                cycle_id: Some(2),
                debit_survival_micro: 55,
            },
            versions(),
        )
        .expect("external debit should apply");

    assert_eq!(ledger.balance_survival_micro(), 445);
}
