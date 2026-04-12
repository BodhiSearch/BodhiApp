use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;

use super::anthropic_auth_middleware::strip_sentinel_headers;

/// Strips the chat-UI sentinel from `Authorization` / `x-api-key` on `/v1/*`
/// so downstream `auth_middleware` can fall through to session-cookie auth.
/// Non-sentinel headers pass through unchanged.
pub async fn openai_auth_middleware(mut req: Request, next: Next) -> Response {
  if req.uri().path().starts_with("/v1/") {
    strip_sentinel_headers(&mut req);
  }
  next.run(req).await
}

#[cfg(test)]
mod tests {
  use crate::middleware::{openai_auth_middleware, SENTINEL_API_KEY};
  use axum::body::Body;
  use axum::http::{Request, StatusCode};
  use axum::middleware::from_fn;
  use axum::routing::get;
  use axum::Router;
  use rstest::rstest;
  use tower::ServiceExt;

  async fn echo_handler(headers: axum::http::HeaderMap) -> String {
    let auth = headers
      .get("authorization")
      .and_then(|v| v.to_str().ok())
      .unwrap_or("")
      .to_string();
    let api_key = headers
      .get("x-api-key")
      .and_then(|v| v.to_str().ok())
      .unwrap_or("")
      .to_string();
    format!("auth={}|x-api-key={}", auth, api_key)
  }

  fn test_app() -> Router {
    Router::new()
      .route("/v1/chat/completions", get(echo_handler))
      .route("/v1/responses", get(echo_handler))
      .route("/v1/models", get(echo_handler))
      .route("/anthropic/v1/messages", get(echo_handler))
      .layer(from_fn(openai_auth_middleware))
  }

  async fn run_case(path: &str, headers: &[(&str, String)]) -> String {
    let mut builder = Request::get(path);
    for (k, v) in headers {
      builder = builder.header(*k, v);
    }
    let response = test_app()
      .oneshot(builder.body(Body::empty()).unwrap())
      .await
      .unwrap();
    assert_eq!(StatusCode::OK, response.status());
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
      .await
      .unwrap();
    String::from_utf8_lossy(&body).to_string()
  }

  #[rstest]
  #[case::auth_stripped_chat("/v1/chat/completions", vec![("authorization", format!("Bearer {}", SENTINEL_API_KEY))], "auth=|x-api-key=".to_string())]
  #[case::auth_stripped_responses("/v1/responses", vec![("authorization", format!("Bearer {}", SENTINEL_API_KEY))], "auth=|x-api-key=".to_string())]
  #[case::auth_stripped_models("/v1/models", vec![("authorization", format!("Bearer {}", SENTINEL_API_KEY))], "auth=|x-api-key=".to_string())]
  #[case::auth_lowercase_bearer("/v1/chat/completions", vec![("authorization", format!("bearer {}", SENTINEL_API_KEY))], "auth=|x-api-key=".to_string())]
  #[case::xapikey_stripped("/v1/chat/completions", vec![("x-api-key", SENTINEL_API_KEY.to_string())], "auth=|x-api-key=".to_string())]
  #[case::real_token_passthrough("/v1/chat/completions", vec![("authorization", "Bearer bodhiapp_realtoken123".to_string())], "auth=Bearer bodhiapp_realtoken123|x-api-key=".to_string())]
  #[case::non_v1_path_passthrough("/anthropic/v1/messages", vec![("authorization", format!("Bearer {}", SENTINEL_API_KEY))], format!("auth=Bearer {}|x-api-key=", SENTINEL_API_KEY))]
  #[tokio::test]
  async fn test_openai_auth_middleware(
    #[case] path: &str,
    #[case] headers: Vec<(&str, String)>,
    #[case] expected: String,
  ) {
    let body = run_case(path, &headers).await;
    assert_eq!(expected, body);
  }

  #[tokio::test]
  async fn test_no_auth_passes_through_unchanged() {
    let body = run_case("/v1/chat/completions", &[]).await;
    assert_eq!("auth=|x-api-key=", body);
  }
}
