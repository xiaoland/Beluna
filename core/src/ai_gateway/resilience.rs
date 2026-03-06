use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use tokio::{
    sync::{Mutex, OwnedSemaphorePermit, Semaphore},
    time::sleep,
};

use crate::ai_gateway::{
    chat::types::TurnLimits,
    error::{GatewayError, GatewayErrorKind},
    types::{BackendCapabilities, BackendId, ResilienceConfig, RetryPolicy},
};

#[derive(Clone)]
pub struct ResilienceEngine {
    config: ResilienceConfig,
    breakers: Arc<Mutex<HashMap<BackendId, BreakerState>>>,
    permits: Arc<Mutex<HashMap<BackendId, Arc<Semaphore>>>>,
    token_buckets: Arc<Mutex<HashMap<BackendId, TokenBucket>>>,
}

#[derive(Debug, Clone)]
struct BreakerState {
    failure_streak: u32,
    open_until: Option<Instant>,
    probe_in_flight: bool,
}

impl Default for BreakerState {
    fn default() -> Self {
        Self {
            failure_streak: 0,
            open_until: None,
            probe_in_flight: false,
        }
    }
}

#[derive(Debug, Clone)]
struct TokenBucket {
    tokens: f64,
    last_refill: Instant,
}

#[derive(Debug)]
pub struct ResilienceLease {
    pub backend_id: BackendId,
    pub effective_timeout: Duration,
    permit: Option<OwnedSemaphorePermit>,
}

impl ResilienceEngine {
    pub fn new(config: ResilienceConfig) -> Self {
        Self {
            config,
            breakers: Arc::new(Mutex::new(HashMap::new())),
            permits: Arc::new(Mutex::new(HashMap::new())),
            token_buckets: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn config(&self) -> &ResilienceConfig {
        &self.config
    }

    pub async fn pre_dispatch(
        &self,
        limits: &TurnLimits,
        backend_id: &BackendId,
    ) -> Result<ResilienceLease, GatewayError> {
        self.acquire_rate_token(backend_id).await;

        let permit = {
            let mut guard = self.permits.lock().await;
            let semaphore = guard
                .entry(backend_id.clone())
                .or_insert_with(|| {
                    Arc::new(Semaphore::new(
                        self.config.max_concurrency_per_backend.max(1) as usize,
                    ))
                })
                .clone();
            semaphore.acquire_owned().await.map_err(|_| {
                GatewayError::new(
                    GatewayErrorKind::Internal,
                    "failed to acquire backend concurrency permit",
                )
                .with_retryable(false)
                .with_backend_id(backend_id.clone())
            })?
        };

        let effective_timeout_ms = limits
            .max_request_time_ms
            .map(|requested| requested.min(self.config.max_request_time_ms))
            .unwrap_or(self.config.request_timeout_ms)
            .max(1);

        Ok(ResilienceLease {
            backend_id: backend_id.clone(),
            effective_timeout: Duration::from_millis(effective_timeout_ms),
            permit: Some(permit),
        })
    }

    pub fn release(&self, mut lease: ResilienceLease) {
        let _ = lease.permit.take();
    }

    pub async fn ensure_backend_allowed(&self, backend_id: &BackendId) -> Result<(), GatewayError> {
        let now = Instant::now();
        let mut guard = self.breakers.lock().await;
        let state = guard.entry(backend_id.clone()).or_default();

        if let Some(open_until) = state.open_until {
            if now < open_until {
                return Err(GatewayError::new(
                    GatewayErrorKind::CircuitOpen,
                    format!("circuit breaker is open for backend '{}'", backend_id),
                )
                .with_retryable(false)
                .with_backend_id(backend_id.clone()));
            }

            if state.probe_in_flight {
                return Err(GatewayError::new(
                    GatewayErrorKind::CircuitOpen,
                    format!(
                        "circuit probe is already in-flight for backend '{}'",
                        backend_id
                    ),
                )
                .with_retryable(false)
                .with_backend_id(backend_id.clone()));
            }

            state.probe_in_flight = true;
        }

        Ok(())
    }

    pub async fn record_success(&self, backend_id: &BackendId) {
        let mut guard = self.breakers.lock().await;
        let state = guard.entry(backend_id.clone()).or_default();
        state.failure_streak = 0;
        state.open_until = None;
        state.probe_in_flight = false;
    }

    pub async fn record_failure(&self, backend_id: &BackendId, count_toward_breaker: bool) {
        if !count_toward_breaker {
            return;
        }

        let mut guard = self.breakers.lock().await;
        let state = guard.entry(backend_id.clone()).or_default();

        state.failure_streak = state.failure_streak.saturating_add(1);
        state.probe_in_flight = false;

        if state.failure_streak >= self.config.breaker_failure_threshold.max(1) {
            state.open_until =
                Some(Instant::now() + Duration::from_millis(self.config.breaker_open_ms.max(1)));
        }
    }

    pub fn backoff_delay(&self, attempt: u32) -> Duration {
        let base = self.config.backoff_base_ms.max(1) as f64;
        let max = self.config.backoff_max_ms.max(1) as f64;
        let exp = (attempt as i32).max(0);
        let without_jitter = (base * 2f64.powi(exp)).min(max);
        let jitter_factor = 0.9 + (attempt as f64 % 3.0) * 0.05;
        Duration::from_millis((without_jitter * jitter_factor) as u64)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn can_retry(
        &self,
        err: &GatewayError,
        attempt: u32,
        emitted_output: bool,
        emitted_tool: bool,
        capabilities: &BackendCapabilities,
        adapter_tool_retry_safe: bool,
    ) -> bool {
        if !err.retryable || attempt >= self.config.max_retries {
            return false;
        }

        if emitted_output && !matches!(self.config.retry_policy, RetryPolicy::AdapterResumable) {
            return false;
        }

        if emitted_output && !capabilities.resumable_streaming {
            return false;
        }

        if emitted_tool && !adapter_tool_retry_safe {
            return false;
        }

        true
    }

    pub fn counts_toward_breaker(err: &GatewayError) -> bool {
        matches!(
            err.kind,
            GatewayErrorKind::BackendTransient
                | GatewayErrorKind::Timeout
                | GatewayErrorKind::RateLimited
        )
    }

    async fn acquire_rate_token(&self, backend_id: &BackendId) {
        let Some(rps) = self.config.rate_smoothing_per_second else {
            return;
        };

        if rps == 0 {
            return;
        }

        loop {
            let mut should_sleep = None;
            {
                let mut guard = self.token_buckets.lock().await;
                let bucket = guard
                    .entry(backend_id.clone())
                    .or_insert_with(|| TokenBucket {
                        tokens: rps as f64,
                        last_refill: Instant::now(),
                    });

                let now = Instant::now();
                let elapsed = now.duration_since(bucket.last_refill).as_secs_f64();
                if elapsed > 0.0 {
                    bucket.tokens = (bucket.tokens + elapsed * rps as f64).min(rps as f64);
                    bucket.last_refill = now;
                }

                if bucket.tokens >= 1.0 {
                    bucket.tokens -= 1.0;
                } else {
                    let deficit = 1.0 - bucket.tokens;
                    let seconds = deficit / rps as f64;
                    should_sleep = Some(Duration::from_secs_f64(seconds.max(0.005)));
                }
            }

            if let Some(duration) = should_sleep {
                sleep(duration).await;
                continue;
            }

            break;
        }
    }
}
