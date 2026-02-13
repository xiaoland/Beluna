# L2 Plan 03 - Stem And Dispatch Algorithms
- Task Name: `core-cortex-act-stem-refactor`
- Stage: `L2`
- Focus: executable algorithms for Stem loop and serial pipeline
- Status: `DRAFT_FOR_APPROVAL`

## 1) Stem Runtime Data

```rust
pub struct StemRuntime {
    pub cycle_id: u64,
    pub cortex: Arc<dyn CortexPort>,
    pub continuity: Arc<tokio::sync::Mutex<dyn ContinuityStagePort>>,
    pub ledger: Arc<tokio::sync::Mutex<dyn LedgerStagePort>>,
    pub spine: Arc<dyn SpineStagePort>,
    pub sense_rx: tokio::sync::mpsc::Receiver<Sense>,
}
```

No `ActQueue` exists. Act dispatch is inline and serial.

## 2) Main Stem Loop

```rust
loop {
    let Some(sense) = sense_rx.recv().await else {
        break; // queue closed
    };

    match &sense {
        Sense::Sleep => break, // do not call cortex
        Sense::NewCapabilities(patch) => {
            continuity.lock().await.apply_capability_patch(patch);
        }
        Sense::DropCapabilities(drop) => {
            continuity.lock().await.apply_capability_drop(drop);
        }
        Sense::Domain(_) => {}
    }

    cycle_id = cycle_id.saturating_add(1);

    let cognition_state = continuity.lock().await.cognition_state_snapshot();
    let physical_state = compose_physical_state_for_cycle(cycle_id).await;

    let output = cortex.cortex(&sense, &physical_state, &cognition_state).await?;

    continuity
        .lock()
        .await
        .persist_cognition_state(output.new_cognition_state.clone())?;

    for (index, act) in output.acts.into_iter().enumerate() {
        dispatch_one_act_serial(
            cycle_id,
            (index as u64) + 1,
            act,
            &output.new_cognition_state,
        )
        .await?;
    }
}
```

Rule: `new_capabilities` and `drop_capabilities` must take effect before same-cycle Cortex call.

## 3) Physical State Composition Algorithm

```rust
async fn compose_physical_state_for_cycle(cycle_id: u64) -> PhysicalState {
    let ledger_snapshot = ledger.lock().await.physical_snapshot();

    let spine_caps = to_cortex_catalog(&spine.capability_catalog_snapshot());
    let continuity_caps = continuity.lock().await.capabilities_snapshot();
    let ledger_caps = ledger_capability_catalog_snapshot(); // default empty for MVP unless configured

    let merged = merge_capability_catalogs(spine_caps, continuity_caps, ledger_caps);

    PhysicalState {
        cycle_id,
        ledger: ledger_snapshot,
        capabilities: merged,
    }
}
```

`merge_capability_catalogs` must be deterministic:
1. deterministic key order (`BTreeMap` by endpoint_id + capability_id),
2. later source overlays earlier source by key,
3. version string derived deterministically from cycle and source versions.

## 4) Serial Act Dispatch Algorithm

```rust
async fn dispatch_one_act_serial(
    cycle_id: u64,
    seq_no: u64,
    act: Act,
    cognition_state: &CognitionState,
) -> Result<(), StemError> {
    let ctx = DispatchContext { cycle_id, act_seq_no: seq_no };

    // Stage 1: Ledger pre-dispatch
    let (ledger_decision, ticket_opt) = ledger.lock().await.pre_dispatch(&act, &ctx)?;
    if ledger_decision == DispatchDecision::Break {
        return Ok(());
    }
    let ticket = ticket_opt.expect("ledger pre-dispatch continue must yield ticket");

    // Stage 2: Continuity gate
    let continuity_decision = continuity
        .lock()
        .await
        .pre_dispatch(&act, cognition_state, &ctx)?;
    if continuity_decision == DispatchDecision::Break {
        // deterministic rollback reference
        let event = synthetic_continuity_break_event(&act, &ticket, &ctx);
        ledger.lock().await.settle_from_spine(&ticket, &event, &ctx)?;
        continuity.lock().await.on_spine_event(&act, &event, &ctx)?;
        return Ok(());
    }

    // Stage 3: Spine dispatch
    let request = ActDispatchRequest {
        cycle_id,
        seq_no,
        act: act.clone(),
        reserve_entry_id: ticket.reserve_entry_id.clone(),
        cost_attribution_id: ticket.cost_attribution_id.clone(),
    };

    let spine_event = match spine.dispatch_act(request).await {
        Ok(event) => event,
        Err(err) => map_spine_error_to_rejected_event(&act, &ticket, &ctx, err),
    };

    ledger.lock().await.settle_from_spine(&ticket, &spine_event, &ctx)?;
    continuity
        .lock()
        .await
        .on_spine_event(&act, &spine_event, &ctx)?;

    Ok(())
}
```

Key contract:
1. `Break` applies only to current `Act`.
2. Next act is still processed.
3. Settlement path executes for every ledger-continued act.

## 5) Control-Sense Handling Rules
1. `Sense::Sleep`:
   - stop stem loop immediately,
   - no Cortex call,
   - no Act dispatch.

2. `Sense::NewCapabilities`:
   - apply patch first,
   - then call Cortex in same cycle.

3. `Sense::DropCapabilities`:
   - apply drop first,
   - then call Cortex in same cycle.

## 6) Determinism Constraints
1. Cycle id increments once per processed non-sleep sense.
2. Act dispatch order is output order from Cortex (`index + 1` seq_no).
3. All map/set operations use deterministic ordering structures.
4. Synthetic break/error events use deterministic reference id templates.

## 7) Failure Handling
1. Cortex error:
   - log error + continue loop,
   - no acts dispatched for that cycle.
2. Continuity patch apply error:
   - convert to diagnostic sense (optional),
   - continue loop.
3. Spine dispatch error:
   - map to deterministic `ActionRejected` event,
   - run normal settlement path.
4. Ledger settle/refund conflict:
   - treat as hard runtime error and stop loop (invariant breach).

## 8) L2-03 Exit Conditions
1. stem loop behavior is fully specified,
2. serial dispatch and break semantics are unambiguous,
3. control senses and immediate capability effects are explicit,
4. deterministic error and settlement handling are defined.

Status: `READY_FOR_REVIEW`
