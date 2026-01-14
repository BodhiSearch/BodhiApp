# Auth & Scopes - Tools Feature

> Layer: `objs`, `auth_middleware` crates | Status: Planning

## Scope Model

Tool scopes are **discrete permissions** (not hierarchical like TokenScope/UserScope).

**Key Decision**: OAuth scope check only for external OAuth tokens. First-party tokens (session, `bodhiapp_`) bypass scope check if user has tool configured.

## ToolScope Enum

```rust
// crates/objs/src/tool_scope.rs
use serde::{Deserialize, Serialize};
use strum::{EnumIter, EnumString};

#[derive(Debug, Clone, Serialize, Deserialize, EnumString, strum::Display, PartialEq, Eq, Hash, EnumIter)]
pub enum ToolScope {
    #[strum(serialize = "scope_tools-builtin-exa-web-search")]
    #[serde(rename = "scope_tools-builtin-exa-web-search")]
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

```rust
// crates/auth_middleware/src/tool_auth_middleware.rs

/// Middleware for tool execution endpoints.
///
/// Authorization rules:
/// 1. First-party (session, bodhiapp_): Check tool is configured for user
/// 2. OAuth tokens: Check tool scope is present in token
pub async fn tool_auth_middleware(
    State(state): State<Arc<dyn RouterState>>,
    Path(tool_id): Path<String>,
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let headers = request.headers();

    // Determine auth type from headers
    let has_role_header = headers.contains_key(KEY_HEADER_BODHIAPP_ROLE);
    let scope_header = headers.get(KEY_HEADER_BODHIAPP_SCOPE)
        .and_then(|v| v.to_str().ok());

    let user_id = extract_user_id_from_headers(headers)?;

    if has_role_header {
        // Session-based auth (first-party)
        // Check if tool is configured for user
        let is_available = state
            .app_service()
            .tool_service()
            .is_tool_available_for_user(&user_id, &tool_id)
            .await
            .map_err(|e| ApiError::from(e))?;

        if !is_available {
            return Err(ToolError::ToolNotConfigured.into());
        }
    } else if let Some(scope) = scope_header {
        // Bearer token auth
        let resource_scope = ResourceScope::try_parse(scope);

        match resource_scope {
            Some(ResourceScope::Token(_)) => {
                // First-party API token (bodhiapp_)
                // Check if tool is configured for user
                let is_available = state
                    .app_service()
                    .tool_service()
                    .is_tool_available_for_user(&user_id, &tool_id)
                    .await?;

                if !is_available {
                    return Err(ToolError::ToolNotConfigured.into());
                }
            }
            Some(ResourceScope::User(_)) => {
                // External OAuth token - check tool scope
                let required_scope = ToolScope::scope_for_tool_id(&tool_id)
                    .ok_or(ToolError::ToolNotFound(tool_id.clone()))?;

                let token_scopes = ToolScope::from_scope_string(scope);

                if !token_scopes.contains(&required_scope) {
                    return Err(ApiError::forbidden(format!(
                        "Missing required scope: {}",
                        required_scope
                    )));
                }

                // Also verify tool is configured for user
                let is_available = state
                    .app_service()
                    .tool_service()
                    .is_tool_available_for_user(&user_id, &tool_id)
                    .await?;

                if !is_available {
                    return Err(ToolError::ToolNotConfigured.into());
                }
            }
            None => {
                return Err(ApiError::unauthorized("Invalid scope"));
            }
        }
    } else {
        return Err(ApiError::unauthorized("Missing authentication"));
    }

    Ok(next.run(request).await)
}
```

## Token Types Summary

| Token Type | Example | Scope Check | Tool Config Check |
|------------|---------|-------------|-------------------|
| Session | HTTP cookie | No | Yes |
| First-party API | `bodhiapp_xxx` | No | Yes |
| External OAuth | JWT from Keycloak | Yes (`scope_tools-*`) | Yes |

## Keycloak Configuration (Out of Scope)

Manual steps for admin:
1. Create client scope: `scope_tools-builtin-exa-web-search`
2. Set consent required: Yes
3. Add to client's optional scopes
