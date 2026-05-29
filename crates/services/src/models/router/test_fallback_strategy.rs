use crate::ai_apis::ai_api_client::MockAiApiClient;
use crate::ai_apis::AiApiClientFactoryError;
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
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
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
    .body(Body::from(format!("{{\"status\":{}}}", status)))
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
  router_with(targets, FallbackConfig::default())
}

fn router_with(targets: Vec<RouterTarget>, strategy: FallbackConfig) -> ModelRouterAlias {
  ModelRouterAlias {
    id: "router-1".to_string(),
    alias: "my-stack".to_string(),
    targets,
    strategy: RoutingStrategyConfig::Fallback(strategy),
    created_at: chrono::Utc::now(),
    updated_at: chrono::Utc::now(),
  }
}

/// Per-target upstream behavior: an HTTP status to return, or a transport-style
/// `Err` from the forward call.
#[derive(Clone)]
enum Upstream {
  Status(u16),
  ForwardErr,
}

/// Builds a context whose alias list is `aliases` and whose forwarded request
/// yields a per-alias `Upstream` outcome (keyed by the inner alias name). The
/// returned counter records how many forward calls were made.
fn ctx_with_outcomes(
  aliases: Vec<Alias>,
  outcomes: HashMap<String, Upstream>,
) -> (RouterContext, Arc<AtomicUsize>) {
  let mut data = MockDataService::new();
  data
    .expect_list_aliases()
    .returning(move |_, _| Ok(aliases.clone()));

  let calls = Arc::new(AtomicUsize::new(0));
  let calls_for_factory = calls.clone();
  let mut factory = MockAiApiClientFactory::new();
  factory.expect_for_alias().returning(move |alias, _| {
    let name = alias.alias_name().to_string();
    let outcome = outcomes.get(&name).cloned();
    let calls = calls_for_factory.clone();
    let mut client = MockAiApiClient::new();
    client
      .expect_forward_request_with_method()
      .returning(move |_, _, _, _, _| {
        calls.fetch_add(1, Ordering::SeqCst);
        match outcome.clone() {
          Some(Upstream::Status(s)) => Ok(resp(s)),
          Some(Upstream::ForwardErr) | None => {
            Err(AiApiClientFactoryError::ApiError("boom".to_string()))
          }
        }
      });
    Ok(Box::new(client))
  });

  let ctx = RouterContext {
    tenant_id: "t1".to_string(),
    user_id: "u1".to_string(),
    request: json!({"model": "my-stack", "messages": [{"role": "user", "content": "hi"}]}),
    query_params: None,
    data_service: Arc::new(data) as Arc<dyn DataService>,
    db_service: Arc::new(MockDbService::new()) as Arc<dyn DbService>,
    ai_api: Arc::new(factory),
  };
  (ctx, calls)
}

/// Convenience: every forwarded request yields the same status.
fn ctx_with(aliases: Vec<Alias>, upstream_status: u16) -> RouterContext {
  let outcomes = aliases
    .iter()
    .map(|a| {
      (
        a.alias_name().to_string(),
        Upstream::Status(upstream_status),
      )
    })
    .collect();
  ctx_with_outcomes(aliases, outcomes).0
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
  // served by the second (first is disabled); first never counted as an attempt
  assert_eq!("local-b", resp.headers()["x-bodhi-routed-alias"]);
  assert_eq!("1", resp.headers()["x-bodhi-router-attempts"]);
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
async fn test_fallback_falls_through_to_secondary_on_retryable() -> anyhow::Result<()> {
  let outcomes = HashMap::from([
    ("local-a".to_string(), Upstream::Status(503)),
    ("local-b".to_string(), Upstream::Status(200)),
  ]);
  let (ctx, calls) = ctx_with_outcomes(
    vec![model_alias("local-a"), model_alias("local-b")],
    outcomes,
  );
  let router = router(vec![target("local-a", true), target("local-b", true)]);
  let resp = route_chat_completion(&router, &ctx).await?;
  assert_eq!(StatusCode::OK, resp.status());
  assert_eq!("local-b", resp.headers()["x-bodhi-routed-alias"]);
  assert_eq!("2", resp.headers()["x-bodhi-router-attempts"]);
  assert_eq!(2, calls.load(Ordering::SeqCst));
  Ok(())
}

#[rstest]
#[case(401)]
#[case(403)]
#[case(404)]
#[case(408)]
#[case(429)]
#[case(500)]
#[case(502)]
#[case(503)]
#[case(504)]
#[tokio::test]
async fn test_fallback_retryable_status_falls_through(#[case] status: u16) -> anyhow::Result<()> {
  let outcomes = HashMap::from([
    ("local-a".to_string(), Upstream::Status(status)),
    ("local-b".to_string(), Upstream::Status(200)),
  ]);
  let (ctx, _) = ctx_with_outcomes(
    vec![model_alias("local-a"), model_alias("local-b")],
    outcomes,
  );
  let router = router(vec![target("local-a", true), target("local-b", true)]);
  let resp = route_chat_completion(&router, &ctx).await?;
  assert_eq!(StatusCode::OK, resp.status());
  assert_eq!("local-b", resp.headers()["x-bodhi-routed-alias"]);
  Ok(())
}

#[rstest]
#[tokio::test]
async fn test_fallback_transport_error_falls_through() -> anyhow::Result<()> {
  let outcomes = HashMap::from([
    ("local-a".to_string(), Upstream::ForwardErr),
    ("local-b".to_string(), Upstream::Status(200)),
  ]);
  let (ctx, _) = ctx_with_outcomes(
    vec![model_alias("local-a"), model_alias("local-b")],
    outcomes,
  );
  let router = router(vec![target("local-a", true), target("local-b", true)]);
  let resp = route_chat_completion(&router, &ctx).await?;
  assert_eq!(StatusCode::OK, resp.status());
  assert_eq!("local-b", resp.headers()["x-bodhi-routed-alias"]);
  Ok(())
}

#[rstest]
#[case(400)]
#[case(422)]
#[tokio::test]
async fn test_fallback_terminal_status_returns_verbatim(#[case] status: u16) -> anyhow::Result<()> {
  let outcomes = HashMap::from([
    ("local-a".to_string(), Upstream::Status(status)),
    ("local-b".to_string(), Upstream::Status(200)),
  ]);
  let (ctx, calls) = ctx_with_outcomes(
    vec![model_alias("local-a"), model_alias("local-b")],
    outcomes,
  );
  let router = router(vec![target("local-a", true), target("local-b", true)]);
  let resp = route_chat_completion(&router, &ctx).await?;
  assert_eq!(status, resp.status().as_u16());
  assert_eq!("local-a", resp.headers()["x-bodhi-routed-alias"]);
  assert_eq!("1", resp.headers()["x-bodhi-router-attempts"]);
  // secondary never tried
  assert_eq!(1, calls.load(Ordering::SeqCst));
  Ok(())
}

#[rstest]
#[tokio::test]
async fn test_fallback_all_retryable_returns_last_verbatim() -> anyhow::Result<()> {
  let outcomes = HashMap::from([
    ("local-a".to_string(), Upstream::Status(503)),
    ("local-b".to_string(), Upstream::Status(502)),
    ("local-c".to_string(), Upstream::Status(500)),
  ]);
  let (ctx, calls) = ctx_with_outcomes(
    vec![
      model_alias("local-a"),
      model_alias("local-b"),
      model_alias("local-c"),
    ],
    outcomes,
  );
  let router = router(vec![
    target("local-a", true),
    target("local-b", true),
    target("local-c", true),
  ]);
  let resp = route_chat_completion(&router, &ctx).await?;
  // last upstream response returned verbatim
  assert_eq!(StatusCode::INTERNAL_SERVER_ERROR, resp.status());
  assert_eq!("local-c", resp.headers()["x-bodhi-routed-alias"]);
  assert_eq!("3", resp.headers()["x-bodhi-router-attempts"]);
  assert_eq!(3, calls.load(Ordering::SeqCst));
  Ok(())
}

#[rstest]
#[tokio::test]
async fn test_fallback_dangling_reference_skipped() -> anyhow::Result<()> {
  // local-a is absent from list_aliases (dangling); local-b serves.
  let outcomes = HashMap::from([("local-b".to_string(), Upstream::Status(200))]);
  let (ctx, _) = ctx_with_outcomes(vec![model_alias("local-b")], outcomes);
  let router = router(vec![target("local-a", true), target("local-b", true)]);
  let resp = route_chat_completion(&router, &ctx).await?;
  assert_eq!(StatusCode::OK, resp.status());
  assert_eq!("local-b", resp.headers()["x-bodhi-routed-alias"]);
  Ok(())
}

#[rstest]
#[tokio::test]
async fn test_fallback_transport_error_only_surfaces_typed_error() -> anyhow::Result<()> {
  let outcomes = HashMap::from([("local-a".to_string(), Upstream::ForwardErr)]);
  let (ctx, _) = ctx_with_outcomes(vec![model_alias("local-a")], outcomes);
  let router = router(vec![target("local-a", true)]);
  let err = route_chat_completion(&router, &ctx).await.unwrap_err();
  assert!(matches!(err, ModelRouterError::Forward(_)));
  Ok(())
}

#[rstest]
#[tokio::test]
async fn test_fallback_dangling_only_surfaces_typed_error() -> anyhow::Result<()> {
  // First enabled target's referenced alias no longer exists; no other target.
  let (ctx, _) = ctx_with_outcomes(vec![], HashMap::new());
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
async fn test_fallback_disabled_not_counted_as_attempt() -> anyhow::Result<()> {
  // [disabled, enabled->503, enabled->200] => attempts = 2
  let outcomes = HashMap::from([
    ("local-b".to_string(), Upstream::Status(503)),
    ("local-c".to_string(), Upstream::Status(200)),
  ]);
  let (ctx, calls) = ctx_with_outcomes(
    vec![
      model_alias("local-a"),
      model_alias("local-b"),
      model_alias("local-c"),
    ],
    outcomes,
  );
  let router = router(vec![
    target("local-a", false),
    target("local-b", true),
    target("local-c", true),
  ]);
  let resp = route_chat_completion(&router, &ctx).await?;
  assert_eq!(StatusCode::OK, resp.status());
  assert_eq!("local-c", resp.headers()["x-bodhi-routed-alias"]);
  assert_eq!("2", resp.headers()["x-bodhi-router-attempts"]);
  assert_eq!(2, calls.load(Ordering::SeqCst));
  Ok(())
}

#[rstest]
#[tokio::test]
async fn test_fallback_max_attempts_caps_chain() -> anyhow::Result<()> {
  // chain of 3 all 503, max_attempts = 2 => stop after 2, return 2nd response.
  let outcomes = HashMap::from([
    ("local-a".to_string(), Upstream::Status(503)),
    ("local-b".to_string(), Upstream::Status(502)),
    ("local-c".to_string(), Upstream::Status(500)),
  ]);
  let (ctx, calls) = ctx_with_outcomes(
    vec![
      model_alias("local-a"),
      model_alias("local-b"),
      model_alias("local-c"),
    ],
    outcomes,
  );
  let strategy = FallbackConfig {
    max_attempts: 2,
    ..FallbackConfig::default()
  };
  let router = router_with(
    vec![
      target("local-a", true),
      target("local-b", true),
      target("local-c", true),
    ],
    strategy,
  );
  let resp = route_chat_completion(&router, &ctx).await?;
  assert_eq!(StatusCode::BAD_GATEWAY, resp.status());
  assert_eq!("local-b", resp.headers()["x-bodhi-routed-alias"]);
  assert_eq!("2", resp.headers()["x-bodhi-router-attempts"]);
  // third target never forwarded
  assert_eq!(2, calls.load(Ordering::SeqCst));
  Ok(())
}

#[rstest]
#[tokio::test]
async fn test_fallback_nested_router_reference_skipped_then_typed() -> anyhow::Result<()> {
  let nested = Alias::ModelRouter(router(vec![]));
  let (ctx, _) = ctx_with_outcomes(vec![nested], HashMap::new());
  // target "my-stack" resolves to a model-router -> nested not allowed
  let r = router(vec![target("my-stack", true)]);
  let err = route_chat_completion(&r, &ctx).await.unwrap_err();
  assert!(matches!(
    err,
    ModelRouterError::NestedRouterNotAllowed { .. }
  ));
  Ok(())
}
