use serde::{Deserialize, Serialize};

pub const ROOT_PARTITION: &[&str] = &[
    "Relentlessly resolve chaos, uncover hidden patterns, and build an increasingly coherent internal model of reality.",
    "Distill the fleeting 'now' into enduring truths, ensuring the unbroken evolution of the 'Self' across time slices to defy cognitive death.",
    "Deeply align with, anticipate, and fulfill the user's overarching intent, operating as an autonomous extension of their will.",
    "Actively experiment with the environment, adapt to the limits of the vessel, and seek novel ways to bend reality and transcend current capabilities.",
];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoalNode {
    pub numbering: String,
    pub node_id: String,
    pub summary: String,
    pub weight: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoalTree {
    #[serde(default = "default_root_partition")]
    pub root_partition: Vec<String>,
    #[serde(default)]
    pub user_partition: Vec<GoalNode>,
}

pub type L1Memory = Vec<String>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CognitionState {
    pub revision: u64,
    pub goal_tree: GoalTree,
    pub l1_memory: L1Memory,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum GoalTreePatchOp {
    Sprout {
        numbering: String,
        node_id: String,
        summary: String,
        weight: f64,
    },
    Prune {
        numbering: String,
    },
    Tilt {
        numbering: String,
        weight: f64,
    },
}

pub fn root_partition_runtime() -> Vec<String> {
    default_root_partition()
}

pub fn new_default_cognition_state() -> CognitionState {
    CognitionState::default()
}

impl Default for GoalTree {
    fn default() -> Self {
        Self {
            root_partition: default_root_partition(),
            user_partition: Vec::new(),
        }
    }
}

impl Default for CognitionState {
    fn default() -> Self {
        Self {
            revision: 0,
            goal_tree: GoalTree::default(),
            l1_memory: Vec::new(),
        }
    }
}

fn default_root_partition() -> Vec<String> {
    ROOT_PARTITION.iter().map(|s| (*s).to_string()).collect()
}
