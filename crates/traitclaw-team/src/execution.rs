//! Team execution engine — `Team.bind()` and `Team.run()`.
//!
//! Connects [`AgentRole`](crate::AgentRole)s to callable agent functions and
//! executes the orchestration pipeline driven by the team's router.
//!
//! # Design
//!
//! `BoundAgent` is a `Box<dyn Fn(&str) -> BoxFuture<Result<String>>>` so
//! that any callable async closure or agent can be bound without requiring
//! `traitclaw-core` to be imported by consumers — they just pass a closure.
//!
//! # Example
//!
//! ```rust
//! use traitclaw_team::execution::TeamRunner;
//!
//! # async fn example() -> traitclaw_core::Result<()> {
//! let mut runner = TeamRunner::new(2); // max 2 iterations
//!
//! // Bind agents as async closures
//! runner.bind("researcher", |input: String| async move {
//!     Ok(format!("Research result for: {input}"))
//! });
//! runner.bind("writer", |input: String| async move {
//!     Ok(format!("Written summary of: {input}"))
//! });
//!
//! // Set sequential order
//! runner.set_sequence(&["researcher", "writer"]);
//!
//! let output = runner.run("Write a report on AI").await?;
//! assert!(output.contains("Written summary"));
//! # Ok(())
//! # }
//! ```

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use traitclaw_core::{Error, Result};

/// Future alias for boxed async agent calls.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Type-erased callable agent.
pub type BoundAgent = Arc<dyn Fn(String) -> BoxFuture<'static, Result<String>> + Send + Sync>;

/// Sequential team execution engine.
///
/// Binds agent roles to async callables and runs them in sequence,
/// passing each agent's output as the next agent's input.
pub struct TeamRunner {
    agents: HashMap<String, BoundAgent>,
    sequence: Vec<String>,
    max_iterations: usize,
}

impl TeamRunner {
    /// Create a new `TeamRunner` with the given max iteration limit.
    ///
    /// # Panics
    ///
    /// Panics if `max_iterations == 0`.
    #[must_use]
    pub fn new(max_iterations: usize) -> Self {
        assert!(max_iterations > 0, "max_iterations must be > 0");
        Self {
            agents: HashMap::new(),
            sequence: Vec::new(),
            max_iterations,
        }
    }

    /// Bind an agent closure to a role name.
    ///
    /// The closure receives the current input and returns the agent's response.
    pub fn bind<F, Fut>(&mut self, role: impl Into<String>, agent: F)
    where
        F: Fn(String) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<String>> + Send + 'static,
    {
        let name = role.into();
        self.agents.insert(
            name,
            Arc::new(move |input: String| Box::pin(agent(input)) as BoxFuture<_>),
        );
    }

    /// Set the execution sequence of role names.
    pub fn set_sequence(&mut self, roles: &[&str]) {
        self.sequence = roles.iter().map(|&r| r.to_string()).collect();
    }

    /// Check if a role has been bound.
    #[must_use]
    pub fn is_bound(&self, role: &str) -> bool {
        self.agents.contains_key(role)
    }

    /// Execute the team pipeline sequentially.
    ///
    /// Each agent in the sequence receives the previous agent's output.
    /// Returns the last agent's output as the final result.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - A bound agent returns an error
    /// - The sequence is empty
    /// - A role in the sequence is not bound
    /// - `max_iterations` is exceeded
    pub async fn run(&self, input: &str) -> Result<String> {
        if self.sequence.is_empty() {
            return Err(Error::Runtime("TeamRunner has no sequence defined".into()));
        }

        let mut current_input = input.to_string();
        let mut iterations = 0;

        for role in &self.sequence {
            if iterations >= self.max_iterations {
                return Err(Error::Runtime(format!(
                    "TeamRunner exceeded max_iterations ({}) at role '{}'",
                    self.max_iterations, role
                )));
            }

            let agent = self.agents.get(role).ok_or_else(|| {
                Error::Runtime(format!("Role '{}' not bound in TeamRunner", role))
            })?;

            current_input = agent(current_input).await?;
            iterations += 1;
        }

        Ok(current_input)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// VerificationChain execution
// ─────────────────────────────────────────────────────────────────────────────

/// Execute a generate-verify-retry loop.
///
/// - Generator produces output from the current prompt
/// - Verifier returns `Ok(output)` to accept, or `Err(feedback)` to reject
/// - On rejection, the prompt is augmented with feedback and generation retried
///
/// # Example
///
/// ```rust
/// use traitclaw_team::execution::run_verification_chain;
///
/// # async fn example() -> traitclaw_core::Result<()> {
/// let result = run_verification_chain(
///     "Write a haiku",
///     3,
///     |input: String| async move {
///         // Pretend the first attempt fails
///         if !input.contains("retry") {
///             Ok(format!("Draft from: {input}"))
///         } else {
///             Ok(format!("Improved draft from: {input}"))
///         }
///     },
///     |output: String| async move {
///         // Accept on 2nd attempt (when output mentions "retry")
///         if output.contains("retry") {
///             Err(format!("Needs improvement: {output}"))
///         } else {
///             Ok(output) // Accept on first try here
///         }
///     },
/// ).await;
/// assert!(result.is_ok());
/// # Ok(())
/// # }
/// ```
pub async fn run_verification_chain<G, GFut, V, VFut>(
    initial_input: &str,
    max_retries: usize,
    generator: G,
    verifier: V,
) -> Result<String>
where
    G: Fn(String) -> GFut,
    GFut: Future<Output = Result<String>>,
    V: Fn(String) -> VFut,
    VFut: Future<Output = std::result::Result<String, String>>,
{
    let mut prompt = initial_input.to_string();
    let mut last_output = String::new();

    for attempt in 0..=max_retries {
        let output = generator(prompt.clone()).await?;
        last_output = output.clone();

        match verifier(output.clone()).await {
            Ok(accepted) => return Ok(accepted),
            Err(feedback) => {
                if attempt == max_retries {
                    // All retries exhausted
                    return Err(Error::Runtime(format!(
                        "VerificationChain exhausted {max_retries} retries. Last output: {last_output}. Last feedback: {feedback}"
                    )));
                }
                // Augment prompt with feedback for retry
                prompt = format!("{initial_input}\n\nPrevious attempt:\n{output}\n\nFeedback: {feedback}\n\nPlease improve.");
            }
        }
    }

    // Unreachable but satisfies compiler
    Ok(last_output)
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── TeamRunner ────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_sequential_team_two_agents() {
        // AC #7: researcher → writer sequential pipeline
        let mut runner = TeamRunner::new(10);

        runner.bind("researcher", |input: String| async move {
            Ok(format!("Research: {input}"))
        });
        runner.bind("writer", |input: String| async move {
            Ok(format!("Written: {input}"))
        });
        runner.set_sequence(&["researcher", "writer"]);

        let result = runner.run("AI history").await.unwrap();
        assert!(
            result.starts_with("Written: Research: AI history"),
            "got: {result}"
        );
    }

    #[tokio::test]
    async fn test_max_iterations_exceeded() {
        // AC #8: max_iterations exceeded → error
        let mut runner = TeamRunner::new(1); // only 1 allowed
        runner.bind("a", |i: String| async move { Ok(i) });
        runner.bind("b", |i: String| async move { Ok(i) });
        runner.set_sequence(&["a", "b"]); // 2 agents, limit=1

        let result = runner.run("test").await;
        assert!(
            result.is_err(),
            "expected error for max_iterations exceeded"
        );
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("max_iterations"),
            "error should mention max_iterations: {msg}"
        );
    }

    #[tokio::test]
    async fn test_unbound_role_returns_error() {
        let mut runner = TeamRunner::new(10);
        runner.set_sequence(&["missing_role"]);
        let result = runner.run("test").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_empty_sequence_returns_error() {
        let runner = TeamRunner::new(10);
        let result = runner.run("test").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no sequence"));
    }

    // ── VerificationChain ─────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_verification_chain_accepts_on_second_try() {
        // AC #7: generator accepts on 2nd try — prompt contains retry feedback
        let attempt_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let attempt_clone = attempt_count.clone();

        let verify_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let verify_clone = verify_count.clone();

        let result = run_verification_chain(
            "Write something",
            3,
            move |_input: String| {
                let c = attempt_clone.clone();
                async move {
                    let n = c.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    Ok(format!("generated-attempt-{n}"))
                }
            },
            move |output: String| {
                let v = verify_clone.clone();
                async move {
                    let n = v.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    if n == 0 {
                        Err("Too brief".to_string())
                    } else {
                        Ok(output) // accept on 2nd verify call
                    }
                }
            },
        )
        .await;

        assert!(
            result.is_ok(),
            "expected success on retry, got: {:?}",
            result
        );
        let output = result.unwrap();
        // Second attempt's output should be returned
        assert!(
            output.contains("attempt-1"),
            "expected 2nd attempt output, got: {output}"
        );
    }

    #[tokio::test]
    async fn test_verification_chain_all_retries_exhausted() {
        // AC #8: all retries exhausted → error with last output
        let result = run_verification_chain(
            "Write something",
            2, // max 2 retries
            |input: String| async move { Ok(format!("draft: {input}")) },
            |output: String| async move { Err(format!("not good enough: {output}")) },
        )
        .await;

        assert!(result.is_err(), "expected error when all retries exhausted");
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("exhausted"),
            "error should mention exhausted: {msg}"
        );
    }

    #[tokio::test]
    async fn test_verification_chain_accepts_immediately() {
        let result = run_verification_chain(
            "input",
            3,
            |_| async { Ok("perfect output".to_string()) },
            |output: String| async move { Ok(output) },
        )
        .await;

        assert_eq!(result.unwrap(), "perfect output");
    }

    #[tokio::test]
    async fn test_verification_chain_feedback_included_in_retry() {
        // AC #7: on rejection, generator retries with feedback appended
        let got_feedback = std::sync::Arc::new(std::sync::Mutex::new(false));
        let got_feedback_clone = got_feedback.clone();

        let _ = run_verification_chain(
            "Write",
            1,
            move |input: String| {
                let f = got_feedback_clone.clone();
                async move {
                    if input.contains("Feedback:") {
                        *f.lock().unwrap() = true;
                    }
                    Ok(format!("output: {input}"))
                }
            },
            |_| async move { Err("needs work".to_string()) },
        )
        .await;

        assert!(
            *got_feedback.lock().unwrap(),
            "retry prompt should contain 'Feedback:'"
        );
    }
}
