use crate::test_utils::RequestAuthContextExt;
use crate::{anthropic_models_get_handler, anthropic_models_list_handler};
use anyhow_trace::anyhow_trace;
use axum::{extract::Request, routing::get, Router};
use pretty_assertions::assert_eq;
use reqwest::StatusCode;
use rstest::rstest;
use server_core::test_utils::ResponseTestExt;
use services::{
  test_utils::{
    anthropic_model, openai_model, AppServiceStubBuilder, TEST_TENANT_ID, TEST_USER_ID,
  },
  ApiAliasBuilder, ApiFormat, AuthContext, ResourceRole,
};
use std::sync::Arc;
use tower::ServiceExt;

async fn seed_anthropic_alias(
  builder: &mut AppServiceStubBuilder,
) -> anyhow::Result<services::ApiAlias> {
  builder.with_data_service().await;
  let db_service = builder.get_db_service().await;
  let api_alias = ApiAliasBuilder::test_default()
    .id("anthropic-alias")
    .api_format(ApiFormat::Anthropic)
    .base_url("https://api.anthropic.com/v1")
    .models(vec![anthropic_model("claude-sonnet-4-5-20250929")])
    .build_with_time(db_service.now())
    .unwrap();
  db_service
    .create_api_model_alias(TEST_TENANT_ID, TEST_USER_ID, &api_alias, None)
    .await?;
  Ok(api_alias)
}

// ============================================================================
// Models list — DB aggregation
// ============================================================================

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_models_list_empty_when_no_anthropic_aliases() -> anyhow::Result<()> {
  let app_service = AppServiceStubBuilder::default()
    .with_data_service()
    .await
    .build()
    .await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route("/anthropic/v1/models", get(anthropic_models_list_handler))
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::get("/anthropic/v1/models")
        .body(axum::body::Body::empty())?
        .with_auth_context(AuthContext::test_session(
          TEST_USER_ID,
          "testuser",
          ResourceRole::User,
        )),
    )
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  let body = response.json::<serde_json::Value>().await?;
  assert_eq!(0, body["data"].as_array().unwrap().len());
  assert_eq!(false, body["has_more"].as_bool().unwrap());
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_models_list_aggregates_from_all_anthropic_aliases() -> anyhow::Result<()> {
  let mut builder = AppServiceStubBuilder::default();
  builder.with_data_service().await;
  let db_service = builder.get_db_service().await;

  // Seed two Anthropic aliases with overlapping models to exercise dedup.
  let alias_a = ApiAliasBuilder::test_default()
    .id("anthropic-a")
    .api_format(ApiFormat::Anthropic)
    .base_url("https://api.anthropic.com/v1")
    .models(vec![
      anthropic_model("claude-sonnet-4-5-20250929"),
      anthropic_model("claude-opus-4-5-20251101"),
    ])
    .build_with_time(db_service.now())
    .unwrap();
  let alias_b = ApiAliasBuilder::test_default()
    .id("anthropic-b")
    .api_format(ApiFormat::Anthropic)
    .base_url("https://api.anthropic.com/v1")
    .models(vec![
      anthropic_model("claude-sonnet-4-5-20250929"), // duplicate
      anthropic_model("claude-haiku-4-5-20251001"),
    ])
    .build_with_time(db_service.now())
    .unwrap();
  db_service
    .create_api_model_alias(TEST_TENANT_ID, TEST_USER_ID, &alias_a, None)
    .await?;
  db_service
    .create_api_model_alias(TEST_TENANT_ID, TEST_USER_ID, &alias_b, None)
    .await?;

  let app_service = builder.build().await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route("/anthropic/v1/models", get(anthropic_models_list_handler))
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::get("/anthropic/v1/models")
        .body(axum::body::Body::empty())?
        .with_auth_context(AuthContext::test_session(
          TEST_USER_ID,
          "testuser",
          ResourceRole::User,
        )),
    )
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  let body = response.json::<serde_json::Value>().await?;
  let data = body["data"].as_array().unwrap();
  assert_eq!(3, data.len()); // 3 unique models
  let ids: Vec<&str> = data.iter().map(|m| m["id"].as_str().unwrap()).collect();
  assert!(ids.contains(&"claude-sonnet-4-5-20250929"));
  assert!(ids.contains(&"claude-opus-4-5-20251101"));
  assert!(ids.contains(&"claude-haiku-4-5-20251001"));
  // All entries carry type: "model"
  for item in data {
    assert_eq!("model", item["type"].as_str().unwrap());
  }
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_models_list_excludes_non_anthropic_aliases() -> anyhow::Result<()> {
  let mut builder = AppServiceStubBuilder::default();
  builder.with_data_service().await;
  let db_service = builder.get_db_service().await;

  // One Anthropic alias and one OpenAI alias owned by the same user.
  let anthropic_alias = ApiAliasBuilder::test_default()
    .id("anthropic-alias")
    .api_format(ApiFormat::Anthropic)
    .base_url("https://api.anthropic.com/v1")
    .models(vec![anthropic_model("claude-sonnet-4-5-20250929")])
    .build_with_time(db_service.now())
    .unwrap();
  let openai_alias = ApiAliasBuilder::test_default()
    .id("openai-alias")
    .api_format(ApiFormat::OpenAI)
    .base_url("https://api.openai.com/v1")
    .models(vec![openai_model("gpt-4o")])
    .build_with_time(db_service.now())
    .unwrap();
  db_service
    .create_api_model_alias(TEST_TENANT_ID, TEST_USER_ID, &anthropic_alias, None)
    .await?;
  db_service
    .create_api_model_alias(TEST_TENANT_ID, TEST_USER_ID, &openai_alias, None)
    .await?;

  let app_service = builder.build().await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route("/anthropic/v1/models", get(anthropic_models_list_handler))
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::get("/anthropic/v1/models")
        .body(axum::body::Body::empty())?
        .with_auth_context(AuthContext::test_session(
          TEST_USER_ID,
          "testuser",
          ResourceRole::User,
        )),
    )
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  let body = response.json::<serde_json::Value>().await?;
  let ids: Vec<&str> = body["data"]
    .as_array()
    .unwrap()
    .iter()
    .map(|m| m["id"].as_str().unwrap())
    .collect();
  assert!(ids.contains(&"claude-sonnet-4-5-20250929"));
  assert!(!ids.contains(&"gpt-4o"));
  Ok(())
}

// ============================================================================
// Single model GET — served from local cache
// ============================================================================

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_models_get_by_path_param_returns_cached_model() -> anyhow::Result<()> {
  let mut builder = AppServiceStubBuilder::default();
  seed_anthropic_alias(&mut builder).await?;

  let app_service = builder.build().await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route(
      "/anthropic/v1/models/{model_id}",
      get(anthropic_models_get_handler),
    )
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::get("/anthropic/v1/models/claude-sonnet-4-5-20250929")
        .body(axum::body::Body::empty())?
        .with_auth_context(AuthContext::test_session(
          TEST_USER_ID,
          "testuser",
          ResourceRole::User,
        )),
    )
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  let body = response.json::<serde_json::Value>().await?;
  assert_eq!("claude-sonnet-4-5-20250929", body["id"].as_str().unwrap());
  assert_eq!("model", body["type"].as_str().unwrap());
  assert_eq!(
    "claude-sonnet-4-5-20250929",
    body["display_name"].as_str().unwrap()
  );
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_models_get_invalid_model_id_slash() -> anyhow::Result<()> {
  // Model ids with characters outside [a-zA-Z0-9._-] are rejected by
  // validate_model_id (path-parameter safety, not body validation).
  // We use a percent-encoded space + bang so the router still dispatches to
  // the handler instead of a 404.
  let app_service = AppServiceStubBuilder::default()
    .with_data_service()
    .await
    .build()
    .await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route(
      "/anthropic/v1/models/{model_id}",
      get(anthropic_models_get_handler),
    )
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::get("/anthropic/v1/models/bad%20id%21")
        .body(axum::body::Body::empty())?
        .with_auth_context(AuthContext::test_session(
          TEST_USER_ID,
          "testuser",
          ResourceRole::User,
        )),
    )
    .await?;

  assert_eq!(StatusCode::BAD_REQUEST, response.status());
  let body = response.json::<serde_json::Value>().await?;
  assert_eq!("error", body["type"].as_str().unwrap());
  assert_eq!(
    "invalid_request_error",
    body["error"]["type"].as_str().unwrap()
  );
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_models_get_valid_id_with_dots_and_hyphens() -> anyhow::Result<()> {
  // Model ids with dots (like claude-3.5-sonnet) must be accepted.
  let mut builder = AppServiceStubBuilder::default();
  builder.with_data_service().await;
  let db_service = builder.get_db_service().await;
  let api_alias = ApiAliasBuilder::test_default()
    .id("anthropic-alias")
    .api_format(ApiFormat::Anthropic)
    .base_url("https://api.anthropic.com/v1")
    .models(vec![anthropic_model("claude-3.5-sonnet")])
    .build_with_time(db_service.now())
    .unwrap();
  db_service
    .create_api_model_alias(TEST_TENANT_ID, TEST_USER_ID, &api_alias, None)
    .await?;

  let app_service = builder.build().await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route(
      "/anthropic/v1/models/{model_id}",
      get(anthropic_models_get_handler),
    )
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::get("/anthropic/v1/models/claude-3.5-sonnet")
        .body(axum::body::Body::empty())?
        .with_auth_context(AuthContext::test_session(
          TEST_USER_ID,
          "testuser",
          ResourceRole::User,
        )),
    )
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  let body = response.json::<serde_json::Value>().await?;
  assert_eq!("claude-3.5-sonnet", body["id"].as_str().unwrap());
  Ok(())
}
