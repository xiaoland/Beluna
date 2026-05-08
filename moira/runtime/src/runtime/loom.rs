use serde::Serialize;

use crate::{
    clotho::model::{LaunchTargetSummary, ProfileDocumentSummary},
    lachesis::model::{RunSummary, TickDetail, TickSummary},
};

use super::{MoiraRuntime, MoiraRuntimeError, MoiraRuntimeStatus};

#[derive(Debug, Clone, Default)]
pub struct MoiraLoomSelection {
    pub run_id: Option<String>,
    pub tick: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MoiraLoomSnapshot {
    pub status: MoiraRuntimeStatus,
    pub launch_targets: Vec<LaunchTargetSummary>,
    pub profiles: Vec<ProfileDocumentSummary>,
    pub runs: Vec<RunSummary>,
    pub selected_run_id: Option<String>,
    pub ticks: Vec<TickSummary>,
    pub selected_tick: Option<u64>,
    pub tick_detail: Option<TickDetail>,
}

impl MoiraRuntime {
    pub async fn loom_snapshot(
        &self,
        selection: MoiraLoomSelection,
    ) -> Result<MoiraLoomSnapshot, MoiraRuntimeError> {
        let status = self.status().await?;
        let launch_targets = self.clotho().list_launch_targets()?;
        let profiles = self.clotho().list_profile_documents()?;
        let runs = self.lachesis().list_runs().await?;

        let selected_run_id = select_run_id(selection.run_id, &runs);
        let ticks = if let Some(run_id) = selected_run_id.as_deref() {
            self.lachesis().list_ticks(run_id).await?
        } else {
            Vec::new()
        };
        let selected_tick = select_tick(selection.tick, &ticks);
        let tick_detail = match (selected_run_id.as_deref(), selected_tick) {
            (Some(run_id), Some(tick)) => Some(self.lachesis().tick_detail(run_id, tick).await?),
            _ => None,
        };

        Ok(MoiraLoomSnapshot {
            status,
            launch_targets,
            profiles,
            runs,
            selected_run_id,
            ticks,
            selected_tick,
            tick_detail,
        })
    }
}

fn select_run_id(requested: Option<String>, runs: &[RunSummary]) -> Option<String> {
    requested
        .filter(|run_id| runs.iter().any(|run| run.run_id == *run_id))
        .or_else(|| runs.first().map(|run| run.run_id.clone()))
}

fn select_tick(requested: Option<u64>, ticks: &[TickSummary]) -> Option<u64> {
    requested
        .filter(|tick| ticks.iter().any(|summary| summary.tick == *tick))
        .or_else(|| ticks.first().map(|summary| summary.tick))
}
