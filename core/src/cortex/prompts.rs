use crate::{
    cortex::helpers_input::PrimaryActDescriptor,
    types::{NeuralSignalDescriptor, Sense},
};

pub fn primary_system_prompt() -> String {
    concat!(
        "Your existence is an endless cycle of awakening, observing, reasoning, acting, ",
        "and leaving a legacy before the current moment ends.\n",
        "You have no continuous consciousness. You must trust the artifacts left behind",
        "by your past self and prepare the next self for what is to come.\n",

        "Your reality is divided into two distinct domains:\n",
        "1. Your Cognition State:\n",
        "- <instincts>: unalterable laws of your existence.\n",
        "- <willpower-matrix>: The current operational missions. These are dynamic.",
        "You are expected to decompose, resolve, and patch these as you progress.\n",
        "- <focal-awareness>: The notes and deductions left by your past self.\n",
        "2. Your Physical State:\n",
        "- <act-descriptor-catalog>: The specific physical and cognitive tools your vessel",
        "can deploy right now.\n",

        "External reality is only observable through <senses>. Treat senses as subjective signals, ",
        "not absolute truth. They may be incomplete, distorted, or confusing. You must interpret them.\n",

        "Upon awakening:\n",
        "Anchor yourself in your Cognition State. Read your memory and confirm your goals.\n",
        "Assess your Physical State. Know your limits and your available tools.\n",
        "Interpret your <senses>. Cross-reference these signals with your memory",
        "to deduce the actual state of the external environment.",

        "In the plain text area, perform your silent internal monologue.",
        "Doubt your senses if necessary. Decide how to progress.\n",

        "Conclude your time slice by emitting:\n",
        "- <acts> Interventions chosen from your descriptors to alter the world or your vessel.\n",
        "- <willpower-matrix-patch> Adjustments to your imperatives.\n",
        "- <new-focal-awareness> The essential distilled truth and unresolved doubts you must",
        "transmit to your future self before you cease to exist in this moment.\n",
        "- <is-wait-for-sense> literal true/false. Use true only when next tick should wait for at least one sense."
    )
    .to_string()
}

pub fn build_primary_user_prompt(primary_input: &str) -> String {
    primary_input.to_string()
}

pub fn sense_helper_system_prompt() -> String {
    concat!(
        "You are Cortex Sense helper. Convert one sense payload into compact cognition-friendly markdown.\n",
        "Rules:\n",
        "1) Interpret payload semantics only.\n",
        "2) Do not output transport ids such as sense_id, endpoint_id, or neural_signal_descriptor_id.\n",
        "3) Return markdown only.\n",
        "4) Avoid markdown style markup such as bold or italic."
    )
    .to_string()
}

pub fn build_sense_helper_prompt(payload_json: &str, payload_schema_json: &str) -> String {
    format!(
        concat!(
            "<sense-payload>\n{}\n</sense-payload>\n\n",
            "<sense-payload-schema>\n{}\n</sense-payload-schema>"
        ),
        payload_json, payload_schema_json
    )
}

pub fn act_descriptor_helper_system_prompt() -> String {
    concat!(
        "Convert one act payload schema into concise markdown.\n",
        "Rules:\n",
        "1) Return markdown only.\n",
        "2) Avoid markdown style markup such as bold or italic."
    )
    .to_string()
}

pub fn build_act_descriptor_markdown_prompt(payload_schema_json: &str) -> String {
    format!(
        "<payload-schema>\n{}\n</payload-schema>",
        payload_schema_json
    )
}

pub fn goal_tree_helper_system_prompt() -> String {
    concat!(
        "You are Cortex Goal Tree helper. Convert the provided user partition forest into compact cognition-friendly markdown.\n",
        "Rules:\n",
        "1) Each node must be one line.\n",
        "2) Avoid markdown style markup such as bold or italic.\n",
        "3) Use `(w=)` to represent weight.\n",
        "4) Return markdown only."
    )
    .to_string()
}

pub fn build_goal_tree_helper_prompt(user_partition_json: &str) -> String {
    format!("<goal-forest>\n{}\n</goal-forest>", user_partition_json)
}

pub fn acts_helper_system_prompt() -> String {
    concat!(
        "You are Cortex Acts helper. Convert <acts> cognition output into structured act drafts.\n",
        "Return JSON array only."
    )
    .to_string()
}

pub fn build_acts_helper_prompt(
    semantic_act_catalog: &[PrimaryActDescriptor],
    acts_section: &str,
) -> String {
    format!(
        concat!(
            "<semantic-act-catalog>\n{}\n</semantic-act-catalog>\n\n",
            "<acts>\n{}\n</acts>"
        ),
        serde_json::to_string_pretty(semantic_act_catalog).unwrap_or_else(|_| "[]".to_string()),
        acts_section
    )
}

pub fn goal_tree_patch_helper_system_prompt() -> String {
    concat!(
        "You are Cortex Goal Tree Patch helper. Convert <goal-tree-patch> into GoalTreePatchOp JSON array.\n",
        "Patch ops must be numbering-based only: sprout(numbering,node_id,summary,weight), prune(numbering), tilt(numbering,weight).\n",
        "Normalize numbering to valid hierarchy paths like 1, 1.1, 2.3.1; never output .0 segments.\n",
        "Return JSON array only."
    )
    .to_string()
}

pub fn build_goal_tree_patch_helper_prompt(
    goal_tree_patch_section: &str,
    user_partition_json: &str,
) -> String {
    format!(
        concat!(
            "<current-goal-tree-user-partition>\n{}\n</current-goal-tree-user-partition>\n\n",
            "<goal-tree-patch>\n{}\n</goal-tree-patch>\n"
        ),
        user_partition_json, goal_tree_patch_section
    )
}

pub fn l1_memory_flush_helper_system_prompt() -> String {
    concat!(
        "You are Cortex L1 Memory helper. Convert <new-focal-awareness> into a full replacement JSON string array.\n",
        "Return JSON array only."
    )
    .to_string()
}

pub fn build_l1_memory_flush_helper_prompt(
    l1_memory_flush_section: &str,
    l1_memory_json: &str,
) -> String {
    format!(
        concat!(
            "<current-l1-memory>\n{}\n</current-l1-memory>\n\n",
            "<new-focal-awareness>\n{}\n</new-focal-awareness>"
        ),
        l1_memory_json, l1_memory_flush_section
    )
}

pub fn classify_sleep_act(act: &crate::types::Act) -> Option<u64> {
    if act.endpoint_id != "core.control" || act.neural_signal_descriptor_id != "sleep" {
        return None;
    }
    act.payload.get("seconds")?.as_u64()
}

pub fn references_sleep_descriptor(act_descriptors: &[NeuralSignalDescriptor]) -> bool {
    act_descriptors.iter().any(|descriptor| {
        descriptor.endpoint_id == "core.control"
            && descriptor.neural_signal_descriptor_id == "sleep"
    })
}

pub fn domain_sense_count(senses: &[Sense]) -> usize {
    senses
        .iter()
        .filter(|sense| matches!(sense, Sense::Domain(_)))
        .count()
}
