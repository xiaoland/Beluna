use tokio::time::Duration;
use std::collections::{BTreeMap, BTreeSet};

use crate::cortex::{
    cognition::{GoalForest, GoalNode},
    helpers::{self, HelperRuntime},
};

const GOAL_FOREST_EMPTY_FALLBACK: &str = concat!(
    "There's no trees in the goal forest currently.\n",
    "Try to plan some trees, and then plant, sprout, prune, trim them."
);

#[derive(Clone, Default)]
pub(crate) struct GoalForestInputHelper;

impl GoalForestInputHelper {
    pub(crate) async fn to_input_ir_section(
        &self,
        _runtime: &impl HelperRuntime,
        cycle_id: u64,
        _deadline: Duration,
        goal_forest: &GoalForest,
    ) -> String {
        let stage = "goal_forest_input_helper";
        let input_payload = helpers::pretty_json(&serde_json::json!({
            "goal_forest": goal_forest,
        }));
        helpers::log_organ_input(cycle_id, stage, &input_payload);

        let output = if goal_forest.nodes.is_empty() {
            goal_forest_empty_one_shot().to_string()
        } else {
            goal_forest_ascii(&goal_forest.nodes)
        };

        helpers::log_organ_output(cycle_id, stage, &output);
        output
    }
}

pub(crate) fn goal_forest_json(goal_forest_nodes: &[GoalNode]) -> String {
    serde_json::to_string_pretty(goal_forest_nodes).unwrap_or_else(|_| "[]".to_string())
}

pub(crate) fn goal_forest_ascii(goal_forest_nodes: &[GoalNode]) -> String {
    let mut children_by_parent: BTreeMap<Option<&str>, Vec<&GoalNode>> = BTreeMap::new();
    for node in goal_forest_nodes {
        children_by_parent
            .entry(node.parent_id.as_deref())
            .or_default()
            .push(node);
    }

    for children in children_by_parent.values_mut() {
        children.sort_by(|lhs, rhs| compare_goal_node(lhs, rhs));
    }

    let mut lines = Vec::new();
    let mut visited = BTreeSet::new();

    if let Some(roots) = children_by_parent.get(&None) {
        for root in roots {
            append_goal_node_lines(
                &mut lines,
                &mut visited,
                &children_by_parent,
                root,
                0,
            );
        }
    }

    for node in goal_forest_nodes {
        if visited.contains(node.id.as_str()) {
            continue;
        }
        append_goal_node_lines(
            &mut lines,
            &mut visited,
            &children_by_parent,
            node,
            0,
        );
    }

    lines.join("\n")
}

fn append_goal_node_lines(
    lines: &mut Vec<String>,
    visited: &mut BTreeSet<String>,
    children_by_parent: &BTreeMap<Option<&str>, Vec<&GoalNode>>,
    node: &GoalNode,
    depth: usize,
) {
    if !visited.insert(node.id.clone()) {
        return;
    }

    let prefix = if depth == 0 {
        "+-- ".to_string()
    } else {
        format!("{}|-- ", "    ".repeat(depth))
    };
    let numbering = node.numbering.as_deref().unwrap_or("null");
    lines.push(format!(
        "{prefix}{} [{}] (w={:.2}) id={} :: {}",
        numbering, node.status, node.weight, node.id, node.summary
    ));

    if let Some(children) = children_by_parent.get(&Some(node.id.as_str())) {
        for child in children {
            append_goal_node_lines(lines, visited, children_by_parent, child, depth + 1);
        }
    }
}

fn compare_goal_node(lhs: &GoalNode, rhs: &GoalNode) -> std::cmp::Ordering {
    compare_numbering(lhs.numbering.as_deref(), rhs.numbering.as_deref())
        .then_with(|| lhs.id.cmp(&rhs.id))
}

pub(crate) fn goal_forest_empty_one_shot() -> &'static str {
    GOAL_FOREST_EMPTY_FALLBACK
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
