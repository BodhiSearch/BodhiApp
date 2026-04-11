use axum::extract::Request;
use axum::http::HeaderValue;
use axum::middleware::Next;
use axum::response::Response;

/// Applied at the `api_protected` level (outside `auth_middleware`) so it can rewrite
/// `x-api-key` to `Authorization: Bearer` before `auth_middleware` processes the token.
///
/// Path-scoped: only activates for Anthropic endpoints (`/anthropic/*` and `/v1/messages`).
/// All other paths pass through unchanged.
///
/// Auth rewrite logic (`x-api-key` is never forwarded upstream):
/// - `x-api-key` absent → request passes through unchanged.
/// - `x-api-key` present, `Authorization` absent → strips `x-api-key`, sets
///   `Authorization: Bearer <value>` so native Anthropic SDK clients work unchanged.
/// - `x-api-key` present, `Authorization` present → strips `x-api-key` only;
///   existing `Authorization` header takes precedence.
///
/// Token validation is handled by downstream `api_auth_middleware` — no BYOK.
pub async fn anthropic_auth_middleware(mut req: Request, next: Next) -> Response {
  let path = req.uri().path();
  if path.starts_with("/anthropic/") || path == "/v1/messages" {
    if let Some(key) = req.headers().get("x-api-key").cloned() {
      req.headers_mut().remove("x-api-key"); // always strip — never forward upstream
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

#[cfg(test)]
mod tests {
  use super::*;
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

  #[tokio::test]
  async fn test_no_auth_headers_passes_through_unchanged() {
    let response = test_app()
      .oneshot(Request::get("/anthropic/v1/messages").body(Body::empty()).unwrap())
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
