mod common;

use std::{fs, path::Path};

use moira_runtime::{
    atropos::model::SupervisionPhase,
    clotho::model::{KnownLocalBuildRegistration, WakeInputRequest},
};

use crate::common::{RuntimeSandbox, wait_for_receiver_ready};

#[cfg(unix)]
#[tokio::test]
async fn runtime_atropos_wakes_and_stops_registered_process() {
    let sandbox = RuntimeSandbox::new();
    let runtime = sandbox.open_runtime().await;
    wait_for_receiver_ready(runtime.as_ref()).await;

    let bin_dir = sandbox.root().join("bin");
    fs::create_dir_all(&bin_dir).expect("bin dir should create");
    let executable = bin_dir.join("sleep-core-fixture.sh");
    fs::write(
        &executable,
        "#!/bin/sh\ntrap 'exit 0' TERM\nwhile true; do sleep 1; done\n",
    )
    .expect("process fixture should write");
    make_executable(&executable);

    let target = runtime
        .clotho()
        .register_known_local_build(KnownLocalBuildRegistration {
            build_id: "sleep-core".to_string(),
            executable_path: executable,
            working_dir: Some(bin_dir),
            source_dir: None,
        })
        .expect("known local build should register");

    let running = runtime
        .atropos()
        .wake(WakeInputRequest {
            target,
            profile: None,
        })
        .await
        .expect("Atropos should wake process fixture");
    assert_eq!(running.phase, SupervisionPhase::Running);
    assert!(running.pid.is_some());

    let stopping = runtime
        .atropos()
        .stop()
        .await
        .expect("Atropos should request graceful stop");
    assert_eq!(stopping.phase, SupervisionPhase::Stopping);

    for _ in 0..40 {
        let status = runtime
            .atropos()
            .runtime_status()
            .await
            .expect("Atropos status should load");
        if status.phase == SupervisionPhase::Terminated {
            assert_eq!(status.pid, None);
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    }

    let _ = runtime.atropos().force_kill().await;
    panic!("process fixture did not terminate after graceful stop");
}

#[cfg(unix)]
fn make_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path)
        .expect("process fixture metadata should load")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).expect("process fixture should become executable");
}
