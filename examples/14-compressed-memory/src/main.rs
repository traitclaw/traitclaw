//! Example: Compressed Memory
//!
//! Demonstrates the CompressedMemory decorator pattern for automatic
//! context window management.

use traitclaw_core::memory::compressed::CompressedMemory;
use traitclaw_core::memory::in_memory::InMemoryMemory;
use traitclaw_core::traits::memory::Memory;
use traitclaw_core::types::message::Message;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Compressed Memory Example ===\n");

    // Create memory with compression: threshold=5, keep_recent=3
    let memory = CompressedMemory::new(InMemoryMemory::new(), 5, 3);

    println!("Config: threshold=5, keep_recent=3\n");

    // Simulate a conversation
    let conversation = [
        ("user", "What is Rust?"),
        (
            "assistant",
            "Rust is a systems programming language focused on safety and performance.",
        ),
        ("user", "What about ownership?"),
        (
            "assistant",
            "Ownership is Rust's unique memory management system with three rules.",
        ),
        ("user", "Explain borrowing"),
        (
            "assistant",
            "Borrowing lets you reference data without taking ownership.",
        ),
        ("user", "What about lifetimes?"),
        (
            "assistant",
            "Lifetimes are annotations that tell the compiler how long references are valid.",
        ),
    ];

    for (i, (role, content)) in conversation.iter().enumerate() {
        let msg = if *role == "user" {
            Message::user(*content)
        } else {
            Message::assistant(*content)
        };
        memory.append("session1", msg).await?;

        let messages = memory.messages("session1").await?;
        let total_stored = i + 1;
        let returned = messages.len();

        if total_stored <= 5 {
            println!(
                "Message {}: stored={total_stored}, returned={returned} (no compression)",
                i + 1
            );
        } else {
            println!(
                "Message {}: stored={total_stored}, returned={returned} ← COMPRESSED!",
                i + 1
            );
            // Show the summary
            if messages[0].content.contains("[Compressed") {
                let preview = if messages[0].content.len() > 80 {
                    format!("{}...", &messages[0].content[..80])
                } else {
                    messages[0].content.clone()
                };
                println!("  Summary: {preview}");
            }
        }
    }

    // --- Stackable decorators ---
    println!("\n--- Stackable Decorators ---\n");

    let inner = CompressedMemory::new(InMemoryMemory::new(), 20, 10);
    let outer = CompressedMemory::new(inner, 8, 3);

    println!("Inner: threshold=20, keep=10");
    println!("Outer: threshold=8, keep=3");
    println!("Effect: aggressive compression for long conversations\n");

    for i in 0..12 {
        outer
            .append("s2", Message::user(format!("Round {i}: Hello!")))
            .await?;
    }

    let messages = outer.messages("s2").await?;
    println!("Stored 12 messages → returned {}", messages.len());
    println!("  First message role: {:?}", messages[0].role);
    println!(
        "  Is compressed: {}",
        messages[0].content.contains("[Compressed")
    );

    Ok(())
}
