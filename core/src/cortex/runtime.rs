use std::{
    collections::{BTreeMap, HashMap},
    future::Future,
    pin::Pin,
    sync::Arc,
    time::Instant,
};

use serde::{Deserialize, Serialize};
use tokio::{
    sync::RwLock,
    time::{Duration, timeout},
};

use crate::{
    ai_gateway::{
        gateway::AIGateway,
        types_chat::{
            BelunaContentPart, BelunaMessage, BelunaRole, ChatRequest, ChatResponse, OutputMode,
            RequestLimitOverrides, ToolChoice,
        },
    },
    config::CortexHelperRoutesConfig,
    cortex::{
        clamp::derive_act_id,
        cognition::{CognitionState, GoalNode, GoalTreePatchOp},
        error::{CortexError, extractor_failed, filler_failed, invalid_input, primary_failed},
        helpers_input,
        helpers_output::{
            acts_json_schema, apply_cognition_patches, goal_tree_patch_ops_json_schema,
            l1_memory_flush_json_schema, materialize_acts, parse_acts_helper_output,
            parse_goal_tree_patch_helper_output, parse_l1_memory_flush_helper_output,
        },
        ir, prompts,
        testing::{
            ActDescriptorHelperRequest as TestActDescriptorHelperRequest,
            ActsHelperRequest as TestActsHelperRequest,
            GoalTreeHelperRequest as TestGoalTreeHelperRequest,
            GoalTreePatchHelperRequest as TestGoalTreePatchRequest,
            L1MemoryFlushHelperRequest as TestL1MemoryFlushRequest,
            PrimaryRequest as TestPrimaryRequest, SenseHelperRequest as TestSenseHelperRequest,
            TestHooks,
        },
        types::{
            ActDraft, ActsHelperOutput, CortexOutput, GoalTreePatchHelperOutput,
            L1MemoryFlushHelperOutput, ReactionLimits,
        },
    },
    observability::metrics as observability_metrics,
    types::{NeuralSignalDescriptor, PhysicalState, Sense},
};

#[derive(Debug, Clone)]
pub enum CortexTelemetryEvent {
    ReactionStarted { cycle_id: u64 },
    StageFailed { cycle_id: u64, stage: &'static str },
    ReactionCompleted { cycle_id: u64, act_count: usize },
    NoopFallback { cycle_id: u64, reason: &'static str },
}

pub type CortexTelemetryHook = Arc<dyn Fn(CortexTelemetryEvent) + Send + Sync>;

type SenseSectionFuture = Pin<Box<dyn Future<Output = Result<String, CortexError>> + Send>>;
type ActDescriptorSectionFuture = Pin<Box<dyn Future<Output = Result<String, CortexError>> + Send>>;
type GoalTreeSectionFuture = Pin<Box<dyn Future<Output = Result<String, CortexError>> + Send>>;
type PrimaryFuture = Pin<Box<dyn Future<Output = Result<String, CortexError>> + Send>>;
type ActsHelperFuture = Pin<Box<dyn Future<Output = Result<ActsHelperOutput, CortexError>> + Send>>;
type GoalTreePatchHelperFuture =
    Pin<Box<dyn Future<Output = Result<GoalTreePatchHelperOutput, CortexError>> + Send>>;
type L1MemoryFlushHelperFuture =
    Pin<Box<dyn Future<Output = Result<L1MemoryFlushHelperOutput, CortexError>> + Send>>;

#[derive(Clone)]
pub struct Cortex {
    gateway: Option<Arc<AIGateway>>,
    helper_routes: CortexHelperRoutesConfig,
    hooks: Option<TestHooks>,
    act_descriptor_cache: Arc<RwLock<HashMap<String, String>>>,
    goal_tree_cache: Arc<RwLock<HashMap<String, String>>>,
    telemetry_hook: Option<CortexTelemetryHook>,
    limits: ReactionLimits,
}

#[derive(Clone, Copy)]
enum CognitionOrgan {
    Primary,
    Sense,
    ActDescriptor,
    GoalTree,
    Acts,
    GoalTreePatch,
    L1MemoryFlush,
}

impl CognitionOrgan {
    fn stage(self) -> &'static str {
        match self {
            Self::Primary => "primary",
            Self::Sense => "sense_helper",
            Self::ActDescriptor => "act_descriptor_helper",
            Self::GoalTree => "goal_tree_helper",
            Self::Acts => "acts_helper",
            Self::GoalTreePatch => "goal_tree_patch_helper",
            Self::L1MemoryFlush => "l1_memory_flush_helper",
        }
    }
}

impl Cortex {
    pub fn from_config(
        config: &crate::config::CortexRuntimeConfig,
        gateway: Arc<AIGateway>,
        telemetry_hook: Option<CortexTelemetryHook>,
    ) -> Self {
        Self {
            gateway: Some(gateway),
            helper_routes: config.helper_routes.clone(),
            hooks: None,
            act_descriptor_cache: Arc::new(RwLock::new(HashMap::new())),
            goal_tree_cache: Arc::new(RwLock::new(HashMap::new())),
            telemetry_hook,
            limits: config.default_limits.clone(),
        }
    }

    pub(crate) fn for_test_with_hooks(hooks: TestHooks, limits: ReactionLimits) -> Self {
        Self {
            gateway: None,
            helper_routes: CortexHelperRoutesConfig::default(),
            hooks: Some(hooks),
            act_descriptor_cache: Arc::new(RwLock::new(HashMap::new())),
            goal_tree_cache: Arc::new(RwLock::new(HashMap::new())),
            telemetry_hook: None,
            limits,
        }
    }

    pub async fn cortex(
        &self,
        senses: &[Sense],
        physical_state: &PhysicalState,
        cognition_state: &CognitionState,
    ) -> Result<CortexOutput, CortexError> {
        if senses.iter().any(|sense| matches!(sense, Sense::Hibernate)) {
            return Err(invalid_input(
                "hibernate sense should not be sent to cortex",
            ));
        }

        self.emit(CortexTelemetryEvent::ReactionStarted {
            cycle_id: physical_state.cycle_id,
        });
        observability_metrics::record_cortex_cycle_id(physical_state.cycle_id);

        let deadline = Duration::from_millis(self.limits.max_cycle_time_ms.max(1));
        let sense_descriptors =
            helpers_input::sense_descriptors(&physical_state.capabilities.entries);
        let act_descriptors = helpers_input::act_descriptors(&physical_state.capabilities.entries);

        let (sense_section_result, act_catalog_result, goal_tree_section_result) = tokio::join!(
            timeout(
                deadline,
                self.build_senses_section(
                    physical_state.cycle_id,
                    senses.to_vec(),
                    sense_descriptors.clone()
                )
            ),
            timeout(
                deadline,
                self.build_act_descriptor_catalog_section(
                    physical_state.cycle_id,
                    act_descriptors.clone()
                )
            ),
            timeout(
                deadline,
                self.build_goal_tree_section(
                    physical_state.cycle_id,
                    cognition_state.goal_tree.user_partition.clone(),
                )
            )
        );

        let senses_section = self.resolve_input_helper_fallback(
            physical_state.cycle_id,
            "sense_helper",
            sense_section_result,
            helpers_input::fallback_senses_section(senses, &sense_descriptors),
        );
        let act_descriptor_catalog_section = self.resolve_input_helper_fallback(
            physical_state.cycle_id,
            "act_descriptor_helper",
            act_catalog_result,
            helpers_input::fallback_act_descriptor_catalog_section(&act_descriptors),
        );
        let willpower_matrix_section = self.resolve_input_helper_fallback(
            physical_state.cycle_id,
            "goal_tree_helper",
            goal_tree_section_result,
            helpers_input::fallback_goal_tree_section(&cognition_state.goal_tree.user_partition),
        );
        let instincts_section =
            helpers_input::instincts_section(&cognition_state.goal_tree.root_partition);
        let focal_awareness_section = helpers_input::l1_memory_section(&cognition_state.l1_memory);

        observability_metrics::record_cortex_input_ir_act_descriptor_catalog_count(
            act_descriptors.len(),
        );
        tracing::debug!(
            target: "cortex",
            cycle_id = physical_state.cycle_id,
            input_ir_sense = %senses_section,
            "input_ir_sense"
        );
        tracing::debug!(
            target: "cortex",
            cycle_id = physical_state.cycle_id,
            input_ir_act_descriptor_catalog = %act_descriptor_catalog_section,
            "input_ir_act_descriptor_catalog"
        );
        tracing::debug!(
            target: "cortex",
            cycle_id = physical_state.cycle_id,
            input_ir_instincts = %instincts_section,
            "input_ir_instincts"
        );
        tracing::debug!(
            target: "cortex",
            cycle_id = physical_state.cycle_id,
            input_ir_willpower_matrix = %willpower_matrix_section,
            "input_ir_willpower_matrix"
        );
        tracing::debug!(
            target: "cortex",
            cycle_id = physical_state.cycle_id,
            input_ir_focal_awareness = %focal_awareness_section,
            "input_ir_focal_awareness"
        );

        let input_ir = ir::build_input_ir(
            &senses_section,
            &act_descriptor_catalog_section,
            &instincts_section,
            &willpower_matrix_section,
            &focal_awareness_section,
        );
        let primary_input_payload = ir::build_primary_input_payload(
            &senses_section,
            &act_descriptor_catalog_section,
            &instincts_section,
            &willpower_matrix_section,
            &focal_awareness_section,
        );

        let primary_result = timeout(
            deadline,
            self.run_primary_helper(
                physical_state.cycle_id,
                primary_input_payload,
                input_ir.text.clone(),
            ),
        )
        .await;
        let primary_output = match primary_result {
            Ok(Ok(output)) => output,
            Ok(Err(err)) => {
                self.emit(CortexTelemetryEvent::StageFailed {
                    cycle_id: physical_state.cycle_id,
                    stage: "primary",
                });
                tracing::warn!(
                    target: "cortex",
                    cycle_id = physical_state.cycle_id,
                    error = %err,
                    "primary_failed_noop"
                );
                return Ok(self.noop_output(
                    cognition_state,
                    physical_state.cycle_id,
                    "primary_failed",
                ));
            }
            Err(_) => {
                self.emit(CortexTelemetryEvent::StageFailed {
                    cycle_id: physical_state.cycle_id,
                    stage: "primary_timeout",
                });
                tracing::warn!(
                    target: "cortex",
                    cycle_id = physical_state.cycle_id,
                    deadline_ms = deadline.as_millis() as u64,
                    "primary_timeout_noop"
                );
                return Ok(self.noop_output(
                    cognition_state,
                    physical_state.cycle_id,
                    "primary_timeout",
                ));
            }
        };

        let (_output_ir, output_sections) = match ir::parse_output_ir(&primary_output) {
            Ok(parsed) => parsed,
            Err(err) => {
                self.emit(CortexTelemetryEvent::StageFailed {
                    cycle_id: physical_state.cycle_id,
                    stage: "primary_contract",
                });
                tracing::warn!(
                    target: "cortex",
                    cycle_id = physical_state.cycle_id,
                    error = %err,
                    "primary_contract_failed_noop"
                );
                return Ok(self.noop_output(
                    cognition_state,
                    physical_state.cycle_id,
                    "primary_contract",
                ));
            }
        };

        let (acts_result, goal_tree_ops_result, l1_memory_flush_result) = tokio::join!(
            timeout(
                deadline,
                self.run_acts_helper(
                    physical_state.cycle_id,
                    output_sections.acts_section.clone(),
                    act_descriptors.clone()
                )
            ),
            timeout(
                deadline,
                self.run_goal_tree_patch_helper(
                    physical_state.cycle_id,
                    output_sections.goal_tree_patch_section.clone(),
                    cognition_state.clone()
                )
            ),
            timeout(
                deadline,
                self.run_l1_memory_flush_helper(
                    physical_state.cycle_id,
                    output_sections.l1_memory_flush_section.clone(),
                    cognition_state.clone()
                )
            )
        );

        let act_drafts = match acts_result {
            Ok(Ok(acts_helper_output)) => acts_helper_output,
            Ok(Err(err)) => {
                self.emit(CortexTelemetryEvent::StageFailed {
                    cycle_id: physical_state.cycle_id,
                    stage: "acts_helper",
                });
                tracing::warn!(
                    target: "cortex",
                    cycle_id = physical_state.cycle_id,
                    error = %err,
                    "acts_helper_failed_fallback_empty"
                );
                Vec::new()
            }
            Err(_) => {
                self.emit(CortexTelemetryEvent::StageFailed {
                    cycle_id: physical_state.cycle_id,
                    stage: "acts_helper_timeout",
                });
                tracing::warn!(
                    target: "cortex",
                    cycle_id = physical_state.cycle_id,
                    deadline_ms = deadline.as_millis() as u64,
                    "acts_helper_timeout_fallback_empty"
                );
                Vec::new()
            }
        };

        let goal_tree_ops = match goal_tree_ops_result {
            Ok(Ok(ops)) => ops,
            Ok(Err(err)) => {
                self.emit(CortexTelemetryEvent::StageFailed {
                    cycle_id: physical_state.cycle_id,
                    stage: "goal_tree_patch_helper",
                });
                tracing::warn!(
                    target: "cortex",
                    cycle_id = physical_state.cycle_id,
                    error = %err,
                    "goal_tree_patch_helper_failed_fallback_empty"
                );
                Vec::<GoalTreePatchOp>::new()
            }
            Err(_) => {
                self.emit(CortexTelemetryEvent::StageFailed {
                    cycle_id: physical_state.cycle_id,
                    stage: "goal_tree_patch_helper_timeout",
                });
                tracing::warn!(
                    target: "cortex",
                    cycle_id = physical_state.cycle_id,
                    deadline_ms = deadline.as_millis() as u64,
                    "goal_tree_patch_helper_timeout_fallback_empty"
                );
                Vec::<GoalTreePatchOp>::new()
            }
        };

        let l1_memory_flush = match l1_memory_flush_result {
            Ok(Ok(entries)) => entries,
            Ok(Err(err)) => {
                self.emit(CortexTelemetryEvent::StageFailed {
                    cycle_id: physical_state.cycle_id,
                    stage: "l1_memory_flush_helper",
                });
                tracing::warn!(
                    target: "cortex",
                    cycle_id = physical_state.cycle_id,
                    error = %err,
                    "l1_memory_flush_helper_failed_fallback_empty"
                );
                Vec::<String>::new()
            }
            Err(_) => {
                self.emit(CortexTelemetryEvent::StageFailed {
                    cycle_id: physical_state.cycle_id,
                    stage: "l1_memory_flush_helper_timeout",
                });
                tracing::warn!(
                    target: "cortex",
                    cycle_id = physical_state.cycle_id,
                    deadline_ms = deadline.as_millis() as u64,
                    "l1_memory_flush_helper_timeout_fallback_empty"
                );
                Vec::<String>::new()
            }
        };

        if cognition_state.goal_tree.user_partition.is_empty()
            && !goal_tree_ops.is_empty()
            && !goal_tree_ops
                .iter()
                .any(|op| matches!(op, GoalTreePatchOp::Sprout { .. }))
        {
            tracing::warn!(
                target: "cortex",
                cycle_id = physical_state.cycle_id,
                op_count = goal_tree_ops.len(),
                "goal_tree_patch_no_sprout_for_empty_user_partition"
            );
        }

        let acts = materialize_acts(
            physical_state.cycle_id,
            act_drafts,
            |cycle_id, endpoint_id, neural_signal_descriptor_id, payload| {
                derive_act_id(
                    cycle_id,
                    &[],
                    endpoint_id,
                    neural_signal_descriptor_id,
                    payload,
                )
            },
        );

        let apply_result = apply_cognition_patches(
            cognition_state,
            &goal_tree_ops,
            &l1_memory_flush,
            self.limits.max_l1_memory_entries,
        );
        if !goal_tree_ops.is_empty()
            && apply_result.new_cognition_state.goal_tree.user_partition
                == cognition_state.goal_tree.user_partition
        {
            tracing::warn!(
                target: "cortex",
                cycle_id = physical_state.cycle_id,
                op_count = goal_tree_ops.len(),
                "goal_tree_patch_ops_no_effect"
            );
        }
        if apply_result.l1_memory_overflow_count > 0 {
            tracing::warn!(
                target: "cortex",
                cycle_id = physical_state.cycle_id,
                max_l1_memory_entries = self.limits.max_l1_memory_entries,
                overflow_entries = apply_result.l1_memory_overflow_count,
                "l1_memory_flush_overflow_discarded"
            );
        }
        let new_cognition_state = apply_result.new_cognition_state;

        let acts_json = serde_json::to_string(&acts)
            .unwrap_or_else(|err| format!("{{\"serialization_error\":\"{}\"}}", err));
        tracing::debug!(
            target: "cortex",
            cycle_id = physical_state.cycle_id,
            output_ir_goal_tree_patch = %output_sections.goal_tree_patch_section,
            "output_ir_goal_tree_patch"
        );
        tracing::debug!(
            target: "cortex",
            cycle_id = physical_state.cycle_id,
            output_ir_l1_memory_flush = %output_sections.l1_memory_flush_section,
            "output_ir_l1_memory_flush"
        );
        tracing::debug!(
            target: "cortex",
            cycle_id = physical_state.cycle_id,
            output_ir_wait_for_sense = output_sections.wait_for_sense,
            "output_ir_wait_for_sense"
        );
        tracing::debug!(
            target: "cortex",
            cycle_id = physical_state.cycle_id,
            act_count = acts.len(),
            final_returned_acts = %acts_json,
            "final_returned_acts"
        );

        if acts.is_empty() {
            self.emit(CortexTelemetryEvent::NoopFallback {
                cycle_id: physical_state.cycle_id,
                reason: "acts_helper_empty",
            });
        }
        self.emit(CortexTelemetryEvent::ReactionCompleted {
            cycle_id: physical_state.cycle_id,
            act_count: acts.len(),
        });

        Ok(CortexOutput {
            acts,
            new_cognition_state,
            wait_for_sense: output_sections.wait_for_sense,
        })
    }

    async fn build_senses_section(
        &self,
        cycle_id: u64,
        senses: Vec<Sense>,
        sense_descriptors: Vec<NeuralSignalDescriptor>,
    ) -> Result<String, CortexError> {
        let stage = CognitionOrgan::Sense.stage();
        let semantic_senses = helpers_input::semantic_sense_events(&senses);
        if semantic_senses.is_empty() {
            let input_payload = pretty_json(&serde_json::json!({
                "senses": &senses,
                "sense_descriptors": &sense_descriptors,
            }));
            log_cortex_organ_input(cycle_id, stage, &input_payload);
            let output = helpers_input::fallback_senses_section(&senses, &sense_descriptors);
            log_cortex_organ_output(cycle_id, stage, &output);
            return Ok(output);
        }

        if let Some(hooks) = &self.hooks {
            let input_payload = pretty_json(&serde_json::json!({
                "senses": &senses,
                "sense_descriptors": &sense_descriptors,
            }));
            log_cortex_organ_input(cycle_id, stage, &input_payload);
            let output = (hooks.sense_helper)(TestSenseHelperRequest {
                cycle_id,
                senses,
                sense_descriptors,
            })
            .await?;
            log_cortex_organ_output(cycle_id, stage, &output);
            return Ok(output);
        }

        let semantic_sense_catalog = helpers_input::semantic_sense_catalog(&sense_descriptors);
        let input_payload = format!(
            "semantic_senses:\n{}\n\nsemantic_sense_catalog:\n{}",
            pretty_json(&semantic_senses),
            pretty_json(&semantic_sense_catalog)
        );
        log_cortex_organ_input(cycle_id, stage, &input_payload);
        let output = self
            .build_senses_with_organs(cycle_id, &semantic_senses, &semantic_sense_catalog)
            .await?;
        log_cortex_organ_output(cycle_id, stage, &output);
        Ok(output)
    }

    async fn build_act_descriptor_catalog_section(
        &self,
        cycle_id: u64,
        act_descriptors: Vec<NeuralSignalDescriptor>,
    ) -> Result<String, CortexError> {
        let stage = CognitionOrgan::ActDescriptor.stage();
        let input_payload = pretty_json(&serde_json::json!({
            "act_descriptors": &act_descriptors,
        }));
        log_cortex_organ_input(cycle_id, stage, &input_payload);

        let cache_key = helpers_input::act_descriptor_cache_key(&act_descriptors);
        if let Some(cached) = self.get_cached_act_descriptor_section(&cache_key).await {
            tracing::debug!(
                target: "cortex",
                cycle_id = cycle_id,
                cache_key = %cache_key,
                "act_descriptor_helper_cache_hit"
            );
            log_cortex_organ_output(cycle_id, stage, &cached);
            return Ok(cached);
        }

        let generated = if let Some(hooks) = &self.hooks {
            (hooks.act_descriptor_helper)(TestActDescriptorHelperRequest {
                cycle_id,
                act_descriptors,
            })
            .await?
        } else {
            self.build_act_descriptor_catalog_with_organs(cycle_id, &act_descriptors)
                .await?
        };

        if !generated.trim().is_empty() {
            self.cache_act_descriptor_section(cache_key, generated.clone())
                .await;
        }
        log_cortex_organ_output(cycle_id, stage, &generated);
        Ok(generated)
    }

    async fn build_goal_tree_section(
        &self,
        cycle_id: u64,
        user_partition: Vec<GoalNode>,
    ) -> Result<String, CortexError> {
        let stage = CognitionOrgan::GoalTree.stage();
        let user_partition_json = helpers_input::goal_tree_user_partition_json(&user_partition);
        let input_payload = pretty_json(&serde_json::json!({
            "goal_tree_user_partition": &user_partition,
        }));
        log_cortex_organ_input(cycle_id, stage, &input_payload);

        if user_partition.is_empty() {
            let output = helpers_input::goal_tree_empty_pursuits_one_shot().to_string();
            log_cortex_organ_output(cycle_id, stage, &output);
            return Ok(output);
        }

        let cache_key = helpers_input::goal_tree_cache_key(&user_partition);
        if let Some(cached) = self.get_cached_goal_tree_section(&cache_key).await {
            tracing::debug!(
                target: "cortex",
                cycle_id = cycle_id,
                cache_key = %cache_key,
                "goal_tree_helper_cache_hit"
            );
            log_cortex_organ_output(cycle_id, stage, &cached);
            return Ok(cached);
        }

        let generated = if let Some(hooks) = &self.hooks {
            (hooks.goal_tree_helper)(TestGoalTreeHelperRequest {
                cycle_id,
                user_partition_json: user_partition_json.clone(),
            })
            .await?
        } else {
            let prompt = prompts::build_goal_tree_helper_prompt(&user_partition_json);
            self.run_text_organ_with_system(
                cycle_id,
                CognitionOrgan::GoalTree,
                self.limits.max_sub_output_tokens,
                prompts::goal_tree_helper_system_prompt(),
                prompt,
            )
            .await?
        };

        self.cache_goal_tree_section(cache_key, generated.clone())
            .await;
        log_cortex_organ_output(cycle_id, stage, &generated);
        Ok(generated)
    }

    async fn build_senses_with_organs(
        &self,
        cycle_id: u64,
        semantic_senses: &[helpers_input::PrimarySenseEvent],
        semantic_sense_catalog: &[helpers_input::PrimarySenseDescriptor],
    ) -> Result<String, CortexError> {
        let mut entries = Vec::with_capacity(semantic_senses.len());
        for sense_event in semantic_senses {
            let payload_json = serde_json::to_string_pretty(&sense_event.payload)
                .unwrap_or_else(|_| "{}".to_string());
            let payload_schema_json = semantic_sense_catalog
                .iter()
                .find(|descriptor| {
                    descriptor.endpoint == sense_event.endpoint
                        && descriptor.sense == sense_event.sense
                })
                .and_then(|descriptor| {
                    serde_json::to_string_pretty(&descriptor.payload_schema).ok()
                })
                .unwrap_or_else(|| "{}".to_string());
            let prompt = prompts::build_sense_helper_prompt(&payload_json, &payload_schema_json);
            let markdown = self
                .run_text_organ_with_system(
                    cycle_id,
                    CognitionOrgan::Sense,
                    self.limits.max_sub_output_tokens,
                    prompts::sense_helper_system_prompt(),
                    prompt,
                )
                .await?;
            entries.push(format!(
                "<sense endpoint-id=\"{}\" sense-id=\"{}\" sense-name=\"{}\">\n{}\n</sense>",
                escape_xml_attr(&sense_event.endpoint),
                escape_xml_attr(&sense_event.sense_id),
                escape_xml_attr(&sense_event.sense),
                markdown.trim(),
            ));
        }
        Ok(entries.join("\n"))
    }

    async fn build_act_descriptor_catalog_with_organs(
        &self,
        cycle_id: u64,
        act_descriptors: &[NeuralSignalDescriptor],
    ) -> Result<String, CortexError> {
        let mut catalog_entries = Vec::with_capacity(act_descriptors.len());
        for act_descriptor in act_descriptors {
            let prompt = prompts::build_act_descriptor_markdown_prompt(
                &serde_json::to_string_pretty(&act_descriptor.payload_schema)
                    .unwrap_or_else(|_| "{}".to_string()),
            );
            let markdown = self
                .run_text_organ_with_system(
                    cycle_id,
                    CognitionOrgan::ActDescriptor,
                    self.limits.max_sub_output_tokens,
                    prompts::act_descriptor_helper_system_prompt(),
                    prompt,
                )
                .await?;
            catalog_entries.push(self.wrap_act_descriptor_catalog_entry(act_descriptor, &markdown));
        }
        Ok(catalog_entries.join("\n"))
    }

    fn wrap_act_descriptor_catalog_entry(
        &self,
        act_descriptor: &NeuralSignalDescriptor,
        markdown: &str,
    ) -> String {
        format!(
            "<act-descriptor endpoint-id=\"{}\" act-id=\"{}\">\n{}\n</act-descriptor>",
            escape_xml_attr(&act_descriptor.endpoint_id),
            escape_xml_attr(&act_descriptor.neural_signal_descriptor_id),
            markdown.trim(),
        )
    }

    async fn run_primary_helper(
        &self,
        cycle_id: u64,
        primary_input: String,
        input_ir_internal: String,
    ) -> Result<String, CortexError> {
        let stage = CognitionOrgan::Primary.stage();
        let input_payload = pretty_json(&serde_json::json!({
            "primary_input": &primary_input,
            "input_ir_internal": &input_ir_internal,
        }));
        log_cortex_organ_input(cycle_id, stage, &input_payload);

        if let Some(hooks) = &self.hooks {
            let output = (hooks.primary)(TestPrimaryRequest {
                cycle_id,
                input_ir: primary_input.clone(),
            })
            .await?;
            log_cortex_organ_output(cycle_id, stage, &output);
            return Ok(output);
        }

        let output = self
            .run_text_organ_with_system(
                cycle_id,
                CognitionOrgan::Primary,
                self.limits.max_primary_output_tokens,
                prompts::primary_system_prompt(),
                prompts::build_primary_user_prompt(&primary_input),
            )
            .await?;
        log_cortex_organ_output(cycle_id, stage, &output);
        Ok(output)
    }

    async fn run_acts_helper(
        &self,
        cycle_id: u64,
        acts_section: String,
        act_descriptors: Vec<NeuralSignalDescriptor>,
    ) -> Result<ActsHelperOutput, CortexError> {
        let stage = CognitionOrgan::Acts.stage();
        let semantic_act_catalog = helpers_input::semantic_act_catalog(&act_descriptors);
        let input_payload = pretty_json(&serde_json::json!({
            "acts_section": &acts_section,
            "semantic_act_catalog": &semantic_act_catalog,
        }));
        log_cortex_organ_input(cycle_id, stage, &input_payload);

        if let Some(hooks) = &self.hooks {
            let raw = (hooks.acts_helper)(TestActsHelperRequest {
                cycle_id,
                acts_section,
            })
            .await?;
            let output: ActsHelperOutput = raw
                .into_iter()
                .map(|act| ActDraft {
                    endpoint_id: act.endpoint_id,
                    neural_signal_descriptor_id: act.neural_signal_descriptor_id,
                    payload: act.payload,
                })
                .collect();
            log_cortex_organ_output(cycle_id, stage, &pretty_json(&output));
            return Ok(output);
        }

        let prompt = prompts::build_acts_helper_prompt(&semantic_act_catalog, &acts_section);
        let response = self
            .run_organ(
                cycle_id,
                CognitionOrgan::Acts,
                self.limits.max_sub_output_tokens,
                prompts::acts_helper_system_prompt(),
                prompt,
                OutputMode::JsonSchema {
                    name: "acts_helper_output".to_string(),
                    schema: acts_json_schema(),
                    strict: true,
                },
            )
            .await?;
        let output = parse_acts_helper_output(&response.output_text)
            .map_err(|err| extractor_failed(err.to_string()))?;
        log_cortex_organ_output(cycle_id, stage, &pretty_json(&output));
        Ok(output)
    }

    async fn run_goal_tree_patch_helper(
        &self,
        cycle_id: u64,
        goal_tree_patch_section: String,
        cognition_state: CognitionState,
    ) -> Result<GoalTreePatchHelperOutput, CortexError> {
        let stage = CognitionOrgan::GoalTreePatch.stage();
        let user_partition_json =
            helpers_input::goal_tree_user_partition_json(&cognition_state.goal_tree.user_partition);
        let input_payload = pretty_json(&serde_json::json!({
            "goal_tree_patch_section": &goal_tree_patch_section,
            "current_user_partition_json": &user_partition_json,
        }));
        log_cortex_organ_input(cycle_id, stage, &input_payload);

        if let Some(hooks) = &self.hooks {
            let output = (hooks.goal_tree_patch_helper)(TestGoalTreePatchRequest {
                cycle_id,
                goal_tree_patch_section,
                cognition_state,
            })
            .await?;
            log_cortex_organ_output(cycle_id, stage, &pretty_json(&output));
            return Ok(output);
        }

        let prompt = prompts::build_goal_tree_patch_helper_prompt(
            &goal_tree_patch_section,
            &user_partition_json,
        );
        let response = self
            .run_organ(
                cycle_id,
                CognitionOrgan::GoalTreePatch,
                self.limits.max_sub_output_tokens,
                prompts::goal_tree_patch_helper_system_prompt(),
                prompt,
                OutputMode::JsonSchema {
                    name: "goal_tree_patch_helper_output".to_string(),
                    schema: goal_tree_patch_ops_json_schema(),
                    strict: true,
                },
            )
            .await?;
        let output = parse_goal_tree_patch_helper_output(&response.output_text)
            .map_err(|err| filler_failed(err.to_string()))?;
        log_cortex_organ_output(cycle_id, stage, &pretty_json(&output));
        Ok(output)
    }

    async fn run_l1_memory_flush_helper(
        &self,
        cycle_id: u64,
        l1_memory_flush_section: String,
        cognition_state: CognitionState,
    ) -> Result<L1MemoryFlushHelperOutput, CortexError> {
        let stage = CognitionOrgan::L1MemoryFlush.stage();
        let l1_memory_json = helpers_input::l1_memory_json(&cognition_state.l1_memory);
        let input_payload = pretty_json(&serde_json::json!({
            "l1_memory_flush_section": &l1_memory_flush_section,
            "current_l1_memory_json": &l1_memory_json,
        }));
        log_cortex_organ_input(cycle_id, stage, &input_payload);

        if let Some(hooks) = &self.hooks {
            let output = (hooks.l1_memory_flush_helper)(TestL1MemoryFlushRequest {
                cycle_id,
                l1_memory_flush_section,
                cognition_state,
            })
            .await?;
            log_cortex_organ_output(cycle_id, stage, &pretty_json(&output));
            return Ok(output);
        }

        let prompt =
            prompts::build_l1_memory_flush_helper_prompt(&l1_memory_flush_section, &l1_memory_json);
        let response = self
            .run_organ(
                cycle_id,
                CognitionOrgan::L1MemoryFlush,
                self.limits.max_sub_output_tokens,
                prompts::l1_memory_flush_helper_system_prompt(),
                prompt,
                OutputMode::JsonSchema {
                    name: "l1_memory_flush_helper_output".to_string(),
                    schema: l1_memory_flush_json_schema(),
                    strict: true,
                },
            )
            .await?;
        let output = parse_l1_memory_flush_helper_output(&response.output_text)
            .map_err(|err| filler_failed(err.to_string()))?;
        log_cortex_organ_output(cycle_id, stage, &pretty_json(&output));
        Ok(output)
    }

    async fn run_text_organ_with_system(
        &self,
        cycle_id: u64,
        organ: CognitionOrgan,
        max_output_tokens: u64,
        system_prompt: String,
        user_prompt: String,
    ) -> Result<String, CortexError> {
        let response = self
            .run_organ(
                cycle_id,
                organ,
                max_output_tokens,
                system_prompt,
                user_prompt,
                OutputMode::Text,
            )
            .await?;
        let text = response.output_text.trim().to_string();
        if text.is_empty() {
            return Err(extractor_failed(format!(
                "{} produced empty output",
                organ.stage()
            )));
        }
        Ok(text)
    }

    async fn run_organ(
        &self,
        cycle_id: u64,
        organ: CognitionOrgan,
        max_output_tokens: u64,
        system_prompt: String,
        user_prompt: String,
        output_mode: OutputMode,
    ) -> Result<ChatResponse, CortexError> {
        let stage = organ.stage();
        let request_id = format!("cortex-{stage}-{cycle_id}");
        let started_at = Instant::now();
        let route = self.resolve_route(organ);
        let request = build_request(
            request_id.clone(),
            route.clone(),
            max_output_tokens,
            self.limits.max_cycle_time_ms,
            system_prompt,
            user_prompt,
            stage,
            output_mode,
        );

        let gateway = self.gateway.as_ref().ok_or_else(|| {
            CortexError::new(
                crate::cortex::error::CortexErrorKind::Internal,
                "AI Gateway is not configured for this Cortex instance",
            )
        })?;

        let response = gateway.chat_once(request).await.map_err(|err| {
            tracing::debug!(
                target: "cortex",
                stage = stage,
                cycle_id = cycle_id,
                request_id = %request_id,
                elapsed_ms = started_at.elapsed().as_millis() as u64,
                error_kind = ?err.kind,
                error = %err.message,
                "llm_call_failed"
            );
            match organ {
                CognitionOrgan::Primary => primary_failed(err.to_string()),
                CognitionOrgan::GoalTreePatch | CognitionOrgan::L1MemoryFlush => {
                    filler_failed(err.to_string())
                }
                CognitionOrgan::Sense
                | CognitionOrgan::ActDescriptor
                | CognitionOrgan::GoalTree
                | CognitionOrgan::Acts => extractor_failed(err.to_string()),
            }
        })?;

        Ok(response)
    }

    fn resolve_input_helper_fallback(
        &self,
        cycle_id: u64,
        stage: &'static str,
        result: Result<Result<String, CortexError>, tokio::time::error::Elapsed>,
        fallback: String,
    ) -> String {
        match result {
            Ok(Ok(text)) if !text.trim().is_empty() => text,
            Ok(Ok(_)) => fallback,
            Ok(Err(err)) => {
                self.emit(CortexTelemetryEvent::StageFailed { cycle_id, stage });
                tracing::warn!(
                    target: "cortex",
                    cycle_id = cycle_id,
                    stage = stage,
                    error = %err,
                    "input_helper_failed_fallback_raw"
                );
                fallback
            }
            Err(_) => {
                self.emit(CortexTelemetryEvent::StageFailed { cycle_id, stage });
                tracing::warn!(
                    target: "cortex",
                    cycle_id = cycle_id,
                    stage = stage,
                    "input_helper_timeout_fallback_raw"
                );
                fallback
            }
        }
    }

    fn noop_output(
        &self,
        cognition_state: &CognitionState,
        cycle_id: u64,
        reason: &'static str,
    ) -> CortexOutput {
        self.emit(CortexTelemetryEvent::NoopFallback { cycle_id, reason });
        CortexOutput {
            acts: Vec::new(),
            new_cognition_state: cognition_state.clone(),
            wait_for_sense: false,
        }
    }

    fn resolve_route(&self, organ: CognitionOrgan) -> Option<String> {
        let stage_route = match organ {
            CognitionOrgan::Primary => self.helper_routes.primary.clone(),
            CognitionOrgan::Sense => self.helper_routes.sense_helper.clone(),
            CognitionOrgan::ActDescriptor => self.helper_routes.act_descriptor_helper.clone(),
            CognitionOrgan::GoalTree => self.helper_routes.goal_tree_helper.clone(),
            CognitionOrgan::Acts => self.helper_routes.acts_helper.clone(),
            CognitionOrgan::GoalTreePatch => self.helper_routes.goal_tree_patch_helper.clone(),
            CognitionOrgan::L1MemoryFlush => self.helper_routes.l1_memory_flush_helper.clone(),
        };
        stage_route.or_else(|| self.helper_routes.default.clone())
    }

    async fn get_cached_act_descriptor_section(&self, cache_key: &str) -> Option<String> {
        self.act_descriptor_cache
            .read()
            .await
            .get(cache_key)
            .cloned()
    }

    async fn cache_act_descriptor_section(&self, cache_key: String, value: String) {
        self.act_descriptor_cache
            .write()
            .await
            .insert(cache_key, value);
    }

    async fn get_cached_goal_tree_section(&self, cache_key: &str) -> Option<String> {
        self.goal_tree_cache.read().await.get(cache_key).cloned()
    }

    async fn cache_goal_tree_section(&self, cache_key: String, value: String) {
        self.goal_tree_cache.write().await.insert(cache_key, value);
    }

    fn emit(&self, event: CortexTelemetryEvent) {
        match &event {
            CortexTelemetryEvent::ReactionStarted { cycle_id } => {
                tracing::debug!(target: "cortex", cycle_id = *cycle_id, "reaction_started");
            }
            CortexTelemetryEvent::StageFailed { cycle_id, stage } => {
                tracing::warn!(
                    target: "cortex",
                    cycle_id = *cycle_id,
                    stage = *stage,
                    "stage_failed"
                );
            }
            CortexTelemetryEvent::ReactionCompleted {
                cycle_id,
                act_count,
            } => {
                tracing::debug!(
                    target: "cortex",
                    cycle_id = *cycle_id,
                    act_count = *act_count,
                    "reaction_completed"
                );
            }
            CortexTelemetryEvent::NoopFallback { cycle_id, reason } => {
                tracing::debug!(
                    target: "cortex",
                    cycle_id = *cycle_id,
                    reason = *reason,
                    "noop_fallback"
                );
            }
        }

        if let Some(hook) = &self.telemetry_hook {
            hook(event);
        }
    }
}

fn build_request(
    request_id: String,
    route: Option<String>,
    max_output_tokens: u64,
    max_request_time_ms: u64,
    system_prompt: String,
    user_prompt: String,
    stage: &'static str,
    output_mode: OutputMode,
) -> ChatRequest {
    let mut metadata = BTreeMap::new();
    metadata.insert("cortex_stage".to_string(), stage.to_string());
    ChatRequest {
        request_id: Some(request_id),
        route,
        messages: vec![
            BelunaMessage {
                role: BelunaRole::System,
                parts: vec![BelunaContentPart::Text {
                    text: system_prompt,
                }],
                tool_call_id: None,
                tool_name: None,
            },
            BelunaMessage {
                role: BelunaRole::User,
                parts: vec![BelunaContentPart::Text { text: user_prompt }],
                tool_call_id: None,
                tool_name: None,
            },
        ],
        tools: vec![],
        tool_choice: ToolChoice::None,
        output_mode,
        limits: RequestLimitOverrides {
            max_output_tokens: Some(max_output_tokens),
            max_request_time_ms: Some(max_request_time_ms),
        },
        metadata,
        cost_attribution_id: None,
    }
}

fn escape_xml_attr(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn log_cortex_organ_input(cycle_id: u64, stage: &str, input_payload: &str) {
    tracing::info!(
        target: "cortex",
        cycle_id = cycle_id,
        stage = stage,
        input_payload = %input_payload,
        "cortex_organ_input"
    );
}

fn log_cortex_organ_output(cycle_id: u64, stage: &str, output_payload: &str) {
    tracing::info!(
        target: "cortex",
        cycle_id = cycle_id,
        stage = stage,
        output_payload = %output_payload,
        "cortex_organ_output"
    );
}

fn pretty_json<T: Serialize>(value: &T) -> String {
    serde_json::to_string_pretty(value)
        .unwrap_or_else(|err| format!("{{\"serialization_error\":\"{}\"}}", err))
}

fn parse_json_output<T: for<'a> Deserialize<'a>>(text: &str) -> Result<T, CortexError> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(CortexError::new(
            crate::cortex::error::CortexErrorKind::Internal,
            "empty JSON output",
        ));
    }

    if let Ok(parsed) = serde_json::from_str::<T>(trimmed) {
        return Ok(parsed);
    }

    if let Some(stripped) = strip_code_fence(trimmed)
        && let Ok(parsed) = serde_json::from_str::<T>(&stripped)
    {
        return Ok(parsed);
    }

    Err(CortexError::new(
        crate::cortex::error::CortexErrorKind::Internal,
        "failed to parse JSON output",
    ))
}

fn strip_code_fence(text: &str) -> Option<String> {
    let text = text.trim();
    if !text.starts_with("```") {
        return None;
    }

    let mut lines = text.lines();
    let _first = lines.next()?;
    let mut body = Vec::new();
    for line in lines {
        if line.trim_start().starts_with("```") {
            break;
        }
        body.push(line);
    }
    Some(body.join("\n"))
}
