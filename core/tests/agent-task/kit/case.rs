#![allow(dead_code)]

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Clone, Deserialize)]
pub struct AgentTaskCase {
    #[serde(skip)]
    pub source_path: PathBuf,
    #[serde(skip)]
    pub base_dir: PathBuf,
    pub schema_version: u64,
    pub id: String,
    pub title: String,
    pub task: TaskSpec,
    pub world: WorldSpec,
    pub ai: AiSpec,
    pub runtime: RuntimeSpec,
    pub oracle: OracleSpec,
    #[serde(default)]
    pub metrics: Option<Value>,
    #[serde(default)]
    pub artifacts: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TaskSpec {
    pub user_intent: String,
    pub success_claim: String,
    pub injected_sense: InjectedSenseSpec,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InjectedSenseSpec {
    pub endpoint_id: String,
    pub neural_signal_descriptor_id: String,
    pub payload: String,
    pub weight: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorldSpec {
    pub root: String,
    #[serde(default)]
    pub files: Vec<Value>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    #[serde(default)]
    pub endpoints: Vec<EndpointSpec>,
    pub continuity: ContinuitySpec,
    pub proprioception: ProprioceptionSpec,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EndpointSpec {
    pub id: String,
    pub kind: String,
    #[serde(default)]
    pub descriptors: Vec<DescriptorSpec>,
    #[serde(default)]
    pub response: Option<EndpointResponseSpec>,
    #[serde(default)]
    pub preflight: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DescriptorSpec {
    #[serde(rename = "type")]
    pub signal_type: String,
    pub neural_signal_descriptor_id: String,
    pub payload_schema: Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EndpointResponseSpec {
    pub outcome: String,
    pub reference_id_template: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ContinuitySpec {
    pub initial_state: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProprioceptionSpec {
    #[serde(default)]
    pub entries: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AiSpec {
    pub mode: String,
    pub provider: String,
    #[serde(default = "default_model")]
    pub model: String,
    pub fixtures: AiFixturesSpec,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AiFixturesSpec {
    pub root: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RuntimeSpec {
    pub harness: String,
    pub tick_source: String,
    pub max_ticks: u64,
    pub max_primary_turns: u8,
    pub max_model_calls: u64,
    pub max_acts: usize,
    pub timeout_ms: u64,
    #[serde(default)]
    pub exercised_path: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OracleSpec {
    pub pass: OraclePassSpec,
    #[serde(default)]
    pub fail: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OraclePassSpec {
    #[serde(default)]
    pub evidence: Vec<EvidenceExpectation>,
    #[serde(default)]
    pub files: Vec<FileExpectationSpec>,
    #[serde(default)]
    pub diagnostics: Option<Value>,
    #[serde(default)]
    pub correlation: Option<CorrelationSpec>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FileExpectationSpec {
    pub path: String,
    #[serde(default)]
    pub exists: Option<bool>,
    #[serde(default)]
    pub absent: bool,
    #[serde(default)]
    pub content_exact: Option<String>,
    #[serde(default)]
    pub content_trimmed_exact: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EvidenceExpectation {
    pub stream: String,
    #[serde(default)]
    pub exact_count: Option<usize>,
    #[serde(rename = "match", default)]
    pub matcher: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CorrelationSpec {
    #[serde(default)]
    pub require_act_instance_id: bool,
    #[serde(default)]
    pub require_tick: bool,
}

impl AgentTaskCase {
    pub fn fixture_root(&self) -> PathBuf {
        self.base_dir.join(&self.ai.fixtures.root)
    }
}

pub fn load_cases(root: &Path) -> Result<Vec<AgentTaskCase>> {
    let mut candidates = Vec::new();
    for entry in fs::read_dir(root)
        .with_context(|| format!("failed to read case directory {}", root.display()))?
    {
        let path = entry?.path();
        if path.is_dir() {
            let case_path = path.join("case.yaml");
            if case_path.exists() {
                candidates.push(case_path);
            }
            continue;
        }
        if is_case_file(&path) {
            candidates.push(path);
        }
    }
    candidates.sort();

    let mut cases = Vec::with_capacity(candidates.len());
    for path in candidates {
        cases.push(load_case_file(&path)?);
    }
    Ok(cases)
}

fn load_case_file(path: &Path) -> Result<AgentTaskCase> {
    let content =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    let mut case: AgentTaskCase = serde_json::from_str(&content).with_context(|| {
        format!(
            concat!(
                "failed to parse {}; current loader accepts the JSON-compatible ",
                "subset used by the first agent task cases"
            ),
            path.display()
        )
    })?;
    if case.id.trim().is_empty() {
        bail!("case id cannot be empty in {}", path.display());
    }
    case.source_path = path.to_path_buf();
    case.base_dir = path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    Ok(case)
}

fn is_case_file(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .map(|ext| matches!(ext, "json" | "yaml" | "yml"))
        .unwrap_or(false)
}

fn default_model() -> String {
    "fixture-model".to_string()
}
