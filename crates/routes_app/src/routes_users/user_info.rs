use crate::{TokenInfo, UserResponse, ENDPOINT_USER_INFO};
use auth_middleware::AuthContext;
use axum::{Extension, Json};
use objs::{ApiError, AppRole, API_TAG_AUTH};
use services::{extract_claims, Claims};
use tracing::debug;

/// Get information about the currently logged in user
#[utoipa::path(
    get,
    path = ENDPOINT_USER_INFO,
    tag = API_TAG_AUTH,
    operation_id = "getCurrentUser",
    summary = "Get Current User Information",
    description = "Retrieves information about the currently authenticated user. This endpoint supports optional authentication - returns `logged_out` status if not authenticated, or user details with roles/scopes if authenticated via any method (session, API token, or OAuth exchange).",
    responses(
        (status = 200, description = "User information (authenticated or not)", body = UserResponse,
         examples(
             ("authenticated" = (summary = "Authenticated user", value = json!({
                 "auth_status": "logged_in",
                 "user_id": "550e8400-e29b-41d4-a716-446655440000",
                 "username": "user@example.com",
                 "role": "resource_admin"
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
pub async fn user_info_handler(
  Extension(auth_context): Extension<AuthContext>,
) -> Result<Json<UserResponse>, ApiError> {
  match auth_context {
    AuthContext::Anonymous => {
      debug!("anonymous request");
      Ok(Json(UserResponse::LoggedOut))
    }
    AuthContext::Session {
      ref token,
      ref role,
      ..
    } => {
      debug!("session auth");
      let claims: Claims = extract_claims::<Claims>(token)?;
      Ok(Json(UserResponse::LoggedIn(objs::UserInfo {
        user_id: claims.sub,
        username: claims.preferred_username,
        first_name: claims.given_name,
        last_name: claims.family_name,
        role: role.map(AppRole::Session),
      })))
    }
    AuthContext::ApiToken { ref role, .. } => {
      debug!("api token auth");
      Ok(Json(UserResponse::Token(TokenInfo { role: *role })))
    }
    AuthContext::ExternalApp {
      ref token,
      ref role,
      ..
    } => {
      debug!("external app auth");
      let claims: Claims = extract_claims::<Claims>(token)?;
      Ok(Json(UserResponse::LoggedIn(objs::UserInfo {
        user_id: claims.sub,
        username: claims.preferred_username,
        first_name: claims.given_name,
        last_name: claims.family_name,
        role: Some(AppRole::ExchangedToken(*role)),
      })))
    }
  }
}
