use crate::cortex::cognition::{CognitionState, GoalNode, GoalTreePatchOp};

pub(crate) struct CognitionPatchApplyResult {
    pub new_cognition_state: CognitionState,
    pub l1_memory_overflow_count: usize,
}

pub(crate) fn apply_cognition_patches(
    previous: &CognitionState,
    goal_tree_ops: &[GoalTreePatchOp],
    l1_memory_flush: &[String],
    max_l1_memory_entries: usize,
) -> CognitionPatchApplyResult {
    let mut next = previous.clone();
    let mut changed = false;

    for op in goal_tree_ops {
        if apply_goal_tree_op(&mut next.goal_tree.user_partition, op) {
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

fn apply_goal_tree_op(user_partition: &mut Vec<GoalNode>, op: &GoalTreePatchOp) -> bool {
    match op {
        GoalTreePatchOp::Sprout {
            numbering,
            node_id,
            summary,
            weight,
        } => {
            if !is_valid_numbering(numbering) {
                return false;
            }
            if user_partition
                .iter()
                .any(|node| node.numbering == *numbering)
            {
                return false;
            }

            let Some(normalized_weight) = normalize_weight(*weight, user_partition) else {
                return false;
            };

            user_partition.push(GoalNode {
                numbering: numbering.clone(),
                node_id: node_id.clone(),
                summary: summary.clone(),
                weight: normalized_weight,
            });
            true
        }
        GoalTreePatchOp::Prune { numbering } => {
            if !is_valid_numbering(numbering) {
                return false;
            }

            let descendant_prefix = format!("{numbering}.");
            let original_len = user_partition.len();
            user_partition.retain(|node| {
                node.numbering != *numbering && !node.numbering.starts_with(&descendant_prefix)
            });
            user_partition.len() != original_len
        }
        GoalTreePatchOp::Tilt { numbering, weight } => {
            if !is_valid_numbering(numbering) {
                return false;
            }
            let Some(idx) = user_partition
                .iter()
                .position(|node| node.numbering == *numbering)
            else {
                return false;
            };

            let Some(normalized_weight) = normalize_weight(*weight, user_partition) else {
                return false;
            };

            if (user_partition[idx].weight - normalized_weight).abs() <= f64::EPSILON {
                return false;
            }
            user_partition[idx].weight = normalized_weight;
            true
        }
    }
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

fn normalize_weight(weight: f64, user_partition: &[GoalNode]) -> Option<f64> {
    if !weight.is_finite() {
        return None;
    }

    if user_partition.is_empty() {
        return Some(0.5);
    }

    let mut min_weight = user_partition[0].weight;
    let mut max_weight = user_partition[0].weight;
    for node in user_partition.iter().skip(1) {
        if node.weight < min_weight {
            min_weight = node.weight;
        }
        if node.weight > max_weight {
            max_weight = node.weight;
        }
    }

    let span = max_weight - min_weight;
    if span.abs() <= f64::EPSILON {
        return Some(0.5);
    }

    if weight < min_weight || weight > max_weight {
        return None;
    }

    Some((weight - min_weight) / span)
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
