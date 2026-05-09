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
    assert_eq!(stopping.phase, SupervisionPhase::Terminated);
    assert_eq!(stopping.pid, None);
    assert!(
        stopping
            .terminal_reason
            .as_deref()
            .is_some_and(|reason| reason.starts_with("graceful_stop("))
    );

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
#[tokio::test(flavor = "current_thread")]
async fn runtime_atropos_restores_child_signal_mask_before_wake() {
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
            build_id: "masked-parent-core".to_string(),
            executable_path: executable,
            working_dir: Some(bin_dir),
            source_dir: None,
        })
        .expect("known local build should register");

    let signal_mask = block_sigterm_for_current_thread();
    let running = runtime
        .atropos()
        .wake(WakeInputRequest {
            target,
            profile: None,
        })
        .await
        .expect("Atropos should wake process fixture");
    drop(signal_mask);

    assert_eq!(running.phase, SupervisionPhase::Running);
    assert!(running.pid.is_some());

    let stopped = runtime
        .atropos()
        .stop()
        .await
        .expect("Atropos should gracefully stop process fixture");
    assert_eq!(stopped.phase, SupervisionPhase::Terminated);
    assert_eq!(stopped.pid, None);
    assert!(
        stopped
            .terminal_reason
            .as_deref()
            .is_some_and(|reason| reason.starts_with("graceful_stop("))
    );
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

#[cfg(unix)]
struct SignalMaskGuard {
    previous: libc::sigset_t,
}

#[cfg(unix)]
impl Drop for SignalMaskGuard {
    fn drop(&mut self) {
        unsafe {
            libc::pthread_sigmask(libc::SIG_SETMASK, &self.previous, std::ptr::null_mut());
        }
    }
}

#[cfg(unix)]
fn block_sigterm_for_current_thread() -> SignalMaskGuard {
    unsafe {
        let mut blocked = std::mem::MaybeUninit::<libc::sigset_t>::uninit();
        assert_eq!(libc::sigemptyset(blocked.as_mut_ptr()), 0);
        let mut blocked = blocked.assume_init();
        assert_eq!(libc::sigaddset(&mut blocked, libc::SIGTERM), 0);

        let mut previous = std::mem::MaybeUninit::<libc::sigset_t>::uninit();
        assert_eq!(
            libc::pthread_sigmask(libc::SIG_BLOCK, &blocked, previous.as_mut_ptr()),
            0
        );

        SignalMaskGuard {
            previous: previous.assume_init(),
        }
    }
}
