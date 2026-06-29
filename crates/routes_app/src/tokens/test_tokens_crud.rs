use crate::test_utils::{make_auth_no_role, make_auth_with_role, RequestAuthContextExt};
use crate::tokens::tokens_api_schemas::{
  CreateTokenRequest, PaginatedTokenResponse, TokenCreated, TokenDetail, UpdateTokenRequest,
};
use crate::{tokens_create, tokens_delete, tokens_index, tokens_update};
use anyhow_trace::anyhow_trace;
use axum::{
  body::Body,
  http::{Method, Request},
  routing::{get, post, put},
  Router,
};
use hyper::StatusCode;
use pretty_assertions::assert_eq;
use rstest::rstest;
use server_core::test_utils::{RequestTestExt, ResponseTestExt};
use services::AuthContext;
use services::{
  test_utils::{
    access_token_claims, build_token, test_db_service, AppServiceStub, AppServiceStubBuilder,
    TestDbService, TEST_TENANT_ID,
  },
  AppService, {TokenEntity, TokenRepository, TokenStatus},
};
use services::{McpGrant, ModelGrant, ResourceRole, TokenGrants, TokenGrantsV1, TokenScope};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

async fn app(app_service_stub: AppServiceStub) -> Router {
  let app_service: Arc<dyn AppService> = Arc::new(app_service_stub);
  Router::new()
    .route("/api/tokens", post(tokens_create))
    .route("/api/tokens", get(tokens_index))
    .route(
      "/api/tokens/{token_id}",
      put(tokens_update).delete(tokens_delete),
    )
    .with_state(app_service)
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
    .build()
    .await?;

  for i in 1..=15 {
    let mut token = TokenEntity {
      id: Uuid::new_v4().to_string(),
      tenant_id: TEST_TENANT_ID.to_string(),
      user_id: user_id.to_string(),
      name: format!("Test Token {}", i),
      token_prefix: format!("bodhiapp_test{:02}", i),
      token_hash: "token_hash".to_string(),
      scopes: "scope_token_user".to_string(),
      status: TokenStatus::Active,
      grants: services::default_grants_json(),
      last_used_at: None,
      created_at: app_service.time_service().utc_now(),
      updated_at: app_service.time_service().utc_now(),
    };
    db_service
      .create_api_token(TEST_TENANT_ID, &mut token)
      .await?;
  }

  let router = app(app_service).await;

  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::GET)
        .uri("/api/tokens?page=1&page_size=10")
        .body(Body::empty())?
        .with_auth_context(AuthContext::test_session_with_token(
          &user_id,
          "user@test.com",
          ResourceRole::Admin,
          &token,
        )),
    )
    .await?;

  assert_eq!(response.status(), StatusCode::OK);
  let list_response = response.json::<PaginatedTokenResponse>().await?;
  assert_eq!(list_response.data.len(), 10);
  assert_eq!(list_response.total, 15);
  assert_eq!(list_response.page, 1);
  assert_eq!(list_response.page_size, 10);

  let response = router
    .oneshot(
      Request::builder()
        .method(Method::GET)
        .uri("/api/tokens?page=2&page_size=10")
        .body(Body::empty())?
        .with_auth_context(AuthContext::test_session_with_token(
          &user_id,
          "user@test.com",
          ResourceRole::Admin,
          &token,
        )),
    )
    .await?;

  assert_eq!(response.status(), StatusCode::OK);
  let list_response = response.json::<PaginatedTokenResponse>().await?;
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
  let claims = access_token_claims();
  let user_id = claims["sub"].as_str().unwrap().to_string();
  let (token, _) = build_token(claims)?;
  let db_service = Arc::new(db_service);
  let app_service = AppServiceStubBuilder::default()
    .db_service(db_service.clone())
    .build()
    .await?;

  let router = app(app_service).await;

  let response = router
    .oneshot(
      Request::builder()
        .method(Method::GET)
        .uri("/api/tokens")
        .body(Body::empty())?
        .with_auth_context(AuthContext::test_session_with_token(
          &user_id,
          "user@test.com",
          ResourceRole::Admin,
          &token,
        )),
    )
    .await?;

  assert_eq!(response.status(), StatusCode::OK);
  let list_response = response.json::<PaginatedTokenResponse>().await?;
  assert_eq!(list_response.data.len(), 0);
  assert_eq!(list_response.total, 0);
  assert_eq!(list_response.page, 1); // Default page
  assert_eq!(list_response.page_size, 30); // Default page size

  Ok(())
}

#[rstest]
#[case::user_role_user_scope(ResourceRole::User, TokenScope::User, "scope_token_user")]
#[case::admin_role_user_scope(ResourceRole::Admin, TokenScope::User, "scope_token_user")]
#[case::admin_role_power_user_scope(
  ResourceRole::Admin,
  TokenScope::PowerUser,
  "scope_token_power_user"
)]
#[case::manager_role_user_scope(ResourceRole::Manager, TokenScope::User, "scope_token_user")]
#[case::manager_role_power_user_scope(
  ResourceRole::Manager,
  TokenScope::PowerUser,
  "scope_token_power_user"
)]
#[case::power_user_role_user_scope(ResourceRole::PowerUser, TokenScope::User, "scope_token_user")]
#[case::power_user_role_power_user_scope(
  ResourceRole::PowerUser,
  TokenScope::PowerUser,
  "scope_token_power_user"
)]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_create_token_handler_role_scope_mapping(
  #[case] role: ResourceRole,
  #[case] requested_scope: TokenScope,
  #[case] expected_scope: &str,
  #[values("session", "multi_tenant")] auth_variant: &str,
  #[future] test_db_service: TestDbService,
) -> anyhow::Result<()> {
  let claims = access_token_claims();
  let user_id = claims["sub"].as_str().unwrap().to_string();
  let (access_token, _) = build_token(claims)?;

  let test_db_service = Arc::new(test_db_service);
  let app_service = AppServiceStubBuilder::default()
    .db_service(test_db_service.clone())
    .build()
    .await?;

  let app = app(app_service).await;

  let response = app
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/tokens")
        .json(&CreateTokenRequest {
          name: Some(format!("Test Token for {:?}", role)),
          scope: requested_scope,
          grants: Default::default(),
        })?
        .with_auth_context(make_auth_with_role(
          auth_variant,
          &user_id,
          "user@test.com",
          role,
          &access_token,
        )),
    )
    .await?;

  assert_eq!(StatusCode::CREATED, response.status());

  let token_response = response.json::<TokenCreated>().await?;
  assert!(
    token_response.token.starts_with("bodhiapp_"),
    "Token should start with 'bodhiapp_' prefix"
  );

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
  #[values("session", "multi_tenant")] auth_variant: &str,
  #[future] test_db_service: TestDbService,
) -> anyhow::Result<()> {
  let claims = access_token_claims();
  let user_id = claims["sub"].as_str().unwrap().to_string();
  let (access_token, _) = build_token(claims)?;

  let test_db_service = Arc::new(test_db_service);
  let app_service = AppServiceStubBuilder::default()
    .db_service(test_db_service.clone())
    .build()
    .await?;

  let app = app(app_service).await;

  let token_name = "Test Integration Token";
  let response = app
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/tokens")
        .json(&CreateTokenRequest {
          name: Some(token_name.to_string()),
          scope: TokenScope::User,
          grants: Default::default(),
        })?
        .with_auth_context(make_auth_with_role(
          auth_variant,
          &user_id,
          "user@test.com",
          ResourceRole::User,
          &access_token,
        )),
    )
    .await?;

  assert_eq!(StatusCode::CREATED, response.status());

  // Verify Cache-Control headers prevent proxy/browser caching of raw token (AUTH-VULN-09)
  assert_eq!(
    response.headers().get("cache-control").unwrap(),
    "no-store, no-cache, must-revalidate"
  );
  assert_eq!(response.headers().get("pragma").unwrap(), "no-cache");

  let token_response = response.json::<TokenCreated>().await?;
  let token_str = &token_response.token;

  assert!(
    token_str.starts_with("bodhiapp_"),
    "Token should start with 'bodhiapp_' prefix"
  );
  assert!(
    token_str.len() > 50,
    "Token should be sufficiently long (prefix + base64)"
  );

  let token_prefix = &token_str[.."bodhiapp_".len() + 8];

  let db_token = test_db_service
    .get_api_token_by_prefix(token_prefix)
    .await?
    .expect("Token should exist in database");

  assert_eq!(token_name, db_token.name);
  assert_eq!(user_id, db_token.user_id);
  assert_eq!(TokenStatus::Active, db_token.status);
  assert_eq!("scope_token_user", db_token.scopes);

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
  #[values("session", "multi_tenant")] auth_variant: &str,
  #[future] test_db_service: TestDbService,
) -> anyhow::Result<()> {
  let claims = access_token_claims();
  let user_id = claims["sub"].as_str().unwrap().to_string();
  let (access_token, _) = build_token(claims)?;

  let test_db_service = Arc::new(test_db_service);
  let app_service = AppServiceStubBuilder::default()
    .db_service(test_db_service.clone())
    .build()
    .await?;

  let app = app(app_service).await;

  let response = app
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/tokens")
        .json(&CreateTokenRequest {
          name: None,
          scope: TokenScope::User,
          grants: Default::default(),
        })?
        .with_auth_context(make_auth_with_role(
          auth_variant,
          &user_id,
          "user@test.com",
          ResourceRole::User,
          &access_token,
        )),
    )
    .await?;

  assert_eq!(StatusCode::CREATED, response.status());

  let token_response = response.json::<TokenCreated>().await?;
  let token_prefix = &token_response.token[.."bodhiapp_".len() + 8];

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
    .build()
    .await?;

  let app = app(app_service).await;

  let response = app
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/tokens")
        .json(&CreateTokenRequest {
          name: Some("Test Token".to_string()),
          scope: TokenScope::User,
          grants: Default::default(),
        })?
        .with_auth_context(AuthContext::Anonymous {
          deployment: services::DeploymentMode::Standalone,
        }),
    )
    .await?;

  assert_eq!(StatusCode::FORBIDDEN, response.status());

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_create_token_handler_invalid_role(
  #[future] test_db_service: TestDbService,
) -> anyhow::Result<()> {
  let user_id = Uuid::new_v4().to_string();

  let test_db_service = Arc::new(test_db_service);
  let app_service = AppServiceStubBuilder::default()
    .db_service(test_db_service.clone())
    .build()
    .await?;

  let app = app(app_service).await;

  let response = app
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/tokens")
        .json(&CreateTokenRequest {
          name: Some("Test Token".to_string()),
          scope: TokenScope::User,
          grants: Default::default(),
        })?
        .with_auth_context(AuthContext::test_session(
          &user_id,
          "user@test.com",
          services::ResourceRole::Guest,
        )),
    )
    .await?;

  assert_eq!(StatusCode::FORBIDDEN, response.status());

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_create_token_handler_missing_role(
  #[values("session", "multi_tenant")] auth_variant: &str,
  #[future] test_db_service: TestDbService,
) -> anyhow::Result<()> {
  let user_id = Uuid::new_v4().to_string();

  let test_db_service = Arc::new(test_db_service);
  let app_service = AppServiceStubBuilder::default()
    .db_service(test_db_service.clone())
    .build()
    .await?;

  let app = app(app_service).await;

  let response = app
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/tokens")
        .json(&CreateTokenRequest {
          name: Some("Test Token".to_string()),
          scope: TokenScope::User,
          grants: Default::default(),
        })?
        .with_auth_context(make_auth_no_role(auth_variant, &user_id, "user@test.com")),
    )
    .await?;

  assert_eq!(StatusCode::FORBIDDEN, response.status());

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
    .build()
    .await?;
  let now = app_service.time_service().utc_now();

  let mut token = TokenEntity {
    id: Uuid::new_v4().to_string(),
    tenant_id: TEST_TENANT_ID.to_string(),
    user_id: user_id.to_string(),
    name: "Initial Name".to_string(),
    token_prefix: "bodhiapp_test123".to_string(),
    token_hash: "token_hash".to_string(),
    scopes: "scope_token_user".to_string(),
    status: TokenStatus::Active,
    grants: services::default_grants_json(),
    last_used_at: None,
    created_at: now,
    updated_at: now,
  };
  test_db_service
    .create_api_token(TEST_TENANT_ID, &mut token)
    .await?;

  let app = app(app_service).await;

  let response = app
    .oneshot(
      Request::builder()
        .method(Method::PUT)
        .uri(format!("/api/tokens/{}", token.id))
        .json(&UpdateTokenRequest {
          name: "Updated Name".to_string(),
          status: TokenStatus::Inactive,
        })?
        .with_auth_context(AuthContext::test_session_with_token(
          &user_id,
          "user@test.com",
          ResourceRole::Admin,
          &access_token,
        )),
    )
    .await?;

  assert_eq!(response.status(), StatusCode::OK);

  let updated_token = response.json::<TokenDetail>().await?;
  assert_eq!(updated_token.name, "Updated Name");
  assert_eq!(updated_token.status, TokenStatus::Inactive);
  assert_eq!(updated_token.id, token.id);

  let db_token = test_db_service
    .get_api_token_by_id(TEST_TENANT_ID, &user_id, &token.id)
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
  let claims = access_token_claims();
  let user_id = claims["sub"].as_str().unwrap().to_string();
  let (token, _) = build_token(claims)?;
  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(test_db_service))
    .build()
    .await?;
  let app = app(app_service).await;

  let response = app
    .oneshot(
      Request::builder()
        .method(Method::PUT)
        .uri("/api/tokens/non-existent-id")
        .json(&UpdateTokenRequest {
          name: "Updated Name".to_string(),
          status: TokenStatus::Inactive,
        })?
        .with_auth_context(AuthContext::test_session_with_token(
          &user_id,
          "user@test.com",
          ResourceRole::Admin,
          &token,
        )),
    )
    .await?;

  assert_eq!(response.status(), StatusCode::NOT_FOUND);
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_create_token_persists_and_returns_grants(
  #[future] test_db_service: TestDbService,
) -> anyhow::Result<()> {
  let claims = access_token_claims();
  let user_id = claims["sub"].as_str().unwrap().to_string();
  let (token, _) = build_token(claims)?;
  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(test_db_service))
    .build()
    .await?;
  let app = app(app_service).await;
  let auth = AuthContext::test_session_with_token(
    &user_id,
    "user@test.com",
    ResourceRole::PowerUser,
    &token,
  );

  let grants = TokenGrants::V1(TokenGrantsV1 {
    list_models: false,
    models: ModelGrant::Specific {
      ids: vec!["my-model".to_string()],
    },
    list_mcps: true,
    mcps: McpGrant::Specific { ids: vec![] },
  });

  let create = app
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/tokens")
        .json(&CreateTokenRequest {
          name: Some("granted".to_string()),
          scope: TokenScope::User,
          grants: grants.clone(),
        })?
        .with_auth_context(auth.clone()),
    )
    .await?;
  assert_eq!(StatusCode::CREATED, create.status());

  // Grants are not returned on create (token-only payload); verify they round-trip via list.
  let list = app
    .oneshot(
      Request::builder()
        .method(Method::GET)
        .uri("/api/tokens?page=1&page_size=10")
        .body(Body::empty())?
        .with_auth_context(auth),
    )
    .await?;
  assert_eq!(StatusCode::OK, list.status());
  let listed = list.json::<PaginatedTokenResponse>().await?;
  let detail = listed
    .data
    .iter()
    .find(|t| t.name == "granted")
    .expect("created token present in list");
  assert_eq!(grants, detail.grants);

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_delete_token_handler(#[future] test_db_service: TestDbService) -> anyhow::Result<()> {
  let claims = access_token_claims();
  let user_id = claims["sub"].as_str().unwrap().to_string();
  let (token, _) = build_token(claims)?;
  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(test_db_service))
    .build()
    .await?;
  let app = app(app_service).await;
  let auth =
    AuthContext::test_session_with_token(&user_id, "user@test.com", ResourceRole::Admin, &token);

  let create = app
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/tokens")
        .json(&CreateTokenRequest {
          name: Some("to-delete".to_string()),
          scope: TokenScope::User,
          grants: Default::default(),
        })?
        .with_auth_context(auth.clone()),
    )
    .await?;
  assert_eq!(StatusCode::CREATED, create.status());

  let list = app
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::GET)
        .uri("/api/tokens?page=1&page_size=10")
        .body(Body::empty())?
        .with_auth_context(auth.clone()),
    )
    .await?;
  let listed = list.json::<PaginatedTokenResponse>().await?;
  let id = listed
    .data
    .iter()
    .find(|t| t.name == "to-delete")
    .expect("created token present")
    .id
    .clone();

  let del = app
    .clone()
    .oneshot(
      Request::builder()
        .method(Method::DELETE)
        .uri(format!("/api/tokens/{id}"))
        .body(Body::empty())?
        .with_auth_context(auth.clone()),
    )
    .await?;
  assert_eq!(StatusCode::NO_CONTENT, del.status());

  // Deleting again is a 404 — the row is gone.
  let del_again = app
    .oneshot(
      Request::builder()
        .method(Method::DELETE)
        .uri(format!("/api/tokens/{id}"))
        .body(Body::empty())?
        .with_auth_context(auth),
    )
    .await?;
  assert_eq!(StatusCode::NOT_FOUND, del_again.status());

  Ok(())
}
