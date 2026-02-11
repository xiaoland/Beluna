use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContinuityErrorKind {
    InvalidRequest,
    InvariantViolation,
    LedgerConflict,
    Arithmetic,
    Internal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContinuityError {
    pub kind: ContinuityErrorKind,
    pub message: String,
}

impl ContinuityError {
    pub fn new(kind: ContinuityErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

impl fmt::Display for ContinuityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ContinuityError {}

pub fn invalid_request(message: impl Into<String>) -> ContinuityError {
    ContinuityError::new(ContinuityErrorKind::InvalidRequest, message)
}

pub fn invariant_violation(message: impl Into<String>) -> ContinuityError {
    ContinuityError::new(ContinuityErrorKind::InvariantViolation, message)
}

pub fn ledger_conflict(message: impl Into<String>) -> ContinuityError {
    ContinuityError::new(ContinuityErrorKind::LedgerConflict, message)
}

pub fn arithmetic_error(message: impl Into<String>) -> ContinuityError {
    ContinuityError::new(ContinuityErrorKind::Arithmetic, message)
}

pub fn internal_error(message: impl Into<String>) -> ContinuityError {
    ContinuityError::new(ContinuityErrorKind::Internal, message)
}
