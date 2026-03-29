use std::collections::BTreeSet;

use serde_json::Value;
use thiserror::Error;

use super::{
    ContractEvent, CortexGoalForestSnapshotEvent, CortexOrganRequestEvent,
    CortexOrganResponseEvent, CortexTickEvent, DescriptorCatalogChangeMode, FIXTURE_SCHEMA_VERSION,
    FixtureBundle, FixtureCase, OrganResponseStatus, SpineAdapterLifecycleEvent,
    SpineDispatchOutcomeEvent, SpineEndpointLifecycleEvent, StemDescriptorCatalogEvent,
    StemDispatchTransitionEvent, StemSignalTransitionEvent, TransitionKind,
};

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{context}: {detail}")]
pub struct ContractValidationError {
    context: String,
    detail: String,
}

impl FixtureBundle {
    pub fn validate(&self) -> Result<(), ContractValidationError> {
        require(
            self.schema_version == FIXTURE_SCHEMA_VERSION,
            "fixture_bundle",
            format!(
                "schema_version must be {FIXTURE_SCHEMA_VERSION}, got {}",
                self.schema_version
            ),
        )?;
        require(
            !self.fixtures.is_empty(),
            "fixture_bundle",
            "fixtures must not be empty",
        )?;

        let mut fixture_ids = BTreeSet::new();
        for fixture in &self.fixtures {
            require(
                fixture_ids.insert(fixture.fixture_id.clone()),
                "fixture_bundle",
                format!("duplicate fixture_id `{}`", fixture.fixture_id),
            )?;
            fixture.validate(self.subsystem)?;
        }
        Ok(())
    }
}

impl FixtureCase {
    fn validate(
        &self,
        subsystem: super::ObservabilitySubsystem,
    ) -> Result<(), ContractValidationError> {
        require_non_empty(&self.fixture_id, "fixture_case", "fixture_id")?;
        require(
            self.event.subsystem() == subsystem,
            &self.fixture_id,
            format!(
                "event family `{}` does not belong to subsystem `{}`",
                self.event.family(),
                subsystem.prefix()
            ),
        )?;
        require(
            self.fixture_id == self.event.family()
                || self
                    .fixture_id
                    .starts_with(&format!("{}.", self.event.family())),
            &self.fixture_id,
            format!(
                "fixture_id must equal or start with family `{}`",
                self.event.family()
            ),
        )?;
        self.event.validate(&self.fixture_id)
    }
}

impl ContractEvent {
    fn validate(&self, fixture_id: &str) -> Result<(), ContractValidationError> {
        match self {
            Self::CortexTick(event) => validate_cortex_tick(event, fixture_id),
            Self::CortexOrganRequest(event) => validate_cortex_request(event, fixture_id),
            Self::CortexOrganResponse(event) => validate_cortex_response(event, fixture_id),
            Self::CortexGoalForestSnapshot(event) => {
                validate_goal_forest_snapshot(event, fixture_id)
            }
            Self::StemSignalTransition(event) => validate_signal_transition(event, fixture_id),
            Self::StemDispatchTransition(event) => validate_dispatch_transition(event, fixture_id),
            Self::StemDescriptorCatalog(event) => validate_catalog_event(event, fixture_id),
            Self::SpineAdapterLifecycle(event) => validate_adapter_lifecycle(event, fixture_id),
            Self::SpineEndpointLifecycle(event) => validate_endpoint_lifecycle(event, fixture_id),
            Self::SpineDispatchOutcome(event) => validate_dispatch_outcome(event, fixture_id),
        }
    }
}

fn validate_cortex_tick(
    event: &CortexTickEvent,
    fixture_id: &str,
) -> Result<(), ContractValidationError> {
    validate_common(&event.run_id, &event.timestamp, fixture_id)?;
    require(event.tick > 0, fixture_id, "tick must be > 0")?;
    require_present(&event.trigger_summary, fixture_id, "trigger_summary")?;
    require_present(&event.senses_summary, fixture_id, "senses_summary")?;
    require_present(
        &event.proprioception_snapshot_or_ref,
        fixture_id,
        "proprioception_snapshot_or_ref",
    )?;
    require_present(&event.acts_summary, fixture_id, "acts_summary")?;
    require_present(&event.goal_forest_ref, fixture_id, "goal_forest_ref")
}

fn validate_cortex_request(
    event: &CortexOrganRequestEvent,
    fixture_id: &str,
) -> Result<(), ContractValidationError> {
    validate_common(&event.run_id, &event.timestamp, fixture_id)?;
    require(event.tick > 0, fixture_id, "tick must be > 0")?;
    require_non_empty(&event.stage, fixture_id, "stage")?;
    require_non_empty(&event.route_or_organ, fixture_id, "route_or_organ")?;
    require_non_empty(&event.request_id, fixture_id, "request_id")?;
    require_present(&event.input_summary, fixture_id, "input_summary")
}

fn validate_cortex_response(
    event: &CortexOrganResponseEvent,
    fixture_id: &str,
) -> Result<(), ContractValidationError> {
    validate_common(&event.run_id, &event.timestamp, fixture_id)?;
    require(event.tick > 0, fixture_id, "tick must be > 0")?;
    require_non_empty(&event.stage, fixture_id, "stage")?;
    require_non_empty(&event.request_id, fixture_id, "request_id")?;
    require_present(&event.response_summary, fixture_id, "response_summary")?;
    require_present(&event.tool_summary, fixture_id, "tool_summary")?;
    require_present(&event.act_summary, fixture_id, "act_summary")?;
    if matches!(event.status, OrganResponseStatus::Error) {
        require(
            event.error_summary_when_present.is_some(),
            fixture_id,
            "error responses must include error_summary_when_present",
        )?;
    }
    Ok(())
}

fn validate_goal_forest_snapshot(
    event: &CortexGoalForestSnapshotEvent,
    fixture_id: &str,
) -> Result<(), ContractValidationError> {
    validate_common(&event.run_id, &event.timestamp, fixture_id)?;
    require(event.tick > 0, fixture_id, "tick must be > 0")?;
    require_present(&event.snapshot_summary, fixture_id, "snapshot_summary")?;
    require_present(&event.snapshot_or_ref, fixture_id, "snapshot_or_ref")
}

fn validate_signal_transition(
    event: &StemSignalTransitionEvent,
    fixture_id: &str,
) -> Result<(), ContractValidationError> {
    validate_common(&event.run_id, &event.timestamp, fixture_id)?;
    require_non_empty(&event.descriptor_id, fixture_id, "descriptor_id")?;
    require(
        event.sense_id.is_some() || event.act_id.is_some(),
        fixture_id,
        "signal transitions must carry sense_id or act_id",
    )
}

fn validate_dispatch_transition(
    event: &StemDispatchTransitionEvent,
    fixture_id: &str,
) -> Result<(), ContractValidationError> {
    validate_common(&event.run_id, &event.timestamp, fixture_id)?;
    require_non_empty(&event.act_id, fixture_id, "act_id")?;
    require_present(
        &event.queue_or_flow_summary,
        fixture_id,
        "queue_or_flow_summary",
    )?;
    if matches!(event.transition_kind, TransitionKind::Result) {
        require(
            event.terminal_outcome_when_present.is_some(),
            fixture_id,
            "result transitions must include terminal_outcome_when_present",
        )?;
    }
    Ok(())
}

fn validate_catalog_event(
    event: &StemDescriptorCatalogEvent,
    fixture_id: &str,
) -> Result<(), ContractValidationError> {
    validate_common(&event.run_id, &event.timestamp, fixture_id)?;
    require_non_empty(&event.catalog_version, fixture_id, "catalog_version")?;
    require_present(
        &event.changed_descriptor_summary,
        fixture_id,
        "changed_descriptor_summary",
    )?;
    if matches!(event.change_mode, DescriptorCatalogChangeMode::Snapshot) {
        require(
            event.catalog_snapshot_when_required.is_some(),
            fixture_id,
            "snapshot changes must include catalog_snapshot_when_required",
        )?;
    }
    Ok(())
}

fn validate_adapter_lifecycle(
    event: &SpineAdapterLifecycleEvent,
    fixture_id: &str,
) -> Result<(), ContractValidationError> {
    validate_common(&event.run_id, &event.timestamp, fixture_id)?;
    require_non_empty(&event.adapter_type, fixture_id, "adapter_type")?;
    require_non_empty(&event.adapter_id, fixture_id, "adapter_id")
}

fn validate_endpoint_lifecycle(
    event: &SpineEndpointLifecycleEvent,
    fixture_id: &str,
) -> Result<(), ContractValidationError> {
    validate_common(&event.run_id, &event.timestamp, fixture_id)?;
    require_non_empty(&event.endpoint_id, fixture_id, "endpoint_id")
}

fn validate_dispatch_outcome(
    event: &SpineDispatchOutcomeEvent,
    fixture_id: &str,
) -> Result<(), ContractValidationError> {
    validate_common(&event.run_id, &event.timestamp, fixture_id)?;
    require_non_empty(&event.act_id, fixture_id, "act_id")?;
    require_non_empty(&event.binding_target, fixture_id, "binding_target")
}

fn validate_common(
    run_id: &str,
    timestamp: &str,
    context: &str,
) -> Result<(), ContractValidationError> {
    require_non_empty(run_id, context, "run_id")?;
    require_non_empty(timestamp, context, "timestamp")
}

fn require_present(
    value: &Value,
    context: &str,
    field: &str,
) -> Result<(), ContractValidationError> {
    require(
        !value.is_null(),
        context,
        format!("{field} must not be null"),
    )
}

fn require_non_empty(
    value: &str,
    context: &str,
    field: &str,
) -> Result<(), ContractValidationError> {
    require(
        !value.trim().is_empty(),
        context,
        format!("{field} must not be empty"),
    )
}

fn require(
    condition: bool,
    context: &str,
    detail: impl Into<String>,
) -> Result<(), ContractValidationError> {
    if condition {
        return Ok(());
    }
    Err(ContractValidationError {
        context: context.to_string(),
        detail: detail.into(),
    })
}
