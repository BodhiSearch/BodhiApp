use crate::ai_apis::ai_api_client::MockAiApiClient;
use crate::models::router::{route_chat_completion, ModelRouterError, RouterContext};
use crate::models::{
  Alias, FallbackConfig, ModelAlias, ModelRouterAlias, Repo, RouterTarget, RoutingStrategyConfig,
};
use crate::test_utils::MockDbService;
use crate::{DataService, DbService, MockAiApiClientFactory, MockDataService};
use axum::body::Body;
use axum::http::StatusCode;
use axum::response::Response;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serde_json::json;
use std::sync::Arc;

fn model_alias(name: &str) -> Alias {
  Alias::Model(ModelAlias {
    alias: name.to_string(),
    repo: Repo::new("meta", "llama"),
    filename: "model.gguf".to_string(),
    snapshot: "main".to_string(),
  })
}

fn resp(status: u16) -> Response {
  Response::builder()
    .status(status)
    .body(Body::from("{\"ok\":true}"))
    .unwrap()
}

fn target(alias: &str, enabled: bool) -> RouterTarget {
  RouterTarget {
    alias: alias.to_string(),
    model: alias.to_string(),
    enabled,
    weight: None,
  }
}

fn router(targets: Vec<RouterTarget>) -> ModelRouterAlias {
  ModelRouterAlias {
    id: "router-1".to_string(),
    alias: "my-stack".to_string(),
    targets,
    strategy: RoutingStrategyConfig::Fallback(FallbackConfig::default()),
    created_at: chrono::Utc::now(),
    updated_at: chrono::Utc::now(),
  }
}

/// Builds a context whose alias list is `aliases` and whose every forwarded request
/// yields an upstream response with `upstream_status`.
fn ctx_with(aliases: Vec<Alias>, upstream_status: u16) -> RouterContext {
  let mut data = MockDataService::new();
  data
    .expect_list_aliases()
    .returning(move |_, _| Ok(aliases.clone()));
  let mut factory = MockAiApiClientFactory::new();
  factory.expect_for_alias().returning(move |_, _| {
    let mut client = MockAiApiClient::new();
    client
      .expect_forward_request_with_method()
      .returning(move |_, _, _, _, _| Ok(resp(upstream_status)));
    Ok(Box::new(client))
  });
  RouterContext {
    tenant_id: "t1".to_string(),
    user_id: "u1".to_string(),
    request: json!({"model": "my-stack", "messages": [{"role": "user", "content": "hi"}]}),
    query_params: None,
    data_service: Arc::new(data) as Arc<dyn DataService>,
    db_service: Arc::new(MockDbService::new()) as Arc<dyn DbService>,
    ai_api: Arc::new(factory),
  }
}

#[rstest]
#[tokio::test]
async fn test_fallback_serves_first_enabled_target() -> anyhow::Result<()> {
  let ctx = ctx_with(vec![model_alias("local-a"), model_alias("local-b")], 200);
  let router = router(vec![target("local-a", true), target("local-b", true)]);
  let resp = route_chat_completion(&router, &ctx).await?;
  assert_eq!(StatusCode::OK, resp.status());
  let headers = resp.headers();
  assert_eq!("local-a", headers["x-bodhi-routed-alias"]);
  assert_eq!("local-a", headers["x-bodhi-routed-model"]);
  assert_eq!("fallback", headers["x-bodhi-router-strategy"]);
  assert_eq!("1", headers["x-bodhi-router-attempts"]);
  Ok(())
}

#[rstest]
#[tokio::test]
async fn test_fallback_skips_disabled_first_target() -> anyhow::Result<()> {
  let ctx = ctx_with(vec![model_alias("local-a"), model_alias("local-b")], 200);
  let router = router(vec![target("local-a", false), target("local-b", true)]);
  let resp = route_chat_completion(&router, &ctx).await?;
  assert_eq!(StatusCode::OK, resp.status());
  // served by the second (first is disabled)
  assert_eq!("local-b", resp.headers()["x-bodhi-routed-alias"]);
  Ok(())
}

#[rstest]
#[tokio::test]
async fn test_fallback_all_disabled_returns_empty_chain() -> anyhow::Result<()> {
  let ctx = ctx_with(vec![model_alias("local-a")], 200);
  let router = router(vec![target("local-a", false)]);
  let err = route_chat_completion(&router, &ctx).await.unwrap_err();
  assert!(matches!(err, ModelRouterError::EmptyChain));
  Ok(())
}

#[rstest]
#[tokio::test]
async fn test_fallback_returns_upstream_error_verbatim() -> anyhow::Result<()> {
  // Phase 1: no fall-through. An upstream 429 is returned verbatim (not an error).
  let ctx = ctx_with(vec![model_alias("local-a")], 429);
  let router = router(vec![target("local-a", true)]);
  let resp = route_chat_completion(&router, &ctx).await?;
  assert_eq!(StatusCode::TOO_MANY_REQUESTS, resp.status());
  assert_eq!("local-a", resp.headers()["x-bodhi-routed-alias"]);
  Ok(())
}

#[rstest]
#[tokio::test]
async fn test_fallback_dangling_reference_surfaces_typed_error() -> anyhow::Result<()> {
  // First enabled target's referenced alias no longer exists.
  let ctx = ctx_with(vec![], 200);
  let router = router(vec![target("local-a", true)]);
  let err = route_chat_completion(&router, &ctx).await.unwrap_err();
  assert!(matches!(
    err,
    ModelRouterError::ReferencedAliasNotFound { .. }
  ));
  Ok(())
}

#[rstest]
#[tokio::test]
async fn test_fallback_nested_router_reference_rejected() -> anyhow::Result<()> {
  let nested = Alias::ModelRouter(router(vec![]));
  let ctx = ctx_with(vec![nested], 200);
  // target "my-stack" resolves to a model-router -> nested not allowed
  let r = router(vec![target("my-stack", true)]);
  let err = route_chat_completion(&r, &ctx).await.unwrap_err();
  assert!(matches!(
    err,
    ModelRouterError::NestedRouterNotAllowed { .. }
  ));
  Ok(())
}
