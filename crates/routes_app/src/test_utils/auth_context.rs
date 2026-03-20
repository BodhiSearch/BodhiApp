use axum::body::Body;
use axum::http::Request;
use services::test_utils::TEST_TENANT_ID;
use services::{AuthContext, ResourceRole};

pub trait RequestAuthContextExt {
  fn with_auth_context(self, ctx: AuthContext) -> Self;
}

impl RequestAuthContextExt for Request<Body> {
  fn with_auth_context(mut self, ctx: AuthContext) -> Self {
    self.extensions_mut().insert(ctx);
    self
  }
}

/// Creates an `AuthContext` for the given auth variant with a role and token.
///
/// Use in tests parameterized with `#[values("session", "multi_tenant")]`.
pub fn make_auth_with_role(
  variant: &str,
  user_id: &str,
  username: &str,
  role: ResourceRole,
  token: &str,
) -> AuthContext {
  match variant {
    "session" => AuthContext::test_session_with_token(user_id, username, role, token),
    "multi_tenant" => AuthContext::test_multi_tenant_session_full(
      user_id,
      username,
      "test-client-id",
      TEST_TENANT_ID,
      role,
      token,
    ),
    _ => panic!("unknown auth variant: {variant}"),
  }
}

/// Creates an `AuthContext` for the given auth variant with a role but no explicit token.
///
/// Uses the default "test-token" for both session and multi-tenant variants.
pub fn make_auth_with_role_default_token(
  variant: &str,
  user_id: &str,
  username: &str,
  role: ResourceRole,
) -> AuthContext {
  make_auth_with_role(variant, user_id, username, role, "test-token")
}

/// Creates an `AuthContext` for the given auth variant without a role.
///
/// For multi-tenant, creates a session with `tenant_id` and `client_id` populated
/// (needed for tests that proceed past auth extraction into auth-scoped services).
pub fn make_auth_no_role(variant: &str, user_id: &str, username: &str) -> AuthContext {
  match variant {
    "session" => AuthContext::test_session(user_id, username, ResourceRole::Guest),
    "multi_tenant" => AuthContext::test_multi_tenant_session_no_role(user_id, username),
    _ => panic!("unknown auth variant: {variant}"),
  }
}
