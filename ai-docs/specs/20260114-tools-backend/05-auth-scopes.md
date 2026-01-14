# Auth & Scopes - Tools Feature

> Layer: `objs`, `auth_middleware` crates | Status: ✅ Complete (7 tests passing)

## Implementation Note

The current implementation (`tool_auth_middleware.rs`) checks tool configuration for all auth types (session, first-party tokens, OAuth tokens). OAuth-specific tool scope validation is simplified and will be enhanced in a future iteration when `auth_middleware` is extended to preserve full JWT scope strings instead of just the ResourceScope enum.

## Scope Model

Tool scopes are **discrete permissions** (not hierarchical like TokenScope/UserScope).

**Current Implementation**: All auth types (session, first-party tokens, OAuth tokens) are validated by checking if the tool is configured (enabled + has API key) for the user.

**Future Enhancement**: OAuth-specific tool scope validation will be added when auth_middleware is enhanced to preserve full JWT scope strings.

## ToolScope Enum

```rust
// crates/objs/src/tool_scope.rs
use serde::{Deserialize, Serialize};
use strum::{EnumIter, EnumString};

#[derive(Debug, Clone, Serialize, Deserialize, EnumString, strum::Display, PartialEq, Eq, Hash, EnumIter)]
pub enum ToolScope {
    #[strum(serialize = "scope_tool-builtin-exa-web-search")]
    #[serde(rename = "scope_tool-builtin-exa-web-search")]
    BuiltinExaWebSearch,
}

impl ToolScope {
    /// Extract tool scopes from space-separated scope string
    pub fn from_scope_string(scope: &str) -> Vec<Self> {
        scope
            .split_whitespace()
            .filter_map(|s| s.parse::<ToolScope>().ok())
            .collect()
    }

    /// Get corresponding tool_id
    pub fn tool_id(&self) -> &'static str {
        match self {
            Self::BuiltinExaWebSearch => "builtin-exa-web-search",
        }
    }

    /// Get scope string for tool_id
    pub fn scope_for_tool_id(tool_id: &str) -> Option<Self> {
        match tool_id {
            "builtin-exa-web-search" => Some(Self::BuiltinExaWebSearch),
            _ => None,
        }
    }
}
```

## Authorization Middleware for Tool Execution

**File**: `crates/auth_middleware/src/tool_auth_middleware.rs` (310 lines, 7 tests passing)

```rust
/// Middleware for tool execution endpoints
///
/// Authorization rules:
/// - Check that user has the tool configured (enabled + API key set)
/// - For all auth types: session, first-party tokens, and OAuth tokens
///
/// Note: OAuth-specific tool scope validation is deferred to future enhancement
/// when auth_middleware is extended to preserve full JWT scope strings.
pub async fn tool_auth_middleware(
    State(state): State<Arc<dyn RouterState>>,
    Path(tool_id): Path<String>,
    req: Request,
    next: Next,
) -> Result<Response, ApiError>

async fn _impl(
    State(state): State<Arc<dyn RouterState>>,
    Path(tool_id): Path<String>,
    req: Request,
    next: Next,
) -> Result<Response, ToolAuthError> {
    let headers = req.headers();

    // Extract user_id
    let user_id = headers
        .get(KEY_HEADER_BODHIAPP_USER_ID)
        .and_then(|v| v.to_str().ok())
        .ok_or(ToolAuthError::MissingUserId)?;

    // Verify authentication exists (either role or scope header)
    let has_auth = headers.contains_key(KEY_HEADER_BODHIAPP_ROLE)
        || headers.contains_key(KEY_HEADER_BODHIAPP_SCOPE);

    if !has_auth {
        return Err(ToolAuthError::MissingAuth);
    }

    // Check if tool is configured and available for user
    let is_available = state
        .app_service()
        .tool_service()
        .is_tool_available_for_user(user_id, &tool_id)
        .await?;

    if !is_available {
        return Err(ToolError::ToolNotConfigured.into());
    }

    Ok(next.run(req).await)
}
```

## Token Types Summary (Current Implementation)

| Token Type | Example | Scope Check | Tool Config Check |
|------------|---------|-------------|-------------------|
| Session | HTTP cookie | No | Yes |
| First-party API | `bodhiapp_xxx` | No | Yes |
| External OAuth | JWT from Keycloak | No (deferred) | Yes |

Note: OAuth scope checking will be added in future enhancement when auth_middleware preserves full JWT scope strings.

## Keycloak Configuration (Out of Scope)

Manual steps for admin:
1. Create client scope: `scope_tool-builtin-exa-web-search`
2. Set consent required: Yes
3. Add to client's optional scopes
