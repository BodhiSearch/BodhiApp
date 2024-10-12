use crate::TokenizerConfigError;
use llama_server_bindings::{GptParamsBuilderError, LlamaCppError};
use objs::{impl_error_from, AppError, ErrorType, ObjValidationError, SerdeJsonError};
use services::DataServiceError;

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum ContextError {
  #[error(transparent)]
  SerdeJson(#[from] SerdeJsonError),
  #[error(transparent)]
  #[error_meta(error_type = ErrorType::InternalServer, status = 500, code = "context_error-llama_cpp_error", args_delegate = false)]
  LlamaCpp(#[from] LlamaCppError),
  #[error(transparent)]
  DataServiceError(#[from] DataServiceError),
  #[error(transparent)]
  #[error_meta(error_type = ErrorType::InternalServer, status = 500, code = "context_error-gpt_params_builder_error", args_delegate = false)]
  BuilderError(#[from] GptParamsBuilderError),
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
  use crate::{test_utils::setup_l10n_server_core, ContextError};
  use llama_server_bindings::{GptParamsBuilderError, LlamaCppError};
  use objs::test_utils::assert_error_message;
  use objs::AppError;
  use objs::FluentLocalizationService;
  use rstest::rstest;
  use std::sync::Arc;

  #[rstest]
  #[case(&ContextError::LlamaCpp(LlamaCppError::GptParamsInit("test".to_string())), "error initializing llama cpp: gpt_params_init: test")]
  #[case(&ContextError::BuilderError(GptParamsBuilderError::UninitializedField("field")), "error building gpt params: `field` must be initialized")]
  #[case(&ContextError::Minijina(minijinja::Error::new(minijinja::ErrorKind::NonKey, "error")), "error rendering template: not a key type: error")]
  #[case(&ContextError::Unreachable("unreachable".to_string()), "should not happen: unreachable")]
  #[serial_test::serial(localization)]
  fn test_error_messages_server_core(
    #[from(setup_l10n_server_core)] localization_service: Arc<FluentLocalizationService>,
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
