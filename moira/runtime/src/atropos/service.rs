use std::{process::ExitStatus, sync::Arc, time::Duration};

use tokio::{
    process::Child,
    sync::Mutex,
    time::{Instant, sleep},
};

use crate::{
    MoiraPaths, MoiraTaskSpawner,
    clotho::{
        ClothoService,
        model::{PreparedRuntimeWakeInput, WakeInputRequest},
    },
    lachesis::LachesisService,
};

use super::{
    model::{RuntimeStatus, SupervisionPhase},
    wake_command::build_wake_command,
};
const MONITOR_INTERVAL: Duration = Duration::from_millis(250);
const GRACEFUL_STOP_SETTLE_TIMEOUT: Duration = Duration::from_secs(8);
const FORCE_KILL_SETTLE_TIMEOUT: Duration = Duration::from_secs(2);

pub struct AtroposService {
    #[allow(dead_code)]
    paths: MoiraPaths,
    clotho: Arc<ClothoService>,
    lachesis: Arc<LachesisService>,
    task_spawner: Arc<dyn MoiraTaskSpawner>,
    state: Arc<Mutex<RuntimeState>>,
}
struct RuntimeState {
    phase: SupervisionPhase,
    terminal_reason: Option<String>,
    last_wake_input: Option<PreparedRuntimeWakeInput>,
    running: Option<RunningProcess>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TerminationIntent {
    GracefulStop,
    ForceKill,
}

struct RunningProcess {
    pid: u32,
    termination_intent: Option<TerminationIntent>,
    wake_input: PreparedRuntimeWakeInput,
    child: Child,
}
impl RuntimeState {
    fn new() -> Self {
        Self {
            phase: SupervisionPhase::Idle,
            terminal_reason: None,
            last_wake_input: None,
            running: None,
        }
    }
    fn wake_input(&self) -> Option<&PreparedRuntimeWakeInput> {
        self.running
            .as_ref()
            .map(|running| &running.wake_input)
            .or(self.last_wake_input.as_ref())
    }
    fn status(&self) -> RuntimeStatus {
        let wake_input = self.wake_input();

        RuntimeStatus {
            phase: self.phase,
            target_label: wake_input.map(|value| value.target.target_label.clone()),
            executable_path: wake_input.map(|value| value.target.executable_path.clone()),
            working_dir: wake_input.map(|value| value.target.working_dir.clone()),
            profile_path: wake_input.and_then(|value| value.profile_path.clone()),
            pid: self.running.as_ref().map(|running| running.pid),
            terminal_reason: self.terminal_reason.clone(),
        }
    }
}
impl AtroposService {
    pub fn new(
        paths: MoiraPaths,
        clotho: Arc<ClothoService>,
        lachesis: Arc<LachesisService>,
        task_spawner: Arc<dyn MoiraTaskSpawner>,
    ) -> Self {
        Self {
            paths,
            clotho,
            lachesis,
            task_spawner,
            state: Arc::new(Mutex::new(RuntimeState::new())),
        }
    }

    #[allow(dead_code)]
    pub fn paths(&self) -> &MoiraPaths {
        &self.paths
    }
    pub async fn runtime_status(&self) -> Result<RuntimeStatus, String> {
        let mut guard = self.state.lock().await;
        sync_terminal_state(&mut guard);
        Ok(guard.status())
    }
    pub async fn wake(&self, request: WakeInputRequest) -> Result<RuntimeStatus, String> {
        self.reserve_wake_slot().await?;

        let result = self.try_wake(request).await;
        if let Err(error) = &result {
            self.mark_terminal(format!("wake_failed: {error}")).await;
        }

        result
    }
    pub async fn stop(&self) -> Result<RuntimeStatus, String> {
        let Some(status) = self.request_graceful_stop().await? else {
            return Err("no supervised Core is currently running".to_string());
        };

        if status.phase == SupervisionPhase::Stopping {
            return self
                .wait_for_terminal_state(GRACEFUL_STOP_SETTLE_TIMEOUT)
                .await;
        }

        Ok(status)
    }

    pub async fn force_kill(&self) -> Result<RuntimeStatus, String> {
        let status = {
            let mut guard = self.state.lock().await;
            sync_terminal_state(&mut guard);

            let Some(running) = guard.running.as_mut() else {
                return Err("no supervised Core is currently running".to_string());
            };
            if matches!(
                running.termination_intent,
                Some(TerminationIntent::ForceKill)
            ) {
                guard.status()
            } else {
                running.child.start_kill().map_err(|err| {
                    format!(
                        "failed to force-kill supervised Core pid={}: {err}",
                        running.pid
                    )
                })?;
                running.termination_intent = Some(TerminationIntent::ForceKill);
                guard.phase = SupervisionPhase::Stopping;
                guard.status()
            }
        };

        if status.phase == SupervisionPhase::Stopping {
            return self
                .wait_for_terminal_state(FORCE_KILL_SETTLE_TIMEOUT)
                .await;
        }

        Ok(status)
    }

    pub async fn stop_if_running(&self) -> Result<Option<RuntimeStatus>, String> {
        let Some(status) = self.request_graceful_stop().await? else {
            return Ok(None);
        };

        if status.phase == SupervisionPhase::Stopping {
            return self
                .wait_for_terminal_state(GRACEFUL_STOP_SETTLE_TIMEOUT)
                .await
                .map(Some);
        }

        Ok(Some(status))
    }

    async fn request_graceful_stop(&self) -> Result<Option<RuntimeStatus>, String> {
        let mut guard = self.state.lock().await;
        sync_terminal_state(&mut guard);

        let Some(running) = guard.running.as_mut() else {
            return Ok(None);
        };
        if running.termination_intent.is_some() {
            return Ok(Some(guard.status()));
        }

        send_graceful_stop(running.pid)?;
        running.termination_intent = Some(TerminationIntent::GracefulStop);
        guard.phase = SupervisionPhase::Stopping;

        Ok(Some(guard.status()))
    }
    async fn wait_for_terminal_state(&self, timeout: Duration) -> Result<RuntimeStatus, String> {
        let deadline = Instant::now() + timeout;

        loop {
            let status = {
                let mut guard = self.state.lock().await;
                sync_terminal_state(&mut guard);
                guard.status()
            };
            if status.phase != SupervisionPhase::Stopping || Instant::now() >= deadline {
                return Ok(status);
            }

            sleep(MONITOR_INTERVAL).await;
        }
    }
    async fn reserve_wake_slot(&self) -> Result<(), String> {
        let mut guard = self.state.lock().await;
        sync_terminal_state(&mut guard);

        if matches!(
            guard.phase,
            SupervisionPhase::Waking | SupervisionPhase::Running | SupervisionPhase::Stopping
        ) {
            return Err(format!(
                "cannot wake Core while Atropos phase is {:?}",
                guard.phase
            ));
        }

        guard.phase = SupervisionPhase::Waking;
        guard.terminal_reason = None;
        guard.last_wake_input = None;
        Ok(())
    }
    async fn try_wake(&self, request: WakeInputRequest) -> Result<RuntimeStatus, String> {
        ensure_receiver_ready(self.lachesis.as_ref()).await?;
        let wake_input = self.clotho.prepare_runtime_wake_input(&request)?;

        let mut command = build_wake_command(&wake_input);

        let mut child = command.spawn().map_err(|err| {
            format!(
                "failed to wake Core from `{}`: {err}",
                wake_input.target.executable_path.display()
            )
        })?;
        let pid = child.id().ok_or_else(|| {
            let _ = child.start_kill();
            "spawned Core without a process id; wake aborted".to_string()
        })?;

        let status = {
            let mut guard = self.state.lock().await;
            guard.phase = SupervisionPhase::Running;
            guard.terminal_reason = None;
            guard.last_wake_input = Some(wake_input.clone());
            guard.running = Some(RunningProcess {
                pid,
                termination_intent: None,
                wake_input,
                child,
            });
            guard.status()
        };

        self.spawn_monitor();

        Ok(status)
    }
    async fn mark_terminal(&self, reason: String) {
        let mut guard = self.state.lock().await;
        if let Some(mut running) = guard.running.take() {
            let _ = running.child.start_kill();
        }
        guard.phase = SupervisionPhase::Terminated;
        guard.terminal_reason = Some(reason);
    }
    fn spawn_monitor(&self) {
        let state = self.state.clone();
        self.task_spawner.spawn(Box::pin(async move {
            loop {
                let should_stop = {
                    let mut guard = state.lock().await;
                    sync_terminal_state(&mut guard);
                    guard.running.is_none()
                };
                if should_stop {
                    break;
                }
                sleep(MONITOR_INTERVAL).await;
            }
        }));
    }
}

async fn ensure_receiver_ready(lachesis: &LachesisService) -> Result<(), String> {
    let status = lachesis.receiver_status().await?;
    if matches!(status.wake_state.as_str(), "listening" | "awake") {
        return Ok(());
    }

    let last_error = status
        .last_error
        .as_ref()
        .map(|value| format!(" last_error={value}"))
        .unwrap_or_default();

    Err(format!(
        "cannot wake Core before Lachesis receiver is ready; receiver_state={} endpoint={}{}",
        status.wake_state, status.endpoint, last_error
    ))
}

fn sync_terminal_state(state: &mut RuntimeState) {
    let Some(running) = state.running.as_mut() else {
        return;
    };

    match running.child.try_wait() {
        Ok(Some(exit_status)) => {
            let terminal_reason = describe_exit_status(&exit_status, running.termination_intent);
            state.running = None;
            state.phase = SupervisionPhase::Terminated;
            state.terminal_reason = Some(terminal_reason);
        }
        Ok(None) => {}
        Err(err) => {
            state.running = None;
            state.phase = SupervisionPhase::Terminated;
            state.terminal_reason = Some(format!("failed_to_poll_exit: {err}"));
        }
    }
}

fn describe_exit_status(status: &ExitStatus, intent: Option<TerminationIntent>) -> String {
    let prefix = match intent {
        Some(TerminationIntent::GracefulStop) => "graceful_stop",
        Some(TerminationIntent::ForceKill) => "force_kill",
        None => "process_exit",
    };

    match status.code() {
        Some(code) => format!("{prefix}(code={code})"),
        None => describe_signal_exit(status, prefix),
    }
}

#[cfg(unix)]
fn describe_signal_exit(status: &ExitStatus, prefix: &str) -> String {
    use std::os::unix::process::ExitStatusExt;

    match status.signal() {
        Some(signal) => format!("{prefix}(signal={signal})"),
        None => format!("{prefix}(unknown)"),
    }
}

#[cfg(not(unix))]
fn describe_signal_exit(_status: &ExitStatus, prefix: &str) -> String {
    format!("{prefix}(unknown)")
}

#[cfg(unix)]
fn send_graceful_stop(pid: u32) -> Result<(), String> {
    let outcome = unsafe { libc::kill(pid as i32, libc::SIGTERM) };
    if outcome == 0 {
        Ok(())
    } else {
        Err(format!(
            "failed to send SIGTERM to supervised Core pid={pid}: {}",
            std::io::Error::last_os_error()
        ))
    }
}

#[cfg(not(unix))]
fn send_graceful_stop(_pid: u32) -> Result<(), String> {
    Err("graceful stop is only implemented for unix targets in this slice".to_string())
}
