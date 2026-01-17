# Auth & Scopes - Toolsets Feature

> Layer: `objs`, `auth_middleware` crates | Status: ✅ Complete

## Scope Model

Toolset scopes are **discrete permissions** (not hierarchical like TokenScope/UserScope). One scope grants access to all tools within that toolset.

**Session/First-party**: Validated by checking if toolset is configured (enabled + has API key) for the user.

**External OAuth**: Additionally validates:
1. App-client is registered for the toolset (via cached `/resources/request-access` response)
2. Token contains the required `scope_toolset-*` claim

## ToolsetScope Enum

```rust
// crates/objs/src/toolsets.rs
use serde::{Deserialize, Serialize};
use strum::{EnumIter, EnumString};

#[derive(Debug, Clone, Serialize, Deserialize, EnumString, strum::Display, PartialEq, Eq, Hash, EnumIter)]
pub enum ToolsetScope {
    #[strum(serialize = "scope_toolset-builtin-exa-web-search")]
    #[serde(rename = "scope_toolset-builtin-exa-web-search")]
    BuiltinExaWebSearch,
}

impl ToolsetScope {
    /// Extract toolset scopes from space-separated scope string
    pub fn from_scope_string(scope: &str) -> Vec<Self> {
        scope
            .split_whitespace()
            .filter_map(|s| s.parse::<ToolsetScope>().ok())
            .collect()
    }

    /// Get corresponding toolset_id
    pub fn toolset_id(&self) -> &'static str {
        match self {
            Self::BuiltinExaWebSearch => "builtin-exa-web-search",
        }
    }

    /// Get scope for toolset_id
    pub fn scope_for_toolset_id(toolset_id: &str) -> Option<Self> {
        match toolset_id {
            "builtin-exa-web-search" => Some(Self::BuiltinExaWebSearch),
            _ => None,
        }
    }
}
```

## Authorization Middleware for Toolset Execution

**File**: `crates/auth_middleware/src/toolset_auth_middleware.rs`

```rust
/// Middleware for toolset execution endpoints
///
/// Authorization rules:
/// - Session/First-party: Check app-level + user config
/// - OAuth: Check app-level + app-client registration + scope + user config
pub async fn toolset_auth_middleware(
    State(state): State<Arc<dyn RouterState>>,
    Path(toolset_id): Path<String>,
    req: Request,
    next: Next,
) -> Result<Response, ApiError>

async fn _impl(
    State(state): State<Arc<dyn RouterState>>,
    Path(toolset_id): Path<String>,
    req: Request,
    next: Next,
) -> Result<Response, ToolsetAuthError> {
    let headers = req.headers();
    let user_id = extract_user_id(headers)?;
    
    // Determine auth type
    let is_session_auth = headers.contains_key(KEY_HEADER_BODHIAPP_ROLE);
    let scope_header = headers.get(KEY_HEADER_BODHIAPP_SCOPE).unwrap_or("");
    let is_first_party_token = scope_header.starts_with("scope_token_");
    let is_oauth_auth = scope_header.starts_with("scope_user_") && !is_session_auth;

    let toolset_service = state.app_service().toolset_service();

    // 1. Check app-level enabled (all auth types)
    if !toolset_service.is_toolset_enabled_for_app(&toolset_id).await? {
        return Err(ToolsetError::ToolsetAppDisabled.into());
    }

    if is_oauth_auth {
        // 2. Check app-client registered for toolset
        let azp = headers.get(KEY_HEADER_BODHIAPP_AZP)?;
        if !toolset_service.is_app_client_registered_for_toolset(azp, &toolset_id).await? {
            return Err(ToolsetAuthError::AppClientNotRegistered);
        }
        
        // 3. Check scope_toolset-* in token
        let toolset_scopes_header = headers.get(KEY_HEADER_BODHIAPP_TOOLSET_SCOPES).unwrap_or("");
        let required_scope = ToolsetScope::scope_for_toolset_id(&toolset_id)
            .ok_or(ToolsetError::ToolsetNotFound(toolset_id.clone()))?;
        if !toolset_scopes_header.split_whitespace().any(|s| s == required_scope.to_string()) {
            return Err(ToolsetAuthError::MissingToolsetScope);
        }
    }

    // 4. Check user has toolset configured (API key required for execution)
    if !toolset_service.is_toolset_available_for_user(user_id, &toolset_id).await? {
        return Err(ToolsetError::ToolsetNotConfigured.into());
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

## Header Constants

```rust
pub const KEY_HEADER_BODHIAPP_TOOLSET_SCOPES: &str = "X-BodhiApp-Toolset-Scopes";
pub const KEY_HEADER_BODHIAPP_AZP: &str = "X-BodhiApp-Azp";
```

These headers are injected after token exchange for OAuth tokens:
- `X-BodhiApp-Toolset-Scopes`: Space-separated toolset scopes from token
- `X-BodhiApp-Azp`: Authorized party (app-client ID)

## Error Types

```rust
#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum ToolsetAuthError {
    #[error("missing_user_id")]
    #[error_meta(error_type = ErrorType::Unauthorized)]
    MissingUserId,

    #[error("missing_auth")]
    #[error_meta(error_type = ErrorType::Unauthorized)]
    MissingAuth,

    #[error("app_client_not_registered")]
    #[error_meta(error_type = ErrorType::Forbidden)]
    AppClientNotRegistered,

    #[error("missing_toolset_scope")]
    #[error_meta(error_type = ErrorType::Forbidden)]
    MissingToolsetScope,

    #[error("missing_azp_header")]
    #[error_meta(error_type = ErrorType::Forbidden)]
    MissingAzpHeader,

    #[error(transparent)]
    ToolsetError(#[from] ToolsetError),
}
```

## Keycloak Configuration

Toolset scopes are configured on app-clients via developer portal:
1. Client scope `scope_toolset-builtin-exa-web-search` exists in realm
2. App-client has `bodhi.toolsets` attribute listing allowed toolsets
3. App-client has toolset scope in optional scopes

See [09-keycloak-extension-contract.md](./09-keycloak-extension-contract.md) for full Keycloak integration details.

## Related Documents

- [05.5-app-level-toolset-config.md](./05.5-app-level-toolset-config.md) - App-level toolset enable/disable
- [05.6-external-app-toolset-access.md](./05.6-external-app-toolset-access.md) - Full OAuth toolset authorization flow
- [09-keycloak-extension-contract.md](./09-keycloak-extension-contract.md) - Keycloak extension API contract
