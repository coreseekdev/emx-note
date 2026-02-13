//! Metadata (frontmatter) management command module

use std::fs;
use std::io;

pub fn run(
    ctx: &emx_note::ResolveContext,
    caps: Option<&str>,
    note_ref: String,
    key: Option<String>,
    value: Vec<String>,
    delete: bool,
) -> io::Result<()> {
    let capsa_ref = super::resolve::resolve_capsa(ctx, caps)?;

    // Resolve note using helper function
    let note_path = emx_note::resolve_note_or_error(
        &capsa_ref.path,
        &note_ref,
        emx_note::DEFAULT_EXTENSIONS
    )?;

    // Read note content
    let content = fs::read_to_string(&note_path)?;

    // Parse or modify frontmatter
    if delete {
        if let Some(k) = key {
            let updated = delete_key(&content, &k)?;
            fs::write(&note_path, updated)?;
            eprintln!("Deleted key '{}'", k);
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Key required for --delete"
            ));
        }
    } else if let Some(k) = key {
        if value.is_empty() {
            // Get key value
            if let Some(v) = get_key(&content, &k) {
                println!("{}", v);
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("Key '{}' not found", k)
                ));
            }
        } else {
            // Set key value
            let yaml_value = if value.len() == 1 {
                serde_yaml::Value::String(value[0].clone())
            } else {
                serde_yaml::Value::Sequence(value.into_iter().map(serde_yaml::Value::String).collect())
            };
            let updated = set_key(&content, &k, yaml_value.clone())?;
            fs::write(&note_path, updated)?;

            // Output the value that was set (for confirmation)
            println!("{}", to_yaml_string(&yaml_value));
        }
    } else {
        // List all frontmatter
        let frontmatter = extract_frontmatter(&content);
        if frontmatter.is_empty() {
            println!("(no frontmatter)");
        } else {
            println!("{}", frontmatter);
        }
    }

    Ok(())
}

/// Extract YAML frontmatter from content
fn extract_frontmatter(content: &str) -> String {
    if !content.starts_with("---") {
        return String::new();
    }

    let rest = &content[3..];
    if let Some(end_pos) = rest.find("\n---") {
        let frontmatter = &rest[..end_pos];

        // Check frontmatter size before parsing
        if frontmatter.len() > emx_note::MAX_FRONTMATTER_SIZE {
            return String::new();
        }

        return frontmatter.to_string();
    }

    String::new()
}

/// Get a key value from frontmatter
fn get_key(content: &str, key: &str) -> Option<String> {
    let frontmatter = extract_frontmatter(content);
    if frontmatter.is_empty() {
        return None;
    }

    let yaml: serde_yaml::Value = serde_yaml::from_str(&frontmatter).ok()?;

    // First try to get the key as-is (handles keys with dots like "abc.efg")
    if let serde_yaml::Value::Mapping(ref map) = yaml {
        if let Some(value) = map.get(&serde_yaml::Value::String(key.to_string())) {
            return Some(to_yaml_string(value));
        }
    }

    // If not found, try nested path lookup (handles "abc.efg" as "abc"."efg")
    let parts: Vec<&str> = key.split('.').collect();
    if parts.len() > 1 {
        let mut current = &yaml;

        for part in parts {
            match current {
                serde_yaml::Value::Mapping(map) => {
                    current = map.get(&serde_yaml::Value::String(part.to_string()))?;
                }
                _ => return None,
            }
        }

        Some(to_yaml_string(current))
    } else {
        None
    }
}

/// Set a key value in frontmatter
fn set_key(content: &str, key: &str, value: serde_yaml::Value) -> io::Result<String> {
    let frontmatter = extract_frontmatter(content);

    let yaml: serde_yaml::Value = if frontmatter.is_empty() {
        serde_yaml::Value::Mapping(serde_yaml::mapping::Mapping::new())
    } else {
        serde_yaml::from_str(&frontmatter).map_err(|e| {
            io::Error::new(io::ErrorKind::InvalidData, format!("Invalid YAML: {}", e))
        })?
    };

    let updated = set_nested_value(yaml, key, value);
    let new_frontmatter = serde_yaml::to_string(&updated).unwrap_or_default();

    // Find body content: skip frontmatter end marker and any following newlines
    let body = if frontmatter.is_empty() {
        content
    } else if let Some(end_marker_pos) = content.find("\n---") {
        let after_marker = end_marker_pos + 4; // Skip "\n---"
        // Skip any newlines after the end marker
        content[after_marker..].trim_start_matches(|c| c == '\n' || c == '\r')
    } else {
        content
    };

    Ok(format!("---\n{}\n---\n{}", new_frontmatter.trim(), body))
}

/// Delete a key from frontmatter
fn delete_key(content: &str, key: &str) -> io::Result<String> {
    let frontmatter = extract_frontmatter(content);
    if frontmatter.is_empty() {
        return Ok(content.to_string());
    }

    let yaml: serde_yaml::Value = serde_yaml::from_str(&frontmatter).map_err(|e| {
        io::Error::new(io::ErrorKind::InvalidData, format!("Invalid YAML: {}", e))
    })?;

    let updated = delete_nested_value(yaml, key);

    // Find body content: skip frontmatter end marker and any following newlines
    let body = if let Some(end_marker_pos) = content.find("\n---") {
        let after_marker = end_marker_pos + 4;
        content[after_marker..].trim_start_matches(|c| c == '\n' || c == '\r')
    } else {
        content
    };

    if updated.is_null() || (updated.is_mapping() && updated.as_mapping().map(|m| m.is_empty()).unwrap_or(false)) {
        // Remove entire frontmatter if empty
        return Ok(body.to_string());
    }

    let new_frontmatter = serde_yaml::to_string(&updated).unwrap_or_default();

    Ok(format!("---\n{}\n---\n{}", new_frontmatter.trim(), body))
}

/// Set nested value in YAML
fn set_nested_value(yaml: serde_yaml::Value, key: &str, value: serde_yaml::Value) -> serde_yaml::Value {
    let parts: Vec<&str> = key.split('.').collect();

    if parts.len() == 1 {
        if let serde_yaml::Value::Mapping(mut map) = yaml {
            map.insert(
                serde_yaml::Value::String(parts[0].to_string()),
                value
            );
            return serde_yaml::Value::Mapping(map);
        }
        return yaml;
    }

    // Nested path - need to insert into a mapping
    let current_key = serde_yaml::Value::String(parts[0].to_string());
    let rest_key = parts[1..].join(".");

    let nested_value = if let serde_yaml::Value::Mapping(ref map) = yaml {
        if let Some(existing) = map.get(&current_key) {
            // If existing value is not a mapping, we need to replace it
            if !matches!(existing, serde_yaml::Value::Mapping(_)) {
                // Create fresh nested structure, discarding the non-mapping value
                let new_map = serde_yaml::mapping::Mapping::new();
                let empty = serde_yaml::Value::Mapping(new_map);
                set_nested_value(empty, &rest_key, value)
            } else {
                // Recurse into existing mapping
                set_nested_value(existing.clone(), &rest_key, value)
            }
        } else {
            // Create new nested structure
            let new_map = serde_yaml::mapping::Mapping::new();
            let empty = serde_yaml::Value::Mapping(new_map);
            set_nested_value(empty, &rest_key, value)
        }
    } else {
        // yaml is not a mapping, create new nested structure from scratch
        let new_map = serde_yaml::mapping::Mapping::new();
        let empty = serde_yaml::Value::Mapping(new_map);
        set_nested_value(empty, &rest_key, value)
    };

    // Insert into mapping, creating one if yaml isn't a mapping
    if let serde_yaml::Value::Mapping(mut map) = yaml {
        map.insert(current_key, nested_value);
        serde_yaml::Value::Mapping(map)
    } else {
        // Convert non-mapping yaml to a new mapping with the nested value
        let mut new_map = serde_yaml::mapping::Mapping::new();
        new_map.insert(current_key, nested_value);
        serde_yaml::Value::Mapping(new_map)
    }
}

/// Delete nested value from YAML
fn delete_nested_value(yaml: serde_yaml::Value, key: &str) -> serde_yaml::Value {
    let parts: Vec<&str> = key.split('.').collect();

    if parts.len() == 1 {
        if let serde_yaml::Value::Mapping(mut map) = yaml {
            map.remove(&serde_yaml::Value::String(parts[0].to_string()));
            return serde_yaml::Value::Mapping(map);
        }
        return yaml;
    }

    // Nested path - need to traverse and delete
    let current_key = serde_yaml::Value::String(parts[0].to_string());
    let rest_key = parts[1..].join(".");

    if let serde_yaml::Value::Mapping(mut map) = yaml {
        if let Some(existing) = map.get(&current_key) {
            let updated = delete_nested_value(existing.clone(), &rest_key);
            if updated.is_null() || (updated.is_mapping() && updated.as_mapping().map(|m| m.is_empty()).unwrap_or(false)) {
                map.remove(&current_key);
            } else {
                map.insert(current_key, updated);
            }
        }
        serde_yaml::Value::Mapping(map)
    } else {
        yaml
    }
}

/// Convert YAML value to string representation
fn to_yaml_string(value: &serde_yaml::Value) -> String {
    match value {
        serde_yaml::Value::String(s) => s.clone(),
        serde_yaml::Value::Number(n) => n.to_string(),
        serde_yaml::Value::Bool(b) => b.to_string(),
        serde_yaml::Value::Null => "null".to_string(),
        serde_yaml::Value::Sequence(seq) => {
            let items: Vec<String> = seq.iter().map(to_yaml_string).collect();
            format!("[{}]", items.join(", "))
        }
        serde_yaml::Value::Mapping(_) => {
            serde_yaml::to_string(value).unwrap_or_default()
        }
        serde_yaml::Value::Tagged(_) => "?".to_string(),
    }
}
