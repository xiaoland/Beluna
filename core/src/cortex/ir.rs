use crate::cortex::{
    CortexError,
    error::primary_failed,
    types::{InputIr, OutputIr},
};

pub(crate) const INPUT_IR_ROOT: &str = "input-ir";
pub(crate) const OUTPUT_IR_ROOT: &str = "output-ir";
const SENSES_TAG: &str = "senses";
const ACT_DESCRIPTOR_CATALOG_TAG: &str = "act-descriptor-catalog";
const GOAL_STACK_TAG: &str = "goal-stack";
const CONTEXT_TAG: &str = "context";
const ACTS_TAG: &str = "acts";
const GOAL_STACK_PATCH_TAG: &str = "goal-stack-patch";

#[derive(Debug, Clone)]
pub(crate) struct OutputIrSections {
    pub acts_section: String,
    pub goal_stack_patch_section: String,
}

pub(crate) fn build_input_ir(
    senses_section: &str,
    act_descriptor_catalog_section: &str,
    goal_stack_section: &str,
    context_section: &str,
) -> InputIr {
    InputIr {
        text: format!(
            "<{root}>\n<{senses}>\n{a}\n</{senses}>\n<{act_catalog}>\n{b}\n</{act_catalog}>\n<{goal_stack}>\n{c}\n</{goal_stack}>\n<{context}>\n{d}\n</{context}>\n</{root}>",
            root = INPUT_IR_ROOT,
            senses = SENSES_TAG,
            act_catalog = ACT_DESCRIPTOR_CATALOG_TAG,
            goal_stack = GOAL_STACK_TAG,
            context = CONTEXT_TAG,
            a = senses_section.trim(),
            b = act_descriptor_catalog_section.trim(),
            c = goal_stack_section.trim(),
            d = context_section.trim(),
        ),
    }
}

pub(crate) fn parse_output_ir(
    output_text: &str,
) -> Result<(OutputIr, OutputIrSections), CortexError> {
    let output_ir = OutputIr {
        text: output_text.trim().to_string(),
    };
    if output_ir.text.is_empty() {
        return Err(primary_failed("primary output is empty"));
    }

    let root_body = extract_tag_body(&output_ir.text, OUTPUT_IR_ROOT)
        .ok_or_else(|| primary_failed("primary output must contain <output-ir> root"))?;
    let acts_section = extract_tag_body(&root_body, ACTS_TAG)
        .ok_or_else(|| primary_failed("primary output must include <acts> section"))?;
    let goal_stack_patch_section = extract_tag_body(&root_body, GOAL_STACK_PATCH_TAG)
        .ok_or_else(|| primary_failed("primary output must include <goal-stack-patch> section"))?;

    Ok((
        output_ir,
        OutputIrSections {
            acts_section,
            goal_stack_patch_section,
        },
    ))
}

fn extract_tag_body(text: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = text.find(&open)?;
    let body_start = start + open.len();
    let end_rel = text[body_start..].find(&close)?;
    let body_end = body_start + end_rel;
    Some(text[body_start..body_end].trim().to_string())
}
