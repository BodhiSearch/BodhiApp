use axum::http::StatusCode;

/// Status-only disposition of an upstream attempt (proposal §4 / Phase-2 doc).
/// The decision is made on the HTTP status alone, before any response body is
/// consumed, so a retryable failure can fall through before the first byte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Disposition {
  /// 2xx — commit and stream back.
  Success,
  /// Fall through to the next target. Covers transport-ish statuses
  /// (408/429/5xx) plus the free-tier stacking rule (401/403/404): a bad key
  /// or missing model on one vendor must not fail the request if another works.
  Retryable,
  /// Stop immediately and return verbatim — the request itself is the problem
  /// (400/422, which is how providers surface content-policy / context-window),
  /// so another vendor won't help.
  Terminal,
}

pub fn classify_status(status: StatusCode) -> Disposition {
  match status.as_u16() {
    200..=299 => Disposition::Success,
    400 | 422 => Disposition::Terminal,
    _ => Disposition::Retryable,
  }
}

#[cfg(test)]
mod tests {
  use super::{classify_status, Disposition};
  use axum::http::StatusCode;
  use pretty_assertions::assert_eq;
  use rstest::rstest;

  #[rstest]
  #[case(200, Disposition::Success)]
  #[case(204, Disposition::Success)]
  #[case(400, Disposition::Terminal)]
  #[case(422, Disposition::Terminal)]
  #[case(401, Disposition::Retryable)]
  #[case(403, Disposition::Retryable)]
  #[case(404, Disposition::Retryable)]
  #[case(408, Disposition::Retryable)]
  #[case(429, Disposition::Retryable)]
  #[case(500, Disposition::Retryable)]
  #[case(502, Disposition::Retryable)]
  #[case(503, Disposition::Retryable)]
  #[case(504, Disposition::Retryable)]
  fn test_classify_status(#[case] status: u16, #[case] expected: Disposition) {
    assert_eq!(
      expected,
      classify_status(StatusCode::from_u16(status).unwrap())
    );
  }
}
