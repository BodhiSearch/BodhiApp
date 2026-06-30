use crate::{
  AppRole, ApprovedResources, DeploymentMode, ErrorType, ResourceRole, TokenGrants, TokenScope,
  UserScope,
};
use errmeta::AppError;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AuthContext {
  Anonymous {
    deployment: DeploymentMode,
  },
  Session {
    client_id: String,
    tenant_id: String,
    user_id: String,
    username: String,
    role: ResourceRole,
    token: String,
  },
  MultiTenantSession {
    client_id: Option<String>,
    tenant_id: Option<String>,
    user_id: String,
    username: String,
    role: ResourceRole,
    token: Option<String>,
    dashboard_token: String,
  },
  ApiToken {
    client_id: String,
    tenant_id: String,
    user_id: String,
    role: TokenScope,
    token: String,
    /// Per-resource grants parsed from the token's stored `grants` JSON.
    grants: TokenGrants,
  },
  ExternalApp {
    client_id: String,
    tenant_id: String,
    user_id: String,
    role: Option<UserScope>,
    token: String,
    external_app_token: String,
    app_client_id: String,
    access_request_id: Option<String>,
    /// Approved per-resource grants from the bound access request, resolved at
    /// token-exchange. `None` when no access request is bound (treated as
    /// unrestricted, preserving the pre-grants behavior).
    grants: Option<ApprovedResources>,
  },
}

impl AuthContext {
  pub fn is_multi_tenant(&self) -> bool {
    match self {
      AuthContext::MultiTenantSession { .. } => true,
      AuthContext::Anonymous { deployment, .. } => *deployment == DeploymentMode::MultiTenant,
      _ => false,
    }
  }

  pub fn client_id(&self) -> Option<&str> {
    match self {
      AuthContext::Anonymous { .. } => None,
      AuthContext::Session { client_id, .. } => Some(client_id),
      AuthContext::MultiTenantSession { client_id, .. } => client_id.as_deref(),
      AuthContext::ApiToken { client_id, .. } => Some(client_id),
      AuthContext::ExternalApp { client_id, .. } => Some(client_id),
    }
  }

  pub fn tenant_id(&self) -> Option<&str> {
    match self {
      AuthContext::Anonymous { .. } => None,
      AuthContext::Session { tenant_id, .. } => Some(tenant_id),
      AuthContext::MultiTenantSession { tenant_id, .. } => tenant_id.as_deref(),
      AuthContext::ApiToken { tenant_id, .. } => Some(tenant_id),
      AuthContext::ExternalApp { tenant_id, .. } => Some(tenant_id),
    }
  }

  pub fn require_tenant_id(&self) -> Result<&str, AuthContextError> {
    self.tenant_id().ok_or(AuthContextError::MissingTenantId)
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
      AuthContext::MultiTenantSession { user_id, .. } => Some(user_id),
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
      AuthContext::MultiTenantSession { token, .. } => token.as_deref(),
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

  pub fn resource_role(&self) -> Option<&ResourceRole> {
    match self {
      AuthContext::Session { role, .. } => Some(role),
      AuthContext::MultiTenantSession { role, .. } => Some(role),
      _ => None,
    }
  }

  pub fn dashboard_token(&self) -> Option<&str> {
    match self {
      AuthContext::MultiTenantSession {
        dashboard_token, ..
      } => Some(dashboard_token),
      _ => None,
    }
  }

  pub fn require_dashboard_token(&self) -> Result<&str, AuthContextError> {
    self
      .dashboard_token()
      .ok_or(AuthContextError::MissingDashboardToken)
  }

  pub fn app_role(&self) -> Option<AppRole> {
    match self {
      AuthContext::Anonymous { .. } => Some(AppRole::Session(ResourceRole::Anonymous)),
      AuthContext::Session { role, .. } => Some(AppRole::Session(*role)),
      AuthContext::MultiTenantSession { role, .. } => Some(AppRole::Session(*role)),
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

  #[error("Tenant ID is required but not present in auth context.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  MissingTenantId,

  #[error("Dashboard token is required but not present in auth context.")]
  #[error_meta(error_type = ErrorType::Authentication)]
  MissingDashboardToken,
}

#[cfg(test)]
mod tests {
  use super::{AuthContext, AuthContextError};
  use crate::{AppRole, DeploymentMode, ResourceRole, TokenGrants, TokenScope};
  use errmeta::AppError;
  use pretty_assertions::assert_eq;
  use rstest::rstest;

  fn anonymous() -> AuthContext {
    AuthContext::Anonymous {
      deployment: DeploymentMode::Standalone,
    }
  }

  fn session() -> AuthContext {
    AuthContext::Session {
      client_id: "test-client-id".to_string(),
      tenant_id: "test-tenant".to_string(),
      user_id: "user1".to_string(),
      username: "testuser".to_string(),
      role: ResourceRole::Guest,
      token: "test-token".to_string(),
    }
  }

  fn api_token() -> AuthContext {
    AuthContext::ApiToken {
      client_id: "test-client".to_string(),
      tenant_id: "tenant-123".to_string(),
      user_id: "user1".to_string(),
      role: TokenScope::User,
      token: "test-token".to_string(),
      grants: TokenGrants::default(),
    }
  }

  /// The three `require_*` accessors share one signature, so the success and
  /// failure matrices collapse into two parameterized tests.
  #[rstest]
  #[case::user_id_anonymous(anonymous(), AuthContext::require_user_id as fn(&AuthContext) -> Result<&str, AuthContextError>, 403, "auth_context_error-anonymous_not_allowed")]
  #[case::client_id_anonymous(
    anonymous(),
    AuthContext::require_client_id,
    403,
    "auth_context_error-missing_client_id"
  )]
  #[case::tenant_id_missing(
    anonymous(),
    AuthContext::require_tenant_id,
    500,
    "auth_context_error-missing_tenant_id"
  )]
  fn require_accessor_errors(
    #[case] ctx: AuthContext,
    #[case] accessor: fn(&AuthContext) -> Result<&str, AuthContextError>,
    #[case] status: u16,
    #[case] code: &str,
  ) {
    let err = accessor(&ctx).unwrap_err();
    assert_eq!(status, err.status());
    assert_eq!(code, err.code());
  }

  #[rstest]
  #[case::user_id_session(session(), AuthContext::require_user_id as fn(&AuthContext) -> Result<&str, AuthContextError>, "user1")]
  #[case::client_id_session(session(), AuthContext::require_client_id, "test-client-id")]
  #[case::tenant_id_api_token(api_token(), AuthContext::require_tenant_id, "tenant-123")]
  fn require_accessor_ok(
    #[case] ctx: AuthContext,
    #[case] accessor: fn(&AuthContext) -> Result<&str, AuthContextError>,
    #[case] expected: &str,
  ) {
    assert_eq!(expected, accessor(&ctx).unwrap());
  }

  #[test]
  fn test_external_app_no_role_is_authenticated() {
    let ctx = AuthContext::ExternalApp {
      client_id: "test-client-id".to_string(),
      tenant_id: "test-tenant".to_string(),
      user_id: "user1".to_string(),
      role: None,
      token: "test-external-token".to_string(),
      external_app_token: "test-external-app-token".to_string(),
      app_client_id: "app1".to_string(),
      access_request_id: None,
      grants: None,
    };
    assert_eq!(true, ctx.is_authenticated());
    assert_eq!(None, ctx.app_role());
    assert_eq!(Some("user1"), ctx.user_id());
    assert_eq!(Some("test-client-id"), ctx.client_id());
  }

  #[test]
  fn test_anonymous_user_id_is_none() {
    let ctx = AuthContext::Anonymous {
      deployment: DeploymentMode::Standalone,
    };
    assert_eq!(None, ctx.user_id());
    assert_eq!(None, ctx.client_id());
    assert_eq!(false, ctx.is_authenticated());
    assert_eq!(false, ctx.is_multi_tenant());
    assert_eq!(
      Some(AppRole::Session(ResourceRole::Anonymous)),
      ctx.app_role()
    );
  }

  #[test]
  fn test_anonymous_multi_tenant() {
    let ctx = AuthContext::Anonymous {
      deployment: DeploymentMode::MultiTenant,
    };
    assert_eq!(true, ctx.is_multi_tenant());
    assert_eq!(false, ctx.is_authenticated());
  }

  #[test]
  fn test_tenant_id_returns_none_when_not_set() {
    let ctx = AuthContext::Anonymous {
      deployment: DeploymentMode::Standalone,
    };
    assert_eq!(None, ctx.tenant_id());
  }

  #[test]
  fn test_tenant_id_returns_some_when_set() {
    let ctx = AuthContext::Session {
      client_id: "test-client".to_string(),
      tenant_id: "test-tenant".to_string(),
      user_id: "user1".to_string(),
      username: "testuser".to_string(),
      role: ResourceRole::Guest,
      token: "test-token".to_string(),
    };
    assert_eq!(Some("test-tenant"), ctx.tenant_id());
  }

  #[test]
  fn test_multi_tenant_session_dashboard_only() {
    let ctx = AuthContext::MultiTenantSession {
      client_id: None,
      tenant_id: None,
      user_id: "user1".to_string(),
      username: "testuser".to_string(),
      role: ResourceRole::Guest,
      token: None,
      dashboard_token: "dashboard-tok".to_string(),
    };
    assert_eq!(true, ctx.is_authenticated());
    assert_eq!(true, ctx.is_multi_tenant());
    assert_eq!(Some("user1"), ctx.user_id());
    assert_eq!(None, ctx.client_id());
    assert_eq!(None, ctx.tenant_id());
    assert_eq!(None, ctx.token());
    assert_eq!(Some("dashboard-tok"), ctx.dashboard_token());
  }

  #[test]
  fn test_multi_tenant_session_full() {
    let ctx = AuthContext::MultiTenantSession {
      client_id: Some("client1".to_string()),
      tenant_id: Some("tenant1".to_string()),
      user_id: "user1".to_string(),
      username: "testuser".to_string(),
      role: crate::ResourceRole::Admin,
      token: Some("resource-tok".to_string()),
      dashboard_token: "dashboard-tok".to_string(),
    };
    assert_eq!(true, ctx.is_authenticated());
    assert_eq!(true, ctx.is_multi_tenant());
    assert_eq!(Some("client1"), ctx.client_id());
    assert_eq!(Some("tenant1"), ctx.tenant_id());
    assert_eq!(Some("resource-tok"), ctx.token());
    assert_eq!(Some(&crate::ResourceRole::Admin), ctx.resource_role());
    assert_eq!(Some("dashboard-tok"), ctx.dashboard_token());
  }

  #[test]
  fn test_require_dashboard_token_missing() {
    let ctx = AuthContext::Session {
      client_id: "c".to_string(),
      tenant_id: "t".to_string(),
      user_id: "u".to_string(),
      username: "n".to_string(),
      role: ResourceRole::Guest,
      token: "tok".to_string(),
    };
    let err = ctx.require_dashboard_token().unwrap_err();
    assert_eq!(401, err.status());
    assert_eq!("auth_context_error-missing_dashboard_token", err.code());
  }

  #[test]
  fn test_session_is_not_multi_tenant() {
    let ctx = AuthContext::Session {
      client_id: "c".to_string(),
      tenant_id: "t".to_string(),
      user_id: "u".to_string(),
      username: "n".to_string(),
      role: ResourceRole::Guest,
      token: "tok".to_string(),
    };
    assert_eq!(false, ctx.is_multi_tenant());
  }
}
