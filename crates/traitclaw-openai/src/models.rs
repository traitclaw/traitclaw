//! Ergonomic model constructor functions.
//!
//! These free functions match the developer-experience shown in the architecture:
//!
//! ```rust,no_run
//! use traitclaw_openai::openai;
//!
//! // Reads OPENAI_API_KEY from the environment automatically:
//! let provider = openai("gpt-4o-mini");
//! ```

use traitclaw_openai_compat::{OpenAiCompatConfig, OpenAiCompatProvider};

// ─── Base URL constants ───────────────────────────────────────────────────────

const OPENAI_BASE: &str = "https://api.openai.com/v1";
const GROQ_BASE: &str = "https://api.groq.com/openai/v1";
const TOGETHER_BASE: &str = "https://api.together.xyz/v1";
const MISTRAL_BASE: &str = "https://api.mistral.ai/v1";
const OLLAMA_BASE: &str = "http://localhost:11434/v1";

// ─── Key resolution helpers ───────────────────────────────────────────────────

fn env_key(var: &str) -> String {
    std::env::var(var).unwrap_or_default()
}

// ─── Constructor Functions ────────────────────────────────────────────────────

/// Create an OpenAI provider, reading `OPENAI_API_KEY` from the environment.
///
/// # Example
///
/// ```rust,no_run
/// use traitclaw_openai::openai;
///
/// let provider = openai("gpt-4o-mini");
/// ```
#[must_use]
pub fn openai(model: impl Into<String>) -> OpenAiCompatProvider {
    OpenAiCompatProvider::new(OpenAiCompatConfig {
        base_url: OPENAI_BASE.to_string(),
        api_key: env_key("OPENAI_API_KEY"),
        model: model.into(),
        model_info: None,
    })
}

/// Create a Groq provider, reading `GROQ_API_KEY` from the environment.
///
/// # Example
///
/// ```rust,no_run
/// use traitclaw_openai::groq;
///
/// let provider = groq("llama-3.3-70b-versatile");
/// ```
#[must_use]
pub fn groq(model: impl Into<String>) -> OpenAiCompatProvider {
    OpenAiCompatProvider::new(OpenAiCompatConfig {
        base_url: GROQ_BASE.to_string(),
        api_key: env_key("GROQ_API_KEY"),
        model: model.into(),
        model_info: None,
    })
}

/// Create a Together AI provider, reading `TOGETHER_API_KEY` from the environment.
///
/// # Example
///
/// ```rust,no_run
/// use traitclaw_openai::together;
///
/// let provider = together("meta-llama/Llama-3-70b-chat-hf");
/// ```
#[must_use]
pub fn together(model: impl Into<String>) -> OpenAiCompatProvider {
    OpenAiCompatProvider::new(OpenAiCompatConfig {
        base_url: TOGETHER_BASE.to_string(),
        api_key: env_key("TOGETHER_API_KEY"),
        model: model.into(),
        model_info: None,
    })
}

/// Create a Mistral provider, reading `MISTRAL_API_KEY` from the environment.
///
/// # Example
///
/// ```rust,no_run
/// use traitclaw_openai::mistral;
///
/// let provider = mistral("mistral-large-latest");
/// ```
#[must_use]
pub fn mistral(model: impl Into<String>) -> OpenAiCompatProvider {
    OpenAiCompatProvider::new(OpenAiCompatConfig {
        base_url: MISTRAL_BASE.to_string(),
        api_key: env_key("MISTRAL_API_KEY"),
        model: model.into(),
        model_info: None,
    })
}

/// Create a local Ollama provider (no authentication needed).
///
/// # Example
///
/// ```rust,no_run
/// use traitclaw_openai::ollama;
///
/// let provider = ollama("llama3.2");
/// ```
#[must_use]
pub fn ollama(model: impl Into<String>) -> OpenAiCompatProvider {
    OpenAiCompatProvider::new(OpenAiCompatConfig {
        base_url: OLLAMA_BASE.to_string(),
        api_key: String::new(),
        model: model.into(),
        model_info: None,
    })
}

/// Create an Azure OpenAI provider.
///
/// - `resource` — your Azure resource name
/// - `deployment` — the deployment name
/// - `api_version` — e.g. `"2024-02-01"` (`None` defaults to `"2024-02-01"`)
///
/// Reads `AZURE_OPENAI_API_KEY` from the environment.
///
/// # Example
///
/// ```rust,no_run
/// use traitclaw_openai::azure_openai;
///
/// let provider = azure_openai("my-resource", "gpt-4o-mini", None);
/// ```
#[must_use]
pub fn azure_openai(
    resource: impl Into<String>,
    deployment: impl Into<String>,
    api_version: Option<&str>,
) -> OpenAiCompatProvider {
    let resource = resource.into();
    let deployment = deployment.into();
    let api_version = api_version.unwrap_or("2024-02-01");
    let base_url = format!(
        "https://{resource}.openai.azure.com/openai/deployments/{deployment}?api-version={api_version}"
    );
    // Azure uses `api-key` header but OpenAI-compat uses Bearer.
    // For simplicity we put the key in api_key; callers can also set
    // AZURE_OPENAI_API_KEY for zero-arg construction.
    OpenAiCompatProvider::new(OpenAiCompatConfig {
        base_url,
        api_key: env_key("AZURE_OPENAI_API_KEY"),
        model: deployment,
        model_info: None,
    })
}

/// Create a provider for any custom OpenAI-compatible endpoint.
///
/// # Example
///
/// ```rust,no_run
/// use traitclaw_openai::custom;
///
/// let provider = custom("http://localhost:8000/v1", "my-model", "");
/// ```
#[must_use]
pub fn custom(
    base_url: impl Into<String>,
    model: impl Into<String>,
    api_key: impl Into<String>,
) -> OpenAiCompatProvider {
    OpenAiCompatProvider::new(OpenAiCompatConfig {
        base_url: base_url.into(),
        api_key: api_key.into(),
        model: model.into(),
        model_info: None,
    })
}
