use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NonCortexErrorKind {
    InvalidRequest,
    InvariantViolation,
    LedgerConflict,
    Arithmetic,
    Internal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NonCortexError {
    pub kind: NonCortexErrorKind,
    pub message: String,
}

impl NonCortexError {
    pub fn new(kind: NonCortexErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

impl fmt::Display for NonCortexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for NonCortexError {}

pub fn invalid_request(message: impl Into<String>) -> NonCortexError {
    NonCortexError::new(NonCortexErrorKind::InvalidRequest, message)
}

pub fn invariant_violation(message: impl Into<String>) -> NonCortexError {
    NonCortexError::new(NonCortexErrorKind::InvariantViolation, message)
}

pub fn ledger_conflict(message: impl Into<String>) -> NonCortexError {
    NonCortexError::new(NonCortexErrorKind::LedgerConflict, message)
}

pub fn arithmetic_error(message: impl Into<String>) -> NonCortexError {
    NonCortexError::new(NonCortexErrorKind::Arithmetic, message)
}

pub fn internal_error(message: impl Into<String>) -> NonCortexError {
    NonCortexError::new(NonCortexErrorKind::Internal, message)
}
