use crate::test_utils::RequestAuthContextExt;
use crate::{models_copy, models_destroy, models_index, models_show};
use anyhow_trace::anyhow_trace;
use axum::{
  body::Body,
  http::Request,
  routing::{get, post},
  Router,
};
use rstest::rstest;
use server_core::test_utils::ResponseTestExt;
use services::{
  test_utils::{anthropic_model, AppServiceStubBuilder, TEST_TENANT_ID},
  AppService, AuthContext, ResourceRole,
};
use services::{AliasResponse, ApiAlias, ApiFormat, PaginatedAliasResponse};
use std::sync::Arc;
use tower::ServiceExt;

fn test_router(app_service: Arc<dyn services::AppService>) -> Router {
  Router::new()
    .route("/api/models", get(models_index))
    .route("/api/models/{id}", get(models_show).delete(models_destroy))
    .route("/api/models/{id}/copy", post(models_copy))
    .with_state(app_service)
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_models_index_includes_anthropic_oauth_alias() -> anyhow::Result<()> {
  let service = AppServiceStubBuilder::default()
    .with_data_service()
    .await
    .with_db_service()
    .await
    .build()
    .await?;
  let db_service = service.db_service();

  let api_alias = ApiAlias::new(
    "anthropic-oauth-alias",
    ApiFormat::AnthropicOAuth,
    "https://api.anthropic.com",
    vec![anthropic_model("claude-sonnet-4-5-20250929")],
    Some("anthropic/".to_string()),
    false,
    db_service.now(),
    None,
    None,
  );
  db_service
    .create_api_model_alias(TEST_TENANT_ID, "test-user", &api_alias, None)
    .await?;

  let app_service: Arc<dyn services::AppService> = Arc::new(service);
  let response = test_router(app_service)
    .oneshot(
      Request::get("/api/models")
        .body(Body::empty())?
        .with_auth_context(AuthContext::test_session(
          "test-user",
          "testuser",
          ResourceRole::User,
        )),
    )
    .await?
    .json::<PaginatedAliasResponse>()
    .await?;

  let api_alias_found = response.data.iter().any(|alias| match alias {
    AliasResponse::Api(a) => a.id == "anthropic-oauth-alias",
    _ => false,
  });
  assert!(
    api_alias_found,
    "anthropic_oauth alias should appear in /bodhi/v1/models response"
  );
  Ok(())
}
