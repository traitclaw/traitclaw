//! Proc macros for the `BaseClaw` AI Agent Framework.
//!
//! # Usage
//!
//! ```rust,ignore
//! use baseclaw::Tool;
//! use schemars::JsonSchema;
//! use serde::Deserialize;
//!
//! #[derive(Tool, Deserialize, JsonSchema)]
//! #[tool(description = "Search the web for information")]
//! struct WebSearch {
//!     /// The search query
//!     query: String,
//! }
//!
//! impl WebSearch {
//!     async fn execute(&self) -> baseclaw_core::Result<serde_json::Value> {
//!         Ok(serde_json::json!({"results": []}))
//!     }
//! }
//! ```

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput};

/// Derive the `ErasedTool` implementation boilerplate for a struct.
///
/// The struct itself acts as the tool's Input type and MUST derive
/// `serde::Deserialize` and `schemars::JsonSchema`.
///
/// The user MUST provide an inherent `execute(&self) -> Result<serde_json::Value>` method.
///
/// # Attributes
///
/// - `#[tool(description = "...")]` — tool description (required)
/// - `#[tool(name = "...")]` — override tool name (optional, defaults to snake_case)
#[proc_macro_derive(Tool, attributes(tool))]
pub fn derive_tool(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match expand_tool(input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn expand_tool(input: DeriveInput) -> syn::Result<TokenStream2> {
    // Validate struct
    match &input.data {
        Data::Struct(_) => {}
        _ => {
            return Err(syn::Error::new_spanned(
                &input.ident,
                "#[derive(Tool)] can only be applied to structs",
            ));
        }
    }

    let struct_name = &input.ident;

    // Parse #[tool(...)] attributes
    let mut description: Option<String> = None;
    let mut name_override: Option<String> = None;

    for attr in &input.attrs {
        if !attr.path().is_ident("tool") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("description") {
                let value: syn::LitStr = meta.value()?.parse()?;
                description = Some(value.value());
            } else if meta.path.is_ident("name") {
                let value: syn::LitStr = meta.value()?.parse()?;
                name_override = Some(value.value());
            }
            Ok(())
        })?;
    }

    let description = description.unwrap_or_else(|| to_title_case(&struct_name.to_string()));
    let tool_name = name_override.unwrap_or_else(|| to_snake_case(&struct_name.to_string()));

    let expanded = quote! {
        // Inherent helper methods for static access
        impl #struct_name {
            /// Returns the statically known tool name.
            pub fn tool_name() -> &'static str {
                #tool_name
            }

            /// Returns the statically known tool description.
            pub fn tool_description() -> &'static str {
                #description
            }

            /// Generate the [`baseclaw_core::ToolSchema`] for this tool.
            pub fn tool_schema() -> baseclaw_core::ToolSchema {
                let schema = schemars::schema_for!(#struct_name);
                baseclaw_core::ToolSchema {
                    name: #tool_name.to_string(),
                    description: #description.to_string(),
                    parameters: serde_json::to_value(schema)
                        .unwrap_or_else(|_| serde_json::Value::Object(Default::default())),
                }
            }
        }

        // ErasedTool impl — the struct IS the input type.
        // The user must provide:
        //   `async fn execute(&self) -> baseclaw_core::Result<serde_json::Value>`
        #[async_trait::async_trait]
        impl baseclaw_core::ErasedTool for #struct_name {
            fn name(&self) -> &str {
                #tool_name
            }

            fn description(&self) -> &str {
                #description
            }

            fn schema(&self) -> baseclaw_core::ToolSchema {
                #struct_name::tool_schema()
            }

            async fn execute_json(
                &self,
                input: serde_json::Value,
            ) -> baseclaw_core::Result<serde_json::Value> {
                let typed: #struct_name = serde_json::from_value(input)
                    .map_err(|e| baseclaw_core::Error::tool_execution(
                        #tool_name,
                        format!("Invalid input: {e}"),
                    ))?;
                typed.execute().await
            }
        }
    };

    Ok(expanded)
}

/// Convert `PascalCase` to `snake_case`.
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.extend(c.to_lowercase());
    }
    result
}

/// Convert `PascalCase` to `Title Case`.
fn to_title_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            result.push(' ');
        }
        result.push(c);
    }
    result
}
