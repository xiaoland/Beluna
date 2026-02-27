use crate::ai_gateway::error::{GatewayError, GatewayErrorKind};

/// Map an HTTP status code + body into a structured [`GatewayError`].
pub(crate) fn map_http_error(status: u16, backend_id: &str, body: &str) -> GatewayError {
    let normalized_body = body.chars().take(240).collect::<String>();

    let mut err = if status == 401 {
        GatewayError::new(GatewayErrorKind::Authentication, "authentication failed")
            .with_retryable(false)
    } else if status == 403 {
        GatewayError::new(GatewayErrorKind::Authorization, "authorization failed")
            .with_retryable(false)
    } else if status == 408 || status == 429 {
        GatewayError::new(
            GatewayErrorKind::RateLimited,
            format!("backend returned status {}", status),
        )
        .with_retryable(true)
    } else if (400..500).contains(&status) {
        GatewayError::new(
            GatewayErrorKind::InvalidRequest,
            format!("backend returned status {}", status),
        )
        .with_retryable(false)
    } else {
        GatewayError::new(
            GatewayErrorKind::BackendTransient,
            format!("backend returned status {}", status),
        )
        .with_retryable(true)
    };

    err = err
        .with_backend_id(backend_id.to_string())
        .with_provider_http_status(status);

    if !normalized_body.is_empty() {
        err.message = format!("{}: {}", err.message, normalized_body);
    }

    err
}
