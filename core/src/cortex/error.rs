use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CortexErrorKind {
    InvalidRequest,
    InvariantViolation,
    PolicyViolation,
    Internal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CortexError {
    pub kind: CortexErrorKind,
    pub message: String,
}

impl CortexError {
    pub fn new(kind: CortexErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

impl fmt::Display for CortexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for CortexError {}

pub fn invalid_request(message: impl Into<String>) -> CortexError {
    CortexError::new(CortexErrorKind::InvalidRequest, message)
}

pub fn invariant_violation(message: impl Into<String>) -> CortexError {
    CortexError::new(CortexErrorKind::InvariantViolation, message)
}

pub fn policy_violation(message: impl Into<String>) -> CortexError {
    CortexError::new(CortexErrorKind::PolicyViolation, message)
}

pub fn internal_error(message: impl Into<String>) -> CortexError {
    CortexError::new(CortexErrorKind::Internal, message)
}
