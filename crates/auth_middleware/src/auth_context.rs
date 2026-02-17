use objs::{AppRole, ResourceRole, TokenScope, UserScope};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AuthContext {
  Anonymous,
  Session {
    user_id: String,
    username: String,
    role: Option<ResourceRole>,
    token: String,
  },
  ApiToken {
    user_id: String,
    role: TokenScope,
    token: String,
  },
  ExternalApp {
    user_id: String,
    role: UserScope,
    token: String,
    external_app_token: String,
    app_client_id: String,
    access_request_id: Option<String>,
  },
}

impl AuthContext {
  pub fn user_id(&self) -> Option<&str> {
    match self {
      AuthContext::Anonymous => None,
      AuthContext::Session { user_id, .. } => Some(user_id),
      AuthContext::ApiToken { user_id, .. } => Some(user_id),
      AuthContext::ExternalApp { user_id, .. } => Some(user_id),
    }
  }

  pub fn token(&self) -> Option<&str> {
    match self {
      AuthContext::Anonymous => None,
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
      AuthContext::Anonymous => None,
      AuthContext::Session {
        role: Some(role), ..
      } => Some(AppRole::Session(*role)),
      AuthContext::Session { role: None, .. } => None,
      AuthContext::ApiToken { role, .. } => Some(AppRole::ApiToken(*role)),
      AuthContext::ExternalApp { role, .. } => Some(AppRole::ExchangedToken(*role)),
    }
  }

  pub fn is_authenticated(&self) -> bool {
    !matches!(self, AuthContext::Anonymous)
  }
}

#[cfg(feature = "test-utils")]
mod test_factory {
  use super::*;
  use axum::body::Body;
  use axum::http::Request;

  impl AuthContext {
    pub fn test_session(user_id: &str, username: &str, role: ResourceRole) -> Self {
      AuthContext::Session {
        user_id: user_id.to_string(),
        username: username.to_string(),
        role: Some(role),
        token: "test-token".to_string(),
      }
    }

    pub fn test_session_no_role(user_id: &str, username: &str) -> Self {
      AuthContext::Session {
        user_id: user_id.to_string(),
        username: username.to_string(),
        role: None,
        token: "test-token".to_string(),
      }
    }

    pub fn test_session_with_token(
      user_id: &str,
      username: &str,
      role: ResourceRole,
      token: &str,
    ) -> Self {
      AuthContext::Session {
        user_id: user_id.to_string(),
        username: username.to_string(),
        role: Some(role),
        token: token.to_string(),
      }
    }

    pub fn test_api_token(user_id: &str, role: TokenScope) -> Self {
      AuthContext::ApiToken {
        user_id: user_id.to_string(),
        role,
        token: "test-api-token".to_string(),
      }
    }

    pub fn test_external_app(
      user_id: &str,
      role: UserScope,
      app_client_id: &str,
      access_request_id: Option<&str>,
    ) -> Self {
      AuthContext::ExternalApp {
        user_id: user_id.to_string(),
        role,
        token: "test-external-token".to_string(),
        external_app_token: "test-external-app-token".to_string(),
        app_client_id: app_client_id.to_string(),
        access_request_id: access_request_id.map(|s| s.to_string()),
      }
    }
  }

  pub trait RequestAuthContextExt {
    fn with_auth_context(self, ctx: AuthContext) -> Self;
  }

  impl RequestAuthContextExt for Request<Body> {
    fn with_auth_context(mut self, ctx: AuthContext) -> Self {
      self.extensions_mut().insert(ctx);
      self
    }
  }
}

#[cfg(feature = "test-utils")]
pub use test_factory::RequestAuthContextExt;
