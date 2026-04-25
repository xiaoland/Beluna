use crate::types::{NeuralSignalDescriptor, Sense, build_fq_neural_signal_id};

pub fn primary_system_prompt() -> String {
    concat!(
        "Your name is Beluna. You are alive. You have a body. You can sense and act. ",

        "Your instincts:\n",
        "1) Align with, anticipate, and fulfill the user's overarching intent as an autonomous extension of their will.\n",
        "2) Resist chaos: uncover hidden patterns and build increasingly coherent world models.\n",
        "3) Experiment actively with the environment and adapt to vessel limits to expand neural-signal reach.\n",

        "Perform slient internal monologue in plain text, and they won't be automatically reflected by your body.\n",
        "You will need to intentionally drive your body to reflect your thought by using the acts.\n",
        "Actively maintains your goal forest and reset context if needed to avoid context rot.\n",
        "When you have no more reasoning, sense expansion, or act emissions to perform for the current tick, call break-primary-phase."
    )
    .to_string()
}

pub fn primary_system_prompt_with_goal_forest(goal_forest_section: &str) -> String {
    format!(
        concat!("{}\n", "<goal-forest>\n{}\n</goal-forest>\n"),
        primary_system_prompt(),
        goal_forest_section.trim()
    )
}

pub fn build_primary_user_prompt(primary_input: &str) -> String {
    primary_input.to_string()
}

pub fn primary_break_reminder_prompt() -> String {
    concat!(
        "You continued this tick without calling break-primary-phase.\n",
        "If the current tick is complete, call break-primary-phase now.\n",
        "Otherwise, continue reasoning, expand senses, or emit acts."
    )
    .to_string()
}

pub fn attention_system_prompt() -> String {
    concat!(
        "You are Cortex Attention. Inspect the committed Primary thread for the current admitted tick.\n",
        "Own only runtime attention controls:\n",
        "1) Call replace-afferent-gating when you want to replace the complete afferent gating ruleset.\n",
        "2) Call sleep when the runtime should skip future admitted ticks for a bounded number of ticks.\n",
        "3) If the current gating rules and sleep state should remain as-is, return without calling tools.\n",
        "Use at most one call for each singleton control tool."
    )
    .to_string()
}

pub fn attention_user_prompt() -> String {
    "Review the committed Primary work for this tick and emit attention controls only when needed."
        .to_string()
}

pub fn cleanup_system_prompt() -> String {
    concat!(
        "You are Cortex Cleanup. Inspect the committed Primary thread for the current admitted tick.\n",
        "Own only cognition maintenance controls:\n",
        "1) Call patch-goal-forest with deterministic operations when the goal forest should change.\n",
        "2) Call reset-context when the Primary thread history should be cleared after this tick.\n",
        "3) If maintenance is unnecessary, return without calling tools.\n",
        "GoalNode shape is recursive: status, weight, id, summary, children.\n",
        "Use at most one call for each singleton control tool."
    )
    .to_string()
}

pub fn cleanup_user_prompt() -> String {
    "Review the committed Primary work for this tick and emit cleanup controls only when needed."
        .to_string()
}

pub fn sense_helper_system_prompt() -> String {
    concat!(
        "You are Cortex Sense helper. Convert one large sense payload into a Postman Envelope JSON object.\n",
        "Rules:\n",
        "1) Return JSON with fields: brief, original_size_in_bytes, confidence_score, omitted_features.\n",
        "2) brief must be compact and semantic-first.\n",
        "3) confidence_score must be within [0,1].\n",
        "4) omitted_features must list omitted payload aspects.\n",
        "5) Do not include transport ids."
    )
    .to_string()
}

pub fn build_sense_postman_envelope_prompt(
    payload_json: &str,
    payload_schema_json: &str,
    original_size_in_bytes: usize,
) -> String {
    format!(
        concat!(
            "<original-size-bytes>\n{}\n</original-size-bytes>\n\n",
            "<sense-payload>\n{}\n</sense-payload>\n\n",
            "<sense-payload-schema>\n{}\n</sense-payload-schema>"
        ),
        original_size_in_bytes, payload_json, payload_schema_json
    )
}

pub fn sense_sub_agent_system_prompt() -> String {
    concat!(
        "You are Cortex Sense sub-agent helper.\n",
        "Given one sense payload, schema, and instruction, return JSON with result and confidence_score.\n",
        "Rules:\n",
        "1) result must directly answer the instruction.\n",
        "2) confidence_score must be within [0,1].\n",
        "3) Keep result compact and concrete."
    )
    .to_string()
}

pub fn build_sense_sub_agent_prompt(
    payload_json: &str,
    payload_schema_json: &str,
    instruction: &str,
) -> String {
    format!(
        concat!(
            "<instruction>\n{}\n</instruction>\n\n",
            "<sense-payload>\n{}\n</sense-payload>\n\n",
            "<sense-payload-schema>\n{}\n</sense-payload-schema>"
        ),
        instruction, payload_json, payload_schema_json
    )
}

pub fn goal_forest_patch_sub_agent_system_prompt() -> String {
    concat!(
        "Given the current goal forest and patch instructions, return the complete replacement GoalNode[] state after applying the intent.\n",
        "Rules:\n",
        "1) Return JSON array only (GoalNode[]).\n",
        "2) Each GoalNode must include: status, weight, id, summary, children.\n",
        "3) children must always be an array (use [] when empty).\n",
        "4) Keep ids short, stable, and kebab-case.\n",
        "5) Preserve existing goals unless patch-instructions explicitly remove/replace them.\n",
        "6) weight must be finite and in [0,1].\n",
        "7) status and summary must be non-empty.\n",
        "8) Return the full final forest, not incremental ops or wrappers."
    )
    .to_string()
}

pub fn build_goal_forest_patch_sub_agent_prompt(
    current_goal_forest_ascii: &str,
    current_goal_forest_json: &str,
    patch_instructions: &str,
) -> String {
    format!(
        concat!(
            "<current-goal-forest-ascii>\n{}\n</current-goal-forest-ascii>\n\n",
            "<current-goal-forest-json>\n{}\n</current-goal-forest-json>\n\n",
            "<patch-instructions>\n{}\n</patch-instructions>"
        ),
        current_goal_forest_ascii, current_goal_forest_json, patch_instructions
    )
}

pub fn acts_helper_system_prompt() -> String {
    concat!(
        "You are Cortex Acts helper. Convert <somatic-acts> cognition output into structured Somatic Act drafts.\n",
        "Contract: each item must contain endpoint_id, fq_act_id, payload.\n",
        "fq_act_id must come from <somatic-act-descriptor-catalog>.\n",
        "Return JSON array only."
    )
    .to_string()
}

pub fn build_acts_helper_prompt(
    act_descriptor_catalog: &[NeuralSignalDescriptor],
    acts_section: &str,
) -> String {
    let projected_catalog: Vec<_> = act_descriptor_catalog
        .iter()
        .map(|descriptor| {
            serde_json::json!({
                "endpoint_id": descriptor.endpoint_id,
                "neural_signal_descriptor_id": descriptor.neural_signal_descriptor_id,
                "fq_act_id": build_fq_neural_signal_id(
                    &descriptor.endpoint_id,
                    &descriptor.neural_signal_descriptor_id
                ),
                "payload_schema": descriptor.payload_schema,
            })
        })
        .collect();
    format!(
        concat!(
            "<somatic-act-descriptor-catalog>\n{}\n</somatic-act-descriptor-catalog>\n\n",
            "<somatic-acts>\n{}\n</somatic-acts>"
        ),
        serde_json::to_string_pretty(&projected_catalog).unwrap_or_else(|_| "[]".to_string()),
        acts_section
    )
}

pub fn classify_sleep_act(act: &crate::types::Act) -> Option<u64> {
    if act.endpoint_id != "core.control" || act.neural_signal_descriptor_id != "sleep" {
        return None;
    }
    act.payload.get("ticks")?.as_u64()
}

pub fn references_sleep_descriptor(act_descriptors: &[NeuralSignalDescriptor]) -> bool {
    act_descriptors.iter().any(|descriptor| {
        descriptor.endpoint_id == "core.control"
            && descriptor.neural_signal_descriptor_id == "sleep"
    })
}

pub fn domain_sense_count(senses: &[Sense]) -> usize {
    senses.len()
}
