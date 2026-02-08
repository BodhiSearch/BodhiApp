use crate::{user_info_handler, TokenInfo, UserResponse};
use anyhow_trace::anyhow_trace;
use auth_middleware::{KEY_HEADER_BODHIAPP_SCOPE, KEY_HEADER_BODHIAPP_TOKEN};
use axum::{
  body::Body,
  http::{status::StatusCode, Request},
  routing::get,
  Router,
};
use objs::{AppRole, ResourceRole, ResourceScope, TokenScope, UserInfo, UserScope};
use pretty_assertions::assert_eq;
use rstest::rstest;
use serde_json::Value;
use server_core::{
  test_utils::{RequestAuthExt, ResponseTestExt},
  DefaultRouterState, MockSharedContext,
};
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
#[anyhow_trace]
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
#[anyhow_trace]
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
    "user_route_error-empty_token",
    response_json["error"]["code"].as_str().unwrap()
  );
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
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
    "token_error-invalid_token",
    response_json["error"]["code"].as_str().unwrap()
  );
  Ok(())
}

#[rstest]
#[case::resource_user(ResourceRole::User)]
#[case::resource_power_user(ResourceRole::PowerUser)]
#[case::resource_manager(ResourceRole::Manager)]
#[case::resource_admin(ResourceRole::Admin)]
#[tokio::test]
#[anyhow_trace]
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
        .with_user_auth(&token, &role.to_string())
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
#[anyhow_trace]
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
        .with_api_token(api_token, &resource_scope.to_string())
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
#[anyhow_trace]
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
        .with_api_token(&token, &resource_scope.to_string())
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
#[anyhow_trace]
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
        .with_user_auth(&token, &ResourceRole::Manager.to_string())
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
#[anyhow_trace]
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
#[anyhow_trace]
async fn test_user_info_handler_malformed_role_header(
  token: (String, String),
) -> anyhow::Result<()> {
  let (token, _) = token;
  let app_service: Arc<dyn AppService> = Arc::new(AppServiceStubBuilder::default().build()?);
  let router = test_router(app_service);

  let response = router
    .oneshot(
      Request::get("/app/user")
        .with_user_auth(&token, "invalid_role")
        .body(Body::empty())?,
    )
    .await?;

  assert_eq!(StatusCode::BAD_REQUEST, response.status());
  let response_json = response.json::<Value>().await?;
  assert_eq!(
    "role_error-invalid_role_name",
    response_json["error"]["code"].as_str().unwrap()
  );
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_user_info_handler_malformed_scope_header(
  token: (String, String),
) -> anyhow::Result<()> {
  let (token, _) = token;
  let app_service: Arc<dyn AppService> = Arc::new(AppServiceStubBuilder::default().build()?);
  let router = test_router(app_service);

  let response = router
    .oneshot(
      Request::get("/app/user")
        .with_api_token(&token, "invalid_scope")
        .body(Body::empty())?,
    )
    .await?;

  assert_eq!(StatusCode::UNAUTHORIZED, response.status());
  let response_json = response.json::<Value>().await?;
  assert_eq!(
    "resource_scope_error-invalid_scope",
    response_json["error"]["code"].as_str().unwrap()
  );
  Ok(())
}
