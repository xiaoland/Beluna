use std::collections::BTreeSet;

use thiserror::Error;

use super::{ContractEvent, FIXTURE_SCHEMA_VERSION, FixtureBundle, FixtureCase};

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
            Self::AiGatewayRequest(event) => {
                validate_common(&event.run_id, &event.timestamp, fixture_id)?;
                require_non_empty(&event.request_id, fixture_id, "request_id")?;
                require_non_empty(&event.span_id, fixture_id, "span_id")?;
                require_non_empty(&event.capability, fixture_id, "capability")?;
                require_non_empty(&event.backend_id, fixture_id, "backend_id")?;
                require_non_empty(&event.model, fixture_id, "model")?;
                require_non_empty(&event.kind, fixture_id, "kind")
            }
            Self::AiGatewayChatTurn(event) => {
                validate_common(&event.run_id, &event.timestamp, fixture_id)?;
                require_non_empty(&event.thread_id, fixture_id, "thread_id")?;
                require_non_empty(&event.span_id, fixture_id, "span_id")?;
                require_non_empty(&event.status, fixture_id, "status")
            }
            Self::AiGatewayChatThread(event) => {
                validate_common(&event.run_id, &event.timestamp, fixture_id)?;
                require_non_empty(&event.thread_id, fixture_id, "thread_id")?;
                require_non_empty(&event.span_id, fixture_id, "span_id")?;
                require_non_empty(&event.kind, fixture_id, "kind")
            }
            Self::CortexPrimary(event)
            | Self::CortexSenseHelper(event)
            | Self::CortexGoalForestHelper(event)
            | Self::CortexActsHelper(event) => {
                validate_common(&event.run_id, &event.timestamp, fixture_id)?;
                require_non_empty(&event.request_id, fixture_id, "request_id")?;
                require_non_empty(&event.span_id, fixture_id, "span_id")?;
                require_non_empty(&event.phase, fixture_id, "phase")
            }
            Self::CortexGoalForest(event) => {
                validate_common(&event.run_id, &event.timestamp, fixture_id)?;
                require_non_empty(&event.span_id, fixture_id, "span_id")?;
                require_non_empty(&event.kind, fixture_id, "kind")
            }
            Self::StemTick(event) => {
                validate_common(&event.run_id, &event.timestamp, fixture_id)?;
                require_non_empty(&event.span_id, fixture_id, "span_id")?;
                require_non_empty(&event.status, fixture_id, "status")
            }
            Self::StemAfferent(event) => {
                validate_common(&event.run_id, &event.timestamp, fixture_id)?;
                require_non_empty(&event.span_id, fixture_id, "span_id")?;
                require_non_empty(&event.kind, fixture_id, "kind")?;
                require_non_empty(&event.descriptor_id, fixture_id, "descriptor_id")
            }
            Self::StemEfferent(event) => {
                validate_common(&event.run_id, &event.timestamp, fixture_id)?;
                require_non_empty(&event.span_id, fixture_id, "span_id")?;
                require_non_empty(&event.kind, fixture_id, "kind")?;
                require_non_empty(&event.act_id, fixture_id, "act_id")
            }
            Self::StemProprioception(event) => {
                validate_common(&event.run_id, &event.timestamp, fixture_id)?;
                require_non_empty(&event.span_id, fixture_id, "span_id")?;
                require_non_empty(&event.kind, fixture_id, "kind")
            }
            Self::StemNsCatalog(event) => {
                validate_common(&event.run_id, &event.timestamp, fixture_id)?;
                require_non_empty(&event.span_id, fixture_id, "span_id")?;
                require_non_empty(&event.catalog_version, fixture_id, "catalog_version")
            }
            Self::StemAfferentRule(event) => {
                validate_common(&event.run_id, &event.timestamp, fixture_id)?;
                require_non_empty(&event.span_id, fixture_id, "span_id")?;
                require_non_empty(&event.kind, fixture_id, "kind")?;
                require_non_empty(&event.rule_id, fixture_id, "rule_id")
            }
            Self::SpineAdapter(event) => {
                validate_common(&event.run_id, &event.timestamp, fixture_id)?;
                require_non_empty(&event.span_id, fixture_id, "span_id")?;
                require_non_empty(&event.adapter_type, fixture_id, "adapter_type")?;
                require_non_empty(&event.adapter_id, fixture_id, "adapter_id")
            }
            Self::SpineEndpoint(event) => {
                validate_common(&event.run_id, &event.timestamp, fixture_id)?;
                require_non_empty(&event.span_id, fixture_id, "span_id")?;
                require_non_empty(&event.endpoint_id, fixture_id, "endpoint_id")
            }
            Self::SpineSense(event) => {
                validate_common(&event.run_id, &event.timestamp, fixture_id)?;
                require_non_empty(&event.span_id, fixture_id, "span_id")?;
                require_non_empty(&event.endpoint_id, fixture_id, "endpoint_id")?;
                require_non_empty(&event.sense_id, fixture_id, "sense_id")?;
                require_non_empty(&event.kind, fixture_id, "kind")
            }
            Self::SpineAct(event) => {
                validate_common(&event.run_id, &event.timestamp, fixture_id)?;
                require_non_empty(&event.span_id, fixture_id, "span_id")?;
                require_non_empty(&event.act_id, fixture_id, "act_id")?;
                require_non_empty(&event.kind, fixture_id, "kind")
            }
        }
    }
}

fn validate_common(
    run_id: &str,
    timestamp: &str,
    context: &str,
) -> Result<(), ContractValidationError> {
    require_non_empty(run_id, context, "run_id")?;
    require_non_empty(timestamp, context, "timestamp")
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
