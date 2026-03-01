use crate::{
  create_token_handler, list_tokens_handler, update_token_handler, CreateApiTokenRequest,
};
use anyhow_trace::anyhow_trace;
use auth_middleware::{test_utils::RequestAuthContextExt, AuthContext};
use axum::{
  http::{Method, Request},
  routing::{get, post, put},
  Router,
};
use hyper::StatusCode;
use pretty_assertions::assert_eq;
use rstest::rstest;
use server_core::{test_utils::RequestTestExt, DefaultRouterState, MockSharedContext};
use services::test_utils::{
  access_token_claims, build_token, test_db_service, AppServiceStub, AppServiceStubBuilder,
  TestDbService,
};
use services::{ResourceRole, TokenScope};
use std::sync::Arc;
use tower::ServiceExt;

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
#[case::user_to_power_user(ResourceRole::User, TokenScope::PowerUser)]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_create_token_privilege_escalation_user(
  #[case] role: ResourceRole,
  #[case] requested_scope: TokenScope,
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

  // User attempting to create higher-privilege token
  let response = app
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/tokens")
        .json(&CreateApiTokenRequest {
          name: Some("Escalation Attempt".to_string()),
          scope: requested_scope,
        })?
        .with_auth_context(AuthContext::test_session_with_token(
          &user_id,
          "user@test.com",
          role,
          &access_token,
        )),
    )
    .await?;

  assert_eq!(StatusCode::BAD_REQUEST, response.status());

  Ok(())
}
