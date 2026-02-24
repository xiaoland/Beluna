use std::collections::BTreeMap;

#[derive(Clone, Default)]
pub(crate) struct ProprioceptionInputHelper;

impl ProprioceptionInputHelper {
    pub(crate) fn to_input_ir_section(
        &self,
        _cycle_id: u64,
        proprioception: &BTreeMap<String, String>,
    ) -> String {
        if proprioception.is_empty() {
            return "[]".to_string();
        }

        proprioception
            .iter()
            .map(|(key, value)| format!("- {key}: {value}"))
            .collect::<Vec<_>>()
            .join("\n")
    }
}
