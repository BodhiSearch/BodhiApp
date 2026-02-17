use crate::AuthContext;
use axum::body::Body;
use axum::http::Request;
use objs::{ResourceRole, TokenScope, UserScope};

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
      role: Some(role),
      token: "test-external-token".to_string(),
      external_app_token: "test-external-app-token".to_string(),
      app_client_id: app_client_id.to_string(),
      access_request_id: access_request_id.map(|s| s.to_string()),
    }
  }

  pub fn test_external_app_no_role(
    user_id: &str,
    app_client_id: &str,
    access_request_id: Option<&str>,
  ) -> Self {
    AuthContext::ExternalApp {
      user_id: user_id.to_string(),
      role: None,
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
