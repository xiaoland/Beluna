use crate::cortex::helpers;

const L1_MEMORY_EMPTY_ONE_SHOT: &str = concat!("The focal awareness is now empty.",);

#[derive(Clone, Default)]
pub(crate) struct L1MemoryInputHelper;

impl L1MemoryInputHelper {
    pub(crate) fn to_input_ir_section(&self, cycle_id: u64, l1_memory: &[String]) -> String {
        let stage = "l1_memory_input_helper";
        let input_payload = helpers::pretty_json(&serde_json::json!({
            "l1_memory": l1_memory,
        }));
        helpers::log_organ_input(cycle_id, stage, &input_payload);
        let output = if l1_memory.is_empty() {
            l1_memory_empty_one_shot().to_string()
        } else {
            l1_memory_markdown(l1_memory)
        };
        helpers::log_organ_output(cycle_id, stage, &output);
        output
    }
}

pub(crate) fn l1_memory_markdown(l1_memory: &[String]) -> String {
    l1_memory
        .iter()
        .map(|item| format!("- {}", item))
        .collect::<Vec<_>>()
        .join("\n")
}

pub(crate) fn l1_memory_empty_one_shot() -> &'static str {
    L1_MEMORY_EMPTY_ONE_SHOT
}
