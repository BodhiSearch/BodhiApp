use crate::{
  create_token_handler, list_tokens_handler, update_token_handler, ApiTokenResponse,
  CreateApiTokenRequest, PaginatedApiTokenResponse, UpdateApiTokenRequest,
};
use anyhow_trace::anyhow_trace;
use axum::{
  body::Body,
  http::{Method, Request},
  routing::{get, post, put},
  Router,
};
use hyper::StatusCode;
use objs::TokenScope;
use pretty_assertions::assert_eq;
use rstest::rstest;
use server_core::{
  test_utils::{RequestAuthExt, RequestTestExt, ResponseTestExt},
  DefaultRouterState, MockSharedContext,
};
use services::{
  db::{ApiToken, TokenRepository, TokenStatus},
  test_utils::{
    access_token_claims, build_token, test_db_service, AppServiceStub, AppServiceStubBuilder,
    TestDbService,
  },
  AppService,
};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

async fn app(app_service_stub: AppServiceStub) -> Router {
  let router_state = DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    Arc::new(app_service_stub),
  );
  Router::new()
    .route("/api/tokens", post(create_token_handler))
    .route("/api/tokens", get(list_tokens_handler))
    .route("/api/tokens/{token_id}", put(update_token_handler))
    .with_state(Arc::new(router_state))
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_list_tokens_pagination(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let claims = access_token_claims();
  let user_id = claims["sub"].as_str().unwrap().to_string();
  let (token, _) = build_token(claims)?;
  let db_service = Arc::new(db_service);
  let app_service = AppServiceStubBuilder::default()
    .db_service(db_service.clone())
    .build()?;

  // Create multiple tokens
  for i in 1..=15 {
    let mut token = ApiToken {
      id: Uuid::new_v4().to_string(),
      user_id: user_id.to_string(),
      name: format!("Test Token {}", i),
      token_prefix: format!("bodhiapp_test{:02}", i),
      token_hash: "token_hash".to_string(),
      scopes: "scope_token_user".to_string(),
      status: TokenStatus::Active,
      created_at: app_service.time_service().utc_now(),
      updated_at: app_service.time_service().utc_now(),
    };
    db_service.create_api_token(&mut token).await?;
  }

  let router = app(app_service).await;

  // Test first page
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::GET)
        .header("X-BodhiApp-Token", &token)
        .uri("/api/tokens?page=1&page_size=10")
        .body(Body::empty())?,
    )
    .await?;

  assert_eq!(response.status(), StatusCode::OK);
  let list_response = response.json::<PaginatedApiTokenResponse>().await?;
  assert_eq!(list_response.data.len(), 10);
  assert_eq!(list_response.total, 15);
  assert_eq!(list_response.page, 1);
  assert_eq!(list_response.page_size, 10);

  // Test second page
  let response = router
    .oneshot(
      Request::builder()
        .method(Method::GET)
        .header("X-BodhiApp-Token", &token)
        .uri("/api/tokens?page=2&page_size=10")
        .body(Body::empty())?,
    )
    .await?;

  assert_eq!(response.status(), StatusCode::OK);
  let list_response = response.json::<PaginatedApiTokenResponse>().await?;
  assert_eq!(list_response.data.len(), 5);
  assert_eq!(list_response.total, 15);
  assert_eq!(list_response.page, 2);
  assert_eq!(list_response.page_size, 10);

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_list_tokens_empty(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let (token, _) = build_token(access_token_claims())?;
  let db_service = Arc::new(db_service);
  let app_service = AppServiceStubBuilder::default()
    .db_service(db_service.clone())
    .build()?;

  let router = app(app_service).await;

  // Test empty results
  let response = router
    .oneshot(
      Request::builder()
        .method(Method::GET)
        .header("X-BodhiApp-Token", &token)
        .uri("/api/tokens")
        .body(Body::empty())?,
    )
    .await?;

  assert_eq!(response.status(), StatusCode::OK);
  let list_response = response.json::<PaginatedApiTokenResponse>().await?;
  assert_eq!(list_response.data.len(), 0);
  assert_eq!(list_response.total, 0);
  assert_eq!(list_response.page, 1); // Default page
  assert_eq!(list_response.page_size, 30); // Default page size

  Ok(())
}

#[rstest]
#[case::user_role_user_scope("resource_user", TokenScope::User, "scope_token_user")]
#[case::admin_role_user_scope("resource_admin", TokenScope::User, "scope_token_user")]
#[case::admin_role_power_user_scope(
  "resource_admin",
  TokenScope::PowerUser,
  "scope_token_power_user"
)]
#[case::manager_role_user_scope("resource_manager", TokenScope::User, "scope_token_user")]
#[case::manager_role_power_user_scope(
  "resource_manager",
  TokenScope::PowerUser,
  "scope_token_power_user"
)]
#[case::power_user_role_user_scope("resource_power_user", TokenScope::User, "scope_token_user")]
#[case::power_user_role_power_user_scope(
  "resource_power_user",
  TokenScope::PowerUser,
  "scope_token_power_user"
)]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_create_token_handler_role_scope_mapping(
  #[case] role: &str,
  #[case] requested_scope: TokenScope,
  #[case] expected_scope: &str,
  #[future] test_db_service: TestDbService,
) -> anyhow::Result<()> {
  let claims = access_token_claims();
  let user_id = claims["sub"].as_str().unwrap().to_string();
  let (access_token, _) = build_token(claims)?;

  let test_db_service = Arc::new(test_db_service);
  let app_service = AppServiceStubBuilder::default()
    .db_service(test_db_service.clone())
    .build()?;

  let app = app(app_service).await;

  // Make create request with specific role and scope
  let response = app
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/tokens")
        .with_user_auth(&access_token, role)
        .json(&CreateApiTokenRequest {
          name: Some(format!("Test Token for {}", role)),
          scope: requested_scope,
        })?,
    )
    .await?;

  assert_eq!(StatusCode::CREATED, response.status());

  let token_response = response.json::<ApiTokenResponse>().await?;
  assert!(
    token_response.token.starts_with("bodhiapp_"),
    "Token should start with 'bodhiapp_' prefix"
  );

  // Verify token in database has correct scope
  let token_prefix = &token_response.token[.."bodhiapp_".len() + 8];
  let db_token = test_db_service
    .get_api_token_by_prefix(token_prefix)
    .await?
    .expect("Token should exist in database");

  assert_eq!(expected_scope, db_token.scopes);
  assert_eq!(user_id, db_token.user_id);
  assert_eq!(TokenStatus::Active, db_token.status);

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_create_token_handler_success(
  #[future] test_db_service: TestDbService,
) -> anyhow::Result<()> {
  let claims = access_token_claims();
  let user_id = claims["sub"].as_str().unwrap().to_string();
  let (access_token, _) = build_token(claims)?;

  let test_db_service = Arc::new(test_db_service);
  let app_service = AppServiceStubBuilder::default()
    .db_service(test_db_service.clone())
    .build()?;

  let app = app(app_service).await;

  // Make create request
  let token_name = "Test Integration Token";
  let response = app
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/tokens")
        .with_user_auth(&access_token, "resource_user")
        .json(&CreateApiTokenRequest {
          name: Some(token_name.to_string()),
          scope: TokenScope::User,
        })?,
    )
    .await?;

  assert_eq!(StatusCode::CREATED, response.status());

  let token_response = response.json::<ApiTokenResponse>().await?;
  let token_str = &token_response.token;

  // Verify token format
  assert!(
    token_str.starts_with("bodhiapp_"),
    "Token should start with 'bodhiapp_' prefix"
  );
  assert!(
    token_str.len() > 50,
    "Token should be sufficiently long (prefix + base64)"
  );

  // Extract prefix for DB lookup
  let token_prefix = &token_str[.."bodhiapp_".len() + 8];

  // Verify token exists in database
  let db_token = test_db_service
    .get_api_token_by_prefix(token_prefix)
    .await?
    .expect("Token should exist in database");

  assert_eq!(token_name, db_token.name);
  assert_eq!(user_id, db_token.user_id);
  assert_eq!(TokenStatus::Active, db_token.status);
  assert_eq!("scope_token_user", db_token.scopes);

  // Verify hash matches
  let mut hasher = Sha256::new();
  hasher.update(token_str.as_bytes());
  let calculated_hash = format!("{:x}", hasher.finalize());
  assert_eq!(calculated_hash, db_token.token_hash);

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_create_token_handler_without_name(
  #[future] test_db_service: TestDbService,
) -> anyhow::Result<()> {
  let claims = access_token_claims();
  let (access_token, _) = build_token(claims)?;

  let test_db_service = Arc::new(test_db_service);
  let app_service = AppServiceStubBuilder::default()
    .db_service(test_db_service.clone())
    .build()?;

  let app = app(app_service).await;

  // Make create request without name
  let response = app
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/tokens")
        .with_user_auth(&access_token, "resource_user")
        .json(&CreateApiTokenRequest {
          name: None,
          scope: TokenScope::User,
        })?,
    )
    .await?;

  assert_eq!(StatusCode::CREATED, response.status());

  let token_response = response.json::<ApiTokenResponse>().await?;
  let token_prefix = &token_response.token[.."bodhiapp_".len() + 8];

  // Verify token has empty name
  let db_token = test_db_service
    .get_api_token_by_prefix(token_prefix)
    .await?
    .expect("Token should exist in database");

  assert_eq!("", db_token.name);

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_create_token_handler_missing_auth(
  #[future] test_db_service: TestDbService,
) -> anyhow::Result<()> {
  let test_db_service = Arc::new(test_db_service);
  let app_service = AppServiceStubBuilder::default()
    .db_service(test_db_service.clone())
    .build()?;

  let app = app(app_service).await;

  // Make create request WITHOUT authentication header
  let response = app
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/tokens")
        .json(&CreateApiTokenRequest {
          name: Some("Test Token".to_string()),
          scope: TokenScope::User,
        })?,
    )
    .await?;

  assert_eq!(StatusCode::BAD_REQUEST, response.status());

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_create_token_handler_invalid_role(
  #[future] test_db_service: TestDbService,
) -> anyhow::Result<()> {
  let claims = access_token_claims();
  let (access_token, _) = build_token(claims)?;

  let test_db_service = Arc::new(test_db_service);
  let app_service = AppServiceStubBuilder::default()
    .db_service(test_db_service.clone())
    .build()?;

  let app = app(app_service).await;

  // Make create request with INVALID role header
  let response = app
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/tokens")
        .with_user_auth(&access_token, "invalid_role_value")
        .json(&CreateApiTokenRequest {
          name: Some("Test Token".to_string()),
          scope: TokenScope::User,
        })?,
    )
    .await?;

  assert_eq!(StatusCode::BAD_REQUEST, response.status());

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_create_token_handler_missing_role(
  #[future] test_db_service: TestDbService,
) -> anyhow::Result<()> {
  let claims = access_token_claims();
  let (access_token, _) = build_token(claims)?;

  let test_db_service = Arc::new(test_db_service);
  let app_service = AppServiceStubBuilder::default()
    .db_service(test_db_service.clone())
    .build()?;

  let app = app(app_service).await;

  // Make create request WITHOUT role header (should fail)
  let response = app
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/tokens")
        .header("X-BodhiApp-Token", &access_token)
        // NO role header - should return error
        .json(&CreateApiTokenRequest {
          name: Some("Test Token".to_string()),
          scope: TokenScope::User,
        })?,
    )
    .await?;

  assert_eq!(StatusCode::BAD_REQUEST, response.status());

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_update_token_handler_success(
  #[future] test_db_service: TestDbService,
) -> anyhow::Result<()> {
  let claims = access_token_claims();
  let user_id = claims["sub"].as_str().unwrap().to_string();
  let (access_token, _) = build_token(claims)?;

  let test_db_service = Arc::new(test_db_service);
  let app_service = AppServiceStubBuilder::default()
    .db_service(test_db_service.clone())
    .build()?;
  let now = app_service.time_service().utc_now();

  // Create initial token
  let mut token = ApiToken {
    id: Uuid::new_v4().to_string(),
    user_id: user_id.to_string(),
    name: "Initial Name".to_string(),
    token_prefix: "bodhiapp_test123".to_string(),
    token_hash: "token_hash".to_string(),
    scopes: "scope_token_user".to_string(),
    status: TokenStatus::Active,
    created_at: now,
    updated_at: now,
  };
  test_db_service.create_api_token(&mut token).await?;

  // Setup app with router
  let app = app(app_service).await;

  // Make update request
  let response = app
    .oneshot(
      Request::builder()
        .method(Method::PUT)
        .uri(format!("/api/tokens/{}", token.id))
        .header("X-BodhiApp-Token", &access_token)
        .json(&UpdateApiTokenRequest {
          name: "Updated Name".to_string(),
          status: TokenStatus::Inactive,
        })?,
    )
    .await?;

  assert_eq!(response.status(), StatusCode::OK);

  let updated_token = response.json::<ApiToken>().await?;
  assert_eq!(updated_token.name, "Updated Name");
  assert_eq!(updated_token.status, TokenStatus::Inactive);
  assert_eq!(updated_token.id, token.id);

  // Verify DB was updated
  let db_token = test_db_service
    .get_api_token_by_id(&user_id, &token.id)
    .await?
    .expect("Token should exist in database");
  assert_eq!(db_token.name, "Updated Name");
  assert_eq!(db_token.status, TokenStatus::Inactive);

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_update_token_handler_not_found(
  #[future] test_db_service: TestDbService,
) -> anyhow::Result<()> {
  let (token, _) = build_token(access_token_claims())?;
  // Setup app with router
  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(test_db_service))
    .build()?;
  let app = app(app_service).await;

  // Make update request with non-existent ID
  let response = app
    .oneshot(
      Request::builder()
        .method(Method::PUT)
        .uri("/api/tokens/non-existent-id")
        .header("X-BodhiApp-Token", &token)
        .json(&UpdateApiTokenRequest {
          name: "Updated Name".to_string(),
          status: TokenStatus::Inactive,
        })?,
    )
    .await?;

  assert_eq!(response.status(), StatusCode::NOT_FOUND);
  Ok(())
}

#[rstest]
#[case::user_to_power_user("resource_user", TokenScope::PowerUser)]
#[case::user_to_manager("resource_user", TokenScope::Manager)]
#[case::user_to_admin("resource_user", TokenScope::Admin)]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_create_token_privilege_escalation_user(
  #[case] role: &str,
  #[case] requested_scope: TokenScope,
  #[future] test_db_service: TestDbService,
) -> anyhow::Result<()> {
  let claims = access_token_claims();
  let (access_token, _) = build_token(claims)?;

  let test_db_service = Arc::new(test_db_service);
  let app_service = AppServiceStubBuilder::default()
    .db_service(test_db_service.clone())
    .build()?;

  let app = app(app_service).await;

  // User attempting to create higher-privilege token
  let response = app
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/tokens")
        .with_user_auth(&access_token, role)
        .json(&CreateApiTokenRequest {
          name: Some("Escalation Attempt".to_string()),
          scope: requested_scope,
        })?,
    )
    .await?;

  assert_eq!(StatusCode::BAD_REQUEST, response.status());

  Ok(())
}

#[rstest]
#[case::power_user_to_manager("resource_power_user", TokenScope::Manager)]
#[case::power_user_to_admin("resource_power_user", TokenScope::Admin)]
#[case::manager_to_admin("resource_manager", TokenScope::Admin)]
#[case::admin_to_manager("resource_admin", TokenScope::Manager)]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_create_token_privilege_escalation_invalid_scopes(
  #[case] role: &str,
  #[case] requested_scope: TokenScope,
  #[future] test_db_service: TestDbService,
) -> anyhow::Result<()> {
  let claims = access_token_claims();
  let (access_token, _) = build_token(claims)?;

  let test_db_service = Arc::new(test_db_service);
  let app_service = AppServiceStubBuilder::default()
    .db_service(test_db_service.clone())
    .build()?;

  let app = app(app_service).await;

  // Any role attempting to create Manager/Admin token
  let response = app
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/tokens")
        .with_user_auth(&access_token, role)
        .json(&CreateApiTokenRequest {
          name: Some("Invalid Scope Attempt".to_string()),
          scope: requested_scope,
        })?,
    )
    .await?;

  assert_eq!(StatusCode::BAD_REQUEST, response.status());

  Ok(())
}
