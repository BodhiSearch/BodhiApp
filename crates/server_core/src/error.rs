use crate::TokenizerConfigError;
use llamacpp_rs::{CommonParamsBuilderError, LlamaCppError};
use objs::{impl_error_from, AppError, ErrorType, ObjValidationError, SerdeJsonError};
use services::DataServiceError;

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum ContextError {
  #[error(transparent)]
  SerdeJson(#[from] SerdeJsonError),
  #[error(transparent)]
  LlamaCpp(#[from] LlamaCppError),
  #[error(transparent)]
  DataServiceError(#[from] DataServiceError),
  #[error(transparent)]
  #[error_meta(error_type = ErrorType::InternalServer, status = 500, code = "context_error-gpt_params_builder_error", args_delegate = false)]
  BuilderError(#[from] CommonParamsBuilderError),
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
  #[error("library_path_missing")]
  #[error_meta(error_type = ErrorType::InternalServer, status = 500)]
  LibraryPathMissing,
  #[error("library_not_exists")]
  #[error_meta(error_type = ErrorType::InternalServer, status = 500)]
  LibraryNotExists(String),
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
  use llamacpp_rs::CommonParamsBuilderError;
  use objs::test_utils::{assert_error_message, setup_l10n};
  use objs::AppError;
  use objs::FluentLocalizationService;
  use rstest::rstest;
  use std::sync::Arc;

  #[rstest]
  #[case(&ContextError::BuilderError(CommonParamsBuilderError::UninitializedField("field")), "error building gpt params: `field` must be initialized")]
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
