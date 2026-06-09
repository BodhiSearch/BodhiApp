use axum::{body::Body, http::Response};
use http::StatusCode;

pub fn assert_auth_rejected(response: &Response<Body>) {
  assert_eq!(
    StatusCode::UNAUTHORIZED,
    response.status(),
    "Expected 401 Unauthorized"
  );
}

pub fn assert_forbidden(response: &Response<Body>, role: &str, method: &str, path: &str) {
  assert_eq!(
    StatusCode::FORBIDDEN,
    response.status(),
    "{role} should be forbidden from {method} {path}"
  );
}

/// Does not assert the response is successful, only that auth did not block it (not 401/403).
pub fn assert_auth_passed(response: &Response<Body>) {
  let status = response.status();
  assert_ne!(
    StatusCode::UNAUTHORIZED,
    status,
    "Expected auth to pass, but got 401 Unauthorized"
  );
  assert_ne!(
    StatusCode::FORBIDDEN,
    status,
    "Expected auth to pass, but got 403 Forbidden"
  );
}
