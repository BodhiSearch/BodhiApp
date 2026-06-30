use crate::middleware::model_inference_grant_middleware;
use crate::test_utils::RequestAuthContextExt;
use axum::{
  body::Body,
  extract::Request,
  middleware::from_fn,
  response::Response,
  routing::{get, post},
  Router,
};
use pretty_assertions::assert_eq;
use reqwest::StatusCode;
use rstest::rstest;
use serde_json::json;
use server_core::test_utils::{RequestTestExt, ResponseTestExt};
use services::{
  test_utils::{TEST_TENANT_ID, TEST_USER_ID},
  ApprovedResources, ApprovedResourcesV1, AuthContext, McpGrant, ModelGrant, ResourceRole,
  TokenGrants, TokenGrantsV1, TokenScope, UserScope,
};
use tower::ServiceExt;

async fn ok_handler() -> Response {
  Response::builder()
    .status(StatusCode::OK)
    .body(Body::empty())
    .unwrap()
}

/// Router mirroring every inference surface + a listing route, all behind the shared
/// grant middleware. Handlers are stubs returning 200 — the middleware is what gates.
fn router() -> Router {
  Router::new()
    .route("/v1/chat/completions", post(ok_handler))
    .route("/v1/embeddings", post(ok_handler))
    .route("/v1/responses", post(ok_handler))
    .route("/v1/messages", post(ok_handler))
    .route("/anthropic/v1/messages", post(ok_handler))
    .route(
      "/v1beta/models/{*model_path}",
      post(ok_handler).get(ok_handler),
    )
    .route("/v1/models", get(ok_handler))
    .layer(from_fn(model_inference_grant_middleware))
}

/// API token granting inference only on `model`.
fn api_token(model: &str) -> AuthContext {
  AuthContext::test_api_token_with_grants(
    TEST_USER_ID,
    TokenScope::User,
    TokenGrants::V1(TokenGrantsV1 {
      models_list: false,
      models: ModelGrant::Specific {
        ids: vec![model.to_string()],
      },
      mcps_list: false,
      mcps: McpGrant::Specific { ids: vec![] },
    }),
  )
}

/// Approved external app granting inference only on `model`.
fn external_app(model: &str) -> AuthContext {
  let approved = ApprovedResources::V1(ApprovedResourcesV1 {
    models_list: false,
    models_access: ModelGrant::Specific {
      ids: vec![model.to_string()],
    },
    mcps_list: false,
    mcps: vec![],
    mcps_access: McpGrant::Specific { ids: vec![] },
  });
  AuthContext::test_external_app(TEST_USER_ID, UserScope::User, "app", Some("ar"))
    .with_tenant_id(TEST_TENANT_ID)
    .with_external_app_grants(approved)
}

// Body-based formats: the model lives in the JSON body.
#[rstest]
#[case::chat("/v1/chat/completions")]
#[case::embeddings("/v1/embeddings")]
#[case::responses("/v1/responses")]
#[case::anthropic_root("/v1/messages")]
#[case::anthropic("/anthropic/v1/messages")]
#[tokio::test]
#[anyhow_trace::anyhow_trace]
async fn body_inference_paths_enforce_model_grant(#[case] path: &str) -> anyhow::Result<()> {
  // Non-granted model → 403 (both principal kinds), rendered in the path's native
  // error envelope (OpenAI vs Anthropic shape).
  for ctx in [api_token("allowed"), external_app("allowed")] {
    let resp = router()
      .oneshot(
        Request::post(path)
          .json(json!({"model": "forbidden"}))?
          .with_auth_context(ctx),
      )
      .await?;
    assert_eq!(StatusCode::FORBIDDEN, resp.status());
    let body = resp.json::<serde_json::Value>().await?;
    match path {
      // OpenAI envelope: {"error":{"type":...,"code":...}}.
      "/v1/chat/completions" | "/v1/embeddings" | "/v1/responses" => {
        assert_eq!("forbidden_error", body["error"]["type"].as_str().unwrap());
        assert_eq!(
          "token_grant_error-model_forbidden",
          body["error"]["code"].as_str().unwrap()
        );
      }
      // Anthropic envelope: {"type":"error","error":{"type":"permission_error"}}.
      _ => {
        assert_eq!("error", body["type"].as_str().unwrap());
        assert_eq!("permission_error", body["error"]["type"].as_str().unwrap());
      }
    }
  }

  // Granted model → passes through to the stub handler (200).
  let resp = router()
    .oneshot(
      Request::post(path)
        .json(json!({"model": "allowed"}))?
        .with_auth_context(api_token("allowed")),
    )
    .await?;
  assert_eq!(StatusCode::OK, resp.status());
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace::anyhow_trace]
async fn gemini_action_path_enforces_model_grant() -> anyhow::Result<()> {
  // Model is in the path: /v1beta/models/{model}:{action}.
  let resp = router()
    .oneshot(
      Request::post("/v1beta/models/forbidden:generateContent")
        .json(json!({}))?
        .with_auth_context(api_token("allowed")),
    )
    .await?;
  assert_eq!(StatusCode::FORBIDDEN, resp.status());
  // Gemini envelope: {"error":{"code":403,"status":"PERMISSION_DENIED"}}.
  let body = resp.json::<serde_json::Value>().await?;
  assert_eq!(403, body["error"]["code"].as_u64().unwrap());
  assert_eq!(
    "PERMISSION_DENIED",
    body["error"]["status"].as_str().unwrap()
  );

  let resp = router()
    .oneshot(
      Request::post("/v1beta/models/allowed:generateContent")
        .json(json!({}))?
        .with_auth_context(api_token("allowed")),
    )
    .await?;
  assert_eq!(StatusCode::OK, resp.status());
  Ok(())
}

/// The middleware lets a request reach the handler without a 403 in two arms:
/// (b) a grant/deny principal whose body carries no `model` (malformed → the
/// handler owns the rejection), and (a) an `Unrestricted` session, which
/// short-circuits before the body is even buffered and is never denied — even for
/// a model no token would grant. Here the stub handler always returns 200, so a 200
/// proves the middleware passed through rather than issuing its own 403.
#[rstest]
#[case::scoped_token_no_model(api_token("granted"), json!({"messages": []}))]
#[case::unrestricted_session_ungranted_model(
  AuthContext::test_session(TEST_USER_ID, "u@email.com", ResourceRole::User),
  json!({"model": "totally-ungranted-model"})
)]
#[tokio::test]
#[anyhow_trace::anyhow_trace]
async fn middleware_passes_request_through_to_handler(
  #[case] ctx: AuthContext,
  #[case] body: serde_json::Value,
) -> anyhow::Result<()> {
  let resp = router()
    .oneshot(
      Request::post("/v1/chat/completions")
        .json(body)?
        .with_auth_context(ctx),
    )
    .await?;
  assert_eq!(StatusCode::OK, resp.status());
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace::anyhow_trace]
async fn unbound_external_app_is_denied() -> anyhow::Result<()> {
  // No bound access request ⇒ fail closed even for the model it might want.
  let unbound = AuthContext::test_external_app(TEST_USER_ID, UserScope::User, "app", None)
    .with_tenant_id(TEST_TENANT_ID);
  let resp = router()
    .oneshot(
      Request::post("/v1/chat/completions")
        .json(json!({"model": "anything"}))?
        .with_auth_context(unbound),
    )
    .await?;
  assert_eq!(StatusCode::FORBIDDEN, resp.status());
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace::anyhow_trace]
async fn listing_and_unrestricted_pass_through() -> anyhow::Result<()> {
  // Listing is filtered in-handler, not gated here — the middleware ignores it.
  let resp = router()
    .oneshot(
      Request::get("/v1/models")
        .body(Body::empty())?
        .with_auth_context(api_token("only-this")),
    )
    .await?;
  assert_eq!(StatusCode::OK, resp.status());

  // Session principals are Unrestricted → inference passes regardless of model.
  let session = AuthContext::test_session(TEST_USER_ID, "u@email.com", ResourceRole::User);
  let resp = router()
    .oneshot(
      Request::post("/v1/chat/completions")
        .json(json!({"model": "anything"}))?
        .with_auth_context(session),
    )
    .await?;
  assert_eq!(StatusCode::OK, resp.status());
  Ok(())
}
