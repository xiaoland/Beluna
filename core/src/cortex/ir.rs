use crate::cortex::{
    CortexError,
    error::primary_failed,
    types::{InputIr, OutputIr},
};

pub(crate) const INPUT_IR_ROOT: &str = "input-ir";
pub(crate) const OUTPUT_IR_ROOT: &str = "output-ir";
const SENSES_TAG: &str = "senses";
const ACT_DESCRIPTOR_CATALOG_TAG: &str = "act-descriptor-catalog";
const PRIMARY_INSTINCTS_TAG: &str = "instincts";
const PRIMARY_WILLPOWER_MATRIX_TAG: &str = "willpower-matrix";
const PRIMARY_FOCAL_AWARENESS_TAG: &str = "focal-awareness";
const ACTS_TAG: &str = "acts";
const PRIMARY_GOAL_TREE_PATCH_TAG: &str = "willpower-matrix-patch";
const PRIMARY_NEW_FOCAL_AWARENESS_TAG: &str = "new-focal-awareness";
const PRIMARY_WAIT_FOR_SENSE_TAG: &str = "is-wait-for-sense";
const INTERNAL_GOAL_TREE_PATCH_TAG: &str = "goal-tree-patch";
const INTERNAL_L1_MEMORY_FLUSH_TAG: &str = "new-focal-awareness";
const INTERNAL_WAIT_FOR_SENSE_TAG: &str = "is-wait-for-sense";

#[derive(Debug, Clone)]
pub(crate) struct OutputIrSections {
    pub acts_section: String,
    pub goal_tree_patch_section: String,
    pub l1_memory_flush_section: String,
    pub wait_for_sense: bool,
}

pub(crate) fn build_input_ir(
    senses_section: &str,
    act_descriptor_catalog_section: &str,
    instincts_section: &str,
    willpower_matrix_section: &str,
    focal_awareness_section: &str,
) -> InputIr {
    let primary_payload = build_primary_input_payload(
        senses_section,
        act_descriptor_catalog_section,
        instincts_section,
        willpower_matrix_section,
        focal_awareness_section,
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
    act_descriptor_catalog_section: &str,
    instincts_section: &str,
    willpower_matrix_section: &str,
    focal_awareness_section: &str,
) -> String {
    format!(
        "<{senses}>\n{a}\n</{senses}>\n<{act_catalog}>\n{b}\n</{act_catalog}>\n<{instincts}>\n{c}\n</{instincts}>\n<{willpower_matrix}>\n{d}\n</{willpower_matrix}>\n<{focal_awareness}>\n{e}\n</{focal_awareness}>",
        senses = SENSES_TAG,
        act_catalog = ACT_DESCRIPTOR_CATALOG_TAG,
        instincts = PRIMARY_INSTINCTS_TAG,
        willpower_matrix = PRIMARY_WILLPOWER_MATRIX_TAG,
        focal_awareness = PRIMARY_FOCAL_AWARENESS_TAG,
        a = senses_section.trim(),
        b = act_descriptor_catalog_section.trim(),
        c = instincts_section.trim(),
        d = willpower_matrix_section.trim(),
        e = focal_awareness_section.trim(),
    )
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

    let parse_target =
        extract_tag_body(&output_ir.text, OUTPUT_IR_ROOT).unwrap_or_else(|| output_ir.text.clone());
    let acts_section = extract_tag_body(&parse_target, ACTS_TAG)
        .ok_or_else(|| primary_failed("primary output must include <acts> section"))?;
    let goal_tree_patch_section = extract_tag_body(&parse_target, PRIMARY_GOAL_TREE_PATCH_TAG)
        .ok_or_else(|| {
            primary_failed("primary output must include <willpower-matrix-patch> section")
        })?;
    let l1_memory_flush_section = extract_tag_body(&parse_target, PRIMARY_NEW_FOCAL_AWARENESS_TAG)
        .ok_or_else(|| {
            primary_failed("primary output must include <new-focal-awareness> section")
        })?;
    let wait_for_sense = extract_tag_body(&parse_target, PRIMARY_WAIT_FOR_SENSE_TAG)
        .map_or(false, |raw| parse_wait_for_sense_flag(&raw));

    Ok((
        OutputIr {
            text: build_internal_output_ir(
                &acts_section,
                &goal_tree_patch_section,
                &l1_memory_flush_section,
                wait_for_sense,
            ),
        },
        OutputIrSections {
            acts_section,
            goal_tree_patch_section,
            l1_memory_flush_section,
            wait_for_sense,
        },
    ))
}

fn build_internal_output_ir(
    acts_section: &str,
    goal_tree_patch_section: &str,
    l1_memory_flush_section: &str,
    wait_for_sense: bool,
) -> String {
    format!(
        "<{root}>\n<{acts}>\n{a}\n</{acts}>\n<{goal_tree_patch}>\n{b}\n</{goal_tree_patch}>\n<{l1_memory_flush}>\n{c}\n</{l1_memory_flush}>\n<{wait_for_sense}>\n{d}\n</{wait_for_sense}>\n</{root}>",
        root = OUTPUT_IR_ROOT,
        acts = ACTS_TAG,
        goal_tree_patch = INTERNAL_GOAL_TREE_PATCH_TAG,
        l1_memory_flush = INTERNAL_L1_MEMORY_FLUSH_TAG,
        wait_for_sense = INTERNAL_WAIT_FOR_SENSE_TAG,
        a = acts_section.trim(),
        b = goal_tree_patch_section.trim(),
        c = l1_memory_flush_section.trim(),
        d = if wait_for_sense { "true" } else { "false" },
    )
}

fn parse_wait_for_sense_flag(value: &str) -> bool {
    let normalized = value.trim().to_ascii_lowercase();
    matches!(normalized.as_str(), "true" | "1" | "yes" | "on")
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
