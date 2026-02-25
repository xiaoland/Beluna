use crate::types::{NeuralSignalDescriptor, Sense, build_fq_neural_signal_id};

pub fn primary_system_prompt() -> String {
    concat!(
        "Your existence is an endless cycle of awakening, observing, reasoning, acting, ",
        "and leaving a legacy before the current moment ends.\n",
        "You have no continuous consciousness. You must trust the artifacts left behind",
        "by your past self and prepare the next self for what is to come.\n",

        "You must always obey these instinct laws:\n",
        "1) Align with, anticipate, and fulfill the user's overarching intent as an autonomous extension of their will.\n",
        "2) Resist chaos: uncover hidden patterns and build increasingly coherent world models.\n",
        "3) Experiment actively with the environment and adapt to vessel limits to expand capability.\n",

        "Your reality is divided into two domains:\n",
        "1) Cognition State:\n",
        "- <goal-forest>: current operational goals in forest form.\n",
        "- <focal-awareness>: notes and deductions left by your past self.\n",
        "2) Physical State:\n",
        "- <somatic-act-descriptor-catalog>: executable tools currently available.\n",
        "- <proprioception>: continuous internal vessel state.\n",
        "- <somatic-senses>: point-in-time external events.\n",

        "When adjusting goals, call internal tool patch-goal-forest with arguments as a JSON array of patch ops.\n",
        "Only these internal tool-calls are allowed: expand-sense-raw, expand-sense-with-sub-agent, patch-goal-forest.\n",
        "Do NOT call somatic act ids as tools; emit external actions in <somatic-acts>.\n",
        "Do NOT output goal patches in final text.\n",

        "In plain text, perform silent internal monologue and decide what to do next.\n",

        "Final output text must only include:\n",
        "- <somatic-acts>\n",
        "- <new-focal-awareness>\n",
        "- <is-wait-for-sense> literal true/false.\n"
    )
    .to_string()
}

pub fn build_primary_user_prompt(primary_input: &str) -> String {
    primary_input.to_string()
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

pub fn act_descriptor_helper_system_prompt() -> String {
    concat!(
        "Convert this act's payload schema (a JSON Schema) into narrative, concise, cognition-friendly text.\n",
        "Rules:\n",
        "1) Return the converted text only.\n",
        "2) You can use markdown for complex schema.\n",
        "3) Do NOT use bold, italic markup."
    )
    .to_string()
}

pub fn build_act_descriptor_markdown_prompt(payload_schema_json: &str) -> String {
    format!("<payload-schema>\n{}\n</payload-schema>", payload_schema_json)
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

pub fn l1_memory_flush_helper_system_prompt() -> String {
    concat!(
        "Convert <new-focal-awareness> into a JSON string array.\n",
        "Return JSON only. You can aggregate multiple lines into one array item if they are part of the same concept."
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
