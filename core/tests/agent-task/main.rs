mod kit;

use std::path::PathBuf;

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
