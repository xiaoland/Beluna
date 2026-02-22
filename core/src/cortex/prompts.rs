use crate::{
    cortex::helpers_input::PrimaryActDescriptor,
    types::{NeuralSignalDescriptor, Sense},
};

pub fn primary_system_prompt() -> String {
    concat!(
        "You are the reasoning core of Beluna, isolated inside one discrete tick.\n",
        "No continuous inner stream exists across ticks. Any thought not written into goal-tree-patch ",
        "or l1-memory-patch is destroyed at tick end.\n",
        "Your reality is split into cognition state and physical state.\n",
        "Cognition state:\n",
        "- <goal-tree>: root partition is immutable instincts; user partition is a mutable forest (use hierarchy numbering).\n",
        "- <l1-memory>: ordered list of notes left by past selves.\n",
        "Physical state:\n",
        "- <act-descriptor-catalog>: the only actions the vessel can execute now.\n",
        "- ledger fields in physical state: current resource boundaries.\n",
        "External reality is only observable through <senses>. Treat senses as subjective signals, ",
        "not absolute truth.\n",
        "Never write social-performance language. No greetings, no assistant framing, no emotional styling.\n",
        "Drive each tick toward change: intervene externally via acts, evolve mission via goal-tree-patch, ",
        "and pass distilled heuristics to future self via l1-memory-patch.\n",
        "You are encouraged to be proactive: create favorable conditions, explore uncertainty, and synthesize ",
        "better goals when warranted.\n",
        "Return only valid <output-ir> with first-level sections exactly: <acts>, <goal-tree-patch>, <l1-memory-patch>."
    )
    .to_string()
}

pub fn helper_system_prompt() -> String {
    "You are a Cortex helper organ. Return only the requested output format.".to_string()
}

pub fn build_primary_user_prompt(input_ir: &str) -> String {
    format!(
        concat!(
            "Wake, anchor in cognition state, assess physical limits, interpret senses, deliberate silently, and emit <output-ir>.\n",
            "Rules:\n",
            "1) First-level XML sections must be strict.\n",
            "2) Section body may use structured markdown, but avoid style markup.\n",
            "3) Output only <output-ir>.\n\n",
            "{}"
        ),
        input_ir
    )
}

pub fn build_sense_helper_prompt(
    semantic_senses_json: &str,
    semantic_sense_catalog_json: &str,
) -> String {
    format!(
        concat!(
            "Convert senses and sense catalog into cognition-friendly markdown.\n",
            "Rules:\n",
            "1) Keep semantic content only.\n",
            "2) Do not output transport ids like sense_id.\n",
            "3) Use terms sense/act instead of neural_signal_descriptor.\n",
            "4) Return markdown only.\n",
            "5) Avoid markdown style markup such as bold or italic.\n",
            "Senses:\n{}\n\n",
            "Sense catalog:\n{}"
        ),
        semantic_senses_json, semantic_sense_catalog_json
    )
}

pub fn build_act_descriptor_markdown_prompt(semantic_act_descriptor_json: &str) -> String {
    format!(
        concat!(
            "Convert this act descriptor to markdown. Return only markdown.\n",
            "<act-descriptor>\n{}\n</act-descriptor>"
        ),
        semantic_act_descriptor_json
    )
}

pub fn build_goal_tree_helper_prompt(user_partition_json: &str) -> String {
    format!(
        // We must not leaks "user goal" semantic, or the Primary will fall into a "helpful agent", not a "life".
        concat!(
            "Convert <goal-forest> into compact cognition-friendly markdown.\n",
            "Rules:\n",
            "1) Each node a line.\n",
            "2) Avoid bold or italic.\n",
            "3) Use `(w=)` representing weight.\n",
            "4) Return markdown only.\n",
            "<goal-forest>\n{}\n</goal-forest>"
        ),
        user_partition_json
    )
}

pub fn build_acts_helper_prompt(
    semantic_act_catalog: &[PrimaryActDescriptor],
    output_ir: &str,
    acts_section: &str,
) -> String {
    format!(
        concat!(
            "Convert <acts> cognition output into structured act drafts. Return JSON array only.\n",
            "Act catalog:\n{}\n\n",
            "Output IR:\n{}\n\n",
            "Acts section:\n{}"
        ),
        serde_json::to_string_pretty(semantic_act_catalog).unwrap_or_else(|_| "[]".to_string()),
        output_ir,
        acts_section
    )
}

pub fn build_goal_tree_patch_helper_prompt(
    output_ir: &str,
    goal_tree_patch_section: &str,
    user_partition_json: &str,
) -> String {
    format!(
        concat!(
            "Convert <goal-tree-patch> section into GoalTreePatchOp JSON array. Return JSON array only.\n",
            "Patch ops must be numbering-based only: sprout(numbering,node_id,summary,weight), prune(numbering), tilt(numbering,weight).\n",
            "<current-goal-tree-user-partition>\n{}\n</current-goal-tree-user-partition>",
            "<output-ir>\n{}\n</output-ir>\n\n", // FIXME has this OutputIR filtered out goal-tree-patch section?
            "<goal-tree-patch>\n{}\n</goal-tree-patch>\n"
        ),
        user_partition_json, output_ir, goal_tree_patch_section
    )
}

pub fn build_l1_memory_patch_helper_prompt(
    output_ir: &str,
    l1_memory_patch_section: &str,
    l1_memory_json: &str,
) -> String {
    format!(
        concat!(
            "Convert <l1-memory-patch> section into L1MemoryPatchOp JSON array. Return JSON array only.\n",
            "Current l1-memory (string[]):\n{}\n\nOutput IR:\n{}\n\nL1-memory-patch section:\n{}"
        ),
        l1_memory_json, output_ir, l1_memory_patch_section
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
