use crate::ai_apis::ai_api_client::MockAiApiClient;
use crate::ai_apis::AiApiClientFactoryError;
use crate::db::TimeService;
use crate::models::router::{
  route_chat_completion, DefaultHealthRegistry, HealthRegistry, ModelRouterError, RouterContext,
};
use crate::models::{
  Alias, FallbackConfig, ModelAlias, ModelRouterAlias, Repo, RouterTarget, RoutingStrategyConfig,
};
use crate::test_utils::{MockDbService, TestTimeService};
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
/// `Err` from the forward call. `StatusWith` carries extra response headers
/// (e.g. `Retry-After`).
#[derive(Clone)]
enum Upstream {
  Status(u16),
  StatusWith(u16, Vec<(String, String)>),
  ForwardErr,
}

/// Shared, mutable per-alias outcomes so a test can change a target's behavior
/// between requests (e.g. primary fails, then recovers).
type Outcomes = Arc<std::sync::Mutex<HashMap<String, Upstream>>>;

/// A controllable `RouterContext` plus the handles a test needs to drive it:
/// the forward-call counter, the mutable outcomes map, the shared health
/// registry, and the settable clock. The same registry/clock can be threaded
/// into a second context (via `with_shared`) to exercise cross-router sharing.
struct TestCtx {
  ctx: RouterContext,
  calls: Arc<AtomicUsize>,
  outcomes: Outcomes,
  health: Arc<DefaultHealthRegistry>,
  time: TestTimeService,
}

fn build_ctx(
  aliases: Vec<Alias>,
  outcomes: Outcomes,
  health: Arc<DefaultHealthRegistry>,
  time: TestTimeService,
) -> (RouterContext, Arc<AtomicUsize>) {
  let mut data = MockDataService::new();
  data
    .expect_list_aliases()
    .returning(move |_, _| Ok(aliases.clone()));

  let calls = Arc::new(AtomicUsize::new(0));
  let calls_for_factory = calls.clone();
  let outcomes_for_factory = outcomes.clone();
  let mut factory = MockAiApiClientFactory::new();
  factory.expect_for_alias().returning(move |alias, _| {
    let name = alias.alias_name().to_string();
    let outcome = outcomes_for_factory.lock().unwrap().get(&name).cloned();
    let calls = calls_for_factory.clone();
    let mut client = MockAiApiClient::new();
    client
      .expect_forward_request_with_method()
      .returning(move |_, _, _, _, _| {
        calls.fetch_add(1, Ordering::SeqCst);
        match outcome.clone() {
          Some(Upstream::Status(s)) => Ok(resp(s)),
          Some(Upstream::StatusWith(s, headers)) => Ok(resp_with(s, &headers)),
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
    time_service: Arc::new(time) as Arc<dyn TimeService>,
    health: health as Arc<dyn HealthRegistry>,
  };
  (ctx, calls)
}

fn resp_with(status: u16, headers: &[(String, String)]) -> Response {
  let mut builder = Response::builder().status(status);
  for (k, v) in headers {
    builder = builder.header(k, v);
  }
  builder
    .body(Body::from(format!("{{\"status\":{}}}", status)))
    .unwrap()
}

/// Builds a fresh `TestCtx` with its own health registry + clock.
fn test_ctx(aliases: Vec<Alias>, outcomes: HashMap<String, Upstream>) -> TestCtx {
  let outcomes: Outcomes = Arc::new(std::sync::Mutex::new(outcomes));
  let health = Arc::new(DefaultHealthRegistry::default());
  let time = TestTimeService::default();
  let (ctx, calls) = build_ctx(aliases, outcomes.clone(), health.clone(), time.clone());
  TestCtx {
    ctx,
    calls,
    outcomes,
    health,
    time,
  }
}

/// Builds a second `TestCtx` sharing an existing registry + clock (for
/// cross-router shared-health tests).
fn test_ctx_sharing(
  aliases: Vec<Alias>,
  outcomes: HashMap<String, Upstream>,
  health: Arc<DefaultHealthRegistry>,
  time: TestTimeService,
) -> TestCtx {
  let outcomes: Outcomes = Arc::new(std::sync::Mutex::new(outcomes));
  let (ctx, calls) = build_ctx(aliases, outcomes.clone(), health.clone(), time.clone());
  TestCtx {
    ctx,
    calls,
    outcomes,
    health,
    time,
  }
}

/// Back-compat shim for the Phase-2 tests: returns just `(ctx, calls)` with a
/// throwaway registry + clock.
fn ctx_with_outcomes(
  aliases: Vec<Alias>,
  outcomes: HashMap<String, Upstream>,
) -> (RouterContext, Arc<AtomicUsize>) {
  let tc = test_ctx(aliases, outcomes);
  (tc.ctx, tc.calls)
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

// ---- Phase 3: health-aware skipping & passive recovery ----

fn aliases_ab() -> Vec<Alias> {
  vec![model_alias("local-a"), model_alias("local-b")]
}

fn router_ab() -> ModelRouterAlias {
  router(vec![target("local-a", true), target("local-b", true)])
}

/// A retryable failure on the primary cools it; the next request skips the
/// (cooled) primary and serves from the secondary without re-hitting the primary.
#[rstest]
#[tokio::test]
async fn test_health_retryable_cools_and_skips_next_request() -> anyhow::Result<()> {
  let outcomes = HashMap::from([
    ("local-a".to_string(), Upstream::Status(503)),
    ("local-b".to_string(), Upstream::Status(200)),
  ]);
  let tc = test_ctx(aliases_ab(), outcomes);
  let r = router_ab();

  // First request: primary 503 (cooled), secondary serves. 2 forwards.
  let resp = route_chat_completion(&r, &tc.ctx).await?;
  assert_eq!("local-b", resp.headers()["x-bodhi-routed-alias"]);
  assert_eq!(2, tc.calls.load(Ordering::SeqCst));

  // Second request: primary is cooled → skipped; only secondary is forwarded.
  let resp = route_chat_completion(&r, &tc.ctx).await?;
  assert_eq!("local-b", resp.headers()["x-bodhi-routed-alias"]);
  assert_eq!("1", resp.headers()["x-bodhi-router-attempts"]);
  assert_eq!(3, tc.calls.load(Ordering::SeqCst)); // only +1
  Ok(())
}

/// After the cooldown window elapses the primary is tried again (half-open); on
/// success its health clears and it is selected first on the following request.
#[rstest]
#[tokio::test]
async fn test_health_half_open_recovers_and_returns_to_primary() -> anyhow::Result<()> {
  let outcomes = HashMap::from([
    ("local-a".to_string(), Upstream::Status(503)),
    ("local-b".to_string(), Upstream::Status(200)),
  ]);
  let tc = test_ctx(aliases_ab(), outcomes);
  let r = router_ab();

  // Cool the primary.
  route_chat_completion(&r, &tc.ctx).await?;
  // Within cooldown: primary still skipped.
  let resp = route_chat_completion(&r, &tc.ctx).await?;
  assert_eq!("local-b", resp.headers()["x-bodhi-routed-alias"]);

  // Primary recovers upstream, advance past the cooldown window.
  tc.outcomes
    .lock()
    .unwrap()
    .insert("local-a".to_string(), Upstream::Status(200));
  tc.time.advance(chrono::Duration::seconds(31));

  // Half-open trial: primary eligible again, tried first, succeeds → served.
  let resp = route_chat_completion(&r, &tc.ctx).await?;
  assert_eq!("local-a", resp.headers()["x-bodhi-routed-alias"]);

  // Following request: health cleared, primary served first directly.
  let resp = route_chat_completion(&r, &tc.ctx).await?;
  assert_eq!("local-a", resp.headers()["x-bodhi-routed-alias"]);
  assert_eq!("1", resp.headers()["x-bodhi-router-attempts"]);
  Ok(())
}

/// A half-open trial that fails re-cools the target.
#[rstest]
#[tokio::test]
async fn test_health_failed_half_open_recools() -> anyhow::Result<()> {
  let outcomes = HashMap::from([
    ("local-a".to_string(), Upstream::Status(503)),
    ("local-b".to_string(), Upstream::Status(200)),
  ]);
  let tc = test_ctx(aliases_ab(), outcomes);
  let r = router_ab();
  let key = crate::models::router::target_key("t1", "local-a", "local-a");

  route_chat_completion(&r, &tc.ctx).await?; // cool primary
  tc.time.advance(chrono::Duration::seconds(31)); // expire
  assert!(!tc.health.is_cooled(&key, tc.time.utc_now()));

  // Half-open trial fails (still 503) → re-cooled.
  let resp = route_chat_completion(&r, &tc.ctx).await?;
  assert_eq!("local-b", resp.headers()["x-bodhi-routed-alias"]);
  assert!(tc.health.is_cooled(&key, tc.time.utc_now()));
  Ok(())
}

/// `Retry-After` extends the cooldown to the larger of it and `cooldown_secs`.
#[rstest]
#[tokio::test]
async fn test_health_retry_after_extends_cooldown() -> anyhow::Result<()> {
  // cooldown_secs default 30; Retry-After 120 → cooled until now+120.
  let outcomes = HashMap::from([
    (
      "local-a".to_string(),
      Upstream::StatusWith(429, vec![("retry-after".to_string(), "120".to_string())]),
    ),
    ("local-b".to_string(), Upstream::Status(200)),
  ]);
  let tc = test_ctx(aliases_ab(), outcomes);
  let r = router_ab();
  let key = crate::models::router::target_key("t1", "local-a", "local-a");

  route_chat_completion(&r, &tc.ctx).await?;
  // At +31s (past default cooldown) it is STILL cooled because Retry-After=120.
  tc.time.advance(chrono::Duration::seconds(31));
  assert!(tc.health.is_cooled(&key, tc.time.utc_now()));
  // At +121s total it is no longer cooled.
  tc.time.advance(chrono::Duration::seconds(90));
  assert!(!tc.health.is_cooled(&key, tc.time.utc_now()));
  Ok(())
}

/// Two routers referencing the same underlying target share its cooldown: once
/// either observes the failure, both skip it.
#[rstest]
#[tokio::test]
async fn test_health_shared_across_routers() -> anyhow::Result<()> {
  let health = Arc::new(DefaultHealthRegistry::default());
  let time = TestTimeService::default();

  // Router 1: primary local-a (503), secondary local-b (200).
  let tc1 = test_ctx_sharing(
    aliases_ab(),
    HashMap::from([
      ("local-a".to_string(), Upstream::Status(503)),
      ("local-b".to_string(), Upstream::Status(200)),
    ]),
    health.clone(),
    time.clone(),
  );
  // Router 2: same primary local-a, different secondary local-c (200).
  let tc2 = test_ctx_sharing(
    vec![model_alias("local-a"), model_alias("local-c")],
    HashMap::from([
      ("local-a".to_string(), Upstream::Status(200)),
      ("local-c".to_string(), Upstream::Status(200)),
    ]),
    health.clone(),
    time.clone(),
  );
  let r1 = router_ab();
  let r2 = router(vec![target("local-a", true), target("local-c", true)]);

  // Router 1 discovers local-a is down.
  route_chat_completion(&r1, &tc1.ctx).await?;

  // Router 2 now skips local-a even though it never failed it itself → serves local-c.
  let resp = route_chat_completion(&r2, &tc2.ctx).await?;
  assert_eq!("local-c", resp.headers()["x-bodhi-routed-alias"]);
  assert_eq!("1", resp.headers()["x-bodhi-router-attempts"]);
  Ok(())
}

/// When every enabled target is cooled, the request still attempts them (ordered
/// by soonest recovery) and returns a real upstream result — never starve.
#[rstest]
#[tokio::test]
async fn test_health_never_starve_all_cooled() -> anyhow::Result<()> {
  let outcomes = HashMap::from([
    ("local-a".to_string(), Upstream::Status(503)),
    ("local-b".to_string(), Upstream::Status(500)),
  ]);
  let tc = test_ctx(aliases_ab(), outcomes);
  let r = router_ab();

  // First request cools both.
  route_chat_completion(&r, &tc.ctx).await?;
  let calls_after_first = tc.calls.load(Ordering::SeqCst);
  assert_eq!(2, calls_after_first);

  // Second request: both cooled, but still attempted → real upstream result.
  let resp = route_chat_completion(&r, &tc.ctx).await?;
  assert_eq!(StatusCode::INTERNAL_SERVER_ERROR, resp.status());
  // both were forwarded again (not an instant synthetic error)
  assert_eq!(4, tc.calls.load(Ordering::SeqCst));
  Ok(())
}

/// A disabled target is never selected, even when all enabled targets are cooled.
#[rstest]
#[tokio::test]
async fn test_health_disabled_never_selected_even_when_all_cooled() -> anyhow::Result<()> {
  // local-a enabled (503 → cooled), local-b disabled (would 200 if tried).
  let outcomes = HashMap::from([
    ("local-a".to_string(), Upstream::Status(503)),
    ("local-b".to_string(), Upstream::Status(200)),
  ]);
  let tc = test_ctx(aliases_ab(), outcomes);
  let r = router(vec![target("local-a", true), target("local-b", false)]);

  route_chat_completion(&r, &tc.ctx).await?; // cool local-a
                                             // Second request: local-a cooled, local-b disabled → only local-a attempted.
  let resp = route_chat_completion(&r, &tc.ctx).await?;
  assert_eq!(StatusCode::SERVICE_UNAVAILABLE, resp.status());
  assert_eq!("local-a", resp.headers()["x-bodhi-routed-alias"]);
  assert_eq!("1", resp.headers()["x-bodhi-router-attempts"]);
  Ok(())
}

/// A structural skip (dangling reference) is NOT cooled — eligibility is
/// unchanged next request.
#[rstest]
#[tokio::test]
async fn test_health_structural_skip_not_cooled() -> anyhow::Result<()> {
  // local-a is dangling (absent from list_aliases); local-b serves.
  let outcomes = HashMap::from([("local-b".to_string(), Upstream::Status(200))]);
  let tc = test_ctx(vec![model_alias("local-b")], outcomes);
  let r = router_ab();
  let key = crate::models::router::target_key("t1", "local-a", "local-a");

  route_chat_completion(&r, &tc.ctx).await?;
  // dangling local-a was skipped but must NOT be cooled.
  assert!(!tc.health.is_cooled(&key, tc.time.utc_now()));
  Ok(())
}

/// A genuine transport failure IS cooled (transient).
#[rstest]
#[tokio::test]
async fn test_health_transport_failure_is_cooled() -> anyhow::Result<()> {
  let outcomes = HashMap::from([
    ("local-a".to_string(), Upstream::ForwardErr),
    ("local-b".to_string(), Upstream::Status(200)),
  ]);
  let tc = test_ctx(aliases_ab(), outcomes);
  let r = router_ab();
  let key = crate::models::router::target_key("t1", "local-a", "local-a");

  route_chat_completion(&r, &tc.ctx).await?;
  assert!(tc.health.is_cooled(&key, tc.time.utc_now()));
  Ok(())
}
