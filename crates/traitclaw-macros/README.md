# traitclaw-macros

[![crates.io](https://img.shields.io/crates/v/traitclaw-macros.svg)](https://crates.io/crates/traitclaw-macros)
[![docs.rs](https://docs.rs/traitclaw-macros/badge.svg)](https://docs.rs/traitclaw-macros)

**Proc macros for the TraitClaw AI Agent Framework.**

Provides `#[derive(Tool)]` for generating type-safe tool implementations with automatic JSON schema generation from Rust structs.

## Usage

```rust
use traitclaw::prelude::*;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Tool)]
#[tool(description = "Search the web for information")]
struct WebSearch {
    /// The search query
    query: String,
    /// Maximum number of results
    max_results: Option<u32>,
}
```

The derive macro automatically:
- Generates `ErasedTool` implementation
- Creates JSON schema from struct fields and doc comments
- Wires up serialization/deserialization

## Note

Most users should use `traitclaw` with the `macros` feature (enabled by default) instead of depending on this crate directly.

## License

Licensed under either of [Apache License, Version 2.0](../../LICENSE-APACHE) or [MIT License](../../LICENSE-MIT) at your option.
