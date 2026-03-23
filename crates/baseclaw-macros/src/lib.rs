//! Proc macros for the `BaseClaw` AI Agent Framework.
//!
//! # Usage
//!
//! ```rust,ignore
//! use baseclaw::Tool;
//!
//! #[derive(Tool)]
//! #[tool(description = "Search the web for information")]
//! struct WebSearch {
//!     /// The search query
//!     query: String,
//! }
//! ```

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput};

/// Derive the `Tool` schema boilerplate for a struct.
///
/// # Attributes
///
/// - `#[tool(description = "...")]` — tool description (required)
/// - `#[tool(name = "...")]` — override tool name (optional)
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

    // Parse #[tool(...)] attributes using syn v2 API
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
