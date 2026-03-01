use crate::{AppError, ErrorMeta, ErrorType};
use rstest::rstest;
use std::str::FromStr;

#[rstest]
#[case(ErrorType::BadRequest, 400, "invalid_request_error")]
#[case(ErrorType::InvalidAppState, 500, "invalid_app_state")]
#[case(ErrorType::InternalServer, 500, "internal_server_error")]
#[case(ErrorType::Authentication, 401, "authentication_error")]
#[case(ErrorType::Forbidden, 403, "forbidden_error")]
#[case(ErrorType::NotFound, 404, "not_found_error")]
#[case(ErrorType::Conflict, 409, "conflict_error")]
#[case(ErrorType::UnprocessableEntity, 422, "unprocessable_entity_error")]
#[case(ErrorType::Unknown, 500, "unknown_error")]
#[case(ErrorType::ServiceUnavailable, 503, "service_unavailable")]
fn test_error_type_status_and_display(
  #[case] error_type: ErrorType,
  #[case] expected_status: u16,
  #[case] expected_display: &str,
) {
  assert_eq!(expected_status, error_type.status());
  assert_eq!(expected_display, error_type.to_string());
}

#[rstest]
#[case("invalid_request_error", true)]
#[case("invalid_app_state", true)]
#[case("internal_server_error", true)]
#[case("authentication_error", true)]
#[case("forbidden_error", true)]
#[case("not_found_error", true)]
#[case("conflict_error", true)]
#[case("unprocessable_entity_error", true)]
#[case("unknown_error", true)]
#[case("service_unavailable", true)]
#[case("nonexistent_error", false)]
fn test_error_type_from_str(#[case] input: &str, #[case] should_succeed: bool) {
  let result = ErrorType::from_str(input);
  assert_eq!(should_succeed, result.is_ok());
}

#[rstest]
fn test_error_type_default() {
  let default = ErrorType::default();
  assert_eq!(500, default.status());
  assert_eq!("unknown_error", default.to_string());
}

#[rstest]
fn test_error_type_partial_eq() {
  assert_eq!(ErrorType::BadRequest, ErrorType::BadRequest);
  assert_ne!(ErrorType::BadRequest, ErrorType::NotFound);
  assert_eq!(ErrorType::Unknown, ErrorType::default());
}

#[rstest]
fn test_error_type_clone() {
  let original = ErrorType::InternalServer;
  let cloned = original.clone();
  assert_eq!(original, cloned);
}

#[derive(Debug, thiserror::Error, ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
enum UnrecognizedTypeError {
  #[error("unrecognized error")]
  #[error_meta(error_type = "totally_unknown_error_string")]
  Unknown,
}

#[rstest]
fn test_app_error_status_returns_500_for_unrecognized_error_type() {
  let error = UnrecognizedTypeError::Unknown;
  assert_eq!("totally_unknown_error_string", error.error_type());
  assert_eq!(500, error.status());
}
