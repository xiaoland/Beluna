#![allow(dead_code)]

use std::{
    net::{SocketAddr, TcpListener},
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use moira_runtime::{
    MoiraPaths, MoiraRuntime, MoiraRuntimeConfig, MoiraRuntimeStatus, NoopEventSink,
    TokioTaskSpawner,
};
use tokio::time::sleep;
use uuid::Uuid;

pub struct RuntimeSandbox {
    root: PathBuf,
    receiver_bind: SocketAddr,
}

impl RuntimeSandbox {
    pub fn new() -> Self {
        let root = std::env::temp_dir().join(format!("moira-runtime-it-{}", Uuid::now_v7()));
        let listener =
            TcpListener::bind("127.0.0.1:0").expect("sandbox should reserve a local port");
        let receiver_bind = listener
            .local_addr()
            .expect("sandbox listener should expose local address");
        drop(listener);

        Self {
            root,
            receiver_bind,
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn receiver_bind(&self) -> SocketAddr {
        self.receiver_bind
    }

    pub async fn open_runtime(&self) -> Arc<MoiraRuntime> {
        MoiraRuntime::open(MoiraRuntimeConfig {
            paths: MoiraPaths::from_root(self.root.clone()),
            receiver_bind: self.receiver_bind,
            event_sink: Arc::new(NoopEventSink),
            task_spawner: Arc::new(TokioTaskSpawner),
        })
        .await
        .expect("runtime should open")
    }
}

impl Drop for RuntimeSandbox {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

pub async fn wait_for_receiver_ready(runtime: &MoiraRuntime) -> MoiraRuntimeStatus {
    wait_for_runtime_status(runtime, |status| {
        matches!(status.receiver.wake_state.as_str(), "listening" | "awake")
    })
    .await
}

pub async fn wait_for_runtime_status(
    runtime: &MoiraRuntime,
    mut predicate: impl FnMut(&MoiraRuntimeStatus) -> bool,
) -> MoiraRuntimeStatus {
    let mut last_status = runtime.status().await.expect("runtime status should load");
    for _ in 0..40 {
        if predicate(&last_status) {
            return last_status;
        }
        sleep(Duration::from_millis(25)).await;
        last_status = runtime.status().await.expect("runtime status should load");
    }

    last_status
}
