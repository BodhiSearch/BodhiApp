use super::error::AppScopeError;
use crate::{ApprovedResourcesV1, McpGrant, ModelGrant, UserScope};

/// Prefix of the dynamic Keycloak scope carrying `<resource-client-id>.<access-request-id>`.
pub const SCOPE_ACCESS_REQUEST_PREFIX: &str = "scope_access_request:";

const SCOPE_APPS_PREFIX: &str = "scope_apps:";

/// Always sent to Keycloak; app-supplied duplicates are collapsed into these.
const BASE_OIDC_SCOPES: [&str; 4] = ["openid", "profile", "email", "roles"];

/// App-facing scope string, parsed. `scope_user_*` and `scope_apps:*` are consumed by
/// BodhiApp (role ceiling, consent-section flags) and never forwarded; every other token
/// passes through to Keycloak verbatim.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedAppScope {
  pub role: UserScope,
  pub llms: bool,
  pub mcps: bool,
  pub passthrough: Vec<String>,
}

/// Exact-match outcome of the app's `redirect_uri` against its registered list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedirectUriValidation {
  Valid,
  /// Older auth-server extension that does not return `redirect_uris` — allow unvalidated.
  Unvalidated,
  /// Registered list present but no exact match — render in-app, never redirect (RFC 6749 §4.1.2.1).
  Mismatch,
}

/// Defaults: no role token → `User`; `scope_apps:*` absent/valueless/`:true` → requested,
/// `:false` → not requested. Two role tokens → higher ceiling wins. `llms:false mcps:false`
/// is a valid role-only request, not an error.
pub fn parse_app_scope(scope: &str) -> Result<ParsedAppScope, AppScopeError> {
  let mut role: Option<UserScope> = None;
  let mut llms: Option<bool> = None;
  let mut mcps: Option<bool> = None;
  let mut passthrough: Vec<String> = Vec::new();

  for token in scope.split_whitespace() {
    if let Ok(parsed) = token.parse::<UserScope>() {
      role = Some(match role {
        Some(current) if current >= parsed => current,
        _ => parsed,
      });
    } else if let Some(rest) = token.strip_prefix(SCOPE_APPS_PREFIX) {
      let (slot, value) = match rest {
        "llms" | "llms:true" => (&mut llms, true),
        "llms:false" => (&mut llms, false),
        "mcps" | "mcps:true" => (&mut mcps, true),
        "mcps:false" => (&mut mcps, false),
        _ => {
          return Err(AppScopeError::MalformedScopeToken {
            token: token.to_string(),
          })
        }
      };
      match slot {
        Some(existing) if *existing != value => {
          return Err(AppScopeError::ConflictingScopeToken {
            token: token.to_string(),
          })
        }
        _ => *slot = Some(value),
      }
    } else if token.starts_with("scope_user_") {
      return Err(AppScopeError::MalformedScopeToken {
        token: token.to_string(),
      });
    } else if token
      .to_ascii_lowercase()
      .starts_with(SCOPE_ACCESS_REQUEST_PREFIX)
    {
      return Err(AppScopeError::ReservedScopeToken {
        token: token.to_string(),
      });
    } else if !passthrough.iter().any(|t| t == token) {
      passthrough.push(token.to_string());
    }
  }

  Ok(ParsedAppScope {
    role: role.unwrap_or(UserScope::User),
    llms: llms.unwrap_or(true),
    mcps: mcps.unwrap_or(true),
    passthrough,
  })
}

/// The dotted dynamic scope value the Keycloak mapper parses by last-dot split. The mapper
/// 500s on any malformed value, so composition refuses inputs that could produce one.
pub fn access_request_scope_value(
  resource_client_id: &str,
  access_request_id: &str,
) -> Result<String, AppScopeError> {
  if resource_client_id.is_empty()
    || access_request_id.is_empty()
    || access_request_id.contains('.')
  {
    return Err(AppScopeError::InvalidScopeComposition {
      resource_client_id: resource_client_id.to_string(),
      access_request_id: access_request_id.to_string(),
    });
  }
  Ok(format!(
    "{SCOPE_ACCESS_REQUEST_PREFIX}{resource_client_id}.{access_request_id}"
  ))
}

/// `openid profile email roles` ∪ passthrough (deduped, stable order) + the dotted dynamic
/// scope.
pub fn compose_keycloak_scope(parsed: &ParsedAppScope, access_request_scope: &str) -> String {
  let mut tokens: Vec<&str> = BASE_OIDC_SCOPES.to_vec();
  for token in &parsed.passthrough {
    if !tokens.contains(&token.as_str()) {
      tokens.push(token);
    }
  }
  format!("{} {}", tokens.join(" "), access_request_scope)
}

/// A grant envelope may not exceed what the scope requested — this is what stops a
/// tampered POST widening a grant beyond what the app asked for.
pub fn validate_grant_against_scope(
  parsed: &ParsedAppScope,
  approved: &ApprovedResourcesV1,
) -> Result<(), AppScopeError> {
  if !parsed.llms {
    let models_empty =
      matches!(&approved.models_access, ModelGrant::Specific { ids } if ids.is_empty());
    if approved.models_list || !models_empty {
      return Err(AppScopeError::GrantExceedsScope {
        section: "models".to_string(),
      });
    }
  }
  if !parsed.mcps {
    let mcps_empty = matches!(&approved.mcps_access, McpGrant::Specific { ids } if ids.is_empty());
    if approved.mcps_list || !approved.mcps.is_empty() || !mcps_empty {
      return Err(AppScopeError::GrantExceedsScope {
        section: "mcps".to_string(),
      });
    }
  }
  Ok(())
}

pub fn match_redirect_uri(requested: &str, registered: Option<&[String]>) -> RedirectUriValidation {
  match registered {
    None => RedirectUriValidation::Unvalidated,
    Some(uris) => {
      if uris.iter().any(|uri| uri == requested) {
        RedirectUriValidation::Valid
      } else {
        RedirectUriValidation::Mismatch
      }
    }
  }
}

#[cfg(test)]
#[path = "test_app_scopes.rs"]
mod test_app_scopes;
