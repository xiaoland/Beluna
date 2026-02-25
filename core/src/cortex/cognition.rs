use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GoalNode {
    pub numbering: Option<String>,
    pub parent_id: Option<String>,
    pub status: String,
    pub weight: f64,
    pub id: String,
    pub summary: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum GoalNodeSerde {
    Current {
        #[serde(default)]
        numbering: Option<String>,
        #[serde(default)]
        parent_id: Option<String>,
        status: String,
        weight: f64,
        id: String,
        summary: String,
    },
    Legacy {
        numbering: String,
        status: String,
        weight: f64,
        summary: String,
        #[serde(default)]
        content: Option<String>,
    },
}

impl<'de> Deserialize<'de> for GoalNode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let repr = GoalNodeSerde::deserialize(deserializer)?;
        Ok(match repr {
            GoalNodeSerde::Current {
                numbering,
                parent_id,
                status,
                weight,
                id,
                summary,
            } => Self {
                numbering,
                parent_id,
                status,
                weight,
                id,
                summary,
            },
            GoalNodeSerde::Legacy {
                numbering,
                status,
                weight,
                summary,
                content: _,
            } => Self {
                id: format!("legacy-{}", numbering.replace('.', "-")),
                numbering: Some(numbering),
                parent_id: None,
                status,
                weight,
                summary,
            },
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Default)]
pub struct GoalForest {
    #[serde(default)]
    pub nodes: Vec<GoalNode>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum GoalForestSerde {
    Current {
        #[serde(default)]
        nodes: Vec<GoalNode>,
    },
    Legacy {
        #[serde(default)]
        user_partition: Vec<GoalNode>,
        #[serde(default)]
        root_partition: Vec<String>,
    },
}

impl<'de> Deserialize<'de> for GoalForest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let repr = GoalForestSerde::deserialize(deserializer)?;
        Ok(match repr {
            GoalForestSerde::Current { nodes } => Self {
                nodes: normalize_legacy_flat_forest(nodes),
            },
            GoalForestSerde::Legacy {
                user_partition,
                root_partition: _,
            } => Self {
                nodes: normalize_legacy_flat_forest(user_partition),
            },
        })
    }
}

pub type L1Memory = Vec<String>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CognitionState {
    #[serde(default)]
    pub revision: u64,
    #[serde(default, alias = "goal_tree")]
    pub goal_forest: GoalForest,
    #[serde(default)]
    pub l1_memory: L1Memory,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum GoalForestPatchOp {
    Sprout {
        #[serde(default)]
        parent_numbering: Option<String>,
        #[serde(default)]
        parent_id: Option<String>,
        #[serde(default)]
        numbering: Option<String>,
        #[serde(default)]
        status: Option<String>,
        #[serde(default)]
        weight: Option<f64>,
        id: String,
        summary: String,
    },
    Plant {
        #[serde(default)]
        status: Option<String>,
        #[serde(default)]
        weight: Option<f64>,
        id: String,
        summary: String,
    },
    Trim {
        #[serde(default)]
        numbering: Option<String>,
        #[serde(default)]
        id: Option<String>,
        #[serde(default)]
        weight: Option<f64>,
        #[serde(default)]
        status: Option<String>,
    },
    Prune {
        #[serde(default)]
        numbering: Option<String>,
        #[serde(default)]
        id: Option<String>,
    },
}

pub fn new_default_cognition_state() -> CognitionState {
    CognitionState::default()
}

impl Default for CognitionState {
    fn default() -> Self {
        Self {
            revision: 0,
            goal_forest: GoalForest::default(),
            l1_memory: Vec::new(),
        }
    }
}

fn normalize_legacy_flat_forest(nodes: Vec<GoalNode>) -> Vec<GoalNode> {
    if nodes.iter().any(|node| node.parent_id.is_some()) {
        return nodes;
    }
    if nodes.iter().all(|node| node.numbering.is_none()) {
        return nodes;
    }

    let mut numbering_to_id = std::collections::BTreeMap::new();
    for node in &nodes {
        let Some(numbering) = node.numbering.as_ref() else {
            return nodes;
        };
        if numbering_to_id
            .insert(numbering.clone(), node.id.clone())
            .is_some()
        {
            return nodes;
        }
    }

    let mut migrated = nodes.clone();
    for node in &mut migrated {
        let Some(numbering) = node.numbering.as_ref() else {
            return nodes;
        };
        let segments: Vec<&str> = numbering.split('.').collect();
        if segments.is_empty() {
            return nodes;
        }

        if segments.len() == 1 {
            node.parent_id = None;
            node.numbering = None;
            continue;
        }

        let parent_old_numbering = segments[..segments.len() - 1].join(".");
        let Some(parent_id) = numbering_to_id.get(parent_old_numbering.as_str()) else {
            return nodes;
        };
        node.parent_id = Some(parent_id.clone());
        node.numbering = Some(segments[1..].join("."));
    }

    migrated
}
