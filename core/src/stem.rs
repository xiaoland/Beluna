use std::{collections::BTreeMap, sync::Arc};

use anyhow::Result;
use tokio::sync::{Mutex, mpsc};

use crate::{
    continuity::{ContinuityEngine, DispatchContext as ContinuityDispatchContext},
    cortex::{AffordanceCapability, CapabilityCatalog, Cortex},
    ledger::{DispatchContext as LedgerDispatchContext, LedgerDispatchTicket, LedgerStage},
    spine::{
        ActDispatchResult, EndpointBinding, EndpointCapabilityDescriptor, NativeFunctionEndpoint,
        RouteKey, Spine, SpineEvent, adapters::catalog_bridge::to_cortex_catalog,
        types::CostVector,
    },
    types::{Act, CognitionState, DispatchDecision, PhysicalState, Sense},
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
                    Sense::NewCapabilities(patch) => {
                        self.continuity.lock().await.apply_capability_patch(patch);
                    }
                    Sense::DropCapabilities(drop_patch) => {
                        self.continuity
                            .lock()
                            .await
                            .apply_capability_drop(drop_patch);
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
                    eprintln!("cortex failed for cycle {}: {}", self.cycle_id, err);
                    continue;
                }
            };

            self.continuity
                .lock()
                .await
                .persist_cognition_state(output.new_cognition_state.clone())?;
            eprintln!(
                "[stem] cycle={} generated_acts={}",
                self.cycle_id,
                output.acts.len()
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

    async fn compose_physical_state(&self, cycle_id: u64) -> Result<PhysicalState> {
        let ledger_snapshot = self.ledger.lock().await.physical_snapshot();
        let spine_catalog = to_cortex_catalog(&self.spine.capability_catalog_snapshot());
        let continuity_catalog = self.continuity.lock().await.capabilities_snapshot();
        let ledger_catalog = CapabilityCatalog::default();
        let merged =
            merge_capability_catalogs(cycle_id, spine_catalog, continuity_catalog, ledger_catalog);

        Ok(PhysicalState {
            cycle_id,
            ledger: ledger_snapshot,
            capabilities: merged,
        })
    }

    async fn dispatch_one_act_serial(
        &self,
        cycle_id: u64,
        seq_no: u64,
        act: Act,
        cognition_state: &CognitionState,
    ) -> Result<()> {
        eprintln!(
            "[stem] dispatch_attempt cycle={} seq={} act_id={} endpoint_id={} capability_id={}",
            cycle_id, seq_no, act.act_id, act.body_endpoint_name, act.capability_id
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
            eprintln!(
                "[stem] dispatch_break cycle={} seq={} reason=ledger_break",
                cycle_id, seq_no
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
            eprintln!(
                "[stem] dispatch_break cycle={} seq={} reason=continuity_break",
                cycle_id, seq_no
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
            Err(err) => map_spine_error_to_rejected_event(cycle_id, seq_no, &act, &ticket, err),
        };
        eprintln!(
            "[stem] dispatch_event cycle={} seq={} kind={}",
            cycle_id,
            seq_no,
            match &event {
                SpineEvent::ActApplied { .. } => "act_applied",
                SpineEvent::ActRejected { .. } => "act_rejected",
                SpineEvent::ActDeferred { .. } => "act_deferred",
            }
        );

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
        capability_instance_id: act.capability_instance_id.clone(),
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
        capability_instance_id: act.capability_instance_id.clone(),
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
            capability_instance_id: act.capability_instance_id.clone(),
            reserve_entry_id: ticket.reserve_entry_id.clone(),
            cost_attribution_id: ticket.cost_attribution_id.clone(),
            actual_cost_micro: act.requested_resources.survival_micro.max(0),
            reference_id,
        },
        ActDispatchResult::Rejected {
            reason_code,
            reference_id,
        } => SpineEvent::ActRejected {
            cycle_id,
            seq_no,
            act_id: act.act_id.clone(),
            capability_instance_id: act.capability_instance_id.clone(),
            reserve_entry_id: ticket.reserve_entry_id.clone(),
            cost_attribution_id: ticket.cost_attribution_id.clone(),
            reason_code,
            reference_id,
        },
    }
}

fn merge_capability_catalogs(
    cycle_id: u64,
    spine_catalog: CapabilityCatalog,
    continuity_catalog: CapabilityCatalog,
    ledger_catalog: CapabilityCatalog,
) -> CapabilityCatalog {
    let mut merged: BTreeMap<String, AffordanceCapability> = BTreeMap::new();

    for affordance in spine_catalog.affordances {
        merged.insert(affordance.endpoint_id.clone(), affordance);
    }
    for affordance in continuity_catalog.affordances {
        merged.insert(affordance.endpoint_id.clone(), affordance);
    }
    for affordance in ledger_catalog.affordances {
        merged.insert(affordance.endpoint_id.clone(), affordance);
    }

    CapabilityCatalog {
        version: format!(
            "stem:{}:{}:{}:{}",
            cycle_id, spine_catalog.version, continuity_catalog.version, ledger_catalog.version
        ),
        affordances: merged.into_values().collect(),
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
            EndpointCapabilityDescriptor {
                route: RouteKey {
                    endpoint_id: "placeholder".to_string(),
                    capability_id: "cap.core".to_string(),
                },
                payload_schema: serde_json::json!({"type":"object"}),
                max_payload_bytes: 16_384,
                default_cost: CostVector {
                    survival_micro: 250,
                    time_ms: 120,
                    io_units: 1,
                    token_units: 128,
                },
                metadata: Default::default(),
            },
            EndpointCapabilityDescriptor {
                route: RouteKey {
                    endpoint_id: "placeholder".to_string(),
                    capability_id: "cap.core.lite".to_string(),
                },
                payload_schema: serde_json::json!({"type":"object"}),
                max_payload_bytes: 16_384,
                default_cost: CostVector {
                    survival_micro: 250,
                    time_ms: 120,
                    io_units: 1,
                    token_units: 128,
                },
                metadata: Default::default(),
            },
            EndpointCapabilityDescriptor {
                route: RouteKey {
                    endpoint_id: "placeholder".to_string(),
                    capability_id: "cap.core.minimal".to_string(),
                },
                payload_schema: serde_json::json!({"type":"object"}),
                max_payload_bytes: 16_384,
                default_cost: CostVector {
                    survival_micro: 250,
                    time_ms: 120,
                    io_units: 1,
                    token_units: 128,
                },
                metadata: Default::default(),
            },
        ],
    )?;

    let _execute = spine.add_endpoint(
        "execute.tool",
        EndpointBinding::Inline(native_endpoint),
        vec![EndpointCapabilityDescriptor {
            route: RouteKey {
                endpoint_id: "placeholder".to_string(),
                capability_id: "cap.core".to_string(),
            },
            payload_schema: serde_json::json!({"type":"object"}),
            max_payload_bytes: 16_384,
            default_cost: CostVector {
                survival_micro: 400,
                time_ms: 200,
                io_units: 2,
                token_units: 256,
            },
            metadata: Default::default(),
        }],
    )?;

    Ok(())
}
