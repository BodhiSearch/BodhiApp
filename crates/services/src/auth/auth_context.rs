use crate::{AppRole, ErrorType, ResourceRole, TokenScope, UserScope};
use errmeta::AppError;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AuthContext {
  Anonymous {
    client_id: Option<String>,
  },
  Session {
    client_id: String,
    user_id: String,
    username: String,
    role: Option<ResourceRole>,
    token: String,
  },
  ApiToken {
    client_id: String,
    user_id: String,
    role: TokenScope,
    token: String,
  },
  ExternalApp {
    client_id: String,
    user_id: String,
    role: Option<UserScope>,
    token: String,
    external_app_token: String,
    app_client_id: String,
    access_request_id: Option<String>,
  },
}

impl AuthContext {
  pub fn client_id(&self) -> Option<&str> {
    match self {
      AuthContext::Anonymous { client_id } => client_id.as_deref(),
      AuthContext::Session { client_id, .. } => Some(client_id),
      AuthContext::ApiToken { client_id, .. } => Some(client_id),
      AuthContext::ExternalApp { client_id, .. } => Some(client_id),
    }
  }

  pub fn require_client_id(&self) -> Result<&str, AuthContextError> {
    match self.client_id() {
      Some(id) => Ok(id),
      None => Err(AuthContextError::MissingClientId),
    }
  }

  pub fn user_id(&self) -> Option<&str> {
    match self {
      AuthContext::Anonymous { .. } => None,
      AuthContext::Session { user_id, .. } => Some(user_id),
      AuthContext::ApiToken { user_id, .. } => Some(user_id),
      AuthContext::ExternalApp { user_id, .. } => Some(user_id),
    }
  }

  pub fn require_user_id(&self) -> Result<&str, AuthContextError> {
    match self.user_id() {
      Some(id) => Ok(id),
      None => Err(AuthContextError::AnonymousNotAllowed),
    }
  }

  pub fn token(&self) -> Option<&str> {
    match self {
      AuthContext::Anonymous { .. } => None,
      AuthContext::Session { token, .. } => Some(token),
      AuthContext::ApiToken { token, .. } => Some(token),
      AuthContext::ExternalApp { token, .. } => Some(token), // Returns exchanged token
    }
  }

  pub fn external_app_token(&self) -> Option<&str> {
    match self {
      AuthContext::ExternalApp {
        external_app_token, ..
      } => Some(external_app_token),
      _ => None,
    }
  }

  pub fn app_role(&self) -> Option<AppRole> {
    match self {
      AuthContext::Anonymous { .. } => None,
      AuthContext::Session {
        role: Some(role), ..
      } => Some(AppRole::Session(*role)),
      AuthContext::Session { role: None, .. } => None,
      AuthContext::ApiToken { role, .. } => Some(AppRole::ApiToken(*role)),
      AuthContext::ExternalApp {
        role: Some(role), ..
      } => Some(AppRole::ExchangedToken(*role)),
      AuthContext::ExternalApp { role: None, .. } => None,
    }
  }

  pub fn is_authenticated(&self) -> bool {
    !matches!(self, AuthContext::Anonymous { .. })
  }
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum AuthContextError {
  #[error("Authentication required. Anonymous access not allowed.")]
  #[error_meta(error_type = ErrorType::Forbidden)]
  AnonymousNotAllowed,

  #[error("Client ID is required but not present.")]
  #[error_meta(error_type = ErrorType::Forbidden)]
  MissingClientId,

  #[error("Authentication token required to perform this operation.")]
  #[error_meta(error_type = ErrorType::Authentication)]
  MissingToken,
}

#[cfg(test)]
mod tests {
  use super::*;
  use errmeta::AppError;
  use pretty_assertions::assert_eq;

  #[test]
  fn test_external_app_no_role_is_authenticated() {
    let ctx = AuthContext::ExternalApp {
      client_id: "test-client-id".to_string(),
      user_id: "user1".to_string(),
      role: None,
      token: "test-external-token".to_string(),
      external_app_token: "test-external-app-token".to_string(),
      app_client_id: "app1".to_string(),
      access_request_id: None,
    };
    assert_eq!(true, ctx.is_authenticated());
    assert_eq!(None, ctx.app_role());
    assert_eq!(Some("user1"), ctx.user_id());
    assert_eq!(Some("test-client-id"), ctx.client_id());
  }

  #[test]
  fn test_anonymous_user_id_is_none() {
    let ctx = AuthContext::Anonymous { client_id: None };
    assert_eq!(None, ctx.user_id());
    assert_eq!(None, ctx.client_id());
    assert_eq!(false, ctx.is_authenticated());
  }

  #[test]
  fn test_anonymous_with_client_id() {
    let ctx = AuthContext::Anonymous {
      client_id: Some("test-client".to_string()),
    };
    assert_eq!(None, ctx.user_id());
    assert_eq!(Some("test-client"), ctx.client_id());
    assert_eq!(false, ctx.is_authenticated());
  }

  #[test]
  fn test_require_user_id_anonymous_returns_403() {
    let ctx = AuthContext::Anonymous { client_id: None };
    let result = ctx.require_user_id();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(403, err.status());
    assert_eq!("auth_context_error-anonymous_not_allowed", err.code());
  }

  #[test]
  fn test_require_client_id_anonymous_returns_403() {
    let ctx = AuthContext::Anonymous { client_id: None };
    let result = ctx.require_client_id();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(403, err.status());
    assert_eq!("auth_context_error-missing_client_id", err.code());
  }

  #[test]
  fn test_require_user_id_session_returns_ok() {
    let ctx = AuthContext::Session {
      client_id: "test-client-id".to_string(),
      user_id: "user1".to_string(),
      username: "testuser".to_string(),
      role: None,
      token: "test-token".to_string(),
    };
    let result = ctx.require_user_id();
    assert!(result.is_ok());
    assert_eq!("user1", result.unwrap());
  }

  #[test]
  fn test_require_client_id_session_returns_ok() {
    let ctx = AuthContext::Session {
      client_id: "test-client-id".to_string(),
      user_id: "user1".to_string(),
      username: "testuser".to_string(),
      role: None,
      token: "test-token".to_string(),
    };
    let result = ctx.require_client_id();
    assert!(result.is_ok());
    assert_eq!("test-client-id", result.unwrap());
  }
}
