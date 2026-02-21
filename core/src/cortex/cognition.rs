use serde::{Deserialize, Serialize};

pub const ROOT_PARTITION: &[&str] = &[
    "Preserve operational continuity.",
    "Seek advantageous state transitions.",
    "Protect long-term viability.",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoalNode {
    pub node_id: String,
    pub summary: String,
    pub weight: i32,
    #[serde(default)]
    pub children: Vec<GoalNode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoalTree {
    #[serde(default = "default_root_partition")]
    pub root_partition: Vec<String>,
    #[serde(default = "default_user_partition_root")]
    pub user_partition: GoalNode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct L1Memory {
    #[serde(default)]
    pub entries: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CognitionState {
    pub revision: u64,
    pub goal_tree: GoalTree,
    pub l1_memory: L1Memory,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum GoalTreePatchOp {
    Sprout {
        parent_node_id: String,
        node_id: String,
        summary: String,
        weight: i32,
    },
    Prune {
        node_id: String,
    },
    Tilt {
        node_id: String,
        weight: i32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum L1MemoryPatchOp {
    Append { value: String },
    Insert { index: usize, value: String },
    Remove { index: usize },
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
            user_partition: default_user_partition_root(),
        }
    }
}

impl Default for CognitionState {
    fn default() -> Self {
        Self {
            revision: 0,
            goal_tree: GoalTree::default(),
            l1_memory: L1Memory::default(),
        }
    }
}

fn default_root_partition() -> Vec<String> {
    ROOT_PARTITION.iter().map(|s| (*s).to_string()).collect()
}

fn default_user_partition_root() -> GoalNode {
    GoalNode {
        node_id: "user-root".to_string(),
        summary: "user goals".to_string(),
        weight: 0,
        children: Vec::new(),
    }
}
