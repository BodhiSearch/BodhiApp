use super::db::TEST_TENANT_ID;
use crate::{AuthContext, ResourceRole, TokenScope, UserScope};

const DEFAULT_CLIENT_ID: &str = "test-client-id";

impl AuthContext {
  pub fn test_anonymous() -> Self {
    AuthContext::Anonymous {
      client_id: Some(DEFAULT_CLIENT_ID.to_string()),
      tenant_id: Some(TEST_TENANT_ID.to_string()),
    }
  }

  pub fn test_anonymous_with_client_id(client_id: &str) -> Self {
    AuthContext::Anonymous {
      client_id: Some(client_id.to_string()),
      tenant_id: Some(TEST_TENANT_ID.to_string()),
    }
  }

  pub fn test_session(user_id: &str, username: &str, role: ResourceRole) -> Self {
    AuthContext::Session {
      client_id: DEFAULT_CLIENT_ID.to_string(),
      tenant_id: TEST_TENANT_ID.to_string(),
      user_id: user_id.to_string(),
      username: username.to_string(),
      role: Some(role),
      token: "test-token".to_string(),
    }
  }

  pub fn test_session_no_role(user_id: &str, username: &str) -> Self {
    AuthContext::Session {
      client_id: DEFAULT_CLIENT_ID.to_string(),
      tenant_id: TEST_TENANT_ID.to_string(),
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
      client_id: DEFAULT_CLIENT_ID.to_string(),
      tenant_id: TEST_TENANT_ID.to_string(),
      user_id: user_id.to_string(),
      username: username.to_string(),
      role: Some(role),
      token: token.to_string(),
    }
  }

  pub fn test_api_token(user_id: &str, role: TokenScope) -> Self {
    AuthContext::ApiToken {
      client_id: DEFAULT_CLIENT_ID.to_string(),
      tenant_id: TEST_TENANT_ID.to_string(),
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
      client_id: DEFAULT_CLIENT_ID.to_string(),
      tenant_id: TEST_TENANT_ID.to_string(),
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
      client_id: DEFAULT_CLIENT_ID.to_string(),
      tenant_id: TEST_TENANT_ID.to_string(),
      user_id: user_id.to_string(),
      role: None,
      token: "test-external-token".to_string(),
      external_app_token: "test-external-app-token".to_string(),
      app_client_id: app_client_id.to_string(),
      access_request_id: access_request_id.map(|s| s.to_string()),
    }
  }

  pub fn with_tenant_id(self, tenant_id: &str) -> Self {
    match self {
      AuthContext::Anonymous { client_id, .. } => AuthContext::Anonymous {
        client_id,
        tenant_id: Some(tenant_id.to_string()),
      },
      AuthContext::Session {
        client_id,
        user_id,
        username,
        role,
        token,
        ..
      } => AuthContext::Session {
        client_id,
        tenant_id: tenant_id.to_string(),
        user_id,
        username,
        role,
        token,
      },
      AuthContext::ApiToken {
        client_id,
        user_id,
        role,
        token,
        ..
      } => AuthContext::ApiToken {
        client_id,
        tenant_id: tenant_id.to_string(),
        user_id,
        role,
        token,
      },
      AuthContext::ExternalApp {
        client_id,
        user_id,
        role,
        token,
        external_app_token,
        app_client_id,
        access_request_id,
        ..
      } => AuthContext::ExternalApp {
        client_id,
        tenant_id: tenant_id.to_string(),
        user_id,
        role,
        token,
        external_app_token,
        app_client_id,
        access_request_id,
      },
    }
  }

  pub fn with_user_id(self, user_id: &str) -> Self {
    match self {
      AuthContext::Anonymous { .. } => self,
      AuthContext::Session {
        client_id,
        tenant_id,
        username,
        role,
        token,
        ..
      } => AuthContext::Session {
        client_id,
        tenant_id,
        user_id: user_id.to_string(),
        username,
        role,
        token,
      },
      AuthContext::ApiToken {
        client_id,
        tenant_id,
        role,
        token,
        ..
      } => AuthContext::ApiToken {
        client_id,
        tenant_id,
        user_id: user_id.to_string(),
        role,
        token,
      },
      AuthContext::ExternalApp {
        client_id,
        tenant_id,
        role,
        token,
        external_app_token,
        app_client_id,
        access_request_id,
        ..
      } => AuthContext::ExternalApp {
        client_id,
        tenant_id,
        user_id: user_id.to_string(),
        role,
        token,
        external_app_token,
        app_client_id,
        access_request_id,
      },
    }
  }
}
