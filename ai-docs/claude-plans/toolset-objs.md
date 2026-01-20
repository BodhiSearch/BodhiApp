# Toolset Multi-Instance: Domain Objects Layer (objs)

## Context Summary

This layer defines the domain types for toolset instances. The key shift: from single `toolset_id` identifying both type and config, to `instance_id` (UUID) + `toolset_type` + `instance_name`.

## Current State Reference

### Existing Types (`crates/objs/src/toolsets.rs`)

| Type | Purpose | Keep/Modify |
|------|---------|-------------|
| `ToolsetDefinition` | Defines a toolset type (id, name, tools) | Keep - represents TYPE |
| `ToolDefinition` | Single tool definition | Keep |
| `ToolsetScope` | OAuth scope parsing | Keep - scope is type-level |
| `ToolsetWithTools` | Type + tools for listing | Keep for admin type listing |

### Existing Error Types (`crates/objs/src/errors.rs`)

`ToolsetError` enum - will need new variants for instance operations.

## New Domain Types

### Toolset

Represents a user's configured instance of a toolset type:

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// A user's configured toolset
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct Toolset {
    /// Unique toolset identifier (UUID)
    pub id: String,

    /// User-defined toolset name (alphanumeric + hyphens, unique per user)
    pub name: String,

    /// The toolset type this is an instance of (e.g., "builtin-exa-web-search")
    pub toolset_type: String,

    /// Optional user description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Whether user has enabled this instance
    pub enabled: bool,

    /// Whether API key is configured (never expose actual key)
    pub has_api_key: bool,

    /// Creation timestamp
    #[schema(value_type = String, format = "date-time")]
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    #[schema(value_type = String, format = "date-time")]
    pub updated_at: DateTime<Utc>,
}
```

### ToolsetWithTools

For API responses including tool definitions:

```rust
/// Toolset instance with full context for API response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct ToolsetWithTools {
    /// Instance details
    #[serde(flatten)]
    pub instance: Toolset,

    /// Whether the toolset TYPE is enabled at app level
    pub app_enabled: bool,

    /// Tools provided by this toolset type
    pub tools: Vec<ToolDefinition>,
}
```

### New Error Variants

Add to `ToolsetError` in `crates/objs/src/errors.rs`:

```rust
// Add these variants to existing ToolsetError enum

#[error("instance_not_found")]
#[error_meta(error_type = ErrorType::NotFound, status = 404)]
ToolsetNotFound(String),

#[error("instance_name_exists")]
#[error_meta(error_type = ErrorType::Conflict, status = 409)]
NameExists(String),

#[error("invalid_instance_name")]
#[error_meta(error_type = ErrorType::BadRequest, status = 400)]
InvalidName(String),

#[error("instance_not_owned")]
#[error_meta(error_type = ErrorType::Forbidden, status = 403)]
NotOwned,
```

## Instance Name Validation

Add validation function or implement as builder pattern:

```rust
use once_cell::sync::Lazy;
use regex::Regex;

/// Regex for valid toolset names: alphanumeric and hyphens only
static TOOLSET_NAME_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z0-9-]+$").unwrap()
});

/// Maximum toolset name length
const MAX_TOOLSET_NAME_LEN: usize = 64;

/// Validate toolset name format
pub fn validate_toolset_name(name: &str) -> Result<(), ToolsetError> {
    if name.is_empty() {
        return Err(ToolsetError::InvalidName(
            "name cannot be empty".to_string()
        ));
    }
    if name.len() > MAX_TOOLSET_NAME_LEN {
        return Err(ToolsetError::InvalidName(
            format!("name exceeds {} characters", MAX_TOOLSET_NAME_LEN)
        ));
    }
    if !TOOLSET_NAME_REGEX.is_match(name) {
        return Err(ToolsetError::InvalidName(
            "name must contain only alphanumeric characters and hyphens".to_string()
        ));
    }
    Ok(())
}
```

## Tool Name Encoding

For LLM tool names, encode toolset name:

```rust
/// Encode toolset name and method into tool name for LLM
/// Format: toolset_{instance_name}__{method}
pub fn encode_tool_name(instance_name: &str, method: &str) -> String {
    format!("toolset_{}__{}", instance_name, method)
}

/// Parse tool name back to (instance_name, method)
/// Returns None if format doesn't match
pub fn parse_tool_name(tool_name: &str) -> Option<(String, String)> {
    let stripped = tool_name.strip_prefix("toolset_")?;
    let parts: Vec<&str> = stripped.splitn(2, "__").collect();
    if parts.len() == 2 {
        Some((parts[0].to_string(), parts[1].to_string()))
    } else {
        None
    }
}
```

## Files to Modify

| File | Changes |
|------|---------|
| `crates/objs/src/toolsets.rs` | Add `Toolset`, `ToolsetWithTools`, validation, encoding functions |
| `crates/objs/src/errors.rs` | Add new error variants to `ToolsetError` |
| `crates/objs/src/lib.rs` | Export new types |

## Dependencies

No new crate dependencies needed. Uses existing:
- `chrono` for timestamps
- `serde` for serialization
- `utoipa` for OpenAPI schema
- `regex` (already in workspace)
- `once_cell` (already in workspace)

## Test Considerations

Unit tests for:
- `validate_toolset_name` - valid/invalid names, edge cases
- `encode_tool_name` / `parse_tool_name` - roundtrip encoding
- Serialization/deserialization of new types
