use crate::{impl_error_from, AppError, ErrorMeta, ErrorType};
use rstest::rstest;

#[derive(Debug, thiserror::Error)]
#[error("external error: {0}")]
struct ExternalError(String);

#[derive(Debug, thiserror::Error)]
#[error("wrapped external: {0}")]
struct WrappedExternalError(String);

impl From<ExternalError> for WrappedExternalError {
  fn from(err: ExternalError) -> Self {
    Self(err.0)
  }
}

#[derive(Debug, thiserror::Error, ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
enum ServiceError {
  #[error("wrapped: {0}")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  Wrapped(WrappedExternalError),
}

impl_error_from!(ExternalError, ServiceError::Wrapped, WrappedExternalError);

#[rstest]
fn test_impl_error_from_creates_from_impl() {
  let external = ExternalError("disk failure".to_string());
  let service_error: ServiceError = external.into();
  assert_eq!(
    "wrapped: wrapped external: disk failure",
    service_error.to_string()
  );
  assert_eq!("service_error-wrapped", service_error.code());
  assert_eq!("internal_server_error", service_error.error_type());
  assert_eq!(500, service_error.status());
}

#[rstest]
fn test_impl_error_from_with_different_source() {
  let external = ExternalError("timeout".to_string());
  let service_error: ServiceError = external.into();
  assert_eq!(
    "wrapped: wrapped external: timeout",
    service_error.to_string()
  );
}
