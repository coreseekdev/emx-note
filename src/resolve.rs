//! Capsa path resolution module
//!
//! This module handles the complex logic of resolving capsa (vault) paths
//! with support for:
//! - Environment variable overrides (EMX_NOTE_HOME, EMX_NOTE_DEFAULT, EMX_AGENT_NAME)
//! - Hierarchical agent namespaces (agent/capsa)
//! - Global shared namespace (@shared/ for agent-less operations)
//! - Link files to external directories
//! - Global vs agent-scoped operations
//!
//! ## Design Philosophy
//!
//! **No agent = Global shared space**
//! - When `EMX_AGENT_NAME` is not set, operations target `@shared/` namespace
//! - This is the default mode for ad-hoc note-taking
//! - Shared across all agents and users
//!
//! **With agent = Private agent space**
//! - When `EMX_AGENT_NAME` is set, operations target `agent/` namespace
//! - Each agent has isolated private capsas
//! - Prevents naming conflicts between agents
//!
//! ## Directory Structure
//!
//! ```text
//! ~/.emx-notes/
//! ├── agent1/
//! │   ├── .          (agent1's private default capsa)
//! │   ├── work/
//! │   └── personal/
//! ├── agent2/
//! │   ├── .
//! │   └── projects/
//! └── @shared/
//!     ├── .          (default when no agent)
//!     ├── common/
//!     └── docs/
//! ```
//!
//! ## Why No Root-Level Capsas?
//!
//! Direct operations on `~/` (the home directory itself) don't make sense because:
//! - Without clear ownership, root-level capsas become ambiguous
//! - `@shared/` provides the same global accessibility with clear intent
//! - Prevents accidental creation of "orphan" capsas outside any namespace

use std::path::{Path, PathBuf};

/// System constant for the default capsa name
/// Must not be user-creatable (starts with dot)
pub const DEFAULT_CAPSA_NAME: &str = ".default";

/// Marker for global shared namespace (cross-agent shared capsas)
pub const GLOBAL_NAMESPACE_MARKER: &str = "@";

/// Name of the global shared namespace
pub const SHARED_NAMESPACE: &str = "shared";

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
    /// Whether this is a global operation (bypasses agent namespace)
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
    ///
    /// Returns hierarchical format:
    /// - With agent: `"agent/."` (agent's private space)
    /// - Without agent: `"@shared/."` (global shared space)
    pub fn default_capsa_name(&self) -> String {
        // Priority 1: EMX_NOTE_DEFAULT environment variable
        if let Some(ref name) = self.default_override {
            return name.clone();
        }

        // Priority 2: EMX_AGENT_NAME (agent's private namespace)
        if let Some(ref agent) = self.agent_name {
            if !self.global {
                // Agent's default capsa is at "agent/."
                return format!("{}/.", agent);
            }
        }

        // Priority 3: Global shared namespace (@shared/.)
        // When no agent is set, operations target the shared space
        format!("{}/.", GLOBAL_NAMESPACE_MARKER.to_string() + SHARED_NAMESPACE)
    }

    /// Apply agent namespace to a capsa name
    /// Returns hierarchical format like "agent/name" or "agent/."
    pub fn apply_agent_namespace(&self, name: &str) -> String {
        // If global operation or no agent, no namespace
        if self.global || self.agent_name.is_none() {
            return name.to_string();
        }

        // Check if name already includes global marker (@shared/...)
        if name.starts_with(GLOBAL_NAMESPACE_MARKER) {
            return name.to_string();
        }

        // Check if name already has a slash (absolute path reference)
        if name.contains('/') {
            return name.to_string();
        }

        let agent = self.agent_name.as_ref().unwrap();

        // Special case: ".default" maps to "agent/."
        if name == DEFAULT_CAPSA_NAME {
            return format!("{}/.", agent);
        }

        // Apply agent namespace: "agent/name"
        format!("{}/{}", agent, name)
    }

    /// Resolve a capsa name to its actual path
    /// Returns None if the capsa doesn't exist
    ///
    /// Resolution order:
    /// 1. Global namespace marker (@name) - treated as @shared/name or just name
    /// 2. Agent namespace (agent/name) - agent's private capsas
    /// 3. Root namespace (name) - resolves directly (typically for @shared/)
    pub fn resolve_capsa(&self, name: &str) -> Option<CapsaRef> {
        // Priority 1: Global namespace marker (@shared/notes)
        if name.starts_with(GLOBAL_NAMESPACE_MARKER) {
            let global_name = &name[1..]; // Strip @ marker
            return self.try_resolve_capsa(global_name, name);
        }

        // Priority 2: Apply agent namespace (if not global mode)
        if !self.global {
            if let Some(ref agent) = self.agent_name {
                // Special case: agent's default (agent/.)
                let capsa_name = if name == DEFAULT_CAPSA_NAME {
                    "."
                } else {
                    name
                };
                let hierarchical = format!("{}/{}", agent, capsa_name);

                if let Some(capsa) = self.try_resolve_capsa(&hierarchical, name) {
                    return Some(capsa);
                }

                // Priority 3: Fallback to root namespace for shared capsas
                // This allows agents to access globally created capsas
                if let Some(capsa) = self.try_resolve_capsa(name, name) {
                    return Some(capsa);
                }

                return None;
            }
        }

        // Priority 4: No agent or global mode - resolve directly in root
        self.try_resolve_capsa(name, name)
    }

    /// Helper to try resolving a specific capsa name
    fn try_resolve_capsa(&self, resolved_name: &str, original_name: &str) -> Option<CapsaRef> {
        let capsas_path = &self.home;
        let full_path = capsas_path.join(resolved_name);

        // Check if it's a link file
        if full_path.is_file() {
            return self.resolve_link(&full_path, resolved_name);
        }

        // Check if it's a directory
        if full_path.is_dir() {
            return Some(CapsaRef {
                name: resolved_name.to_string(),
                path: full_path.clone(),
                is_link: false,
                is_default: original_name == DEFAULT_CAPSA_NAME || resolved_name.ends_with("/."),
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
            is_default: name == DEFAULT_CAPSA_NAME || name.ends_with("/."),
        })
    }

    /// List all available capsas
    /// Returns flat list of paths like "agent1/.", "agent1/work", "@shared/."
    pub fn list_capsas(&self) -> Vec<String> {
        let mut capsas = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&self.home) {
            for entry in entries.filter_map(|e| e.ok()) {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();

                // Skip hidden files that aren't special
                if name_str.starts_with('.') && name_str != DEFAULT_CAPSA_NAME && name_str != "." {
                    continue;
                }

                let path = entry.path();

                // It's a directory - list its contents (for agent namespaces)
                if path.is_dir() {
                    // List contents of subdirectories
                    if let Ok(sub_entries) = std::fs::read_dir(&path) {
                        for sub_entry in sub_entries.filter_map(|e| e.ok()) {
                            let sub_name = sub_entry.file_name();
                            let sub_name_str = sub_name.to_string_lossy();

                            // Skip hidden files in subdirs
                            if sub_name_str.starts_with('.') && sub_name_str != "." {
                                continue;
                            }

                            let sub_path = sub_entry.path();
                            if sub_path.is_dir() || sub_path.is_file() {
                                capsas.push(format!("{}/{}", name_str, sub_name_str));
                            }
                        }
                    }
                } else if path.is_file() {
                    // It's a link file
                    capsas.push(name_str.into_owned());
                }
            }
        }

        capsas.sort();
        capsas.dedup();
        capsas
    }

    /// Check if a path is in hierarchical format (contains '/')
    pub fn is_hierarchical(name: &str) -> bool {
        name.contains('/')
    }

    /// Extract agent name from hierarchical path
    /// Returns None if not a hierarchical path or if path starts with '@'
    pub fn extract_agent(name: &str) -> Option<&str> {
        if name.starts_with(GLOBAL_NAMESPACE_MARKER) {
            return None;
        }
        // Only return agent if it's a hierarchical path (contains '/')
        if !name.contains('/') {
            return None;
        }
        name.split('/').next()
    }
}

/// Reference to a resolved capsa
#[derive(Debug, Clone)]
pub struct CapsaRef {
    /// The (possibly hierarchical) name of the capsa
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

        // Priority 2: EMX_AGENT_NAME (returns "agent/.")
        std::env::remove_var(ENV_NOTE_DEFAULT);
        std::env::set_var(ENV_AGENT_NAME, "my-agent");
        let ctx = ResolveContext::new("/tmp".into(), false, false);
        assert_eq!(ctx.default_capsa_name(), "my-agent/.");

        // Priority 3: @shared/. (when no agent)
        std::env::remove_var(ENV_NOTE_DEFAULT);
        std::env::remove_var(ENV_AGENT_NAME);
        let ctx = ResolveContext::new("/tmp".into(), false, false);
        assert_eq!(ctx.default_capsa_name(), "@shared/.");
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
    fn test_agent_namespace() {
        // With agent, non-global
        let mut ctx = test_context("/tmp");
        ctx.agent_name = Some("agent1".to_string());
        assert_eq!(ctx.apply_agent_namespace("my-notes"), "agent1/my-notes");

        // Default capsa
        assert_eq!(ctx.apply_agent_namespace(".default"), "agent1/.");

        // Without agent
        let ctx = test_context("/tmp");
        assert_eq!(ctx.apply_agent_namespace("my-notes"), "my-notes");

        // Global bypasses agent namespace
        let ctx = ResolveContext {
            home: "/tmp".into(),
            global: true,
            agent_name: Some("agent1".to_string()),
            default_override: None,
            json: false,
        };
        assert_eq!(ctx.apply_agent_namespace("my-notes"), "my-notes");

        // Global namespace marker preserved
        let mut ctx = test_context("/tmp");
        ctx.agent_name = Some("agent1".to_string());
        assert_eq!(ctx.apply_agent_namespace("@shared/notes"), "@shared/notes");
    }

    #[test]
    fn test_helpers() {
        // Test is_hierarchical
        assert!(ResolveContext::is_hierarchical("agent1/work"));
        assert!(ResolveContext::is_hierarchical("@shared/notes"));
        assert!(!ResolveContext::is_hierarchical("work"));

        // Test extract_agent
        assert_eq!(ResolveContext::extract_agent("agent1/work"), Some("agent1"));
        assert_eq!(ResolveContext::extract_agent("@shared/notes"), None);
        assert_eq!(ResolveContext::extract_agent("work"), None);
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
