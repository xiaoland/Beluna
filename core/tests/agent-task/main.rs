mod kit;

use std::{env, path::PathBuf};

use kit::{case::load_cases, runner::AgentTaskRunner};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn run_agent_task_cases() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let cases_dir = manifest_dir.join("tests/agent-task/cases");
    let cases = load_cases(&cases_dir).expect("agent task cases should load");
    assert!(
        !cases.is_empty(),
        "expected at least one agent task case under {}",
        cases_dir.display()
    );

    let runner = AgentTaskRunner::new(manifest_dir.join("target/agent-task-runs"));
    let mut failures = Vec::new();

    for case in &cases {
        match runner.run(case).await {
            Ok(result) if result.passed => {}
            Ok(result) => {
                failures.push(format!(
                    "{} failed: {}\nartifacts: {}",
                    case.id,
                    result.failures.join("; "),
                    result.artifact_dir.display()
                ));
            }
            Err(err) => {
                failures.push(format!("{} errored: {err}", case.id));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "agent task failures:\n{}",
        failures.join("\n\n")
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "live agent task runs require BELUNA_AGENT_TASK_CONFIG and BELUNA_AGENT_TASK_CASE"]
async fn run_live_agent_task_case() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let cases_dir = manifest_dir.join("tests/agent-task/cases");
    let case_id = env::var("BELUNA_AGENT_TASK_CASE")
        .expect("BELUNA_AGENT_TASK_CASE must name the live case id");
    let config_path = env::var("BELUNA_AGENT_TASK_CONFIG")
        .expect("BELUNA_AGENT_TASK_CONFIG must point to a Core config file");
    let config = beluna::config::Config::load(&PathBuf::from(config_path))
        .expect("live Core config should load");
    let cases = load_cases(&cases_dir).expect("agent task cases should load");
    let case = cases
        .iter()
        .find(|case| case.id == case_id)
        .unwrap_or_else(|| panic!("case '{}' not found under {}", case_id, cases_dir.display()));

    let runner = AgentTaskRunner::new(manifest_dir.join("target/agent-task-runs"));
    let result = runner
        .run_live(case, &config)
        .await
        .expect("live agent task run should complete");

    assert!(
        result.passed,
        "{} failed: {}\nartifacts: {}",
        case.id,
        result.failures.join("; "),
        result.artifact_dir.display()
    );
}
