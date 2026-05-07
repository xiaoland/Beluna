use crate::{atropos::model::RuntimeStatus, lachesis::model::IngestPulse};

use super::status::MoiraResourceStatus;

#[derive(Debug, Clone)]
pub enum MoiraEvent {
    LachesisUpdated(IngestPulse),
    ResourceStatusChanged(MoiraResourceStatus),
    CoreSupervisionChanged(RuntimeStatus),
}

pub trait MoiraEventSink: Send + Sync {
    fn emit(&self, event: MoiraEvent);
}

#[derive(Debug, Default)]
pub struct NoopEventSink;

impl MoiraEventSink for NoopEventSink {
    fn emit(&self, _event: MoiraEvent) {}
}
