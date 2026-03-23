//! Retry wrapper for LLM providers with exponential backoff.
//!
//! Wraps any [`Provider`] and automatically retries transient errors.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;

use crate::traits::provider::Provider;
use crate::types::completion::{CompletionRequest, CompletionResponse};
use crate::types::model_info::ModelInfo;
use crate::types::stream::CompletionStream;

/// Configuration for retry behavior.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts (default: 3).
    pub max_retries: usize,
    /// Initial delay before the first retry (default: 500ms).
    pub initial_delay: Duration,
    /// Maximum delay cap (default: 30s).
    pub max_delay: Duration,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(30),
        }
    }
}

impl RetryConfig {
    /// Create a retry config with custom parameters.
    #[must_use]
    pub fn new(max_retries: usize, initial_delay: Duration, max_delay: Duration) -> Self {
        Self {
            max_retries,
            initial_delay,
            max_delay,
        }
    }
}

/// A provider decorator that retries transient errors with exponential backoff.
///
/// Uses [`Error::is_retryable()`](crate::Error::is_retryable) to classify errors.
pub struct RetryProvider {
    inner: Arc<dyn Provider>,
    config: RetryConfig,
}

impl RetryProvider {
    /// Wrap a provider with retry behavior.
    #[must_use]
    pub fn new(inner: Arc<dyn Provider>, config: RetryConfig) -> Self {
        Self { inner, config }
    }

    /// Calculate delay for a given attempt (0-indexed), capped at `max_delay`.
    #[allow(clippy::cast_possible_truncation)]
    fn delay_for_attempt(&self, attempt: usize) -> Duration {
        let delay = self
            .config
            .initial_delay
            .saturating_mul(1u32.wrapping_shl(attempt as u32));
        delay.min(self.config.max_delay)
    }
}

#[async_trait]
impl Provider for RetryProvider {
    async fn complete(&self, req: CompletionRequest) -> crate::Result<CompletionResponse> {
        let mut last_error = None;

        for attempt in 0..=self.config.max_retries {
            let result = self.inner.complete(req.clone()).await;
            match result {
                Ok(response) => return Ok(response),
                Err(e) => {
                    if !e.is_retryable() || attempt == self.config.max_retries {
                        return Err(e);
                    }
                    let delay = self.delay_for_attempt(attempt);
                    tracing::warn!(
                        attempt = attempt + 1,
                        max_retries = self.config.max_retries,
                        delay_ms = delay.as_millis(),
                        error = %e,
                        "Retrying provider call"
                    );
                    tokio::time::sleep(delay).await;
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| crate::Error::provider("retry exhausted")))
    }

    async fn stream(&self, req: CompletionRequest) -> crate::Result<CompletionStream> {
        let mut last_error = None;

        for attempt in 0..=self.config.max_retries {
            let result = self.inner.stream(req.clone()).await;
            match result {
                Ok(stream) => return Ok(stream),
                Err(e) => {
                    if !e.is_retryable() || attempt == self.config.max_retries {
                        return Err(e);
                    }
                    let delay = self.delay_for_attempt(attempt);
                    tracing::warn!(
                        attempt = attempt + 1,
                        max_retries = self.config.max_retries,
                        delay_ms = delay.as_millis(),
                        error = %e,
                        "Retrying provider stream"
                    );
                    tokio::time::sleep(delay).await;
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| crate::Error::provider("retry exhausted")))
    }

    fn model_info(&self) -> &ModelInfo {
        self.inner.model_info()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::completion::{ResponseContent, Usage};
    use crate::types::model_info::ModelTier;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct FailThenSucceedProvider {
        fail_count: AtomicUsize,
        info: ModelInfo,
    }

    impl FailThenSucceedProvider {
        fn new(fail_n_times: usize) -> Self {
            Self {
                fail_count: AtomicUsize::new(fail_n_times),
                info: ModelInfo::new("test", ModelTier::Small, 4096, false, false, false),
            }
        }
    }

    #[async_trait]
    impl Provider for FailThenSucceedProvider {
        async fn complete(&self, _req: CompletionRequest) -> crate::Result<CompletionResponse> {
            let remaining = self.fail_count.fetch_sub(1, Ordering::SeqCst);
            if remaining > 0 {
                Err(crate::Error::provider_with_status("server error", 500))
            } else {
                Ok(CompletionResponse {
                    content: ResponseContent::Text("ok".into()),
                    usage: Usage {
                        prompt_tokens: 1,
                        completion_tokens: 1,
                        total_tokens: 2,
                    },
                })
            }
        }

        async fn stream(&self, _req: CompletionRequest) -> crate::Result<CompletionStream> {
            unimplemented!()
        }

        fn model_info(&self) -> &ModelInfo {
            &self.info
        }
    }

    fn make_request() -> CompletionRequest {
        CompletionRequest {
            model: "test".into(),
            messages: vec![],
            tools: vec![],
            max_tokens: None,
            temperature: None,
            stream: false,
        }
    }

    #[tokio::test]
    async fn test_retry_succeeds_on_second_attempt() {
        let inner = Arc::new(FailThenSucceedProvider::new(1));
        let config = RetryConfig {
            max_retries: 3,
            initial_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(10),
        };
        let provider = RetryProvider::new(inner, config);

        let result = provider.complete(make_request()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_max_retries_exhausted() {
        let inner = Arc::new(FailThenSucceedProvider::new(10));
        let config = RetryConfig {
            max_retries: 2,
            initial_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(10),
        };
        let provider = RetryProvider::new(inner, config);

        let result = provider.complete(make_request()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_non_retryable_error_propagated_immediately() {
        struct NonRetryableProvider {
            info: ModelInfo,
        }

        #[async_trait]
        impl Provider for NonRetryableProvider {
            async fn complete(&self, _req: CompletionRequest) -> crate::Result<CompletionResponse> {
                Err(crate::Error::provider_with_status("unauthorized", 401))
            }
            async fn stream(&self, _req: CompletionRequest) -> crate::Result<CompletionStream> {
                unimplemented!()
            }
            fn model_info(&self) -> &ModelInfo {
                &self.info
            }
        }

        let inner = Arc::new(NonRetryableProvider {
            info: ModelInfo::new("test", ModelTier::Small, 4096, false, false, false),
        });
        let config = RetryConfig {
            max_retries: 3,
            initial_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(10),
        };
        let provider = RetryProvider::new(inner, config);

        let result = provider.complete(make_request()).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("unauthorized"));
    }

    #[test]
    fn test_exponential_backoff_timing() {
        let config = RetryConfig {
            max_retries: 5,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
        };
        let provider = RetryProvider::new(Arc::new(FailThenSucceedProvider::new(0)), config);

        assert_eq!(provider.delay_for_attempt(0), Duration::from_millis(100));
        assert_eq!(provider.delay_for_attempt(1), Duration::from_millis(200));
        assert_eq!(provider.delay_for_attempt(2), Duration::from_millis(400));
        assert_eq!(provider.delay_for_attempt(3), Duration::from_millis(800));
    }

    #[test]
    fn test_max_delay_cap() {
        let config = RetryConfig {
            max_retries: 5,
            initial_delay: Duration::from_secs(10),
            max_delay: Duration::from_secs(30),
        };
        let provider = RetryProvider::new(Arc::new(FailThenSucceedProvider::new(0)), config);

        // 10 * 2^2 = 40s, but capped at 30s
        assert_eq!(provider.delay_for_attempt(2), Duration::from_secs(30));
    }
}
