//! # Streaming — Real-time incremental responses with BaseClaw
//!
//! Demonstrates using `agent.stream()` to receive and display text
//! as it arrives from the LLM, providing a real-time chat experience.

use baseclaw::prelude::*;
use baseclaw_openai_compat::OpenAiCompatProvider;
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let provider = OpenAiCompatProvider::openai(
        "gpt-4o-mini",
        std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "demo-key".into()),
    );

    let agent = Agent::builder()
        .provider(provider)
        .system("You are a storyteller. Tell engaging short stories.")
        .build()?;

    // Stream the response — each event arrives as it's generated
    println!("📖 Streaming story...\n");
    let mut stream = agent.stream("Tell me a 3-sentence story about a brave robot");

    while let Some(event) = stream.next().await {
        match event {
            Ok(StreamEvent::TextDelta(text)) => {
                print!("{text}");
                use std::io::Write;
                std::io::stdout().flush().ok();
            }
            Ok(StreamEvent::Done) => break,
            Ok(_) => {} // Other events (tool calls, etc.)
            Err(e) => {
                eprintln!("\n⚠️ Stream error: {e}");
                break;
            }
        }
    }

    println!("\n\n✅ Stream complete!");
    Ok(())
}
