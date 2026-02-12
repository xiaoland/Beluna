use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CortexErrorKind {
    InvalidReactionInput,
    PrimaryInferenceFailed,
    ExtractorInferenceFailed,
    FillerInferenceFailed,
    ClampRejectedAll,
    BudgetExceeded,
    CycleTimeout,
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

pub fn invalid_input(message: impl Into<String>) -> CortexError {
    CortexError::new(CortexErrorKind::InvalidReactionInput, message)
}

pub fn primary_failed(message: impl Into<String>) -> CortexError {
    CortexError::new(CortexErrorKind::PrimaryInferenceFailed, message)
}

pub fn extractor_failed(message: impl Into<String>) -> CortexError {
    CortexError::new(CortexErrorKind::ExtractorInferenceFailed, message)
}

pub fn filler_failed(message: impl Into<String>) -> CortexError {
    CortexError::new(CortexErrorKind::FillerInferenceFailed, message)
}

pub fn clamp_rejected(message: impl Into<String>) -> CortexError {
    CortexError::new(CortexErrorKind::ClampRejectedAll, message)
}

pub fn budget_exceeded(message: impl Into<String>) -> CortexError {
    CortexError::new(CortexErrorKind::BudgetExceeded, message)
}

pub fn cycle_timeout(message: impl Into<String>) -> CortexError {
    CortexError::new(CortexErrorKind::CycleTimeout, message)
}

pub fn internal_error(message: impl Into<String>) -> CortexError {
    CortexError::new(CortexErrorKind::Internal, message)
}
