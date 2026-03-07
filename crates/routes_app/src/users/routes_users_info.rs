use crate::tenants::DASHBOARD_ACCESS_TOKEN_KEY;
use crate::{
  ApiError, AuthScope, TokenInfo, UserInfoEnvelope, UserResponse, API_TAG_AUTH, ENDPOINT_USER_INFO,
};
use axum::Json;
use services::AppRole;
use services::AuthContext;
use services::{extract_claims, Claims};
use tower_sessions::Session;
use tracing::debug;

/// Get information about the currently logged in user
#[utoipa::path(
    get,
    path = ENDPOINT_USER_INFO,
    tag = API_TAG_AUTH,
    operation_id = "getCurrentUser",
    summary = "Get Current User Information",
    description = "Retrieves information about the currently authenticated user. This endpoint supports optional authentication - returns `logged_out` status if not authenticated, or user details with roles/scopes if authenticated via any method (session, API token, or OAuth exchange). Includes `has_dashboard_session` when the user has an active dashboard session.",
    responses(
        (status = 200, description = "User information (authenticated or not)", body = UserInfoEnvelope,
         examples(
             ("authenticated" = (summary = "Authenticated user", value = json!({
                 "auth_status": "logged_in",
                 "user_id": "550e8400-e29b-41d4-a716-446655440000",
                 "username": "user@example.com",
                 "role": "resource_admin",
                 "has_dashboard_session": true
             }))),
             ("unauthenticated" = (summary = "Unauthenticated request", value = json!({
                 "auth_status": "logged_out"
             })))
         ))
    ),
    security(
        (),
        ("bearer_api_token" = []),
        ("bearer_oauth_token" = []),
        ("session_auth" = [])
    )
)]
pub async fn users_info(
  auth_scope: AuthScope,
  session: Session,
) -> Result<Json<UserInfoEnvelope>, ApiError> {
  let has_dashboard_session = session
    .get::<String>(DASHBOARD_ACCESS_TOKEN_KEY)
    .await
    .unwrap_or(None)
    .is_some();

  let user = match auth_scope.auth_context().clone() {
    AuthContext::Anonymous { .. } => {
      debug!("anonymous request");
      UserResponse::LoggedOut
    }
    AuthContext::Session {
      ref token,
      ref role,
      ..
    } => {
      debug!("session auth");
      let claims: Claims = extract_claims::<Claims>(token)?;
      UserResponse::LoggedIn(services::UserInfo {
        user_id: claims.sub,
        username: claims.preferred_username,
        first_name: claims.given_name,
        last_name: claims.family_name,
        role: role.map(AppRole::Session),
      })
    }
    AuthContext::ApiToken { ref role, .. } => {
      debug!("api token auth");
      UserResponse::Token(TokenInfo { role: *role })
    }
    AuthContext::ExternalApp {
      ref token,
      ref role,
      ..
    } => {
      debug!("external app auth");
      let claims: Claims = extract_claims::<Claims>(token)?;
      UserResponse::LoggedIn(services::UserInfo {
        user_id: claims.sub,
        username: claims.preferred_username,
        first_name: claims.given_name,
        last_name: claims.family_name,
        role: role.as_ref().map(|&r| AppRole::ExchangedToken(r)),
      })
    }
  };

  Ok(Json(UserInfoEnvelope {
    user,
    has_dashboard_session,
  }))
}

#[cfg(test)]
#[path = "test_user_info.rs"]
mod test_user_info;
