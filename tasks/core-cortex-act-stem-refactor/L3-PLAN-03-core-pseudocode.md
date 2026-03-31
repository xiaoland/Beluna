# L3 Plan 03 - Core Pseudocode
- Task Name: `core-cortex-act-stem-refactor`
- Stage: `L3`
- Focus: implementation pseudocode for stem and serial act dispatch
- Status: `DRAFT_FOR_APPROVAL`

## 1) `main` Boot Pseudocode

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let config = load_config()?;

    let (sense_tx, sense_rx) = mpsc::channel::<Sense>(config.loop.sense_queue_capacity);
    let ingress = SenseIngress::new(sense_tx);

    let spine = build_spine(config.clone(), ingress.clone()).await?;
    let ledger = build_ledger();
    let continuity = build_continuity();
    let cortex = build_cortex(config.clone());

    let stem = StemRuntime::new(cortex, continuity, ledger, spine, sense_rx);
    let stem_task = tokio::spawn(async move { stem.run().await });

    wait_for_sigint_or_sigterm().await;

    ingress.close_gate();
    ingress.send_sleep_blocking().await?;

    stem_task.await??;
    flush_and_cleanup().await?;
    Ok(())
}
```

## 2) Stem Loop Pseudocode

```rust
impl StemRuntime {
    pub async fn run(mut self) -> Result<(), StemError> {
        loop {
            let sense = match self.sense_rx.recv().await {
                Some(sense) => sense,
                None => break,
            };

            if matches!(sense, Sense::Sleep) {
                break;
            }

            self.apply_control_sense(&sense).await?;

            self.cycle_id = self.cycle_id.saturating_add(1);

            let cognition = self.continuity.lock().await.cognition_state_snapshot();
            let physical = self.compose_physical_state(self.cycle_id).await?;

            let output = self
                .cortex
                .cortex(&sense, &physical, &cognition)
                .await?;

            self.continuity
                .lock()
                .await
                .persist_cognition_state(output.new_cognition_state.clone())?;

            for (index, act) in output.acts.into_iter().enumerate() {
                self.dispatch_act_serial(
                    self.cycle_id,
                    (index as u64) + 1,
                    act,
                    &output.new_cognition_state,
                )
                .await?;
            }
        }
        Ok(())
    }
}
```

## 3) Control Sense Pseudocode

```rust
async fn apply_control_sense(&self, sense: &Sense) -> Result<(), StemError> {
    match sense {
        Sense::NewCapabilities(patch) => {
            self.continuity.lock().await.apply_capability_patch(patch);
        }
        Sense::DropCapabilities(drop) => {
            self.continuity.lock().await.apply_capability_drop(drop);
        }
        Sense::Domain(_) | Sense::Sleep => {}
    }
    Ok(())
}
```

## 4) Physical State Compose Pseudocode

```rust
async fn compose_physical_state(&self, cycle_id: u64) -> Result<PhysicalState, StemError> {
    let ledger_snapshot = self.ledger.lock().await.physical_snapshot();

    let spine_catalog = to_cortex_catalog(&self.spine.capability_catalog_snapshot());
    let continuity_catalog = self.continuity.lock().await.capabilities_snapshot();
    let ledger_catalog = empty_or_configured_ledger_catalog();

    let merged = merge_catalogs(spine_catalog, continuity_catalog, ledger_catalog);

    Ok(PhysicalState {
        cycle_id,
        ledger: ledger_snapshot,
        capabilities: merged,
    })
}
```

## 5) Serial Dispatch Pseudocode

```rust
async fn dispatch_act_serial(
    &self,
    cycle_id: u64,
    seq_no: u64,
    act: Act,
    cognition: &CognitionState,
) -> Result<(), StemError> {
    let ctx = DispatchContext { cycle_id, act_seq_no: seq_no };

    let (ledger_decision, ticket_opt) = self.ledger.lock().await.pre_dispatch(&act, &ctx)?;
    if matches!(ledger_decision, DispatchDecision::Break) {
        return Ok(());
    }

    let ticket = ticket_opt.ok_or(StemError::Internal("missing ticket"))?;

    let continuity_decision = self
        .continuity
        .lock()
        .await
        .pre_dispatch(&act, cognition, &ctx)?;
    if matches!(continuity_decision, DispatchDecision::Break) {
        let event = synthetic_break_rejected_event(&act, &ticket, &ctx, "continuity_break");
        self.ledger.lock().await.settle_from_spine(&ticket, &event, &ctx)?;
        self.continuity.lock().await.on_spine_event(&act, &event, &ctx)?;
        return Ok(());
    }

    let req = ActDispatchRequest {
        cycle_id,
        seq_no,
        act: act.clone(),
        reserve_entry_id: ticket.reserve_entry_id.clone(),
        cost_attribution_id: ticket.cost_attribution_id.clone(),
    };

    let event = match self.spine.dispatch_act(req).await {
        Ok(event) => event,
        Err(err) => map_spine_error_to_rejected_event(&act, &ticket, &ctx, err),
    };

    self.ledger.lock().await.settle_from_spine(&ticket, &event, &ctx)?;
    self.continuity.lock().await.on_spine_event(&act, &event, &ctx)?;
    Ok(())
}
```

## 6) Deterministic Helpers
1. `cost_attribution_id = hash("cat", cycle_id, act.act_id)`.
2. synthetic rejection `reference_id`:
   - `stem:break:{cycle_id}:{seq_no}:{act_id}`,
   - `stem:spine_error:{cycle_id}:{seq_no}:{act_id}`.
3. catalog merge uses deterministic key sort and stable overwrite order.

Status: `READY_FOR_EXECUTION`
