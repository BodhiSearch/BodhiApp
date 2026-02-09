use axum::{body::Body, http::Response};
use http::StatusCode;

/// Asserts that a response indicates authentication was rejected (401 Unauthorized).
///
/// # Panics
/// Panics if the response status is not 401.
pub fn assert_auth_rejected(response: &Response<Body>) {
  assert_eq!(
    StatusCode::UNAUTHORIZED,
    response.status(),
    "Expected 401 Unauthorized"
  );
}

/// Asserts that a response indicates insufficient privileges (403 Forbidden).
///
/// Includes a descriptive message showing which role was tested against which endpoint.
///
/// # Panics
/// Panics if the response status is not 403.
pub fn assert_forbidden(response: &Response<Body>, role: &str, method: &str, path: &str) {
  assert_eq!(
    StatusCode::FORBIDDEN,
    response.status(),
    "{role} should be forbidden from {method} {path}"
  );
}

/// Asserts that a response passed authentication checks (status is not 401 or 403).
///
/// This does not assert the response is successful, only that auth did not block it.
///
/// # Panics
/// Panics if the response status is 401 or 403.
pub fn assert_auth_passed(response: &Response<Body>) {
  let status = response.status();
  assert_ne!(
    StatusCode::UNAUTHORIZED, status,
    "Expected auth to pass, but got 401 Unauthorized"
  );
  assert_ne!(
    StatusCode::FORBIDDEN, status,
    "Expected auth to pass, but got 403 Forbidden"
  );
}
