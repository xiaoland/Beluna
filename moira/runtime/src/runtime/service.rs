use std::{net::SocketAddr, sync::Arc};

use thiserror::Error;

use crate::{
    atropos::{AtroposService, model::RuntimeStatus},
    clotho::ClothoService,
    lachesis::LachesisService,
};

use super::{
    MoiraEventSink, MoiraPaths, MoiraResourceKind, MoiraResourceState, MoiraResourceStatus,
    MoiraRuntimeLifecycle, MoiraRuntimeStatus, MoiraTaskSpawner,
};

pub struct MoiraRuntimeConfig {
    pub paths: MoiraPaths,
    pub receiver_bind: SocketAddr,
    pub event_sink: Arc<dyn MoiraEventSink>,
    pub task_spawner: Arc<dyn MoiraTaskSpawner>,
}

pub struct MoiraRuntime {
    clotho: Arc<ClothoService>,
    lachesis: Arc<LachesisService>,
    atropos: Arc<AtroposService>,
}

#[derive(Debug, Error)]
pub enum MoiraRuntimeError {
    #[error("{message}")]
    Message { message: String },
}

impl From<String> for MoiraRuntimeError {
    fn from(message: String) -> Self {
        Self::Message { message }
    }
}

#[derive(Debug, Clone)]
pub struct MoiraShutdownOutcome {
    pub core: Option<RuntimeStatus>,
    pub resources: Vec<MoiraResourceStatus>,
}

impl MoiraRuntime {
    pub async fn open(config: MoiraRuntimeConfig) -> Result<Arc<Self>, MoiraRuntimeError> {
        config.paths.ensure_dirs()?;

        let clotho = Arc::new(ClothoService::new(config.paths.clone()));
        let lachesis = LachesisService::open(
            config.paths.telemetry_db_path(),
            config.receiver_bind.to_string(),
        )
        .await?;
        let atropos = Arc::new(AtroposService::new(
            config.paths,
            clotho.clone(),
            lachesis.clone(),
            config.task_spawner.clone(),
        ));

        let runtime = Arc::new(Self {
            clotho,
            lachesis,
            atropos,
        });

        let lachesis = runtime.lachesis.clone();
        config.task_spawner.spawn(Box::pin(async move {
            lachesis
                .start_receiver(config.receiver_bind, config.event_sink)
                .await;
        }));

        Ok(runtime)
    }

    pub fn clotho(&self) -> &ClothoService {
        self.clotho.as_ref()
    }

    pub fn lachesis(&self) -> &LachesisService {
        self.lachesis.as_ref()
    }

    pub fn atropos(&self) -> &AtroposService {
        self.atropos.as_ref()
    }

    pub async fn status(&self) -> Result<MoiraRuntimeStatus, MoiraRuntimeError> {
        let receiver = self.lachesis.receiver_status().await?;
        let core = self.atropos.runtime_status().await?;
        let lifecycle = if receiver.wake_state == "faulted" {
            MoiraRuntimeLifecycle::Degraded
        } else {
            MoiraRuntimeLifecycle::Ready
        };

        Ok(MoiraRuntimeStatus {
            lifecycle,
            resources: runtime_resources(&receiver, &core),
            receiver,
            core,
        })
    }

    pub async fn shutdown(&self) -> Result<MoiraShutdownOutcome, MoiraRuntimeError> {
        let core = self.atropos.stop_if_running().await?;
        let status = self.status().await?;

        Ok(MoiraShutdownOutcome {
            core,
            resources: status.resources,
        })
    }
}

fn runtime_resources(
    receiver: &crate::lachesis::model::ReceiverStatus,
    core: &RuntimeStatus,
) -> Vec<MoiraResourceStatus> {
    vec![
        MoiraResourceStatus {
            kind: MoiraResourceKind::Directory,
            state: MoiraResourceState::Available,
            label: "Moira data directories".to_string(),
            detail: None,
        },
        MoiraResourceStatus {
            kind: MoiraResourceKind::TelemetryStore,
            state: MoiraResourceState::Claimed,
            label: "Lachesis telemetry store".to_string(),
            detail: Some(receiver.db_path.clone()),
        },
        MoiraResourceStatus {
            kind: MoiraResourceKind::OtlpReceiver,
            state: receiver_resource_state(receiver),
            label: "Lachesis OTLP receiver".to_string(),
            detail: receiver
                .last_error
                .clone()
                .or_else(|| Some(receiver.endpoint.clone())),
        },
        MoiraResourceStatus {
            kind: MoiraResourceKind::CoreSupervisor,
            state: MoiraResourceState::Available,
            label: format!("Atropos phase {:?}", core.phase),
            detail: core.terminal_reason.clone(),
        },
    ]
}

fn receiver_resource_state(
    receiver: &crate::lachesis::model::ReceiverStatus,
) -> MoiraResourceState {
    match receiver.wake_state.as_str() {
        "awakening" => MoiraResourceState::Claiming,
        "listening" | "awake" => MoiraResourceState::Claimed,
        "faulted"
            if receiver
                .last_error
                .as_deref()
                .is_some_and(|error| error.contains("Address already in use"))
                || receiver
                    .last_error
                    .as_deref()
                    .is_some_and(|error| error.contains("os error 48")) =>
        {
            MoiraResourceState::Conflict
        }
        "faulted" => MoiraResourceState::Faulted,
        _ => MoiraResourceState::Degraded,
    }
}
