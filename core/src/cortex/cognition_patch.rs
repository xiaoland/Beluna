use std::collections::BTreeSet;

use crate::cortex::cognition::{CognitionState, GoalForestPatchOp, GoalNode};

pub(crate) struct CognitionPatchApplyResult {
    pub new_cognition_state: CognitionState,
    pub l1_memory_overflow_count: usize,
}

pub(crate) fn apply_cognition_patches(
    previous: &CognitionState,
    goal_forest_ops: &[GoalForestPatchOp],
    l1_memory_flush: &[String],
    max_l1_memory_entries: usize,
) -> CognitionPatchApplyResult {
    let mut next = previous.clone();
    let mut changed = false;

    for op in goal_forest_ops {
        if apply_goal_forest_op(&mut next.goal_forest.nodes, op) {
            changed = true;
        }
    }

    let (l1_memory_changed, l1_memory_overflow_count) =
        apply_l1_memory_flush(&mut next.l1_memory, l1_memory_flush, max_l1_memory_entries);
    if l1_memory_changed {
        changed = true;
    }

    if changed {
        next.revision = next.revision.saturating_add(1);
    }
    CognitionPatchApplyResult {
        new_cognition_state: next,
        l1_memory_overflow_count,
    }
}

pub(crate) fn apply_goal_forest_op(nodes: &mut Vec<GoalNode>, op: &GoalForestPatchOp) -> bool {
    match op {
        GoalForestPatchOp::Plant {
            status,
            weight,
            id,
            summary,
        } => insert_goal_node(nodes, None, None, *weight, status.as_deref(), id, summary),
        GoalForestPatchOp::Sprout {
            parent_numbering,
            parent_id,
            numbering,
            status,
            weight,
            id,
            summary,
        } => {
            let Some(parent_index) =
                resolve_selector_index(nodes, parent_numbering.as_deref(), parent_id.as_deref())
            else {
                return false;
            };

            let parent_id_value = nodes[parent_index].id.clone();
            let parent_numbering_value = nodes[parent_index].numbering.as_deref();
            let resolved_numbering = match numbering {
                Some(value) => {
                    let trimmed = value.trim();
                    if trimmed.is_empty() {
                        return false;
                    }
                    trimmed.to_string()
                }
                None => next_child_numbering(nodes, parent_id_value.as_str(), parent_numbering_value),
            };

            if !is_direct_child_numbering(&resolved_numbering, parent_numbering_value) {
                return false;
            }

            insert_goal_node(
                nodes,
                Some(parent_id_value),
                Some(resolved_numbering),
                *weight,
                status.as_deref(),
                id,
                summary,
            )
        }
        GoalForestPatchOp::Trim {
            numbering,
            id,
            weight,
            status,
        } => {
            let Some(index) = resolve_selector_index(nodes, numbering.as_deref(), id.as_deref())
            else {
                return false;
            };

            let mut changed = false;
            if let Some(new_weight) = weight {
                if !is_valid_weight(*new_weight) {
                    return false;
                }
                if (nodes[index].weight - *new_weight).abs() > f64::EPSILON {
                    nodes[index].weight = *new_weight;
                    changed = true;
                }
            }

            if let Some(new_status) = status {
                let trimmed_status = new_status.trim();
                if trimmed_status.is_empty() {
                    return false;
                }
                if nodes[index].status != trimmed_status {
                    nodes[index].status = trimmed_status.to_string();
                    changed = true;
                }
            }

            changed
        }
        GoalForestPatchOp::Prune { numbering, id } => {
            let Some(index) = resolve_selector_index(nodes, numbering.as_deref(), id.as_deref())
            else {
                return false;
            };
            let target_id = nodes[index].id.clone();
            let remove_ids = collect_descendant_ids(nodes, target_id.as_str());
            let original_len = nodes.len();
            nodes.retain(|node| !remove_ids.contains(node.id.as_str()));
            nodes.len() != original_len
        }
    }
}

fn insert_goal_node(
    nodes: &mut Vec<GoalNode>,
    parent_id: Option<String>,
    numbering: Option<String>,
    weight: Option<f64>,
    status: Option<&str>,
    id: &str,
    summary: &str,
) -> bool {
    if id.trim().is_empty() || summary.trim().is_empty() {
        return false;
    }
    if nodes.iter().any(|node| node.id == id) {
        return false;
    }

    let resolved_status = status.unwrap_or("open").trim().to_string();
    if resolved_status.is_empty() {
        return false;
    }

    let resolved_weight = weight.unwrap_or(0.0);
    if !is_valid_weight(resolved_weight) {
        return false;
    }

    match parent_id.as_deref() {
        None => {
            if numbering.is_some() {
                return false;
            }
        }
        Some(parent_id_value) => {
            let Some(parent_index) = nodes.iter().position(|node| node.id == parent_id_value) else {
                return false;
            };
            let Some(numbering_value) = numbering.as_deref() else {
                return false;
            };
            if !is_valid_numbering(numbering_value) {
                return false;
            }
            if !is_direct_child_numbering(numbering_value, nodes[parent_index].numbering.as_deref()) {
                return false;
            }
            if nodes.iter().any(|node| {
                node.parent_id.as_deref() == Some(parent_id_value)
                    && node.numbering.as_deref() == Some(numbering_value)
            }) {
                return false;
            }
        }
    }

    nodes.push(GoalNode {
        numbering,
        parent_id,
        status: resolved_status,
        weight: resolved_weight,
        id: id.to_string(),
        summary: summary.to_string(),
    });
    sort_goal_nodes(nodes);
    true
}

fn collect_descendant_ids(nodes: &[GoalNode], target_id: &str) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    ids.insert(target_id.to_string());
    loop {
        let before = ids.len();
        for node in nodes {
            if let Some(parent_id) = node.parent_id.as_deref()
                && ids.contains(parent_id)
            {
                ids.insert(node.id.clone());
            }
        }
        if ids.len() == before {
            break;
        }
    }
    ids
}

fn apply_l1_memory_flush(
    l1_memory: &mut Vec<String>,
    flush_entries: &[String],
    max_l1_memory_entries: usize,
) -> (bool, usize) {
    let max_entries = max_l1_memory_entries.max(1);
    let truncated: Vec<String> = flush_entries.iter().take(max_entries).cloned().collect();
    let overflow_count = flush_entries.len().saturating_sub(max_entries);
    let changed = *l1_memory != truncated;
    if changed {
        *l1_memory = truncated;
    }
    (changed, overflow_count)
}

fn is_valid_weight(weight: f64) -> bool {
    weight.is_finite() && (0.0..=1.0).contains(&weight)
}

fn resolve_selector_index(nodes: &[GoalNode], numbering: Option<&str>, id: Option<&str>) -> Option<usize> {
    if numbering.is_none() && id.is_none() {
        return None;
    }

    let index_by_numbering = numbering.and_then(|value| unique_index_by_numbering(nodes, value));
    let index_by_id = id.and_then(|value| unique_index_by_id(nodes, value));

    match (index_by_numbering, index_by_id) {
        (Some(a), Some(b)) if a == b => Some(a),
        (Some(_), Some(_)) => None,
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}

fn unique_index_by_numbering(nodes: &[GoalNode], numbering: &str) -> Option<usize> {
    let needle = numbering.trim();
    if needle.is_empty() {
        return None;
    }
    let mut matched: Option<usize> = None;
    for (index, node) in nodes.iter().enumerate() {
        if node.numbering.as_deref() != Some(needle) {
            continue;
        }
        if matched.is_some() {
            return None;
        }
        matched = Some(index);
    }
    matched
}

fn unique_index_by_id(nodes: &[GoalNode], id: &str) -> Option<usize> {
    let needle = id.trim();
    if needle.is_empty() {
        return None;
    }
    let mut matched: Option<usize> = None;
    for (index, node) in nodes.iter().enumerate() {
        if node.id != needle {
            continue;
        }
        if matched.is_some() {
            return None;
        }
        matched = Some(index);
    }
    matched
}

fn next_child_numbering(
    nodes: &[GoalNode],
    parent_id: &str,
    parent_numbering: Option<&str>,
) -> String {
    let mut max_child = 0_u64;
    for node in nodes {
        if node.parent_id.as_deref() != Some(parent_id) {
            continue;
        }
        let Some(numbering) = node.numbering.as_deref() else {
            continue;
        };
        let Some(value) = direct_child_index(numbering, parent_numbering) else {
            continue;
        };
        if value > max_child {
            max_child = value;
        }
    }

    let next = max_child + 1;
    match parent_numbering {
        Some(parent) => format!("{parent}.{next}"),
        None => next.to_string(),
    }
}

fn is_direct_child_numbering(numbering: &str, parent_numbering: Option<&str>) -> bool {
    direct_child_index(numbering, parent_numbering).is_some()
}

fn direct_child_index(numbering: &str, parent_numbering: Option<&str>) -> Option<u64> {
    if !is_valid_numbering(numbering) {
        return None;
    }

    match parent_numbering {
        None => {
            if numbering.contains('.') {
                return None;
            }
            numbering.parse::<u64>().ok()
        }
        Some(parent) => {
            let prefix = format!("{parent}.");
            let suffix = numbering.strip_prefix(prefix.as_str())?;
            if suffix.contains('.') {
                return None;
            }
            suffix.parse::<u64>().ok()
        }
    }
}

fn is_valid_numbering(numbering: &str) -> bool {
    if numbering.is_empty() {
        return false;
    }

    for segment in numbering.split('.') {
        if segment.is_empty() {
            return false;
        }
        if !segment.chars().all(|ch| ch.is_ascii_digit()) {
            return false;
        }
        if segment.starts_with('0') && segment.len() > 1 {
            return false;
        }
        if segment == "0" {
            return false;
        }
    }

    true
}

fn sort_goal_nodes(nodes: &mut [GoalNode]) {
    nodes.sort_by(|lhs, rhs| {
        lhs.parent_id
            .cmp(&rhs.parent_id)
            .then_with(|| compare_numbering(lhs.numbering.as_deref(), rhs.numbering.as_deref()))
            .then_with(|| lhs.id.cmp(&rhs.id))
    });
}

fn compare_numbering(lhs: Option<&str>, rhs: Option<&str>) -> std::cmp::Ordering {
    match (lhs, rhs) {
        (None, None) => std::cmp::Ordering::Equal,
        (None, Some(_)) => std::cmp::Ordering::Less,
        (Some(_), None) => std::cmp::Ordering::Greater,
        (Some(lhs), Some(rhs)) => compare_numbering_str(lhs, rhs),
    }
}

fn compare_numbering_str(lhs: &str, rhs: &str) -> std::cmp::Ordering {
    let lhs_parts = parse_numbering(lhs);
    let rhs_parts = parse_numbering(rhs);
    let shared_len = lhs_parts.len().min(rhs_parts.len());

    for idx in 0..shared_len {
        match lhs_parts[idx].cmp(&rhs_parts[idx]) {
            std::cmp::Ordering::Equal => continue,
            ordering => return ordering,
        }
    }

    lhs_parts.len().cmp(&rhs_parts.len())
}

fn parse_numbering(numbering: &str) -> Vec<u64> {
    numbering
        .split('.')
        .map(|segment| segment.parse::<u64>().unwrap_or(0))
        .collect()
}
