//! `RateLimitGuard` — limits agent loop iterations by time window.
//!
//! Prevents runaway agents from exceeding a configured call rate.
//!
//! # Example
//!
//! ```rust
//! use traitclaw_steering::guards::rate_limit::RateLimitGuard;
//! use traitclaw_core::traits::guard::{Guard, GuardResult};
//! use traitclaw_core::types::action::Action;
//!
//! // Allow max 60 calls per minute
//! let guard = RateLimitGuard::new(60);
//!
//! // First call goes through
//! let result = guard.check(&Action::RawOutput { content: "hello".to_string() });
//! assert!(matches!(result, GuardResult::Allow));
//! ```

use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use traitclaw_core::traits::guard::{Guard, GuardResult, GuardSeverity};
use traitclaw_core::types::action::Action;

/// A [`Guard`] that limits calls to `max_calls_per_minute` within a rolling 60-second window.
///
/// When the limit is exceeded, returns `GuardResult::Deny` with a rate limit message.
/// Thread-safe via `Mutex<VecDeque<Instant>>` for interior mutability.
pub struct RateLimitGuard {
    /// Maximum number of calls allowed per minute.
    max_calls_per_minute: usize,
    /// Timestamps of recent calls within the window.
    call_times: Mutex<VecDeque<Instant>>,
}

impl RateLimitGuard {
    /// Create a new `RateLimitGuard` with the given rate limit.
    #[must_use]
    pub fn new(max_calls_per_minute: usize) -> Self {
        Self {
            max_calls_per_minute,
            call_times: Mutex::new(VecDeque::new()),
        }
    }

    /// Remove call timestamps older than 60 seconds given `now`.
    fn evict_stale(times: &mut VecDeque<Instant>, now: Instant) {
        // Use checked_sub to safely handle systems where now < 60s (e.g., early in boot or tests)
        if let Some(cutoff) = now.checked_sub(Duration::from_secs(60)) {
            while times.front().map_or(false, |&t| t < cutoff) {
                times.pop_front();
            }
        }
        // If now < 60s elapsed, no timestamps can be older than the window — nothing to evict
    }

    /// Number of calls recorded in the current 60-second window.
    #[must_use]
    pub fn current_call_count(&self) -> usize {
        let now = Instant::now();
        let mut times = self.call_times.lock().expect("mutex poisoned");
        Self::evict_stale(&mut times, now);
        times.len()
    }
}

impl Guard for RateLimitGuard {
    fn name(&self) -> &'static str {
        "rate_limit"
    }

    fn check(&self, _action: &Action) -> GuardResult {
        let now = Instant::now();
        let mut times = self.call_times.lock().expect("mutex poisoned");
        Self::evict_stale(&mut times, now);

        if times.len() >= self.max_calls_per_minute {
            GuardResult::Deny {
                reason: format!(
                    "RateLimitGuard: rate limit of {} calls/minute exceeded",
                    self.max_calls_per_minute
                ),
                severity: GuardSeverity::Medium,
            }
        } else {
            times.push_back(now);
            GuardResult::Allow
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn any_action() -> Action {
        Action::RawOutput {
            content: "hello".to_string(),
        }
    }

    #[test]
    fn test_rate_limit_allows_within_limit() {
        // AC #1: calls up to limit are allowed
        let guard = RateLimitGuard::new(5);
        for _ in 0..5 {
            let result = guard.check(&any_action());
            assert!(matches!(result, GuardResult::Allow));
        }
    }

    #[test]
    fn test_rate_limit_denies_at_limit() {
        // AC #2: call beyond limit is denied
        let guard = RateLimitGuard::new(60);
        for _ in 0..60 {
            guard.check(&any_action());
        }
        let result = guard.check(&any_action());
        assert!(
            matches!(result, GuardResult::Deny { .. }),
            "Expected Deny but got: {result:?}"
        );
    }

    #[test]
    fn test_rate_limit_small_window() {
        // AC #2: exact boundary test
        let guard = RateLimitGuard::new(3);
        assert!(matches!(guard.check(&any_action()), GuardResult::Allow));
        assert!(matches!(guard.check(&any_action()), GuardResult::Allow));
        assert!(matches!(guard.check(&any_action()), GuardResult::Allow));
        // 4th call should be denied
        assert!(matches!(
            guard.check(&any_action()),
            GuardResult::Deny { .. }
        ));
    }

    #[test]
    fn test_rate_limit_zero_limit() {
        // Edge case: 0 limit means every call is denied
        let guard = RateLimitGuard::new(0);
        let result = guard.check(&any_action());
        assert!(matches!(result, GuardResult::Deny { .. }));
    }
}
