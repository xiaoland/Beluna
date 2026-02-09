use std::fmt;

use serde::{Deserialize, Serialize};

use crate::ai_gateway::types::BackendId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GatewayErrorKind {
    InvalidRequest,
    UnsupportedCapability,
    Authentication,
    Authorization,
    RateLimited,
    Timeout,
    CircuitOpen,
    BudgetExceeded,
    BackendTransient,
    BackendPermanent,
    ProtocolViolation,
    Internal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayError {
    pub kind: GatewayErrorKind,
    pub message: String,
    pub retryable: bool,
    pub backend_id: Option<BackendId>,
    pub provider_code: Option<String>,
    pub provider_http_status: Option<u16>,
}

impl GatewayError {
    pub fn new(kind: GatewayErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            retryable: matches!(
                kind,
                GatewayErrorKind::RateLimited
                    | GatewayErrorKind::Timeout
                    | GatewayErrorKind::BackendTransient
            ),
            backend_id: None,
            provider_code: None,
            provider_http_status: None,
        }
    }

    pub fn with_retryable(mut self, retryable: bool) -> Self {
        self.retryable = retryable;
        self
    }

    pub fn with_backend_id(mut self, backend_id: impl Into<String>) -> Self {
        self.backend_id = Some(backend_id.into());
        self
    }

    pub fn with_provider_code(mut self, provider_code: impl Into<String>) -> Self {
        self.provider_code = Some(provider_code.into());
        self
    }

    pub fn with_provider_http_status(mut self, status: u16) -> Self {
        self.provider_http_status = Some(status);
        self
    }
}

impl fmt::Display for GatewayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (&self.backend_id, &self.provider_code) {
            (Some(backend_id), Some(provider_code)) => {
                write!(
                    f,
                    "{} (backend={}, provider_code={})",
                    self.message, backend_id, provider_code
                )
            }
            (Some(backend_id), None) => write!(f, "{} (backend={})", self.message, backend_id),
            (None, Some(provider_code)) => {
                write!(f, "{} (provider_code={})", self.message, provider_code)
            }
            (None, None) => write!(f, "{}", self.message),
        }
    }
}

impl std::error::Error for GatewayError {}

pub fn invalid_request(message: impl Into<String>) -> GatewayError {
    GatewayError::new(GatewayErrorKind::InvalidRequest, message).with_retryable(false)
}

pub fn unsupported_capability(message: impl Into<String>) -> GatewayError {
    GatewayError::new(GatewayErrorKind::UnsupportedCapability, message).with_retryable(false)
}

pub fn internal_error(message: impl Into<String>) -> GatewayError {
    GatewayError::new(GatewayErrorKind::Internal, message).with_retryable(false)
}
