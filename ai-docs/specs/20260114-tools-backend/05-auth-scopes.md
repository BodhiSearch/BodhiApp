# Auth & Scopes - Tools Feature

> Layer: `objs`, `auth_middleware` crates | Status: ✅ Complete (7 tests passing) | Updated in Phase 7.6

## Implementation Note

> **Phase 7.6 Update**: OAuth-specific tool scope validation is now implemented. See [05.6-external-app-tool-access.md](./05.6-external-app-tool-access.md) for details.

The `tool_auth_middleware.rs` now handles different authorization flows:
- **Session/First-party**: Two-tier (app-level + user config)
- **External OAuth**: Four-tier (app-level + app-client + scope + user)

## Scope Model

Tool scopes are **discrete permissions** (not hierarchical like TokenScope/UserScope).

**Session/First-party**: Validated by checking if tool is configured (enabled + has API key) for the user.

**External OAuth (Phase 7.6)**: Additionally validates:
1. App-client is registered for the tool (via cached `/resources/request-access` response)
2. Token contains the required `scope_tool-*` claim

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

## Token Types Summary

| Token Type | Example | App Check | App-Client Check | Scope Check | User Config Check |
|------------|---------|-----------|------------------|-------------|-------------------|
| Session | HTTP cookie | Yes | No | No | Yes |
| First-party API | `bodhiapp_xxx` | Yes | No | No | Yes |
| External OAuth | JWT from Keycloak | Yes | Yes | Yes | Yes |

**Phase 7.6**: OAuth scope checking is now implemented via:
- `X-BodhiApp-Tool-Scopes` header (space-separated tool scopes from token)
- `X-BodhiApp-Azp` header (authorized party / app-client ID)

## Keycloak Configuration

Tool scopes are configured on app-clients via developer portal:
1. Client scope `scope_tool-builtin-exa-web-search` exists in realm
2. App-client has `bodhi.tools` attribute listing allowed tools
3. App-client has tool scope in optional scopes

See [09-keycloak-extension-contract.md](./09-keycloak-extension-contract.md) for full Keycloak integration details.

## Related Documents

- [05.6-external-app-tool-access.md](./05.6-external-app-tool-access.md) - Full OAuth tool authorization flow
- [09-keycloak-extension-contract.md](./09-keycloak-extension-contract.md) - Keycloak extension API contract
