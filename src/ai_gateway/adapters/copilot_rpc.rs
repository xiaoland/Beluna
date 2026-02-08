use std::collections::BTreeMap;

use serde_json::{Value, json};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    process::{ChildStdin, ChildStdout},
};

use crate::ai_gateway::error::{GatewayError, GatewayErrorKind};

pub struct RpcIo {
    reader: BufReader<ChildStdout>,
    writer: ChildStdin,
}

impl RpcIo {
    pub fn new(stdout: ChildStdout, stdin: ChildStdin) -> Self {
        Self {
            reader: BufReader::new(stdout),
            writer: stdin,
        }
    }

    pub async fn send_request(
        &mut self,
        id: u64,
        method: &str,
        params: Value,
    ) -> Result<(), GatewayError> {
        let payload = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });
        self.write_message(payload).await
    }

    pub async fn send_notification(
        &mut self,
        method: &str,
        params: Value,
    ) -> Result<(), GatewayError> {
        let payload = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        });
        self.write_message(payload).await
    }

    pub async fn request(
        &mut self,
        id: u64,
        method: &str,
        params: Value,
    ) -> Result<Value, GatewayError> {
        self.send_request(id, method, params).await?;

        loop {
            let msg = self.read_message().await?;
            if msg.get("id").and_then(Value::as_u64) != Some(id) {
                continue;
            }

            if let Some(err) = msg.get("error") {
                return Err(GatewayError::new(
                    GatewayErrorKind::BackendPermanent,
                    format!("copilot rpc error for '{}': {}", method, err),
                )
                .with_retryable(false));
            }

            return Ok(msg.get("result").cloned().unwrap_or(Value::Null));
        }
    }

    async fn write_message(&mut self, payload: Value) -> Result<(), GatewayError> {
        let bytes = serde_json::to_vec(&payload).map_err(|err| {
            GatewayError::new(
                GatewayErrorKind::Internal,
                format!("failed to encode rpc payload: {}", err),
            )
            .with_retryable(false)
        })?;

        let header = format!("Content-Length: {}\r\n\r\n", bytes.len());
        self.writer
            .write_all(header.as_bytes())
            .await
            .map_err(|err| {
                GatewayError::new(
                    GatewayErrorKind::BackendTransient,
                    format!("failed to write rpc header: {}", err),
                )
                .with_retryable(true)
            })?;
        self.writer.write_all(&bytes).await.map_err(|err| {
            GatewayError::new(
                GatewayErrorKind::BackendTransient,
                format!("failed to write rpc body: {}", err),
            )
            .with_retryable(true)
        })?;
        self.writer.flush().await.map_err(|err| {
            GatewayError::new(
                GatewayErrorKind::BackendTransient,
                format!("failed to flush rpc body: {}", err),
            )
            .with_retryable(true)
        })?;

        Ok(())
    }

    async fn read_message(&mut self) -> Result<Value, GatewayError> {
        let mut headers: BTreeMap<String, String> = BTreeMap::new();

        loop {
            let mut line = String::new();
            let n = self.reader.read_line(&mut line).await.map_err(|err| {
                GatewayError::new(
                    GatewayErrorKind::BackendTransient,
                    format!("failed to read rpc header line: {}", err),
                )
                .with_retryable(true)
            })?;

            if n == 0 {
                return Err(GatewayError::new(
                    GatewayErrorKind::ProtocolViolation,
                    "rpc stream closed unexpectedly",
                )
                .with_retryable(true));
            }

            let line = line.trim_end_matches(['\r', '\n']);
            if line.is_empty() {
                break;
            }

            if let Some((name, value)) = line.split_once(':') {
                headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_string());
            }
        }

        let content_length = headers
            .get("content-length")
            .and_then(|v| v.parse::<usize>().ok())
            .ok_or_else(|| {
                GatewayError::new(
                    GatewayErrorKind::ProtocolViolation,
                    "rpc message missing content-length",
                )
                .with_retryable(false)
            })?;

        let mut body = vec![0_u8; content_length];
        self.reader.read_exact(&mut body).await.map_err(|err| {
            GatewayError::new(
                GatewayErrorKind::BackendTransient,
                format!("failed to read rpc message body: {}", err),
            )
            .with_retryable(true)
        })?;

        serde_json::from_slice::<Value>(&body).map_err(|err| {
            GatewayError::new(
                GatewayErrorKind::ProtocolViolation,
                format!("failed to decode rpc message: {}", err),
            )
            .with_retryable(false)
        })
    }
}
