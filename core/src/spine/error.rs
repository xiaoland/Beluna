use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpineErrorKind {
    InvalidBatch,
    InvariantViolation,
    BackendFailure,
    Internal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpineError {
    pub kind: SpineErrorKind,
    pub message: String,
}

impl SpineError {
    pub fn new(kind: SpineErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

impl fmt::Display for SpineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for SpineError {}

pub fn invalid_batch(message: impl Into<String>) -> SpineError {
    SpineError::new(SpineErrorKind::InvalidBatch, message)
}

pub fn invariant_violation(message: impl Into<String>) -> SpineError {
    SpineError::new(SpineErrorKind::InvariantViolation, message)
}

pub fn backend_failure(message: impl Into<String>) -> SpineError {
    SpineError::new(SpineErrorKind::BackendFailure, message)
}

pub fn internal_error(message: impl Into<String>) -> SpineError {
    SpineError::new(SpineErrorKind::Internal, message)
}
