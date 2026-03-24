//! # Memory Persistence — SQLite-backed conversation history
//!
//! Demonstrates using `SqliteMemory` to persist conversations across
//! agent restarts. The memory backend stores messages, working memory,
//! and long-term knowledge in a single SQLite database file.

use baseclaw::prelude::*;
use baseclaw_core::traits::memory::MemoryEntry;
use baseclaw_memory_sqlite::SqliteMemory;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("💾 Memory Persistence Demo\n");

    // ── 1. Create SQLite-backed memory ──────────────────────
    // Use in-memory DB for this demo (use a file path for real persistence)
    let memory = SqliteMemory::in_memory()?;

    // Create a named session for conversation isolation
    let session_id = memory.create_session().await?;
    println!("📝 Created session: {session_id}\n");

    // ── 2. Simulate a multi-turn conversation ───────────────
    let turns: Vec<(&str, &str)> = vec![
        ("user", "My name is Alex and I'm learning Rust."),
        ("assistant", "Nice to meet you, Alex! Rust is a great choice."),
        ("user", "What should I learn first?"),
        ("assistant", "Start with ownership and borrowing — they're Rust's core concepts."),
        ("user", "Thanks! I'll focus on that."),
        ("assistant", "Great plan! Let me know if you need help with examples."),
    ];

    for (role, content) in &turns {
        let message = match *role {
            "user" => Message::user(*content),
            "assistant" => Message::assistant(*content),
            _ => continue,
        };
        memory.append(&session_id, message).await?;
    }

    println!("💬 Stored {} messages\n", turns.len());

    // ── 3. Retrieve conversation history ────────────────────
    let history = memory.messages(&session_id).await?;
    println!("📜 Conversation history:");
    for msg in &history {
        let icon = match msg.role {
            MessageRole::User => "👤",
            MessageRole::Assistant => "🤖",
            _ => "📋",
        };
        println!("  {icon} {}", msg.content);
    }

    // ── 4. Working memory (key-value per session) ───────────
    println!("\n🧠 Working memory:");
    memory
        .set_context(&session_id, "user_name", serde_json::json!("Alex"))
        .await?;
    memory
        .set_context(&session_id, "topic", serde_json::json!("Rust ownership"))
        .await?;

    let name = memory.get_context(&session_id, "user_name").await?;
    let topic = memory.get_context(&session_id, "topic").await?;
    println!("  Name: {}", name.unwrap_or_default());
    println!("  Topic: {}", topic.unwrap_or_default());

    // ── 5. Long-term knowledge store + recall ───────────────
    println!("\n📚 Long-term memory:");

    memory
        .store(MemoryEntry::now("fact-1", "Ownership is Rust's key innovation"))
        .await?;
    memory
        .store(MemoryEntry::now(
            "fact-2",
            "Borrowing allows references without ownership transfer",
        ))
        .await?;
    memory
        .store(MemoryEntry::now(
            "fact-3",
            "Lifetimes ensure references are valid",
        ))
        .await?;

    let recalled = memory.recall("ownership borrowing", 5).await?;
    println!(
        "  Recalled {} entries for 'ownership borrowing':",
        recalled.len()
    );
    for entry in &recalled {
        println!("    → {}", entry.content);
    }

    // ── 6. Session management ───────────────────────────────
    println!("\n📋 Sessions:");
    let sessions = memory.list_sessions().await?;
    println!("  Active sessions: {}", sessions.len());

    // Clean up
    memory.delete_session(&session_id).await?;
    println!("  Deleted session {}", &session_id[..8]);

    println!("\n✅ Done!");
    Ok(())
}
