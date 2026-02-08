use crate::{TokenInfo, UserResponse, UserRouteError, ENDPOINT_USER_INFO};
use auth_middleware::{
  KEY_HEADER_BODHIAPP_ROLE, KEY_HEADER_BODHIAPP_SCOPE, KEY_HEADER_BODHIAPP_TOKEN,
};
use axum::{http::header::HeaderMap, Json};
use objs::{ApiError, AppRole, ResourceRole, ResourceScope, API_TAG_AUTH};
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
pub async fn user_info_handler(headers: HeaderMap) -> Result<Json<UserResponse>, ApiError> {
  let not_loggedin = UserResponse::LoggedOut;
  let Some(token) = headers.get(KEY_HEADER_BODHIAPP_TOKEN) else {
    debug!("no token header");
    return Ok(Json(not_loggedin));
  };
  let token = token
    .to_str()
    .map_err(|err| UserRouteError::InvalidHeader(err.to_string()))?;
  if token.is_empty() {
    debug!("injected token is empty");
    return Err(UserRouteError::EmptyToken)?;
  }
  let role_header = headers.get(KEY_HEADER_BODHIAPP_ROLE);
  let scope_header = headers.get(KEY_HEADER_BODHIAPP_SCOPE);
  match (role_header, scope_header) {
    (Some(role_header), _) => {
      debug!("role header present");
      let role = role_header
        .to_str()
        .map_err(|err| UserRouteError::InvalidHeader(err.to_string()))?;
      let role = role.parse::<ResourceRole>()?;
      let claims: Claims = extract_claims::<Claims>(token)?;
      Ok(Json(UserResponse::LoggedIn(objs::UserInfo {
        user_id: claims.sub,
        username: claims.preferred_username,
        first_name: claims.given_name,
        last_name: claims.family_name,
        role: Some(AppRole::Session(role)),
      })))
    }
    (None, Some(scope_header)) => {
      debug!("scope header present");
      let scope = scope_header
        .to_str()
        .map_err(|err| UserRouteError::InvalidHeader(err.to_string()))?;
      let resource_scope = ResourceScope::try_parse(scope)?;
      match resource_scope {
        ResourceScope::Token(token_scope) => {
          debug!("token scope present, returning api token info");
          Ok(Json(UserResponse::Token(TokenInfo { role: token_scope })))
        }
        ResourceScope::User(user_scope) => {
          debug!("user scope present, extracting claims");
          let claims: Claims = extract_claims::<Claims>(token)?;
          Ok(Json(UserResponse::LoggedIn(objs::UserInfo {
            user_id: claims.sub,
            username: claims.preferred_username,
            first_name: claims.given_name,
            last_name: claims.family_name,
            role: Some(AppRole::ExchangedToken(user_scope)),
          })))
        }
      }
    }
    (None, None) => {
      debug!("no role or scope header, returning logged in user without role");
      let claims: Claims = extract_claims::<Claims>(token)?;
      Ok(Json(UserResponse::LoggedIn(objs::UserInfo {
        user_id: claims.sub,
        username: claims.preferred_username,
        first_name: claims.given_name,
        last_name: claims.family_name,
        role: None,
      })))
    }
  }
}
