use axum::extract::Request;
use axum::http::HeaderValue;
use axum::middleware::Next;
use axum::response::Response;

use crate::middleware::SENTINEL_API_KEY;

/// Gemini-path auth normalization. Runs before `auth_middleware` on `/v1beta/*`.
///
/// 1. Calls `strip_sentinel_headers` (strips `x-api-key` / `Authorization: Bearer SENTINEL`).
/// 2. Additionally strips `x-goog-api-key` if it equals `SENTINEL_API_KEY`.
/// 3. If `x-goog-api-key` is present (non-sentinel) and no `Authorization` header exists,
///    rewrites it as `Authorization: Bearer <value>` and removes `x-goog-api-key`.
/// 4. If `Authorization` already present, `x-goog-api-key` is removed (Authorization wins).
///
/// **Rationale**: pi-ai's `@google/genai` SDK sets `x-goog-api-key` to the dummy sentinel;
/// stripping it allows session-cookie auth to fall through. External clients using
/// `x-goog-api-key` as their sole credential get it transparently rewritten to
/// `Authorization: Bearer` so BodhiApp's token auth picks it up.
pub async fn gemini_auth_middleware(mut req: Request, next: Next) -> Response {
  if req.uri().path().starts_with("/v1beta/") {
    use crate::middleware::anthropic_auth_middleware::strip_sentinel_headers;
    strip_sentinel_headers(&mut req);

    // Strip sentinel from x-goog-api-key specifically
    if let Some(key) = req.headers().get("x-goog-api-key") {
      if key.to_str().ok() == Some(SENTINEL_API_KEY) {
        req.headers_mut().remove("x-goog-api-key");
      }
    }

    // Rewrite non-sentinel x-goog-api-key -> Authorization: Bearer (if no Auth header present)
    if let Some(key) = req.headers().get("x-goog-api-key").cloned() {
      req.headers_mut().remove("x-goog-api-key");
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
  use crate::middleware::{gemini_auth_middleware, SENTINEL_API_KEY};
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
    let goog_key = headers
      .get("x-goog-api-key")
      .and_then(|v| v.to_str().ok())
      .unwrap_or("")
      .to_string();
    format!("auth={}|x-goog-api-key={}", auth, goog_key)
  }

  fn test_app() -> Router {
    Router::new()
      .route("/v1beta/models", get(echo_handler))
      .route("/v1/models", get(echo_handler))
      .route("/anthropic/v1/messages", get(echo_handler))
      .layer(from_fn(gemini_auth_middleware))
  }

  #[tokio::test]
  async fn test_goog_api_key_rewritten_to_bearer_on_v1beta_path() {
    let response = test_app()
      .oneshot(
        Request::get("/v1beta/models")
          .header("x-goog-api-key", "bodhiapp_testtoken")
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
    assert_eq!("auth=Bearer bodhiapp_testtoken|x-goog-api-key=", body_str);
  }

  #[tokio::test]
  async fn test_goog_api_key_not_rewritten_on_non_v1beta_path() {
    let response = test_app()
      .oneshot(
        Request::get("/v1/models")
          .header("x-goog-api-key", "bodhiapp_testtoken")
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
    // x-goog-api-key NOT touched on non-/v1beta/* paths
    assert_eq!("auth=|x-goog-api-key=bodhiapp_testtoken", body_str);
  }

  #[tokio::test]
  async fn test_authorization_takes_precedence_over_goog_api_key() {
    let response = test_app()
      .oneshot(
        Request::get("/v1beta/models")
          .header("authorization", "Bearer bodhiapp_existing")
          .header("x-goog-api-key", "bodhiapp_ignored")
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
    // Authorization kept; x-goog-api-key removed without overwriting auth
    assert_eq!("auth=Bearer bodhiapp_existing|x-goog-api-key=", body_str);
  }

  #[tokio::test]
  async fn test_no_auth_headers_passes_through_unchanged() {
    let response = test_app()
      .oneshot(Request::get("/v1beta/models").body(Body::empty()).unwrap())
      .await
      .unwrap();
    assert_eq!(StatusCode::OK, response.status());
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
      .await
      .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert_eq!("auth=|x-goog-api-key=", body_str);
  }

  #[rstest]
  #[case::sentinel_xgoog_v1beta("/v1beta/models", vec![("x-goog-api-key", SENTINEL_API_KEY.to_string())], "auth=|x-goog-api-key=")]
  #[case::sentinel_bearer_v1beta("/v1beta/models", vec![("authorization", format!("Bearer {}", SENTINEL_API_KEY))], "auth=|x-goog-api-key=")]
  #[case::sentinel_bearer_lowercase_v1beta("/v1beta/models", vec![("authorization", format!("bearer {}", SENTINEL_API_KEY))], "auth=|x-goog-api-key=")]
  #[case::sentinel_xgoog_plus_real_auth("/v1beta/models", vec![
    ("x-goog-api-key", SENTINEL_API_KEY.to_string()),
    ("authorization", "Bearer real-token".to_string()),
  ], "auth=Bearer real-token|x-goog-api-key=")]
  #[case::non_v1beta_path_passthrough("/v1/models", vec![("authorization", format!("Bearer {}", SENTINEL_API_KEY))], &format!("auth=Bearer {}|x-goog-api-key=", SENTINEL_API_KEY))]
  #[case::anthropic_path_passthrough("/anthropic/v1/messages", vec![("x-goog-api-key", SENTINEL_API_KEY.to_string())], &format!("auth=|x-goog-api-key={}", SENTINEL_API_KEY))]
  #[tokio::test]
  async fn test_sentinel_stripped_on_v1beta(
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
}
