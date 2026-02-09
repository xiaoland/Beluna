use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MindErrorKind {
    InvalidRequest,
    InvariantViolation,
    PolicyViolation,
    ConflictResolutionError,
    Internal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MindError {
    pub kind: MindErrorKind,
    pub message: String,
}

impl MindError {
    pub fn new(kind: MindErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

impl fmt::Display for MindError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for MindError {}

pub fn invalid_request(message: impl Into<String>) -> MindError {
    MindError::new(MindErrorKind::InvalidRequest, message)
}

pub fn invariant_violation(message: impl Into<String>) -> MindError {
    MindError::new(MindErrorKind::InvariantViolation, message)
}

pub fn policy_violation(message: impl Into<String>) -> MindError {
    MindError::new(MindErrorKind::PolicyViolation, message)
}

pub fn conflict_resolution_error(message: impl Into<String>) -> MindError {
    MindError::new(MindErrorKind::ConflictResolutionError, message)
}

pub fn internal_error(message: impl Into<String>) -> MindError {
    MindError::new(MindErrorKind::Internal, message)
}
