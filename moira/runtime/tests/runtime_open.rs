mod common;

use std::net::TcpListener;

use moira_runtime::{
    MoiraPaths, MoiraResourceKind, MoiraResourceState, MoiraRuntime, MoiraRuntimeConfig,
    MoiraRuntimeLifecycle, NoopEventSink, TokioTaskSpawner,
};

use crate::common::{RuntimeSandbox, wait_for_receiver_ready, wait_for_runtime_status};

#[tokio::test]
async fn open_creates_runtime_paths_and_reports_ready_status() {
    let sandbox = RuntimeSandbox::new();
    let runtime = sandbox.open_runtime().await;

    let status = wait_for_receiver_ready(runtime.as_ref()).await;

    assert_eq!(status.lifecycle, MoiraRuntimeLifecycle::Ready);
    assert_eq!(status.receiver.wake_state, "listening");
    assert!(sandbox.root().join("artifacts").is_dir());
    assert!(sandbox.root().join("profiles").is_dir());
    assert!(sandbox.root().join("telemetry").is_dir());
    assert!(status.resources.iter().any(|resource| {
        resource.kind == MoiraResourceKind::OtlpReceiver
            && resource.state == MoiraResourceState::Claimed
    }));
}

#[tokio::test]
async fn open_reports_receiver_bind_conflict_as_resource_status() {
    let root =
        std::env::temp_dir().join(format!("moira-runtime-conflict-{}", uuid::Uuid::now_v7()));
    let listener = TcpListener::bind("127.0.0.1:0").expect("fixture listener should bind");
    let receiver_bind = listener
        .local_addr()
        .expect("fixture listener should expose local address");

    let runtime = MoiraRuntime::open(MoiraRuntimeConfig {
        paths: MoiraPaths::from_root(root.clone()),
        receiver_bind,
        event_sink: std::sync::Arc::new(NoopEventSink),
        task_spawner: std::sync::Arc::new(TokioTaskSpawner),
    })
    .await
    .expect("runtime should open even when receiver bind later faults");

    let status = wait_for_runtime_status(runtime.as_ref(), |status| {
        status.receiver.wake_state == "faulted"
    })
    .await;

    assert_eq!(status.lifecycle, MoiraRuntimeLifecycle::Degraded);
    assert!(status.resources.iter().any(|resource| {
        resource.kind == MoiraResourceKind::OtlpReceiver
            && resource.state == MoiraResourceState::Conflict
    }));

    drop(runtime);
    drop(listener);
    let _ = std::fs::remove_dir_all(root);
}
