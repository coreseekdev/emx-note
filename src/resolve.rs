//! Capsa path resolution module
//!
//! This module handles the complex logic of resolving capsa (vault) paths
//! with support for:
//! - Environment variable overrides (EMX_NOTE_HOME, EMX_NOTE_DEFAULT, EMX_AGENT_NAME)
//! - Default capsa (.default directory)
//! - Agent prefixing
//! - Link files to external directories
//! - Global vs agent-scoped operations

use std::path::{Path, PathBuf};

/// System constant for the default capsa name
/// Must not be user-creatable (starts with dot)
pub const DEFAULT_CAPSA_NAME: &str = ".default";

/// Environment variable names
pub const ENV_NOTE_HOME: &str = "EMX_NOTE_HOME";
pub const ENV_NOTE_DEFAULT: &str = "EMX_NOTE_DEFAULT";
pub const ENV_AGENT_NAME: &str = "EMX_AGENT_NAME";

/// Link file section and key
pub const LINK_SECTION: &str = "link";
pub const LINK_TARGET_KEY: &str = "target";

/// Resolution context for capsa operations
#[derive(Debug, Clone)]
pub struct ResolveContext {
    /// The base notes directory (EMX_NOTE_HOME)
    pub home: PathBuf,
    /// Whether this is a global operation (bypasses agent prefixing)
    pub global: bool,
    /// The agent name from environment (if set)
    pub agent_name: Option<String>,
    /// The explicitly specified default capsa name (EMX_NOTE_DEFAULT)
    pub default_override: Option<String>,
    /// Whether to output in JSON format
    pub json: bool,
}

impl ResolveContext {
    /// Create a new resolve context
    pub fn new(home: PathBuf, global: bool, json: bool) -> Self {
        // Treat empty strings as None
        let agent_name = std::env::var(ENV_AGENT_NAME)
            .ok()
            .filter(|s| !s.is_empty());
        let default_override = std::env::var(ENV_NOTE_DEFAULT)
            .ok()
            .filter(|s| !s.is_empty());

        Self {
            home,
            global,
            agent_name,
            default_override,
            json,
        }
    }

    /// Get the default capsa name based on resolution priority
    pub fn default_capsa_name(&self) -> String {
        // Priority 1: EMX_NOTE_DEFAULT environment variable
        if let Some(ref name) = self.default_override {
            return name.clone();
        }

        // Priority 2: EMX_AGENT_NAME (as capsa name)
        if let Some(ref agent) = self.agent_name {
            if !self.global {
                return agent.clone();
            }
        }

        // Priority 3: hardcoded .default
        DEFAULT_CAPSA_NAME.to_string()
    }

    /// Apply agent prefix to a capsa name if needed
    pub fn apply_agent_prefix(&self, name: &str) -> String {
        // If global operation or no agent, no prefix
        if self.global || self.agent_name.is_none() {
            return name.to_string();
        }

        // Special case: "default" maps directly to agent name
        // (agent's personal default capsa, not "agent-default")
        if name == DEFAULT_CAPSA_NAME {
            return self.agent_name.as_ref().unwrap().to_string();
        }

        // Apply agent prefix to other names
        format!("{}-{}", self.agent_name.as_ref().unwrap(), name)
    }

    /// Resolve a capsa name to its actual path
    /// Returns None if the capsa doesn't exist
    pub fn resolve_capsa(&self, name: &str) -> Option<CapsaRef> {
        let prefixed_name = self.apply_agent_prefix(name);
        let capsas_path = &self.home;

        // Check if it's a link file
        let link_path = capsas_path.join(&prefixed_name);
        if link_path.is_file() {
            return self.resolve_link(&link_path, &prefixed_name);
        }

        // Check if it's a directory
        let dir_path = link_path;
        if dir_path.is_dir() {
            return Some(CapsaRef {
                name: prefixed_name.clone(),
                path: dir_path.clone(),
                is_link: false,
                is_default: name == DEFAULT_CAPSA_NAME,
            });
        }

        None
    }

    /// Resolve the default capsa
    pub fn resolve_default(&self) -> Option<CapsaRef> {
        let name = self.default_capsa_name();
        self.resolve_capsa(&name)
    }

    /// Resolve a link file to its target
    fn resolve_link(&self, link_path: &Path, name: &str) -> Option<CapsaRef> {
        // Read link file content
        let content = std::fs::read_to_string(link_path).ok()?;
        let target = parse_link_content(&content)?;

        // Validate the link target is a valid directory
        let validated = crate::util::validate_link_target(&target, &self.home).ok()?;

        Some(CapsaRef {
            name: name.to_string(),
            path: validated,
            is_link: true,
            is_default: name == DEFAULT_CAPSA_NAME,
        })
    }

    /// List all available capsas (excluding system .default if it's a directory)
    pub fn list_capsas(&self) -> Vec<String> {
        let mut capsas = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&self.home) {
            for entry in entries.filter_map(|e| e.ok()) {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();

                // Skip hidden files that aren't .default
                if name_str.starts_with('.') && name_str != DEFAULT_CAPSA_NAME {
                    continue;
                }

                // Include both directories and link files
                if entry.path().is_dir() || entry.path().is_file() {
                    capsas.push(name_str.into_owned());
                }
            }
        }

        capsas.sort();
        capsas.dedup();
        capsas
    }
}

/// Reference to a resolved capsa
#[derive(Debug, Clone)]
pub struct CapsaRef {
    /// The (possibly prefixed) name of the capsa
    pub name: String,
    /// The actual path to the capsa (may be external if link)
    pub path: PathBuf,
    /// Whether this is a link to an external directory
    pub is_link: bool,
    /// Whether this is the default capsa
    pub is_default: bool,
}

/// Parse link file content and extract target path
/// Format: INI-style [link]\ntarget = /path/to/target
fn parse_link_content(content: &str) -> Option<PathBuf> {
    let mut target = None;

    for line in content.lines() {
        let line = line.trim();

        // Parse [section]
        if line.starts_with('[') && line.ends_with(']') {
            continue;
        }

        // Parse key = value
        if let Some(eq_pos) = line.find('=') {
            let key = line[..eq_pos].trim();
            let value = line[eq_pos + 1..].trim();

            if key == LINK_TARGET_KEY {
                target = Some(PathBuf::from(value));
                break;
            }
        }
    }

    target
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_capsa_name_priority() {
        // Priority 1: EMX_NOTE_DEFAULT
        std::env::set_var(ENV_NOTE_DEFAULT, "explicit-default");
        std::env::remove_var(ENV_AGENT_NAME);
        let ctx = ResolveContext::new("/tmp".into(), false, false);
        assert_eq!(ctx.default_capsa_name(), "explicit-default");

        // Priority 2: EMX_AGENT_NAME
        std::env::remove_var(ENV_NOTE_DEFAULT);
        std::env::set_var(ENV_AGENT_NAME, "my-agent");
        let ctx = ResolveContext::new("/tmp".into(), false, false);
        assert_eq!(ctx.default_capsa_name(), "my-agent");

        // Priority 3: .default
        std::env::remove_var(ENV_NOTE_DEFAULT);
        std::env::remove_var(ENV_AGENT_NAME);
        let ctx = ResolveContext::new("/tmp".into(), false, false);
        assert_eq!(ctx.default_capsa_name(), DEFAULT_CAPSA_NAME);
    }

    fn test_context(home: &str) -> ResolveContext {
        ResolveContext {
            home: PathBuf::from(home),
            global: false,
            agent_name: None,
            default_override: None,
            json: false,
        }
    }

    #[test]
    fn test_agent_prefix() {
        // With agent, non-global
        let mut ctx = test_context("/tmp");
        ctx.agent_name = Some("agent1".to_string());
        assert_eq!(ctx.apply_agent_prefix("my-notes"), "agent1-my-notes");

        // Without agent
        let ctx = test_context("/tmp");
        assert_eq!(ctx.apply_agent_prefix("my-notes"), "my-notes");

        // Global bypasses agent prefix
        let ctx = ResolveContext {
            home: "/tmp".into(),
            global: true,
            agent_name: Some("agent1".to_string()),
            default_override: None,
            json: false,
        };
        assert_eq!(ctx.apply_agent_prefix("my-notes"), "my-notes");

        // Special case: "default" maps to agent name
        let mut ctx = test_context("/tmp");
        ctx.agent_name = Some("agent1".to_string());
        ctx.json = false;
        assert_eq!(ctx.apply_agent_prefix(".default"), "agent1");
    }

    #[test]
    fn test_link_parsing() {
        let content = r#"[link]
target = /absolute/path/to/vault"#;
        let target = parse_link_content(content);
        assert_eq!(target, Some(PathBuf::from("/absolute/path/to/vault")));

        // Invalid format
        let content = "invalid content";
        let target = parse_link_content(content);
        assert_eq!(target, None);
    }
}
