//! `RetryPolicy` — configurable exponential backoff for transient API errors.
//!
//! # Example
//!
//! ```rust
//! use traitclaw_openai_compat::retry::RetryPolicy;
//!
//! let policy = RetryPolicy::exponential(3, std::time::Duration::from_millis(100));
//! assert_eq!(policy.max_retries, 3);
//! assert_eq!(policy.initial_delay, std::time::Duration::from_millis(100));
//! ```

use std::time::Duration;

/// HTTP status codes that should trigger a retry.
pub const RETRYABLE_STATUS_CODES: &[u16] = &[429, 500, 502, 503, 504];

/// Controls retry behaviour for transient API errors.
///
/// Retries on HTTP 429, 500, 502, 503, 504.
/// Uses exponential backoff: `initial_delay * 2^attempt`.
/// Optionally adds `±25%` random jitter.
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts (not counting the initial request).
    pub max_retries: usize,
    /// Delay before the first retry.
    pub initial_delay: Duration,
    /// Maximum delay between retries (caps backoff growth).
    pub max_delay: Duration,
    /// Whether to add random jitter (±25% of computed delay).
    pub jitter: bool,
}

impl RetryPolicy {
    /// Create a `RetryPolicy` with pure exponential backoff (no jitter).
    #[must_use]
    pub fn exponential(max_retries: usize, initial_delay: Duration) -> Self {
        Self {
            max_retries,
            initial_delay,
            max_delay: Duration::from_secs(60),
            jitter: false,
        }
    }

    /// Enable jitter (±25% of computed delay).
    ///
    /// Uses lightweight deterministic pseudo-randomness based on attempt number and
    /// initial delay hash. Concurrent callers with different `initial_delay` values
    /// will naturally desynchronize their retry intervals, preventing thundering-herd.
    /// For true randomness in production, a `rand`-based implementation is recommended.
    #[must_use]
    pub fn with_jitter(mut self) -> Self {
        self.jitter = true;
        self
    }

    /// Set the maximum delay cap.
    #[must_use]
    pub fn with_max_delay(mut self, max_delay: Duration) -> Self {
        self.max_delay = max_delay;
        self
    }

    /// Compute the delay for `attempt` (0-indexed: attempt 0 = first retry).
    ///
    /// Returns `initial_delay * 2^attempt`, capped at `max_delay`.
    #[must_use]
    pub fn delay_for(&self, attempt: usize) -> Duration {
        let multiplier = (2usize).saturating_pow(attempt as u32);
        let base = self.initial_delay.saturating_mul(multiplier as u32);
        let capped = base.min(self.max_delay);

        if self.jitter {
            // Pseudo-random jitter: combine attempt with initial_delay nanos to decouple
            // concurrent callers. Result varies between 0.75× and 1.25× of capped delay.
            let seed = self
                .initial_delay
                .as_nanos()
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(attempt as u128);
            // Map seed into [0, 50] then shift to [−25, +25] → factor in [0.75, 1.25]
            let jitter_pct = (seed % 51) as f64; // 0..=50
            let factor = 0.75 + jitter_pct / 100.0; // 0.75..=1.25
            Duration::from_nanos((capped.as_nanos() as f64 * factor) as u64).min(self.max_delay)
        } else {
            capped
        }
    }

    /// Whether `status_code` should trigger a retry.
    #[must_use]
    pub fn is_retryable(status_code: u16) -> bool {
        RETRYABLE_STATUS_CODES.contains(&status_code)
    }
}

impl Default for RetryPolicy {
    /// Default: 3 retries, 1s initial delay, no jitter.
    fn default() -> Self {
        Self::exponential(3, Duration::from_secs(1))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exponential_backoff_delay() {
        // AC #5: delay doubles each attempt
        let policy = RetryPolicy::exponential(3, Duration::from_millis(100));

        let d0 = policy.delay_for(0); // 100ms × 2^0 = 100ms
        let d1 = policy.delay_for(1); // 100ms × 2^1 = 200ms
        let d2 = policy.delay_for(2); // 100ms × 2^2 = 400ms

        assert_eq!(d0, Duration::from_millis(100));
        assert_eq!(d1, Duration::from_millis(200));
        assert_eq!(d2, Duration::from_millis(400));
    }

    #[test]
    fn test_max_delay_cap() {
        let policy = RetryPolicy::exponential(10, Duration::from_millis(100))
            .with_max_delay(Duration::from_millis(300));

        // After the cap kicks in, delay should not exceed max_delay
        let d5 = policy.delay_for(5); // 100 * 32 = 3200ms → capped at 300ms
        assert_eq!(d5, Duration::from_millis(300));
    }

    #[test]
    fn test_retryable_status_codes() {
        // AC #4: retries on 429, 500, 502, 503, 504
        assert!(RetryPolicy::is_retryable(429));
        assert!(RetryPolicy::is_retryable(500));
        assert!(RetryPolicy::is_retryable(502));
        assert!(RetryPolicy::is_retryable(503));
        assert!(RetryPolicy::is_retryable(504));
    }

    #[test]
    fn test_non_retryable_status_codes() {
        assert!(!RetryPolicy::is_retryable(200));
        assert!(!RetryPolicy::is_retryable(400)); // Bad Request — not retried
        assert!(!RetryPolicy::is_retryable(401)); // Auth — not retried
        assert!(!RetryPolicy::is_retryable(404)); // Not Found — not retried
    }

    #[test]
    fn test_max_retries_field() {
        // AC #1: max_retries is stored correctly
        let policy = RetryPolicy::exponential(5, Duration::from_secs(1));
        assert_eq!(policy.max_retries, 5);
    }

    #[test]
    fn test_jitter_changes_delay() {
        // With jitter, delay should differ from base
        let base_policy = RetryPolicy::exponential(3, Duration::from_millis(100));
        let jitter_policy = RetryPolicy::exponential(3, Duration::from_millis(100)).with_jitter();

        let base_d = base_policy.delay_for(0);
        let jitter_d = jitter_policy.delay_for(0);

        // They will differ because jitter applies a factor
        assert_ne!(base_d, jitter_d, "jitter should modify the delay");
    }

    #[test]
    fn test_default_policy() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.max_retries, 3);
        assert_eq!(policy.initial_delay, Duration::from_secs(1));
        assert!(!policy.jitter);
    }
}
