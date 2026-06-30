//! Unified per-model inference enforcement.
//!
//! ONE middleware for every inference surface BodhiApp proxies. It infers the API
//! format from the request path, extracts the target model (from the JSON body for
//! OpenAI / OpenAI-Responses / Anthropic, from the URL path for Gemini), and runs
//! [`AccessPolicy::ensure_model_inference`]. A denial is rendered in that format's
//! native error envelope (OpenAI / Anthropic / Gemini).
//!
//! Centralizing the check here means a new inference endpoint cannot silently skip
//! the grant the way a per-handler call could — the bug class that left `/v1/embeddings`
//! and `/v1/responses` unguarded. Listing endpoints filter their response in-handler
//! and are intentionally NOT covered here (a pre-handler middleware has no response to
//! filter).

use crate::{AccessPolicy, AnthropicApiError, GeminiApiError, OaiApiError, TokenGrantError};
use axum::{
  body::Body,
  extract::Request,
  http::Method,
  middleware::Next,
  response::{IntoResponse, Response},
};
use services::{AuthContext, DeploymentMode};

/// Inference API format, inferred from the request path. Drives model extraction and
/// the error-envelope shape. OpenAI and OpenAI-Responses share both, so they collapse
/// into one variant.
#[derive(Debug, Clone, Copy, PartialEq)]
enum InferenceFormat {
  /// OpenAI chat/completions, embeddings, and Responses create — model in the body.
  OpenAi,
  /// Anthropic messages — model in the body.
  Anthropic,
  /// Gemini generate/embed actions — model in the URL path (`{model}:{action}`).
  Gemini,
}

/// Cap when buffering the request body to read its `model` field (10 MiB).
const MAX_BODY_BYTES: usize = 10 * 1024 * 1024;

/// Minimal projection of an inference body — only the `model` field is needed for
/// enforcement. Cheaper than parsing the whole payload into `serde_json::Value`.
#[derive(serde::Deserialize)]
struct ModelField {
  model: Option<String>,
}

/// Classify an inference request by `(method, path)`. Returns `None` for anything that
/// is not a model-inference call (listings, GETs, lookups, unrelated routes) — those
/// pass through untouched. Only POST requests carry an inference body/action.
fn classify(method: &Method, path: &str) -> Option<InferenceFormat> {
  if method != Method::POST {
    return None;
  }
  match path {
    "/v1/chat/completions" | "/v1/embeddings" | "/v1/responses" => Some(InferenceFormat::OpenAi),
    "/v1/messages" | "/anthropic/v1/messages" => Some(InferenceFormat::Anthropic),
    // Gemini actions: /v1beta/models/{model}:{action}. A bare /v1beta/models/{id}
    // (no ':') is a model lookup, not inference.
    p if p.starts_with("/v1beta/models/") && p.contains(':') => Some(InferenceFormat::Gemini),
    _ => None,
  }
}

/// Extract the Gemini model from `/v1beta/models/{model}:{action}` (model before the
/// last `:`), mirroring `gemini_action_handler`.
fn gemini_model_from_path(path: &str) -> Option<String> {
  let tail = path.strip_prefix("/v1beta/models/")?;
  let (model, _action) = tail.rsplit_once(':')?;
  Some(model.to_string())
}

/// Render a model-forbidden denial in the request format's native error envelope.
fn forbidden(format: InferenceFormat, model: &str) -> Response {
  let err = TokenGrantError::ModelForbidden(model.to_string());
  match format {
    InferenceFormat::OpenAi => OaiApiError::from(err).into_response(),
    InferenceFormat::Anthropic => AnthropicApiError::from(err).into_response(),
    InferenceFormat::Gemini => GeminiApiError::from(err).into_response(),
  }
}

pub async fn model_inference_grant_middleware(req: Request, next: Next) -> Response {
  let method = req.method().clone();
  let path = req.uri().path().to_string();
  let Some(format) = classify(&method, &path) else {
    return next.run(req).await;
  };

  // Resolve the principal's policy BEFORE touching the body. The dominant chat-UI
  // principal is a session (`Unrestricted`) that needs no model, so it must not pay
  // the body-buffering + parse cost; only grant/deny principals need the model.
  let ctx = req
    .extensions()
    .get::<AuthContext>()
    .cloned()
    .unwrap_or(AuthContext::Anonymous {
      deployment: DeploymentMode::Standalone,
    });
  let policy = AccessPolicy::of(&ctx);
  if matches!(policy, AccessPolicy::Unrestricted) {
    return next.run(req).await;
  }

  // TODO: inefficient interceptor — buffers the complete body to read "model", which the
  // handler's own extractor then parses again. Lazy single-field extraction is not feasible
  // with axum/serde (the body must be scanned whole to find field boundaries). This is the
  // only interceptor that reads the body (the Gemini path takes the model from the URL and
  // the MCP proxy has no grant middleware), so there is no interceptor-level read to merge.
  let (model, req) = match format {
    InferenceFormat::Gemini => (gemini_model_from_path(&path), req),
    InferenceFormat::OpenAi | InferenceFormat::Anthropic => {
      let (parts, body) = req.into_parts();
      match axum::body::to_bytes(body, MAX_BODY_BYTES).await {
        Ok(bytes) => {
          let model = serde_json::from_slice::<ModelField>(&bytes)
            .ok()
            .and_then(|m| m.model);
          (model, Request::from_parts(parts, Body::from(bytes)))
        }
        // Unreadable/oversized body: forward empty so the handler's extractor produces
        // its own native 4xx (we can't reconstruct a consumed stream).
        Err(_) => (None, Request::from_parts(parts, Body::empty())),
      }
    }
  };

  // No model in the request ⇒ malformed; let the handler reject it in its own envelope.
  let Some(model) = model else {
    return next.run(req).await;
  };

  if policy.ensure_model_inference(&model).is_err() {
    return forbidden(format, &model);
  }
  next.run(req).await
}

#[cfg(test)]
#[path = "test_model_grant.rs"]
mod test_model_grant;
