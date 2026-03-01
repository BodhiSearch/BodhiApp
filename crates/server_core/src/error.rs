use llama_server_proc::ServerError;
use services::{
  impl_error_from, AppError, BuilderError, DataServiceError, ErrorType, HubServiceError,
  ObjValidationError, SerdeJsonError,
};

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum ContextError {
  #[error(transparent)]
  HubService(#[from] HubServiceError),
  #[error(transparent)]
  Builder(#[from] BuilderError),
  #[error(transparent)]
  Server(#[from] ServerError),
  #[error(transparent)]
  SerdeJson(#[from] SerdeJsonError),
  #[error(transparent)]
  DataServiceError(#[from] DataServiceError),
  #[error("Internal error: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  Unreachable(String),
  #[error(transparent)]
  ObjValidationError(#[from] ObjValidationError),
  #[error("Model executable not found: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  ExecNotExists(String),
}

impl_error_from!(
  ::serde_json::Error,
  ContextError::SerdeJson,
  ::services::SerdeJsonError
);
impl_error_from!(
  ::validator::ValidationErrors,
  ContextError::ObjValidationError,
  services::ObjValidationError
);

#[cfg(test)]
mod tests {
  use crate::ContextError;
  use rstest::rstest;
  use services::AppError;

  #[rstest]
  #[case(&ContextError::Unreachable("unreachable".to_string()), "Internal error: unreachable.")]
  #[case(&ContextError::ExecNotExists("/path/to/exec".to_string()), "Model executable not found: /path/to/exec.")]
  fn test_error_display(#[case] error: &dyn AppError, #[case] expected_message: &str) {
    assert_eq!(expected_message, error.to_string());
  }

  #[rstest]
  #[case(&ContextError::Unreachable("test".to_string()), "internal_server_error")]
  #[case(&ContextError::ExecNotExists("/path".to_string()), "internal_server_error")]
  fn test_error_type(#[case] error: &dyn AppError, #[case] expected_type: &str) {
    assert_eq!(expected_type, error.error_type());
  }

  #[rstest]
  #[case(&ContextError::Unreachable("test".to_string()), "context_error-unreachable")]
  #[case(&ContextError::ExecNotExists("/path".to_string()), "context_error-exec_not_exists")]
  fn test_error_code(#[case] error: &dyn AppError, #[case] expected_code: &str) {
    assert_eq!(expected_code, error.code());
  }
}
