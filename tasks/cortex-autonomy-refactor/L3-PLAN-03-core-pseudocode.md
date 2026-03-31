# L3-03 Core Pseudocode
- Task: `cortex-autonomy-refactor`
- Stage: `L3`

## 1) Stem Loop
```rust
async fn run(mut self) -> Result<()> {
    let mut mode = StemMode::Active;
    let mut tick = tokio::time::interval(Duration::from_millis(self.tick_interval_ms));
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        match mode {
            StemMode::Active => {
                tick.tick().await;

                let senses = self.drain_senses_nonblocking();
                if senses.iter().any(|s| matches!(s, Sense::Hibernate)) {
                    break;
                }

                self.apply_capability_control_senses(&senses).await;
                let domain_senses = extract_domain_senses(&senses);

                let cognition_state = self.continuity.lock().await.cognition_state_snapshot();
                let physical_state = self.compose_physical_state().await?;
                let output = self
                    .cortex
                    .cortex(&domain_senses, &physical_state, &cognition_state)
                    .await
                    .unwrap_or_else(|_| noop_output(cognition_state));

                self.continuity
                    .lock()
                    .await
                    .persist_cognition_state(output.new_cognition_state.clone())?;

                for act in output.acts {
                    if let Some(sleep_deadline) = self.try_handle_sleep_act(&act) {
                        mode = StemMode::SleepingUntil(Some(sleep_deadline));
                        continue;
                    }

                    let ctx = DispatchContext::from_cycle(self.cycle_id);
                    if self.continuity.lock().await.on_act(&act, &ctx)? == DispatchDecision::Break {
                        continue;
                    }

                    if self.spine.on_act(act.clone()).await? == DispatchDecision::Break {
                        continue;
                    }
                }
            }
            StemMode::SleepingUntil(deadline_opt) => {
                let wake = self.wait_sleep_wake(deadline_opt).await;
                match wake {
                    WakeReason::Hibernate => break,
                    WakeReason::SenseArrived | WakeReason::Timeout => {
                        mode = StemMode::Active;
                        self.run_immediate_wake_cycle().await?;
                    }
                }
            }
        }

        self.cycle_id = self.cycle_id.saturating_add(1);
    }

    Ok(())
}
```

## 2) Sleep Act Interception
```rust
fn try_handle_sleep_act(&mut self, act: &Act) -> Option<Instant> {
    if act.endpoint_id != "core.control" || act.neural_signal_descriptor_id != "sleep" {
        return None;
    }

    let seconds = act.payload.get("seconds")?.as_u64()?;
    Some(Instant::now() + Duration::from_secs(seconds.max(1)))
}
```

## 3) Cortex Single-Cycle Pipeline
```rust
async fn cortex(
    &self,
    senses: &[Sense],
    physical: &PhysicalState,
    cognition: &CognitionState,
) -> Result<CortexOutput, CortexError> {
    let (senses_md, act_catalog_md, goal_tree_md) = tokio::join!(
        build_senses_section(...),
        build_act_descriptor_section(...),
        build_goal_tree_section_user_partition_only(...),
    );

    let l1_memory_passthrough = render_l1_memory_section(&cognition.l1_memory.entries);

    let input_ir = build_input_ir(senses_md?, act_catalog_md?, goal_tree_md?, l1_memory_passthrough);
    let primary_output = run_primary(input_ir).await?;
    let sections = parse_output_ir(primary_output)?;

    let (acts, goal_ops, mem_ops) = tokio::join!(
        run_acts_helper(sections.acts),
        run_goal_tree_patch_helper(sections.goal_tree_patch),
        run_l1_memory_patch_helper(sections.l1_memory_patch),
    );

    let acts = materialize_acts_or_empty(acts);
    let goal_ops = goal_ops.unwrap_or_default();
    let mem_ops = mem_ops.unwrap_or_default();

    let new_cognition = apply_patches_in_cortex(cognition, goal_ops, mem_ops)?;
    Ok(CortexOutput { acts, new_cognition_state: new_cognition })
}
```

## 4) Continuity Persist Guardrail
```rust
fn persist_cognition_state(&mut self, candidate: CognitionState) -> Result<(), ContinuityError> {
    ensure_root_partition_immutable(&candidate.goal_tree.root_partition, ROOT_PARTITION_CONST)?;
    ensure_user_tree_valid(&candidate.goal_tree.user_partition)?;
    ensure_l1_memory_is_string_array(&candidate.l1_memory)?;

    self.persistence.write_atomic_json(&candidate)?;
    self.state = candidate;
    Ok(())
}
```

## 5) Spine Middleware Entry
```rust
async fn on_act(&self, act: Act) -> Result<DispatchDecision, SpineError> {
    match self.dispatch_act(act.clone()).await {
        Ok(_ok) => Ok(DispatchDecision::Continue),
        Err(err) => {
            self.emit_dispatch_failure_sense(act, err).await;
            Ok(DispatchDecision::Break)
        }
    }
}
```

