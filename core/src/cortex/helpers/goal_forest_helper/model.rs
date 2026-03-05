use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoalNode {
    pub status: String,
    pub weight: f64,
    pub id: String,
    pub summary: String,
    #[serde(default, alias = "childrens")]
    pub children: Vec<GoalNode>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct GoalForest {
    #[serde(default)]
    pub nodes: Vec<GoalNode>,
}
