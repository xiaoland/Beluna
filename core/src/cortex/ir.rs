use crate::cortex::{
    CortexError,
    error::primary_failed,
    types::{InputIr, OutputIr},
};

pub(crate) const INPUT_IR_ROOT: &str = "input-ir";
pub(crate) const OUTPUT_IR_ROOT: &str = "output-ir";
const SENSES_TAG: &str = "senses";
const ACT_DESCRIPTOR_CATALOG_TAG: &str = "act-descriptor-catalog";
const GOAL_TREE_TAG: &str = "goal-tree";
const L1_MEMORY_TAG: &str = "l1-memory";
const ACTS_TAG: &str = "acts";
const GOAL_TREE_PATCH_TAG: &str = "goal-tree-patch";
const L1_MEMORY_PATCH_TAG: &str = "l1-memory-patch";

#[derive(Debug, Clone)]
pub(crate) struct OutputIrSections {
    pub acts_section: String,
    pub goal_tree_patch_section: String,
    pub l1_memory_patch_section: String,
}

pub(crate) fn build_input_ir(
    senses_section: &str,
    act_descriptor_catalog_section: &str,
    goal_tree_section: &str,
    l1_memory_section: &str,
) -> InputIr {
    InputIr {
        text: format!(
            "<{root}>\n<{senses}>\n{a}\n</{senses}>\n<{act_catalog}>\n{b}\n</{act_catalog}>\n<{goal_tree}>\n{c}\n</{goal_tree}>\n<{l1_memory}>\n{d}\n</{l1_memory}>\n</{root}>",
            root = INPUT_IR_ROOT,
            senses = SENSES_TAG,
            act_catalog = ACT_DESCRIPTOR_CATALOG_TAG,
            goal_tree = GOAL_TREE_TAG,
            l1_memory = L1_MEMORY_TAG,
            a = senses_section.trim(),
            b = act_descriptor_catalog_section.trim(),
            c = goal_tree_section.trim(),
            d = l1_memory_section.trim(),
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
    let goal_tree_patch_section = extract_tag_body(&root_body, GOAL_TREE_PATCH_TAG)
        .ok_or_else(|| primary_failed("primary output must include <goal-tree-patch> section"))?;
    let l1_memory_patch_section = extract_tag_body(&root_body, L1_MEMORY_PATCH_TAG)
        .ok_or_else(|| primary_failed("primary output must include <l1-memory-patch> section"))?;

    Ok((
        output_ir,
        OutputIrSections {
            acts_section,
            goal_tree_patch_section,
            l1_memory_patch_section,
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
