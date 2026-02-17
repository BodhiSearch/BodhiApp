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
    role: Option<UserScope>,
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
      AuthContext::ExternalApp {
        role: Some(role), ..
      } => Some(AppRole::ExchangedToken(*role)),
      AuthContext::ExternalApp { role: None, .. } => None,
    }
  }

  pub fn is_authenticated(&self) -> bool {
    !matches!(self, AuthContext::Anonymous)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_external_app_no_role_is_authenticated() {
    let ctx = AuthContext::test_external_app_no_role("user1", "app1", None);
    assert_eq!(true, ctx.is_authenticated());
    assert_eq!(None, ctx.app_role());
    assert_eq!(Some("user1"), ctx.user_id());
  }
}
