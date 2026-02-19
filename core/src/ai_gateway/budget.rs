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
    error::{GatewayError, GatewayErrorKind},
    types::{BackendId, BudgetConfig},
    types_chat::{CanonicalRequest, ChatEvent},
};

#[derive(Clone)]
pub struct BudgetEnforcer {
    config: BudgetConfig,
    permits: Arc<Mutex<HashMap<BackendId, Arc<Semaphore>>>>,
    token_buckets: Arc<Mutex<HashMap<BackendId, TokenBucket>>>,
    post_usage_overages: Arc<Mutex<HashMap<BackendId, u64>>>,
}

#[derive(Debug)]
pub struct BudgetLease {
    pub backend_id: BackendId,
    pub effective_timeout: Duration,
    permit: Option<OwnedSemaphorePermit>,
}

#[derive(Debug, Clone)]
struct TokenBucket {
    tokens: f64,
    last_refill: Instant,
}

impl BudgetEnforcer {
    pub fn new(config: BudgetConfig) -> Self {
        Self {
            config,
            permits: Arc::new(Mutex::new(HashMap::new())),
            token_buckets: Arc::new(Mutex::new(HashMap::new())),
            post_usage_overages: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn pre_dispatch(
        &self,
        request: &CanonicalRequest,
        backend_id: &BackendId,
    ) -> Result<BudgetLease, GatewayError> {
        if let Some(limit) = request.limits.max_output_tokens {
            if let Some(max_usage_tokens_per_request) = self.config.max_usage_tokens_per_request {
                if limit > max_usage_tokens_per_request {
                    return Err(GatewayError::new(
                        GatewayErrorKind::BudgetExceeded,
                        format!(
                            "requested max_output_tokens {} exceeds configured budget {}",
                            limit, max_usage_tokens_per_request
                        ),
                    )
                    .with_retryable(false)
                    .with_backend_id(backend_id.clone()));
                }
            }
        }

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

        let effective_timeout_ms = request
            .limits
            .max_request_time_ms
            .map(|requested| requested.min(self.config.max_request_time_ms))
            .unwrap_or(self.config.max_request_time_ms)
            .max(1);

        Ok(BudgetLease {
            backend_id: backend_id.clone(),
            effective_timeout: Duration::from_millis(effective_timeout_ms),
            permit: Some(permit),
        })
    }

    pub async fn observe_event(&self, backend_id: &BackendId, event: &ChatEvent) {
        let ChatEvent::Usage { usage, .. } = event else {
            return;
        };

        let Some(max_usage_tokens_per_request) = self.config.max_usage_tokens_per_request else {
            return;
        };

        let Some(total_tokens) = usage.total_tokens else {
            return;
        };

        if total_tokens > max_usage_tokens_per_request {
            let mut guard = self.post_usage_overages.lock().await;
            let count = guard.entry(backend_id.clone()).or_insert(0);
            *count += 1;
        }
    }

    pub fn release(&self, mut lease: BudgetLease) {
        let _ = lease.permit.take();
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
