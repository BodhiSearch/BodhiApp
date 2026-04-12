use crate::test_utils::RequestAuthContextExt;
use crate::{models_copy, models_destroy, models_index, models_show};
use anyhow_trace::anyhow_trace;
use axum::{
  body::Body,
  http::Request,
  routing::{get, post},
  Router,
};
use pretty_assertions::assert_eq;
use rstest::rstest;
use serde_json::Value;
use server_core::test_utils::{router_state_stub, ResponseTestExt};
use services::{AliasResponse, PaginatedAliasResponse};
use services::{AuthContext, ResourceRole};
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
async fn test_list_local_aliases_handler(
  #[future] router_state_stub: Arc<dyn services::AppService>,
) -> anyhow::Result<()> {
  let response = test_router(router_state_stub)
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
    .json::<Value>()
    .await?;
  assert_eq!(1, response["page"]);
  assert_eq!(30, response["page_size"]);
  assert_eq!(8, response["total"]);
  let data = response["data"].as_array().unwrap();
  assert!(!data.is_empty());
  assert_eq!(
    "FakeFactory/fakemodel-gguf:Q4_0",
    data.first().unwrap()["alias"].as_str().unwrap(),
  );
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_list_local_aliases_page_size(
  #[future] router_state_stub: Arc<dyn services::AppService>,
) -> anyhow::Result<()> {
  let response = test_router(router_state_stub)
    .oneshot(
      Request::get("/api/models?page=2&page_size=4")
        .body(Body::empty())?
        .with_auth_context(AuthContext::test_session(
          "test-user",
          "testuser",
          ResourceRole::User,
        )),
    )
    .await?
    .json::<Value>()
    .await?;
  assert_eq!(2, response["page"]);
  assert_eq!(4, response["page_size"]);
  assert_eq!(8, response["total"]);
  let data = response["data"].as_array().unwrap();
  assert_eq!(4, data.len());
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_list_local_aliases_over_limit_page_size(
  #[future] router_state_stub: Arc<dyn services::AppService>,
) -> anyhow::Result<()> {
  let response = test_router(router_state_stub)
    .oneshot(
      Request::get("/api/models?page_size=150")
        .body(Body::empty())?
        .with_auth_context(AuthContext::test_session(
          "test-user",
          "testuser",
          ResourceRole::User,
        )),
    )
    .await?
    .json::<Value>()
    .await?;

  assert_eq!(100, response["page_size"]);
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_list_local_aliases_response_structure(
  #[future] router_state_stub: Arc<dyn services::AppService>,
) -> anyhow::Result<()> {
  let response = test_router(router_state_stub)
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

  assert!(!response.data.is_empty());
  let user_alias_response = response
    .data
    .iter()
    .find_map(|alias| match alias {
      AliasResponse::User(user_alias) if user_alias.alias == "llama3:instruct" => Some(user_alias),
      _ => None,
    })
    .unwrap();
  assert_eq!("llama3:instruct", user_alias_response.alias);
  assert_eq!(
    "QuantFactory/Meta-Llama-3-8B-Instruct-GGUF",
    user_alias_response.repo
  );
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_list_local_aliases_sorting(
  #[future] router_state_stub: Arc<dyn services::AppService>,
) -> anyhow::Result<()> {
  let response = test_router(router_state_stub)
    .oneshot(
      Request::get("/api/models?sort=repo&sort_order=desc")
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

  assert!(!response.data.is_empty());
  Ok(())
}
