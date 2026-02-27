//! Generic HTTP streaming utilities.
//!
//! This module provides composable building blocks for HTTP adapters that
//! communicate over HTTP POST. It contains **no** chat-specific or
//! adapter-specific logic.

use std::time::Duration;

use reqwest::{Client, header};
use serde_json::Value;

use crate::ai_gateway::{
    error::{GatewayError, GatewayErrorKind},
    types::ResolvedCredential,
};

use super::http_errors::map_http_error;

// ---------------------------------------------------------------------------
// HTTP request configuration
// ---------------------------------------------------------------------------

/// Configuration for a single HTTP POST request.
pub(crate) struct HttpRequestConfig {
    pub client: Client,
    pub url: String,
    pub body: Value,
    pub backend_id: String,
    pub request_id: String,
    pub credential: ResolvedCredential,
    pub timeout: Duration,
}

// ---------------------------------------------------------------------------
// send_post — shared request builder + error mapping
// ---------------------------------------------------------------------------

/// Build and send an HTTP POST with standard auth headers and timeout.
///
/// Returns the raw `reqwest::Response` on success (HTTP 2xx).
/// Maps non-success status codes through [`map_http_error`].
pub(crate) async fn send_post(
    config: &HttpRequestConfig,
) -> Result<reqwest::Response, GatewayError> {
    let mut req_builder = config
        .client
        .post(&config.url)
        .timeout(config.timeout)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-request-id", &config.request_id)
        .json(&config.body);

    if let Some(auth_header) = &config.credential.auth_header {
        req_builder = req_builder.header(header::AUTHORIZATION, auth_header);
    }
    for (k, v) in &config.credential.extra_headers {
        req_builder = req_builder.header(k, v);
    }

    let response = req_builder.send().await.map_err(|err| {
        GatewayError::new(
            GatewayErrorKind::BackendTransient,
            format!("HTTP request failed: {}", err),
        )
        .with_retryable(true)
        .with_backend_id(config.backend_id.clone())
    })?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();
        return Err(map_http_error(status, &config.backend_id, &body));
    }

    Ok(response)
}

/// Send an HTTP POST and read the full response body as JSON.
///
/// Convenience wrapper over [`send_post`] for non-streaming (complete) calls.
pub(crate) async fn post_json(config: &HttpRequestConfig) -> Result<Value, GatewayError> {
    let response = send_post(config).await?;
    response.json::<Value>().await.map_err(|err| {
        GatewayError::new(
            GatewayErrorKind::ProtocolViolation,
            format!("invalid JSON response payload: {}", err),
        )
        .with_retryable(false)
        .with_backend_id(config.backend_id.clone())
    })
}

// ---------------------------------------------------------------------------
// Line-frame extraction for streaming protocols
// ---------------------------------------------------------------------------

/// Extract complete SSE data frames from a byte buffer.
///
/// Processes all complete lines (terminated by `\n`) in the buffer,
/// consuming them.
///
///  - Skips lines that don't start with `data:`.
///  - Returns `done = true` when `data: [DONE]` is encountered.
///  - Parses each `data:` payload as JSON.
pub(crate) fn extract_sse_frames(
    buffer: &mut String,
    backend_id: &str,
) -> Result<(Vec<Value>, bool), GatewayError> {
    let mut frames = Vec::new();

    while let Some(idx) = buffer.find('\n') {
        let line = buffer[..idx].trim_end_matches('\r').to_string();
        *buffer = buffer[idx + 1..].to_string();

        if !line.starts_with("data:") {
            continue;
        }

        let data = line[5..].trim();
        if data.is_empty() {
            continue;
        }
        if data == "[DONE]" {
            return Ok((frames, true));
        }

        let value = serde_json::from_str::<Value>(data).map_err(|err| {
            GatewayError::new(
                GatewayErrorKind::ProtocolViolation,
                format!("failed to parse SSE data payload: {}", err),
            )
            .with_retryable(false)
            .with_backend_id(backend_id.to_string())
        })?;
        frames.push(value);
    }

    Ok((frames, false))
}

/// Extract complete NDJSON frames from a byte buffer.
///
/// Processes all complete lines (terminated by `\n`) in the buffer,
/// consuming them. Empty lines are skipped.
pub(crate) fn extract_ndjson_frames(
    buffer: &mut String,
    backend_id: &str,
) -> Result<Vec<Value>, GatewayError> {
    let mut frames = Vec::new();

    while let Some(idx) = buffer.find('\n') {
        let line = buffer[..idx].trim_end_matches('\r').to_string();
        *buffer = buffer[idx + 1..].to_string();

        if line.trim().is_empty() {
            continue;
        }

        let value = serde_json::from_str::<Value>(&line).map_err(|err| {
            GatewayError::new(
                GatewayErrorKind::ProtocolViolation,
                format!("failed to parse NDJSON payload: {}", err),
            )
            .with_retryable(false)
            .with_backend_id(backend_id.to_string())
        })?;
        frames.push(value);
    }

    Ok(frames)
}
