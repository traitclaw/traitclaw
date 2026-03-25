//! Example: Custom AgentStrategy
//!
//! Demonstrates how to implement a custom reasoning loop using the
//! AgentStrategy trait introduced in v0.2.0.
//!
//! This example implements a "ReflectiveStrategy" that asks the LLM to
//! reflect on its response before finalizing — a simple self-critique loop.

use async_trait::async_trait;
use traitclaw_core::agent::{AgentOutput, RunUsage};
use traitclaw_core::traits::strategy::{AgentRuntime, AgentStrategy};
use traitclaw_core::types::agent_state::AgentState;
use traitclaw_core::types::completion::{CompletionRequest, ResponseContent};
use traitclaw_core::types::message::Message;

/// A strategy that generates a response, then asks the LLM to improve it.
#[allow(dead_code)]
struct ReflectiveStrategy {
    reflect_prompt: String,
}

#[allow(dead_code)]
impl ReflectiveStrategy {
    fn new(reflect_prompt: impl Into<String>) -> Self {
        Self {
            reflect_prompt: reflect_prompt.into(),
        }
    }
}

#[async_trait]
impl AgentStrategy for ReflectiveStrategy {
    async fn execute(
        &self,
        runtime: &AgentRuntime,
        input: &str,
        session_id: &str,
    ) -> traitclaw_core::Result<AgentOutput> {
        let start = std::time::Instant::now();
        let model_info = runtime.provider.model_info();
        let mut state = AgentState::new(model_info.tier, model_info.context_window);

        // Step 1: Generate initial response
        let mut messages = vec![];
        if let Some(ref sys) = runtime.config.system_prompt {
            messages.push(Message::system(sys));
        }
        messages.push(Message::user(input));

        let request = CompletionRequest {
            model: model_info.name.clone(),
            messages: messages.clone(),
            tools: vec![],
            max_tokens: runtime.config.max_tokens,
            temperature: runtime.config.temperature,
            response_format: None,
            stream: false,
        };

        let response = runtime.provider.complete(request).await?;
        state.token_usage += response.usage.total_tokens;

        let initial = match response.content {
            ResponseContent::Text(text) => text,
            _ => {
                return Err(traitclaw_core::Error::Runtime(
                    "Unexpected tool call".into(),
                ))
            }
        };

        // Step 2: Ask model to reflect and improve
        messages.push(Message::assistant(&initial));
        messages.push(Message::user(&self.reflect_prompt));

        let reflect_request = CompletionRequest {
            model: model_info.name.clone(),
            messages,
            tools: vec![],
            max_tokens: runtime.config.max_tokens,
            temperature: runtime.config.temperature,
            response_format: None,
            stream: false,
        };

        let reflect_response = runtime.provider.complete(reflect_request).await?;
        state.token_usage += reflect_response.usage.total_tokens;
        state.iteration_count = 2;

        let final_text = match reflect_response.content {
            ResponseContent::Text(text) => text,
            _ => initial, // Fallback to initial if reflection fails
        };

        // Save to memory
        let _ = runtime
            .memory
            .append(session_id, Message::user(input))
            .await;
        let _ = runtime
            .memory
            .append(session_id, Message::assistant(&final_text))
            .await;

        Ok(AgentOutput::text_with_usage(
            final_text,
            RunUsage {
                tokens: state.token_usage,
                iterations: state.iteration_count,
                duration: start.elapsed(),
            },
        ))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Custom Strategy Example ===");
    println!();
    println!("This example shows how to implement a ReflectiveStrategy");
    println!("that generates a response, then asks the LLM to improve it.");
    println!();
    println!("Usage: Set OPENAI_API_KEY environment variable and run.");
    println!();

    // Uncomment to run with a real provider:
    //
    // use traitclaw::prelude::*;
    // use traitclaw_openai_compat::OpenAICompatProvider;
    //
    // let provider = OpenAICompatProvider::new("https://api.openai.com/v1", "gpt-4o-mini")
    //     .with_api_key(std::env::var("OPENAI_API_KEY")?);
    //
    // let agent = Agent::builder()
    //     .model(provider)
    //     .system("You are a helpful assistant")
    //     .strategy(ReflectiveStrategy::new(
    //         "Review your response. Is it clear and accurate? Improve it if needed."
    //     ))
    //     .build()?;
    //
    // let output = agent.run("Explain Rust's ownership model").await?;
    // println!("{}", output.text());

    Ok(())
}
