use crate::{
  ApiError, AuthScope, DashboardUser, TokenInfo, UserInfoEnvelope, UserResponse, API_TAG_AUTH,
  ENDPOINT_USER_INFO,
};
use axum::Json;
use services::AppRole;
use services::AuthContext;
use services::{extract_claims, Claims};
use tracing::debug;

/// Get information about the currently logged in user
#[utoipa::path(
    get,
    path = ENDPOINT_USER_INFO,
    tag = API_TAG_AUTH,
    operation_id = "getCurrentUser",
    summary = "Get Current User Information",
    description = "Retrieves information about the currently authenticated user. This endpoint supports optional authentication - returns `logged_out` status if not authenticated, or user details with roles/scopes if authenticated via any method (session, API token, or OAuth exchange). Includes `dashboard` object when the user has an active dashboard session.",
    responses(
        (status = 200, description = "User information (authenticated or not)", body = UserInfoEnvelope,
         examples(
             ("authenticated" = (summary = "Fully authenticated with dashboard", value = json!({
                 "auth_status": "logged_in",
                 "user_id": "550e8400-e29b-41d4-a716-446655440000",
                 "username": "user@example.com",
                 "role": "resource_admin",
                 "dashboard": {
                     "user_id": "550e8400-e29b-41d4-a716-446655440000",
                     "username": "user@example.com",
                     "first_name": "Test",
                     "last_name": "User"
                 }
             }))),
             ("dashboard_only" = (summary = "Dashboard session only (no tenant login)", value = json!({
                 "auth_status": "logged_out",
                 "dashboard": {
                     "user_id": "550e8400-e29b-41d4-a716-446655440000",
                     "username": "user@example.com",
                     "first_name": "Test",
                     "last_name": "User"
                 }
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
pub async fn users_info(auth_scope: AuthScope) -> Result<Json<UserInfoEnvelope>, ApiError> {
  let (user, dashboard) = match auth_scope.auth_context().clone() {
    AuthContext::Anonymous { .. } => {
      debug!("anonymous request");
      (UserResponse::LoggedOut, None)
    }
    AuthContext::Session {
      ref token,
      ref role,
      ..
    } => {
      debug!("session auth");
      let claims: Claims = extract_claims::<Claims>(token)?;
      (
        UserResponse::LoggedIn(services::UserInfo {
          user_id: claims.sub,
          username: claims.preferred_username,
          first_name: claims.given_name,
          last_name: claims.family_name,
          role: Some(AppRole::Session(*role)),
        }),
        None,
      )
    }
    AuthContext::MultiTenantSession {
      ref token,
      ref role,
      ref dashboard_token,
      ..
    } => {
      debug!("multi-tenant session auth");
      let dashboard_claims: Claims = extract_claims::<Claims>(dashboard_token)?;
      let dashboard = DashboardUser {
        user_id: dashboard_claims.sub,
        username: dashboard_claims.preferred_username,
        first_name: dashboard_claims.given_name,
        last_name: dashboard_claims.family_name,
      };
      let user = if let Some(ref token) = token {
        let claims: Claims = extract_claims::<Claims>(token)?;
        UserResponse::LoggedIn(services::UserInfo {
          user_id: claims.sub,
          username: claims.preferred_username,
          first_name: claims.given_name,
          last_name: claims.family_name,
          role: Some(AppRole::Session(*role)),
        })
      } else {
        UserResponse::LoggedOut
      };
      (user, Some(dashboard))
    }
    AuthContext::ApiToken { ref role, .. } => {
      debug!("api token auth");
      (UserResponse::Token(TokenInfo { role: *role }), None)
    }
    AuthContext::ExternalApp {
      ref token,
      ref role,
      ..
    } => {
      debug!("external app auth");
      let claims: Claims = extract_claims::<Claims>(token)?;
      (
        UserResponse::LoggedIn(services::UserInfo {
          user_id: claims.sub,
          username: claims.preferred_username,
          first_name: claims.given_name,
          last_name: claims.family_name,
          role: role.as_ref().map(|&r| AppRole::ExchangedToken(r)),
        }),
        None,
      )
    }
  };

  Ok(Json(UserInfoEnvelope { user, dashboard }))
}

#[cfg(test)]
#[path = "test_user_info.rs"]
mod test_user_info;
