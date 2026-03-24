//! `WorkspaceBoundaryGuard` — restricts file operations to allowed directories.

use std::path::{Path, PathBuf};
use traitclaw_core::traits::guard::{Guard, GuardResult, GuardSeverity};
use traitclaw_core::types::action::Action;

/// A [`Guard`] that restricts file paths to configured allowed directories.
pub struct WorkspaceBoundaryGuard {
    allowed_dirs: Vec<PathBuf>,
}

impl WorkspaceBoundaryGuard {
    /// Create a guard that allows operations only within `allowed_dirs`.
    #[must_use]
    pub fn new(allowed_dirs: impl IntoIterator<Item = impl Into<PathBuf>>) -> Self {
        Self {
            allowed_dirs: allowed_dirs.into_iter().map(Into::into).collect(),
        }
    }

    /// Create a guard allowing only the current working directory.
    #[must_use]
    pub fn cwd() -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self::new([cwd])
    }

    fn is_path_allowed(&self, path: &Path) -> bool {
        if self.allowed_dirs.is_empty() {
            return true; // No restriction configured
        }
        let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        self.allowed_dirs.iter().any(|allowed| {
            let canonical_allowed = allowed.canonicalize().unwrap_or_else(|_| allowed.clone());
            canonical.starts_with(&canonical_allowed)
        })
    }
}

impl Guard for WorkspaceBoundaryGuard {
    fn name(&self) -> &'static str {
        "workspace_boundary"
    }

    fn check(&self, action: &Action) -> GuardResult {
        match action {
            Action::FileWrite { path, .. } => {
                if self.is_path_allowed(path) {
                    GuardResult::Allow
                } else {
                    GuardResult::Deny {
                        reason: format!(
                            "WorkspaceBoundaryGuard: path '{}' is outside allowed workspace",
                            path.display()
                        ),
                        severity: GuardSeverity::High,
                    }
                }
            }
            Action::ToolCall { arguments, name } => {
                // Heuristic: scan string values in arguments for path-like content
                if let Some(bad) = find_bad_path_in_value(arguments, &self.allowed_dirs) {
                    GuardResult::Deny {
                        reason: format!(
                            "WorkspaceBoundaryGuard: tool '{name}' arguments contain path '{bad}' \
                             outside allowed workspace"
                        ),
                        severity: GuardSeverity::High,
                    }
                } else {
                    GuardResult::Allow
                }
            }
            _ => GuardResult::Allow,
        }
    }
}

fn find_bad_path_in_value(value: &serde_json::Value, allowed: &[PathBuf]) -> Option<String> {
    match value {
        serde_json::Value::String(s) => {
            if (s.contains('/') || s.contains('\\')) && !s.starts_with("http") {
                let path = Path::new(s);
                let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
                let blocked = !allowed.is_empty()
                    && !allowed.iter().any(|a| {
                        let ca = a.canonicalize().unwrap_or_else(|_| a.clone());
                        canonical.starts_with(&ca)
                    });
                if blocked {
                    return Some(s.clone());
                }
            }
            None
        }
        serde_json::Value::Object(map) => map
            .values()
            .find_map(|v| find_bad_path_in_value(v, allowed)),
        serde_json::Value::Array(arr) => {
            arr.iter().find_map(|v| find_bad_path_in_value(v, allowed))
        }
        _ => None,
    }
}
