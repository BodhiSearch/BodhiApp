use crate::ENDPOINT_USER_INFO;
use auth_middleware::{
  KEY_HEADER_BODHIAPP_ROLE, KEY_HEADER_BODHIAPP_SCOPE, KEY_HEADER_BODHIAPP_TOKEN,
};
use axum::{http::header::HeaderMap, Json};
use objs::{ApiError, AppRole, BadRequestError, ResourceScope, Role, UserInfo, API_TAG_AUTH};
use serde::{Deserialize, Serialize};
use services::{extract_claims, Claims};
use tracing::debug;
use utoipa::ToSchema;

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
}

/// Get information about the currently logged in user
#[utoipa::path(
    get,
    path = ENDPOINT_USER_INFO,
    tag = API_TAG_AUTH,
    operation_id = "getCurrentUser",
    summary = "Get Current User Information",
    description = "Retrieves information about the currently authenticated user including email, roles, token type, and authentication source.",
    responses(
        (status = 200, description = "Current user information retrieved successfully", body = UserResponse,
         example = json!({
             "auth_status": "logged_in",
             "user_id": "550e8400-e29b-41d4-a716-446655440000",
             "username": "user@example.com",
             "role": "resource_admin"
         }))
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
    .map_err(|err| BadRequestError::new(err.to_string()))?;
  if token.is_empty() {
    debug!("injected token is empty");
    return Err(BadRequestError::new("injected token is empty".to_string()))?;
  }
  let claims: Claims = extract_claims::<Claims>(token)?;
  let role_header = headers.get(KEY_HEADER_BODHIAPP_ROLE);
  let token_header = headers.get(KEY_HEADER_BODHIAPP_SCOPE);
  match (role_header, token_header) {
    (Some(role_header), _) => {
      debug!("role header present");
      let role = role_header
        .to_str()
        .map_err(|err| BadRequestError::new(err.to_string()))?;
      let role = role.parse::<Role>()?;
      Ok(Json(UserResponse::LoggedIn(UserInfo {
        user_id: claims.sub,
        username: claims.preferred_username,
        first_name: claims.given_name,
        last_name: claims.family_name,
        role: Some(AppRole::Session(role)),
      })))
    }
    (None, Some(token_header)) => {
      debug!("token header present");
      let token = token_header
        .to_str()
        .map_err(|err| BadRequestError::new(err.to_string()))?;
      let token = ResourceScope::try_parse(token)?;
      let app_role = match token {
        ResourceScope::Token(token_scope) => AppRole::ApiToken(token_scope),
        ResourceScope::User(user_scope) => AppRole::ExchangedToken(user_scope),
      };
      Ok(Json(UserResponse::LoggedIn(UserInfo {
        user_id: claims.sub,
        username: claims.preferred_username,
        first_name: claims.given_name,
        last_name: claims.family_name,
        role: Some(app_role),
      })))
    }
    (None, None) => {
      debug!("no role or token header, returning logged in user without role");
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
  use crate::{user_info_handler, UserResponse};
  use auth_middleware::{
    KEY_HEADER_BODHIAPP_ROLE, KEY_HEADER_BODHIAPP_SCOPE, KEY_HEADER_BODHIAPP_TOKEN,
  };
  use axum::{
    body::Body,
    http::{status::StatusCode, Request},
    routing::get,
    Router,
  };
  use objs::{
    test_utils::setup_l10n, AppRole, FluentLocalizationService, ResourceScope, Role, TokenScope,
    UserInfo, UserScope,
  };
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
  async fn test_user_info_handler_empty_token(
    #[from(setup_l10n)] _localization_service: &Arc<FluentLocalizationService>,
  ) -> anyhow::Result<()> {
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
          "message": "invalid request, reason: \u{2068}injected token is empty\u{2069}",
          "type": "invalid_request_error",
          "code": "bad_request_error"
        }
      }),
      response_json
    );
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_user_info_handler_invalid_token(
    #[from(setup_l10n)] _localization_service: &Arc<FluentLocalizationService>,
  ) -> anyhow::Result<()> {
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
          "message": "token is invalid: \u{2068}malformed token format\u{2069}",
          "code": "token_error-invalid_token",
          "type": "authentication_error"
        }
      }),
      response_json
    );
    Ok(())
  }

  #[rstest]
  #[case::resource_user(Role::User)]
  #[case::resource_power_user(Role::PowerUser)]
  #[case::resource_manager(Role::Manager)]
  #[case::resource_admin(Role::Admin)]
  #[tokio::test]
  async fn test_user_info_handler_session_token_with_role(
    token: (String, String),
    #[case] role: Role,
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
  async fn test_user_info_handler_bearer_token_with_token_scope(
    token: (String, String),
    #[case] token_scope: TokenScope,
  ) -> anyhow::Result<()> {
    let (token, _) = token;
    let app_service: Arc<dyn AppService> = Arc::new(AppServiceStubBuilder::default().build()?);
    let router = test_router(app_service);

    let resource_scope = ResourceScope::Token(token_scope);
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
        role: Some(AppRole::ApiToken(token_scope)),
      }),
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
          .header(KEY_HEADER_BODHIAPP_ROLE, Role::Manager.to_string())
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
        role: Some(AppRole::Session(Role::Manager)),
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
          "message": "invalid role name: \u{2068}invalid_role\u{2069}",
          "type": "invalid_request_error",
          "code": "role_error-invalid_role_name"
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
          "message": "invalid resource scope: \u{2068}invalid_scope\u{2069}",
          "type": "authentication_error",
          "code": "resource_scope_error-invalid_scope"
        }
      }),
      response_json
    );
    Ok(())
  }
}
