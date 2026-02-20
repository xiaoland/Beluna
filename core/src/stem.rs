use std::{collections::BTreeMap, sync::Arc};

use anyhow::Result;
use tokio::sync::{Mutex, mpsc};

use crate::{
    continuity::{ContinuityEngine, DispatchContext as ContinuityDispatchContext},
    cortex::Cortex,
    ledger::{DispatchContext as LedgerDispatchContext, LedgerDispatchTicket, LedgerStage},
    spine::{ActDispatchResult, EndpointBinding, NativeFunctionEndpoint, Spine, SpineEvent},
    types::{
        Act, CognitionState, DispatchDecision, NeuralSignalDescriptor,
        NeuralSignalDescriptorCatalog, NeuralSignalDescriptorRouteKey, NeuralSignalType,
        PhysicalState, Sense,
    },
};

pub struct Stem {
    cycle_id: u64,
    cortex: Arc<Cortex>,
    continuity: Arc<Mutex<ContinuityEngine>>,
    ledger: Arc<Mutex<LedgerStage>>,
    spine: Arc<Spine>,
    sense_rx: mpsc::Receiver<Sense>,
}

impl Stem {
    pub fn new(
        cortex: Arc<Cortex>,
        continuity: Arc<Mutex<ContinuityEngine>>,
        ledger: Arc<Mutex<LedgerStage>>,
        spine: Arc<Spine>,
        sense_rx: mpsc::Receiver<Sense>,
    ) -> Self {
        Self {
            cycle_id: 0,
            cortex,
            continuity,
            ledger,
            spine,
            sense_rx,
        }
    }

    #[tracing::instrument(name = "stem_run", target = "stem", skip(self))]
    pub async fn run(mut self) -> Result<()> {
        loop {
            let Some(first_sense) = self.sense_rx.recv().await else {
                break;
            };

            if matches!(first_sense, Sense::Sleep) {
                break;
            }

            let mut sense_batch = vec![first_sense];
            let mut stop_after_cycle = false;
            while let Ok(next_sense) = self.sense_rx.try_recv() {
                if matches!(next_sense, Sense::Sleep) {
                    stop_after_cycle = true;
                    break;
                }
                sense_batch.push(next_sense);
            }

            for sense in &sense_batch {
                match sense {
                    Sense::NewNeuralSignalDescriptors(patch) => {
                        self.continuity
                            .lock()
                            .await
                            .apply_neural_signal_descriptor_patch(patch);
                    }
                    Sense::DropNeuralSignalDescriptors(drop_patch) => {
                        self.continuity
                            .lock()
                            .await
                            .apply_neural_signal_descriptor_drop(drop_patch);
                    }
                    Sense::Domain(_) | Sense::Sleep => {}
                }
            }

            self.cycle_id = self.cycle_id.saturating_add(1);

            let cognition_state = self.continuity.lock().await.cognition_state_snapshot();
            let physical_state = self.compose_physical_state(self.cycle_id).await?;

            let output = match self
                .cortex
                .cortex(&sense_batch, &physical_state, &cognition_state)
                .await
            {
                Ok(output) => output,
                Err(err) => {
                    tracing::warn!(
                        target: "stem",
                        cycle_id = self.cycle_id,
                        error = %err,
                        "cortex_failed_for_cycle"
                    );
                    continue;
                }
            };

            self.continuity
                .lock()
                .await
                .persist_cognition_state(output.new_cognition_state.clone())?;
            tracing::debug!(
                target: "stem",
                cycle_id = self.cycle_id,
                generated_acts = output.acts.len(),
                "cycle_generated_acts"
            );

            for (index, act) in output.acts.into_iter().enumerate() {
                self.dispatch_one_act_serial(
                    self.cycle_id,
                    (index as u64) + 1,
                    act,
                    &output.new_cognition_state,
                )
                .await?;
            }

            if stop_after_cycle {
                break;
            }
        }

        Ok(())
    }

    #[tracing::instrument(
        name = "stem_compose_physical_state",
        target = "stem",
        skip(self),
        fields(cycle_id = cycle_id)
    )]
    async fn compose_physical_state(&self, cycle_id: u64) -> Result<PhysicalState> {
        let ledger_snapshot = self.ledger.lock().await.physical_snapshot();
        let spine_catalog = self.spine.neural_signal_descriptor_catalog_snapshot();
        let continuity_catalog = self
            .continuity
            .lock()
            .await
            .neural_signal_descriptor_snapshot();
        let ledger_catalog = NeuralSignalDescriptorCatalog::default();
        let merged = merge_neural_signal_descriptor_catalogs(
            cycle_id,
            spine_catalog,
            continuity_catalog,
            ledger_catalog,
        );

        Ok(PhysicalState {
            cycle_id,
            ledger: ledger_snapshot,
            capabilities: merged,
        })
    }

    #[tracing::instrument(
        name = "stem_dispatch_one_act_serial",
        target = "stem",
        skip(self, act, cognition_state),
        fields(
            cycle_id = cycle_id,
            seq_no = seq_no,
            act_id = %act.act_id,
            endpoint_id = %act.endpoint_id,
            neural_signal_descriptor_id = %act.neural_signal_descriptor_id
        )
    )]
    async fn dispatch_one_act_serial(
        &self,
        cycle_id: u64,
        seq_no: u64,
        act: Act,
        cognition_state: &CognitionState,
    ) -> Result<()> {
        tracing::debug!(
            target: "stem",
            cycle_id = cycle_id,
            seq_no = seq_no,
            act_id = %act.act_id,
            endpoint_id = %act.endpoint_id,
            neural_signal_descriptor_id = %act.neural_signal_descriptor_id,
            "dispatch_attempt"
        );
        let ledger_ctx = LedgerDispatchContext {
            cycle_id,
            act_seq_no: seq_no,
        };
        let continuity_ctx = ContinuityDispatchContext {
            cycle_id,
            act_seq_no: seq_no,
        };

        let (ledger_decision, ticket_opt) =
            self.ledger.lock().await.pre_dispatch(&act, &ledger_ctx)?;
        if matches!(ledger_decision, DispatchDecision::Break) {
            tracing::info!(
                target: "stem",
                cycle_id = cycle_id,
                seq_no = seq_no,
                reason = "ledger_break",
                "dispatch_break"
            );
            return Ok(());
        }
        let ticket = ticket_opt.expect("ledger continue must return ticket");

        let continuity_decision =
            self.continuity
                .lock()
                .await
                .pre_dispatch(&act, cognition_state, &continuity_ctx)?;
        if matches!(continuity_decision, DispatchDecision::Break) {
            tracing::info!(
                target: "stem",
                cycle_id = cycle_id,
                seq_no = seq_no,
                reason = "continuity_break",
                "dispatch_break"
            );
            let event = synthetic_continuity_break_event(cycle_id, seq_no, &act, &ticket);
            self.ledger
                .lock()
                .await
                .settle_from_spine(&ticket, &event, &ledger_ctx)?;
            self.continuity
                .lock()
                .await
                .on_spine_event(&act, &event, &continuity_ctx)?;
            return Ok(());
        }

        let event = match self.spine.dispatch_act(act.clone()).await {
            Ok(dispatch_result) => {
                map_dispatch_result_to_spine_event(cycle_id, seq_no, &act, &ticket, dispatch_result)
            }
            Err(err) => {
                tracing::warn!(
                    target: "stem.dispatch",
                    cycle_id = cycle_id,
                    seq_no = seq_no,
                    act_id = %act.act_id,
                    endpoint_id = %act.endpoint_id,
                    neural_signal_descriptor_id = %act.neural_signal_descriptor_id,
                    error = %err,
                    "dispatch_failed_with_spine_error"
                );
                map_spine_error_to_rejected_event(cycle_id, seq_no, &act, &ticket, err)
            }
        };
        match &event {
            SpineEvent::ActApplied { reference_id, .. } => {
                tracing::info!(
                    target: "stem.dispatch",
                    cycle_id = cycle_id,
                    seq_no = seq_no,
                    act_id = %act.act_id,
                    endpoint_id = %act.endpoint_id,
                    neural_signal_descriptor_id = %act.neural_signal_descriptor_id,
                    reference_id = %reference_id,
                    "dispatch_applied"
                );
            }
            SpineEvent::ActRejected {
                reason_code,
                reference_id,
                ..
            } => {
                tracing::warn!(
                    target: "stem.dispatch",
                    cycle_id = cycle_id,
                    seq_no = seq_no,
                    act_id = %act.act_id,
                    endpoint_id = %act.endpoint_id,
                    neural_signal_descriptor_id = %act.neural_signal_descriptor_id,
                    reason_code = %reason_code,
                    reference_id = %reference_id,
                    "dispatch_rejected"
                );
            }
            SpineEvent::ActDeferred {
                reason_code,
                reference_id,
                ..
            } => {
                tracing::warn!(
                    target: "stem.dispatch",
                    cycle_id = cycle_id,
                    seq_no = seq_no,
                    act_id = %act.act_id,
                    endpoint_id = %act.endpoint_id,
                    neural_signal_descriptor_id = %act.neural_signal_descriptor_id,
                    reason_code = %reason_code,
                    reference_id = %reference_id,
                    "dispatch_deferred"
                );
            }
        }

        self.ledger
            .lock()
            .await
            .settle_from_spine(&ticket, &event, &ledger_ctx)?;
        self.continuity
            .lock()
            .await
            .on_spine_event(&act, &event, &continuity_ctx)?;

        Ok(())
    }
}

fn synthetic_continuity_break_event(
    cycle_id: u64,
    seq_no: u64,
    act: &Act,
    ticket: &LedgerDispatchTicket,
) -> SpineEvent {
    SpineEvent::ActRejected {
        cycle_id,
        seq_no,
        act_id: act.act_id.clone(),
        reserve_entry_id: ticket.reserve_entry_id.clone(),
        cost_attribution_id: ticket.cost_attribution_id.clone(),
        reason_code: "continuity_break".to_string(),
        reference_id: format!("stem:break:{}:{}:{}", cycle_id, seq_no, act.act_id),
    }
}

fn map_spine_error_to_rejected_event(
    cycle_id: u64,
    seq_no: u64,
    act: &Act,
    ticket: &LedgerDispatchTicket,
    err: crate::spine::SpineError,
) -> SpineEvent {
    SpineEvent::ActRejected {
        cycle_id,
        seq_no,
        act_id: act.act_id.clone(),
        reserve_entry_id: ticket.reserve_entry_id.clone(),
        cost_attribution_id: ticket.cost_attribution_id.clone(),
        reason_code: "spine_error".to_string(),
        reference_id: format!(
            "stem:spine_error:{}:{}:{}:{}",
            cycle_id, seq_no, act.act_id, err.kind as u8
        ),
    }
}

fn map_dispatch_result_to_spine_event(
    cycle_id: u64,
    seq_no: u64,
    act: &Act,
    ticket: &LedgerDispatchTicket,
    dispatch_result: ActDispatchResult,
) -> SpineEvent {
    match dispatch_result {
        ActDispatchResult::Acknowledged { reference_id } => SpineEvent::ActApplied {
            cycle_id,
            seq_no,
            act_id: act.act_id.clone(),
            reserve_entry_id: ticket.reserve_entry_id.clone(),
            cost_attribution_id: ticket.cost_attribution_id.clone(),
            actual_cost_micro: 0,
            reference_id,
        },
        ActDispatchResult::Rejected {
            reason_code,
            reference_id,
        } => SpineEvent::ActRejected {
            cycle_id,
            seq_no,
            act_id: act.act_id.clone(),
            reserve_entry_id: ticket.reserve_entry_id.clone(),
            cost_attribution_id: ticket.cost_attribution_id.clone(),
            reason_code,
            reference_id,
        },
    }
}

fn merge_neural_signal_descriptor_catalogs(
    cycle_id: u64,
    spine_catalog: NeuralSignalDescriptorCatalog,
    continuity_catalog: NeuralSignalDescriptorCatalog,
    ledger_catalog: NeuralSignalDescriptorCatalog,
) -> NeuralSignalDescriptorCatalog {
    let spine_version = spine_catalog.version.clone();
    let continuity_version = continuity_catalog.version.clone();
    let ledger_version = ledger_catalog.version.clone();

    let mut merged: BTreeMap<NeuralSignalDescriptorRouteKey, NeuralSignalDescriptor> =
        BTreeMap::new();

    for descriptor in spine_catalog.entries {
        merged.insert(
            NeuralSignalDescriptorRouteKey {
                r#type: descriptor.r#type,
                endpoint_id: descriptor.endpoint_id.clone(),
                neural_signal_descriptor_id: descriptor.neural_signal_descriptor_id.clone(),
            },
            descriptor,
        );
    }
    for descriptor in continuity_catalog.entries {
        merged.insert(
            NeuralSignalDescriptorRouteKey {
                r#type: descriptor.r#type,
                endpoint_id: descriptor.endpoint_id.clone(),
                neural_signal_descriptor_id: descriptor.neural_signal_descriptor_id.clone(),
            },
            descriptor,
        );
    }
    for descriptor in ledger_catalog.entries {
        merged.insert(
            NeuralSignalDescriptorRouteKey {
                r#type: descriptor.r#type,
                endpoint_id: descriptor.endpoint_id.clone(),
                neural_signal_descriptor_id: descriptor.neural_signal_descriptor_id.clone(),
            },
            descriptor,
        );
    }

    let mut entries = merged.into_values().collect::<Vec<_>>();
    entries.sort_by(|lhs, rhs| {
        lhs.r#type
            .cmp(&rhs.r#type)
            .then_with(|| lhs.endpoint_id.cmp(&rhs.endpoint_id))
            .then_with(|| {
                lhs.neural_signal_descriptor_id
                    .cmp(&rhs.neural_signal_descriptor_id)
            })
    });

    NeuralSignalDescriptorCatalog {
        version: format!(
            "stem:{}:{}:{}:{}",
            cycle_id, spine_version, continuity_version, ledger_version
        ),
        entries,
    }
}

pub fn register_default_native_endpoints(spine: Arc<Spine>) -> Result<()> {
    let native_endpoint = Arc::new(NativeFunctionEndpoint::new(Arc::new(|act| {
        Ok(crate::spine::types::ActDispatchResult::Acknowledged {
            reference_id: format!("native:settle:{}", act.act_id),
        })
    })));

    let _deliberate = spine.add_endpoint(
        "deliberate.plan",
        EndpointBinding::Inline(native_endpoint.clone()),
        vec![
            NeuralSignalDescriptor {
                r#type: NeuralSignalType::Act,
                endpoint_id: "placeholder".to_string(),
                neural_signal_descriptor_id: "cap.core".to_string(),
                payload_schema: serde_json::json!({"type":"object"}),
            },
            NeuralSignalDescriptor {
                r#type: NeuralSignalType::Act,
                endpoint_id: "placeholder".to_string(),
                neural_signal_descriptor_id: "cap.core.lite".to_string(),
                payload_schema: serde_json::json!({"type":"object"}),
            },
            NeuralSignalDescriptor {
                r#type: NeuralSignalType::Act,
                endpoint_id: "placeholder".to_string(),
                neural_signal_descriptor_id: "cap.core.minimal".to_string(),
                payload_schema: serde_json::json!({"type":"object"}),
            },
        ],
    )?;

    let _execute = spine.add_endpoint(
        "execute.tool",
        EndpointBinding::Inline(native_endpoint),
        vec![NeuralSignalDescriptor {
            r#type: NeuralSignalType::Act,
            endpoint_id: "placeholder".to_string(),
            neural_signal_descriptor_id: "cap.core".to_string(),
            payload_schema: serde_json::json!({"type":"object"}),
        }],
    )?;

    Ok(())
}
