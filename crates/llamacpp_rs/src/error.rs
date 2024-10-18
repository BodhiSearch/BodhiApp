use objs::ErrorType;
use std::str::Utf8Error;

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = objs::AppError)]
pub enum LlamaCppError {
  #[error("common_params_get_u32")]
  #[error_meta(error_type = ErrorType::BadRequest, status = 400)]
  CommonParamsGetU32(u32),
  #[error("common_params_update_u32")]
  #[error_meta(error_type = ErrorType::BadRequest, status = 400)]
  CommonParamsUpdateU32(u32, u32),
  #[error("common_params_get_i32")]
  #[error_meta(error_type = ErrorType::BadRequest, status = 400)]
  CommonParamsGetI32(u32),
  #[error("common_params_update_i32")]
  #[error_meta(error_type = ErrorType::BadRequest, status = 400)]
  CommonParamsUpdateI32(u32, i32),
  #[error("common_params_get_string")]
  #[error_meta(error_type = ErrorType::BadRequest, status = 400)]
  CommonParamsGetString(u32),
  #[error("common_params_update_string")]
  #[error_meta(error_type = ErrorType::BadRequest, status = 400)]
  CommonParamsUpdateString(u32, String),
  #[error("common_params_convert_string")]
  #[error_meta(error_type = ErrorType::BadRequest, status = 400)]
  CommonParamsConvertString {
    #[source]
    source: Utf8Error,
    option: u32,
    value: String,
  },
  #[error("gpt_params_init")]
  #[error_meta(error_type = ErrorType::BadRequest, status = 400)]
  GptParamsInit(String),
  #[error("bodhi_context")]
  #[error_meta(error_type = ErrorType::BadRequest, status = 400)]
  BodhiContextInit(String),
  #[error("bodhi_server_start")]
  #[error_meta(error_type = ErrorType::BadRequest, status = 400)]
  BodhiServerStart(String),
  #[error("bodhi_server_chat_completion")]
  #[error_meta(error_type = ErrorType::BadRequest, status = 400)]
  BodhiServerChatCompletion(String),
  #[error("bodhi_server_stop")]
  #[error_meta(error_type = ErrorType::BadRequest, status = 400)]
  BodhiServerStop(String),
}

pub type Result<T> = std::result::Result<T, LlamaCppError>;

#[cfg(test)]
mod tests {
  use crate::error::LlamaCppError;
  use objs::test_utils::setup_l10n;
  use objs::{test_utils::assert_error_message, AppError, FluentLocalizationService};
  use rstest::rstest;
  use std::sync::Arc;

  #[rstest]
  #[case::getu32(&LlamaCppError::CommonParamsGetU32(1), "common_params_get: error retrieving common_params u32 value for option 1")]
  #[case::updateu32(&LlamaCppError::CommonParamsUpdateU32(1, 2), "common_params_update: error updating common_params u32 value for option 1 with value 2")]
  #[case::geti32(&LlamaCppError::CommonParamsGetI32(1), "common_params_get: error retrieving common_params i32 value for option 1")]
  #[case::updatei32(&LlamaCppError::CommonParamsUpdateI32(1, 2), "common_params_update: error updating common_params i32 value for option 1 with value 2")]
  #[case::getstring(&LlamaCppError::CommonParamsGetString(1), "common_params_get: error retrieving common_params string value for option 1")]
  #[case::updatestring(&LlamaCppError::CommonParamsUpdateString(1, "newvalue".to_string()), "common_params_update: error updating common_params string value for option 1, value: newvalue")]
  #[case::init(&LlamaCppError::GptParamsInit("unknown error".to_string()), "common_params_init: unknown error")]
  #[case::context_init(&LlamaCppError::BodhiContextInit("unknown error".to_string()), "bodhi_context: unknown error")]
  #[case::server_start(&LlamaCppError::BodhiServerStart("unknown error".to_string()), "bodhi_server_start: unknown error")]
  #[case::chat_completions(&LlamaCppError::BodhiServerChatCompletion("unknown error".to_string()), "bodhi_server_chat_completion: unknown error")]
  #[case::server_stop(&LlamaCppError::BodhiServerStop("unknown error".to_string()), "bodhi_server_stop: unknown error")]
  fn test_error_messages_llama_cpp_rs(
    #[from(setup_l10n)] localization_service: &Arc<FluentLocalizationService>,
    #[case] error: &dyn AppError,
    #[case] expected: &str,
  ) {
    assert_error_message(localization_service, &error.code(), error.args(), expected);
  }
}
