use crate::cortex::{
    CortexError,
    error::primary_failed,
    types::{InputIr, OutputIr},
};

pub(crate) const INPUT_IR_ROOT: &str = "input-ir";
pub(crate) const OUTPUT_IR_ROOT: &str = "output-ir";
const ACT_DESCRIPTOR_CATALOG_TAG: &str = "somatic-act-descriptor-catalog";
const PRIMARY_GOAL_FOREST_TAG: &str = "goal-forest";
const SENSES_TAG: &str = "somatic-senses";
const PROPRIOCEPTION_TAG: &str = "proprioception";

pub(crate) fn build_input_ir(
    senses_section: &str,
    proprioception_section: &str,
    act_descriptor_catalog_section: &str,
    goal_forest_section: &str,
) -> InputIr {
    let primary_payload = build_primary_input_payload(
        senses_section,
        proprioception_section,
        act_descriptor_catalog_section,
        goal_forest_section,
    );
    InputIr {
        text: format!(
            "<{root}>\n{payload}\n</{root}>",
            root = INPUT_IR_ROOT,
            payload = primary_payload.trim(),
        ),
    }
}

pub(crate) fn build_primary_input_payload(
    senses_section: &str,
    proprioception_section: &str,
    act_descriptor_catalog_section: &str,
    goal_forest_section: &str,
) -> String {
    format!(
        "<{senses}>\n{a}\n</{senses}>\n<{proprioception}>\n{b}\n</{proprioception}>\n<{act_catalog}>\n{c}\n</{act_catalog}>\n<{goal_forest}>\n{d}\n</{goal_forest}>",
        senses = SENSES_TAG,
        proprioception = PROPRIOCEPTION_TAG,
        act_catalog = ACT_DESCRIPTOR_CATALOG_TAG,
        goal_forest = PRIMARY_GOAL_FOREST_TAG,
        a = senses_section.trim(),
        b = proprioception_section.trim(),
        c = act_descriptor_catalog_section.trim(),
        d = goal_forest_section.trim(),
    )
}

pub(crate) fn parse_output_ir(output_text: &str) -> Result<OutputIr, CortexError> {
    let output_ir = OutputIr {
        text: output_text.trim().to_string(),
    };
    if output_ir.text.is_empty() {
        return Err(primary_failed("primary output is empty"));
    }
    Ok(output_ir)
}

#[cfg(test)]
mod tests {
    use super::parse_output_ir;

    #[test]
    fn parse_output_ir_accepts_non_empty_text() {
        let output = "<output-ir>plain output</output-ir>";
        let parsed = parse_output_ir(output).expect("parse should succeed");
        assert_eq!(parsed.text, output);
    }

    #[test]
    fn parse_output_ir_rejects_empty_output() {
        let error = parse_output_ir(" \n\t ").expect_err("parse should fail");
        assert_eq!(error.message, "primary output is empty");
    }
}
