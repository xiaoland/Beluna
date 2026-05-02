use anyhow::{Result, bail};
use serde_json::Value;

use super::{
    case::AgentTaskCase,
    evidence::EvidenceJournal,
    workspace::{CaseWorkspace, FileTreeSnapshot},
};

#[derive(Debug, Clone)]
pub struct RunResult {
    pub passed: bool,
    pub failures: Vec<String>,
    pub artifact_dir: std::path::PathBuf,
}

pub struct OracleEngine;

impl OracleEngine {
    pub fn evaluate(
        case: &AgentTaskCase,
        journal: &EvidenceJournal,
        workspace: &CaseWorkspace,
        after_snapshot: &FileTreeSnapshot,
    ) -> Result<Vec<String>> {
        let events = journal.events();
        let mut failures = Vec::new();

        for expectation in &case.oracle.pass.evidence {
            let matching = events
                .iter()
                .filter(|event| {
                    event.get("stream").and_then(Value::as_str) == Some(&expectation.stream)
                })
                .filter(|event| matcher_matches(event, &expectation.matcher))
                .collect::<Vec<_>>();

            match expectation.exact_count {
                Some(expected) if matching.len() != expected => failures.push(format!(
                    "stream '{}' expected exact_count={}, got {}",
                    expectation.stream,
                    expected,
                    matching.len()
                )),
                None if matching.is_empty() => failures.push(format!(
                    "stream '{}' did not contain a matching event",
                    expectation.stream
                )),
                _ => {}
            }
        }

        if let Some(correlation) = &case.oracle.pass.correlation {
            if correlation.require_act_instance_id
                && !events.iter().any(|event| {
                    event
                        .get("act_instance_id")
                        .and_then(Value::as_str)
                        .map(|value| !value.trim().is_empty())
                        .unwrap_or(false)
                })
            {
                failures.push("correlation requires act_instance_id evidence".to_string());
            }
            if correlation.require_tick
                && !events
                    .iter()
                    .any(|event| event.get("tick").and_then(Value::as_u64).is_some())
            {
                failures.push("correlation requires tick evidence".to_string());
            }
        }

        failures.extend(workspace.evaluate_expectations(&case.oracle.pass.files, after_snapshot));

        Ok(failures)
    }
}

fn matcher_matches(event: &Value, matcher: &std::collections::BTreeMap<String, Value>) -> bool {
    matcher.iter().all(|(key, expected)| {
        if key == "reference_id_prefix" {
            return expected
                .as_str()
                .and_then(|prefix| {
                    event
                        .get("reference_id")
                        .and_then(Value::as_str)
                        .map(|actual| actual.starts_with(prefix))
                })
                .unwrap_or(false);
        }

        event
            .get(key)
            .map(|actual| value_matches(actual, expected))
            .unwrap_or(false)
    })
}

fn value_matches(actual: &Value, expected: &Value) -> bool {
    match (actual, expected) {
        (Value::Object(actual_map), Value::Object(expected_map)) => {
            expected_map.iter().all(|(key, expected_value)| {
                actual_map
                    .get(key)
                    .map(|actual_value| value_matches(actual_value, expected_value))
                    .unwrap_or(false)
            })
        }
        _ => actual == expected,
    }
}

pub fn assert_no_oracle_internal_error(failures: &[String]) -> Result<()> {
    if failures.iter().any(|failure| failure.trim().is_empty()) {
        bail!("oracle produced an empty failure message");
    }
    Ok(())
}
