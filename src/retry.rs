//! Retry + exponential-backoff-with-jitter middleware.
//!
//! Constants mirror the Go SDK's defaults so behavior matches across the two.

use std::future::Future;
use std::time::Duration;

use rand::Rng;

use crate::error::{Error, Result};

/// Jitter as a fraction of the computed delay (±25 %).
pub const DEFAULT_JITTER_FACTOR: f64 = 0.25;
/// Maximum single-step delay between retry attempts.
pub const DEFAULT_MAX_DELAY: Duration = Duration::from_secs(30);
/// Exponential multiplier between attempts.
pub const DEFAULT_MULTIPLIER: f64 = 2.0;
/// Cap on stream-reconnect backoff (used by Phase 2 streaming).
pub const MAX_RECONNECT_BACKOFF: Duration = Duration::from_secs(10);
/// Default retry budget.
pub const DEFAULT_MAX_RETRIES: u32 = 3;
/// Default first-attempt delay.
pub const DEFAULT_INITIAL_DELAY: Duration = Duration::from_secs(1);

/// Retry / backoff configuration.
#[derive(Clone, Debug)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub multiplier: f64,
    pub jitter_factor: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: DEFAULT_MAX_RETRIES,
            initial_delay: DEFAULT_INITIAL_DELAY,
            max_delay: DEFAULT_MAX_DELAY,
            multiplier: DEFAULT_MULTIPLIER,
            jitter_factor: DEFAULT_JITTER_FACTOR,
        }
    }
}

#[allow(dead_code)] // Wired into the request layer in Phase 2.
impl RetryConfig {
    /// Compute the (jittered) delay before the *next* retry, given the
    /// 1-indexed attempt number that just failed.
    pub(crate) fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let base = self.initial_delay.as_secs_f64() * self.multiplier.powi(attempt as i32 - 1);
        let capped = base.min(self.max_delay.as_secs_f64());
        let jitter_span = capped * self.jitter_factor;
        let lo = (capped - jitter_span).max(0.0);
        let hi = capped + jitter_span;
        let secs = if lo >= hi {
            capped
        } else {
            rand::thread_rng().gen_range(lo..hi)
        };
        Duration::from_secs_f64(secs)
    }
}

/// Run `op` with retries on transient errors.
///
/// `op` is invoked at least once. On a transient failure we sleep
/// `delay_for_attempt(n)` (or the error's `Retry-After`, when larger)
/// before the next try. Non-transient failures return immediately.
#[allow(dead_code)] // Wired into the request layer in Phase 2.
pub(crate) async fn run_with_retry<F, Fut, T>(cfg: &RetryConfig, mut op: F) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T>>,
{
    let mut attempt: u32 = 0;
    loop {
        attempt += 1;
        match op().await {
            Ok(v) => return Ok(v),
            Err(e) => {
                if !e.is_transient() || attempt > cfg.max_retries {
                    if attempt > 1 {
                        return Err(Error::RetryExhausted {
                            attempts: attempt,
                            source: Box::new(e),
                        });
                    }
                    return Err(e);
                }
                let computed = cfg.delay_for_attempt(attempt);
                let delay = e
                    .retry_after()
                    .map(|ra| ra.max(computed))
                    .unwrap_or(computed);
                tokio::time::sleep(delay).await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    fn api(status: u16) -> Error {
        Error::Api {
            status,
            code: None,
            message: "x".into(),
            metadata: None,
            provider: None,
            retry_after: None,
        }
    }

    #[tokio::test(start_paused = true)]
    async fn succeeds_first_try() {
        let calls = Arc::new(AtomicU32::new(0));
        let c = calls.clone();
        let cfg = RetryConfig::default();
        let r: Result<u32> = run_with_retry(&cfg, || {
            let c = c.clone();
            async move {
                c.fetch_add(1, Ordering::SeqCst);
                Ok(42)
            }
        })
        .await;
        assert_eq!(r.unwrap(), 42);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test(start_paused = true)]
    async fn retries_transient_then_succeeds() {
        let calls = Arc::new(AtomicU32::new(0));
        let c = calls.clone();
        let cfg = RetryConfig {
            max_retries: 3,
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(50),
            multiplier: 2.0,
            jitter_factor: 0.0,
        };
        let r: Result<u32> = run_with_retry(&cfg, || {
            let c = c.clone();
            async move {
                let n = c.fetch_add(1, Ordering::SeqCst) + 1;
                if n < 3 {
                    Err(api(503))
                } else {
                    Ok(7)
                }
            }
        })
        .await;
        assert_eq!(r.unwrap(), 7);
        assert_eq!(calls.load(Ordering::SeqCst), 3);
    }

    #[tokio::test(start_paused = true)]
    async fn non_transient_errors_short_circuit() {
        let calls = Arc::new(AtomicU32::new(0));
        let c = calls.clone();
        let cfg = RetryConfig::default();
        let r: Result<()> = run_with_retry(&cfg, || {
            let c = c.clone();
            async move {
                c.fetch_add(1, Ordering::SeqCst);
                Err(api(400))
            }
        })
        .await;
        assert!(matches!(r, Err(Error::Api { status: 400, .. })));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test(start_paused = true)]
    async fn exhausts_budget() {
        let calls = Arc::new(AtomicU32::new(0));
        let c = calls.clone();
        let cfg = RetryConfig {
            max_retries: 2,
            initial_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(5),
            multiplier: 2.0,
            jitter_factor: 0.0,
        };
        let r: Result<()> = run_with_retry(&cfg, || {
            let c = c.clone();
            async move {
                c.fetch_add(1, Ordering::SeqCst);
                Err(api(503))
            }
        })
        .await;
        match r {
            Err(Error::RetryExhausted { attempts, source }) => {
                assert_eq!(attempts, 3);
                assert!(matches!(*source, Error::Api { status: 503, .. }));
            }
            other => panic!("expected RetryExhausted, got {other:?}"),
        }
        assert_eq!(calls.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn delay_is_within_jitter_window() {
        let cfg = RetryConfig {
            max_retries: 5,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(30),
            multiplier: 2.0,
            jitter_factor: 0.25,
        };
        for attempt in 1..=4 {
            let base = (cfg.initial_delay.as_secs_f64() * 2f64.powi(attempt as i32 - 1))
                .min(cfg.max_delay.as_secs_f64());
            let lo = base * 0.75;
            let hi = base * 1.25;
            for _ in 0..100 {
                let d = cfg.delay_for_attempt(attempt).as_secs_f64();
                assert!(d >= lo - 1e-9 && d <= hi + 1e-9, "{d} not in [{lo},{hi}]");
            }
        }
    }

    #[tokio::test(start_paused = true)]
    async fn retry_after_overrides_when_larger() {
        let calls = Arc::new(AtomicU32::new(0));
        let c = calls.clone();
        let cfg = RetryConfig {
            max_retries: 1,
            initial_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(1),
            multiplier: 2.0,
            jitter_factor: 0.0,
        };
        let start = tokio::time::Instant::now();
        let r: Result<()> = run_with_retry(&cfg, || {
            let c = c.clone();
            async move {
                let n = c.fetch_add(1, Ordering::SeqCst) + 1;
                if n == 1 {
                    Err(Error::Api {
                        status: 429,
                        code: None,
                        message: "rate".into(),
                        metadata: None,
                        provider: None,
                        retry_after: Some(Duration::from_secs(5)),
                    })
                } else {
                    Ok(())
                }
            }
        })
        .await;
        assert!(r.is_ok());
        let elapsed = start.elapsed();
        assert!(
            elapsed >= Duration::from_secs(5),
            "expected >= 5s, got {elapsed:?}"
        );
    }
}
