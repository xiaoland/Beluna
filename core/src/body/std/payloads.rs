use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct ShellExecRequest {
    pub argv: Vec<String>,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    #[serde(default)]
    pub timeout_ms: Option<u64>,
    #[serde(default)]
    pub stdout_max_bytes: Option<usize>,
    #[serde(default)]
    pub stderr_max_bytes: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellLimits {
    pub default_timeout_ms: u64,
    pub max_timeout_ms: u64,
    pub default_stdout_max_bytes: usize,
    pub max_stdout_max_bytes: usize,
    pub default_stderr_max_bytes: usize,
    pub max_stderr_max_bytes: usize,
}

impl Default for ShellLimits {
    fn default() -> Self {
        Self {
            default_timeout_ms: 10_000,
            max_timeout_ms: 60_000,
            default_stdout_max_bytes: 16 * 1024,
            max_stdout_max_bytes: 256 * 1024,
            default_stderr_max_bytes: 16 * 1024,
            max_stderr_max_bytes: 256 * 1024,
        }
    }
}

impl ShellExecRequest {
    pub fn timeout_ms(&self, limits: &ShellLimits) -> u64 {
        self.timeout_ms
            .unwrap_or(limits.default_timeout_ms)
            .min(limits.max_timeout_ms)
    }

    pub fn stdout_max_bytes(&self, limits: &ShellLimits) -> usize {
        self.stdout_max_bytes
            .unwrap_or(limits.default_stdout_max_bytes)
            .min(limits.max_stdout_max_bytes)
    }

    pub fn stderr_max_bytes(&self, limits: &ShellLimits) -> usize {
        self.stderr_max_bytes
            .unwrap_or(limits.default_stderr_max_bytes)
            .min(limits.max_stderr_max_bytes)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct WebFetchRequest {
    pub url: String,
    #[serde(default)]
    pub method: Option<String>,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    #[serde(default)]
    pub body_text: Option<String>,
    #[serde(default)]
    pub timeout_ms: Option<u64>,
    #[serde(default)]
    pub response_max_bytes: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebLimits {
    pub default_timeout_ms: u64,
    pub max_timeout_ms: u64,
    pub default_response_max_bytes: usize,
    pub max_response_max_bytes: usize,
}

impl Default for WebLimits {
    fn default() -> Self {
        Self {
            default_timeout_ms: 15_000,
            max_timeout_ms: 60_000,
            default_response_max_bytes: 64 * 1024,
            max_response_max_bytes: 512 * 1024,
        }
    }
}

impl WebFetchRequest {
    pub fn timeout_ms(&self, limits: &WebLimits) -> u64 {
        self.timeout_ms
            .unwrap_or(limits.default_timeout_ms)
            .min(limits.max_timeout_ms)
    }

    pub fn response_max_bytes(&self, limits: &WebLimits) -> usize {
        self.response_max_bytes
            .unwrap_or(limits.default_response_max_bytes)
            .min(limits.max_response_max_bytes)
    }

    pub fn method(&self) -> &str {
        self.method.as_deref().unwrap_or("GET")
    }
}
