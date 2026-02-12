//! Capsa resolution utility for commands

use std::io;
use std::fs;
use emx_note::util;

/// Resolve a capsa by name, or use the default.
/// Auto-creates the default capsa if it doesn't exist.
pub fn resolve_capsa(ctx: &emx_note::ResolveContext, caps: Option<&str>) -> io::Result<emx_note::CapsaRef> {
    let default = ctx.default_capsa_name();
    let capsa_name = caps.unwrap_or(&default);

    // Try to resolve the capsa
    if let Some(capsa_ref) = ctx.resolve_capsa(capsa_name) {
        return Ok(capsa_ref);
    }

    // If not found and it's the default capsa, auto-create it
    if caps.is_none() {
        return create_default_capsa(ctx, &capsa_name);
    }

    Err(io::Error::new(io::ErrorKind::NotFound, format!("Capsa '{}' not found", capsa_name)))
}

/// Create the default capsa if it doesn't exist
fn create_default_capsa(ctx: &emx_note::ResolveContext, name: &str) -> io::Result<emx_note::CapsaRef> {
    // Get the prefixed name (with agent prefix if applicable)
    let prefixed_name = ctx.apply_agent_prefix(name);
    let capsa_path = ctx.home.join(&prefixed_name);

    // Create the directory and subdirectories
    fs::create_dir_all(&capsa_path)?;
    fs::create_dir_all(capsa_path.join("#daily"))?;

    eprintln!("Auto-created default capsa: {}", name);
    eprintln!("  Path: {}", util::display_path(&capsa_path));

    Ok(emx_note::CapsaRef {
        name: prefixed_name,
        path: capsa_path,
        is_link: false,
        is_default: name == emx_note::DEFAULT_CAPSA_NAME,
    })
}
