use crate::TokenizerConfigError;
use llama_server_proc::ServerError;
use objs::{
  impl_error_from, AppError, BuilderError, ErrorType, ObjValidationError, SerdeJsonError,
};
use services::DataServiceError;

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum ContextError {
  #[error(transparent)]
  Builder(#[from] BuilderError),
  #[error(transparent)]
  Server(#[from] ServerError),
  #[error(transparent)]
  SerdeJson(#[from] SerdeJsonError),
  #[error(transparent)]
  DataServiceError(#[from] DataServiceError),
  #[error(transparent)]
  ObjValidationError(#[from] ObjValidationError),
  #[error(transparent)]
  #[error_meta(error_type = ErrorType::InternalServer, status = 500, code = "context_error-minijina_error", args_delegate = false)]
  Minijina(#[from] minijinja::Error),
  #[error("unreachable")]
  #[error_meta(error_type = ErrorType::InternalServer, status = 500)]
  Unreachable(String),
  #[error(transparent)]
  TokenizerConfig(#[from] TokenizerConfigError),
  #[error("exec_not_exists")]
  #[error_meta(error_type = ErrorType::InternalServer, status = 500)]
  ExecNotExists(String),
}

impl_error_from!(
  ::serde_json::Error,
  ContextError::SerdeJson,
  ::objs::SerdeJsonError
);
impl_error_from!(
  ::validator::ValidationErrors,
  ContextError::ObjValidationError,
  ::objs::ObjValidationError
);

#[cfg(test)]
mod tests {
  use crate::ContextError;
  use objs::test_utils::{assert_error_message, setup_l10n};
  use objs::AppError;
  use objs::FluentLocalizationService;
  use rstest::rstest;
  use std::sync::Arc;

  #[rstest]
  #[case(&ContextError::Minijina(minijinja::Error::new(minijinja::ErrorKind::NonKey, "error")), "error rendering template: not a key type: error")]
  #[case(&ContextError::Unreachable("unreachable".to_string()), "should not happen: unreachable")]
  fn test_error_messages_server_core(
    #[from(setup_l10n)] localization_service: &Arc<FluentLocalizationService>,
    #[case] error: &dyn AppError,
    #[case] expected_message: &str,
  ) {
    assert_error_message(
      localization_service,
      &error.code(),
      error.args(),
      expected_message,
    );
  }
}
