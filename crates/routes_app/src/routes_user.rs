use crate::ENDPOINT_USER_INFO;
use auth_middleware::{
  KEY_HEADER_BODHIAPP_ROLE, KEY_HEADER_BODHIAPP_SCOPE, KEY_HEADER_BODHIAPP_TOKEN,
};
use axum::{http::header::HeaderMap, Json};
use objs::{
  ApiError, AppError, AppRole, ErrorType, ResourceRole, ResourceScope, TokenScope, UserInfo,
  API_TAG_AUTH,
};
use serde::{Deserialize, Serialize};
use services::{extract_claims, Claims};
use tracing::debug;
use utoipa::ToSchema;

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum UserRouteError {
  #[error("Invalid header value: {0}.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  InvalidHeader(String),
  #[error("Injected token is empty.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  EmptyToken,
  #[error("Failed to list users: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  ListFailed(String),
  #[error("Failed to change user role: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  RoleChangeFailed(String),
  #[error("Failed to remove user: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  RemoveFailed(String),
}

/// Token Type
/// `session` - token stored in cookie based http session
/// `bearer` - token received from http authorization header as bearer token
#[derive(Debug, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum TokenType {
  Session,
  Bearer,
}

/// Role Source
/// `role` - client level user role
/// `scope_token` - scope granted token role
/// `scope_user` - scope granted user role
#[derive(Debug, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum RoleSource {
  Role,
  ScopeToken,
  ScopeUser,
}

/// API Token information response
#[derive(Debug, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct TokenInfo {
  pub role: TokenScope,
}

/// User authentication response with discriminated union
#[derive(Debug, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(tag = "auth_status")]
#[schema(example = json!({
    "auth_status": "logged_in",
    "user_id": "550e8400-e29b-41d4-a716-446655440000",
    "username": "user@example.com",
    "role": "resource_user"
}))]
pub enum UserResponse {
  /// User is not authenticated
  #[serde(rename = "logged_out")]
  LoggedOut,
  /// User is authenticated with details
  #[serde(rename = "logged_in")]
  LoggedIn(UserInfo),
  /// API token authentication
  #[serde(rename = "api_token")]
  Token(TokenInfo),
}

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
      Ok(Json(UserResponse::LoggedIn(UserInfo {
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
          Ok(Json(UserResponse::LoggedIn(UserInfo {
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
      Ok(Json(UserResponse::LoggedIn(UserInfo {
        user_id: claims.sub,
        username: claims.preferred_username,
        first_name: claims.given_name,
        last_name: claims.family_name,
        role: None,
      })))
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::{user_info_handler, TokenInfo, UserResponse};
  use auth_middleware::{
    KEY_HEADER_BODHIAPP_ROLE, KEY_HEADER_BODHIAPP_SCOPE, KEY_HEADER_BODHIAPP_TOKEN,
  };
  use axum::{
    body::Body,
    http::{status::StatusCode, Request},
    routing::get,
    Router,
  };
  use objs::{AppRole, ResourceRole, ResourceScope, TokenScope, UserInfo, UserScope};
  use pretty_assertions::assert_eq;
  use rstest::rstest;
  use serde_json::{json, Value};
  use server_core::{test_utils::ResponseTestExt, DefaultRouterState, MockSharedContext};
  use services::{
    test_utils::{token, AppServiceStubBuilder},
    AppService,
  };
  use std::sync::Arc;
  use tower::ServiceExt;

  fn test_router(app_service: Arc<dyn AppService>) -> Router {
    let state = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContext::default()),
      app_service,
    ));
    Router::new()
      .route("/app/user", get(user_info_handler))
      .with_state(state)
  }

  #[rstest]
  #[tokio::test]
  async fn test_user_info_handler_no_token_header() -> anyhow::Result<()> {
    let app_service: Arc<dyn AppService> = Arc::new(AppServiceStubBuilder::default().build()?);
    let router = test_router(app_service);

    let response = router
      .oneshot(Request::get("/app/user").body(Body::empty())?)
      .await?;

    assert_eq!(StatusCode::OK, response.status());
    let response_json = response.json::<UserResponse>().await?;
    assert_eq!(UserResponse::LoggedOut, response_json);
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_user_info_handler_empty_token() -> anyhow::Result<()> {
    let app_service: Arc<dyn AppService> = Arc::new(AppServiceStubBuilder::default().build()?);
    let router = test_router(app_service);

    let response = router
      .oneshot(
        Request::get("/app/user")
          .header(KEY_HEADER_BODHIAPP_TOKEN, "")
          .body(Body::empty())?,
      )
      .await?;

    assert_eq!(StatusCode::BAD_REQUEST, response.status());
    let response_json = response.json::<Value>().await?;
    assert_eq!(
      json!({
        "error": {
          "message": "Injected token is empty.",
          "type": "invalid_request_error",
          "code": "user_route_error-empty_token",
        }
      }),
      response_json
    );
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_user_info_handler_invalid_token() -> anyhow::Result<()> {
    let app_service: Arc<dyn AppService> = Arc::new(AppServiceStubBuilder::default().build()?);
    let router = test_router(app_service);

    let response = router
      .oneshot(
        Request::get("/app/user")
          .header(KEY_HEADER_BODHIAPP_TOKEN, "invalid_token")
          .body(Body::empty())?,
      )
      .await?;

    assert_eq!(StatusCode::UNAUTHORIZED, response.status());
    let response_json = response.json::<Value>().await?;
    assert_eq!(
      json!({
        "error": {
          "message": "Invalid token: malformed token format.",
          "code": "token_error-invalid_token",
          "type": "authentication_error",
          "param": {
            "var_0": "malformed token format"
          }
        }
      }),
      response_json
    );
    Ok(())
  }

  #[rstest]
  #[case::resource_user(ResourceRole::User)]
  #[case::resource_power_user(ResourceRole::PowerUser)]
  #[case::resource_manager(ResourceRole::Manager)]
  #[case::resource_admin(ResourceRole::Admin)]
  #[tokio::test]
  async fn test_user_info_handler_session_token_with_role(
    token: (String, String),
    #[case] role: ResourceRole,
  ) -> anyhow::Result<()> {
    let (token, _) = token;
    let app_service: Arc<dyn AppService> = Arc::new(AppServiceStubBuilder::default().build()?);
    let router = test_router(app_service);

    let response = router
      .oneshot(
        Request::get("/app/user")
          .header(KEY_HEADER_BODHIAPP_TOKEN, &token)
          .header(KEY_HEADER_BODHIAPP_ROLE, role.to_string())
          .body(Body::empty())?,
      )
      .await?;

    assert_eq!(StatusCode::OK, response.status());
    let response_json = response.json::<UserResponse>().await?;

    // Extract user_id from the token to verify correct response
    let claims = services::extract_claims::<services::Claims>(&token)?;

    assert_eq!(
      UserResponse::LoggedIn(UserInfo {
        user_id: claims.sub,
        username: "testuser@email.com".to_string(),
        first_name: Some("Test".to_string()),
        last_name: Some("User".to_string()),
        role: Some(AppRole::Session(role)),
      }),
      response_json
    );
    Ok(())
  }

  #[rstest]
  #[case::scope_token_user(TokenScope::User)]
  #[case::scope_token_power_user(TokenScope::PowerUser)]
  #[case::scope_token_manager(TokenScope::Manager)]
  #[case::scope_token_admin(TokenScope::Admin)]
  #[tokio::test]
  async fn test_user_info_handler_api_token_with_token_scope(
    #[case] token_scope: TokenScope,
  ) -> anyhow::Result<()> {
    let app_service: Arc<dyn AppService> = Arc::new(AppServiceStubBuilder::default().build()?);
    let router = test_router(app_service);

    // API tokens are random strings, not JWT - simulate what middleware injects
    let api_token = "bodhiapp_test_random_token_string";
    let resource_scope = ResourceScope::Token(token_scope);
    let response = router
      .oneshot(
        Request::get("/app/user")
          .header(KEY_HEADER_BODHIAPP_TOKEN, api_token)
          .header(KEY_HEADER_BODHIAPP_SCOPE, resource_scope.to_string())
          .body(Body::empty())?,
      )
      .await?;

    assert_eq!(StatusCode::OK, response.status());
    let response_json = response.json::<UserResponse>().await?;

    // API tokens should return TokenInfo, not UserInfo
    assert_eq!(
      UserResponse::Token(TokenInfo { role: token_scope }),
      response_json
    );
    Ok(())
  }

  #[rstest]
  #[case::scope_user_user(UserScope::User)]
  #[case::scope_user_power_user(UserScope::PowerUser)]
  #[case::scope_user_manager(UserScope::Manager)]
  #[case::scope_user_admin(UserScope::Admin)]
  #[tokio::test]
  async fn test_user_info_handler_bearer_token_with_user_scope(
    token: (String, String),
    #[case] user_scope: UserScope,
  ) -> anyhow::Result<()> {
    let (token, _) = token;
    let app_service: Arc<dyn AppService> = Arc::new(AppServiceStubBuilder::default().build()?);
    let router = test_router(app_service);

    let resource_scope = ResourceScope::User(user_scope);
    let response = router
      .oneshot(
        Request::get("/app/user")
          .header(KEY_HEADER_BODHIAPP_TOKEN, &token)
          .header(KEY_HEADER_BODHIAPP_SCOPE, resource_scope.to_string())
          .body(Body::empty())?,
      )
      .await?;

    assert_eq!(StatusCode::OK, response.status());
    let response_json = response.json::<UserResponse>().await?;

    // Extract user_id from the token to verify correct response
    let claims = services::extract_claims::<services::Claims>(&token)?;

    assert_eq!(
      UserResponse::LoggedIn(UserInfo {
        user_id: claims.sub,
        username: "testuser@email.com".to_string(),
        first_name: Some("Test".to_string()),
        last_name: Some("User".to_string()),
        role: Some(AppRole::ExchangedToken(user_scope)),
      }),
      response_json
    );
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_user_info_handler_role_takes_precedence_over_scope(
    token: (String, String),
  ) -> anyhow::Result<()> {
    let (token, _) = token;
    let app_service: Arc<dyn AppService> = Arc::new(AppServiceStubBuilder::default().build()?);
    let router = test_router(app_service);

    // Both role and scope headers present - role should take precedence
    let response = router
      .oneshot(
        Request::get("/app/user")
          .header(KEY_HEADER_BODHIAPP_TOKEN, &token)
          .header(KEY_HEADER_BODHIAPP_ROLE, ResourceRole::Manager.to_string())
          .header(
            KEY_HEADER_BODHIAPP_SCOPE,
            ResourceScope::Token(TokenScope::User).to_string(),
          )
          .body(Body::empty())?,
      )
      .await?;

    assert_eq!(StatusCode::OK, response.status());
    let response_json = response.json::<UserResponse>().await?;

    // Extract user_id from the token to verify correct response
    let claims = services::extract_claims::<services::Claims>(&token)?;

    assert_eq!(
      UserResponse::LoggedIn(UserInfo {
        user_id: claims.sub,
        username: "testuser@email.com".to_string(),
        first_name: Some("Test".to_string()),
        last_name: Some("User".to_string()),
        role: Some(AppRole::Session(ResourceRole::Manager)),
      }),
      response_json
    );
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_user_info_handler_missing_role_and_scope_headers(
    token: (String, String),
  ) -> anyhow::Result<()> {
    let (token, _) = token;
    let app_service: Arc<dyn AppService> = Arc::new(AppServiceStubBuilder::default().build()?);
    let router = test_router(app_service);

    let response = router
      .oneshot(
        Request::get("/app/user")
          .header(KEY_HEADER_BODHIAPP_TOKEN, &token)
          .body(Body::empty())?,
      )
      .await?;

    assert_eq!(StatusCode::OK, response.status());
    let response_json = response.json::<UserResponse>().await?;

    // Extract user_id from the token to verify correct response
    let claims = services::extract_claims::<services::Claims>(&token)?;

    assert_eq!(
      UserResponse::LoggedIn(UserInfo {
        user_id: claims.sub,
        username: "testuser@email.com".to_string(),
        first_name: Some("Test".to_string()),
        last_name: Some("User".to_string()),
        role: None,
      }),
      response_json
    );
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_user_info_handler_malformed_role_header(
    token: (String, String),
  ) -> anyhow::Result<()> {
    let (token, _) = token;
    let app_service: Arc<dyn AppService> = Arc::new(AppServiceStubBuilder::default().build()?);
    let router = test_router(app_service);

    let response = router
      .oneshot(
        Request::get("/app/user")
          .header(KEY_HEADER_BODHIAPP_TOKEN, &token)
          .header(KEY_HEADER_BODHIAPP_ROLE, "invalid_role")
          .body(Body::empty())?,
      )
      .await?;

    assert_eq!(StatusCode::BAD_REQUEST, response.status());
    let response_json = response.json::<Value>().await?;
    assert_eq!(
      json!({
        "error": {
          "message": "invalid_role_name",
          "type": "invalid_request_error",
          "code": "role_error-invalid_role_name",
          "param": {
            "var_0": "invalid_role"
          }
        }
      }),
      response_json
    );
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_user_info_handler_malformed_scope_header(
    token: (String, String),
  ) -> anyhow::Result<()> {
    let (token, _) = token;
    let app_service: Arc<dyn AppService> = Arc::new(AppServiceStubBuilder::default().build()?);
    let router = test_router(app_service);

    let response = router
      .oneshot(
        Request::get("/app/user")
          .header(KEY_HEADER_BODHIAPP_TOKEN, &token)
          .header(KEY_HEADER_BODHIAPP_SCOPE, "invalid_scope")
          .body(Body::empty())?,
      )
      .await?;

    assert_eq!(StatusCode::UNAUTHORIZED, response.status());
    let response_json = response.json::<Value>().await?;
    assert_eq!(
      json!({
        "error": {
          "message": "invalid resource scope: invalid_scope",
          "type": "authentication_error",
          "code": "resource_scope_error-invalid_scope",
          "param": {
            "var_0": "invalid_scope"
          }
        }
      }),
      response_json
    );
    Ok(())
  }
}
