use objs::{AppRole, ResourceRole, TokenScope, UserScope};

#[derive(Debug, Clone)]
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
    scope: TokenScope,
    token: String,
  },
  ExternalApp {
    user_id: String,
    scope: UserScope,
    token: String,
    azp: String,
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
      AuthContext::ExternalApp { token, .. } => Some(token),
    }
  }

  pub fn app_role(&self) -> Option<AppRole> {
    match self {
      AuthContext::Anonymous => None,
      AuthContext::Session {
        role: Some(role), ..
      } => Some(AppRole::Session(*role)),
      AuthContext::Session { role: None, .. } => None,
      AuthContext::ApiToken { scope, .. } => Some(AppRole::ApiToken(*scope)),
      AuthContext::ExternalApp { scope, .. } => Some(AppRole::ExchangedToken(*scope)),
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

    pub fn test_api_token(user_id: &str, scope: TokenScope) -> Self {
      AuthContext::ApiToken {
        user_id: user_id.to_string(),
        scope,
        token: "test-api-token".to_string(),
      }
    }

    pub fn test_external_app(
      user_id: &str,
      scope: UserScope,
      azp: &str,
      access_request_id: Option<&str>,
    ) -> Self {
      AuthContext::ExternalApp {
        user_id: user_id.to_string(),
        scope,
        token: "test-external-token".to_string(),
        azp: azp.to_string(),
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
