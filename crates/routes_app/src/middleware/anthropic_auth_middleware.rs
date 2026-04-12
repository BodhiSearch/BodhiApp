use axum::extract::Request;
use axum::http::HeaderValue;
use axum::middleware::Next;
use axum::response::Response;

use crate::middleware::{SENTINEL_API_KEY, SENTINEL_BEARER_LOWER};

/// Anthropic-path auth normalization. Runs before `auth_middleware` on
/// `/anthropic/*` and `/v1/messages`. First strips the chat-UI sentinel from
/// `x-api-key` / `Authorization`, then rewrites any remaining `x-api-key` to
/// `Authorization: Bearer <value>`. Existing `Authorization` wins over `x-api-key`.
pub async fn anthropic_auth_middleware(mut req: Request, next: Next) -> Response {
  let path = req.uri().path();
  if path.starts_with("/anthropic/") || path == "/v1/messages" {
    strip_sentinel_headers(&mut req);
    if let Some(key) = req.headers().get("x-api-key").cloned() {
      req.headers_mut().remove("x-api-key");
      if req.headers().get("authorization").is_none() {
        if let Ok(key_str) = key.to_str() {
          let bearer = format!("Bearer {}", key_str);
          if let Ok(val) = HeaderValue::from_str(&bearer) {
            req.headers_mut().insert("authorization", val);
          }
        }
      }
    }
  }
  next.run(req).await
}

/// Drops `x-api-key` / `Authorization` whose value is the chat-UI sentinel.
/// Bearer-scheme match is case-insensitive.
pub(crate) fn strip_sentinel_headers(req: &mut Request) {
  if let Some(key) = req.headers().get("x-api-key") {
    if key.to_str().ok() == Some(SENTINEL_API_KEY) {
      req.headers_mut().remove("x-api-key");
    }
  }
  if let Some(auth) = req.headers().get("authorization") {
    if auth
      .to_str()
      .ok()
      .map(|s| s.eq_ignore_ascii_case(SENTINEL_BEARER_LOWER))
      .unwrap_or(false)
    {
      req.headers_mut().remove("authorization");
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::middleware::{anthropic_auth_middleware, SENTINEL_API_KEY};
  use axum::body::Body;
  use axum::http::{Request, StatusCode};
  use axum::middleware::from_fn;
  use axum::routing::get;
  use axum::Router;
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
      .route("/anthropic/v1/messages", get(echo_handler))
      .route("/v1/messages", get(echo_handler))
      .route("/v1/chat/completions", get(echo_handler))
      .layer(from_fn(anthropic_auth_middleware))
  }

  #[tokio::test]
  async fn test_x_api_key_rewritten_to_bearer_on_anthropic_path() {
    let response = test_app()
      .oneshot(
        Request::get("/anthropic/v1/messages")
          .header("x-api-key", "bodhiapp_testtoken")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(StatusCode::OK, response.status());
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
      .await
      .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert_eq!("auth=Bearer bodhiapp_testtoken|x-api-key=", body_str);
  }

  #[tokio::test]
  async fn test_x_api_key_rewritten_on_v1_messages() {
    let response = test_app()
      .oneshot(
        Request::get("/v1/messages")
          .header("x-api-key", "bodhiapp_testtoken")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(StatusCode::OK, response.status());
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
      .await
      .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert_eq!("auth=Bearer bodhiapp_testtoken|x-api-key=", body_str);
  }

  #[tokio::test]
  async fn test_x_api_key_not_rewritten_on_non_anthropic_path() {
    let response = test_app()
      .oneshot(
        Request::get("/v1/chat/completions")
          .header("x-api-key", "bodhiapp_testtoken")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(StatusCode::OK, response.status());
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
      .await
      .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    // x-api-key is NOT rewritten and NOT stripped on non-Anthropic paths
    assert_eq!("auth=|x-api-key=bodhiapp_testtoken", body_str);
  }

  #[tokio::test]
  async fn test_authorization_header_takes_precedence() {
    let response = test_app()
      .oneshot(
        Request::get("/anthropic/v1/messages")
          .header("authorization", "Bearer bodhiapp_existing")
          .header("x-api-key", "bodhiapp_ignored")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(StatusCode::OK, response.status());
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
      .await
      .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert_eq!("auth=Bearer bodhiapp_existing|x-api-key=", body_str);
  }

  #[rstest::rstest]
  #[case::xapikey_anthropic_path("/anthropic/v1/messages", vec![("x-api-key", SENTINEL_API_KEY.to_string())], "auth=|x-api-key=")]
  #[case::bearer_anthropic_path("/anthropic/v1/messages", vec![("authorization", format!("Bearer {}", SENTINEL_API_KEY))], "auth=|x-api-key=")]
  #[case::bearer_lowercase_anthropic_path("/anthropic/v1/messages", vec![("authorization", format!("bearer {}", SENTINEL_API_KEY))], "auth=|x-api-key=")]
  #[case::sentinel_plus_real_auth_strip_xapikey_keep_auth(
    "/anthropic/v1/messages",
    vec![("x-api-key", SENTINEL_API_KEY.to_string()), ("authorization", "Bearer real-token".to_string())],
    "auth=Bearer real-token|x-api-key="
  )]
  #[case::xapikey_v1_messages("/v1/messages", vec![("x-api-key", SENTINEL_API_KEY.to_string())], "auth=|x-api-key=")]
  #[case::bearer_v1_messages("/v1/messages", vec![("authorization", format!("Bearer {}", SENTINEL_API_KEY))], "auth=|x-api-key=")]
  #[tokio::test]
  async fn test_sentinel_stripped_on_anthropic(
    #[case] path: &str,
    #[case] headers: Vec<(&str, String)>,
    #[case] expected: &str,
  ) {
    let mut builder = Request::get(path);
    for (k, v) in &headers {
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
    assert_eq!(expected, String::from_utf8_lossy(&body));
  }

  #[tokio::test]
  async fn test_no_auth_headers_passes_through_unchanged() {
    let response = test_app()
      .oneshot(
        Request::get("/anthropic/v1/messages")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(StatusCode::OK, response.status());
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
      .await
      .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert_eq!("auth=|x-api-key=", body_str);
  }
}
