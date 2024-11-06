use llamacpp_sys::LlamaCppSysError;
use objs::ErrorType;
use std::{ffi::NulError, str::Utf8Error};

#[derive(Debug, PartialEq, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = objs::AppError)]
pub enum LlamaCppError {
  #[error("context_already_initialized")]
  #[error_meta(error_type = ErrorType::BadRequest, status = 400)]
  ContextAlreadyInitialized,
  #[error("lock_error")]
  #[error_meta(error_type = ErrorType::InternalServer, status = 500)]
  LockError,
  #[error("context_not_initialized")]
  #[error_meta(error_type = ErrorType::BadRequest, status = 400)]
  ContextNotInitialized,
  #[error(transparent)]
  #[error_meta(error_type = ErrorType::BadRequest, status = 400, code = "llama_cpp_error-llama_cpp_sys_error", args_delegate = false)]
  LlamaCppSys(#[from] LlamaCppSysError),
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
  #[error("common_params_init")]
  #[error_meta(error_type = ErrorType::BadRequest, status = 400)]
  CommonParamsInit(String),
  #[error("bodhi_params_new")]
  #[error_meta(error_type = ErrorType::BadRequest, status = 400)]
  BodhiParamsNew(String),
  #[error("bodhi_context_init")]
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
  #[error(transparent)]
  #[error_meta(error_type = ErrorType::BadRequest, status = 400, code = "bodhi_server_nul_error", args_delegate = false)]
  BodhiServerNullError(#[from] NulError),
  #[error("bodhi_server_callback_missing")]
  #[error_meta(error_type = ErrorType::BadRequest, status = 400)]
  CallbackMissing,
}

pub type Result<T> = std::result::Result<T, LlamaCppError>;

#[cfg(test)]
mod tests {
  use crate::error::LlamaCppError;
  use llamacpp_sys::LlamaCppSysError;
use objs::test_utils::setup_l10n;
  use objs::{test_utils::assert_error_message, AppError, FluentLocalizationService};
  use rstest::rstest;
  use std::ffi::CString;
  use std::sync::Arc;

  #[rstest]
  #[case::getu32(&LlamaCppError::CommonParamsGetU32(1), "common_params_get: error retrieving common_params u32 value for option 1")]
  #[case::updateu32(&LlamaCppError::CommonParamsUpdateU32(1, 2), "common_params_update: error updating common_params u32 value for option 1 with value 2")]
  #[case::geti32(&LlamaCppError::CommonParamsGetI32(1), "common_params_get: error retrieving common_params i32 value for option 1")]
  #[case::updatei32(&LlamaCppError::CommonParamsUpdateI32(1, 2), "common_params_update: error updating common_params i32 value for option 1 with value 2")]
  #[case::getstring(&LlamaCppError::CommonParamsGetString(1), "common_params_get: error retrieving common_params string value for option 1")]
  #[case::updatestring(&LlamaCppError::CommonParamsUpdateString(1, "newvalue".to_string()), "common_params_update: error updating common_params string value for option 1, value: newvalue")]
  #[case::init(&LlamaCppError::CommonParamsInit("unknown error".to_string()), "common_params_init: unknown error")]
  #[case::context_init(&LlamaCppError::BodhiContextInit("unknown error".to_string()), "bodhi_context: unknown error")]
  #[case::server_start(&LlamaCppError::BodhiServerStart("unknown error".to_string()), "bodhi_server_start: unknown error")]
  #[case::chat_completions(&LlamaCppError::BodhiServerChatCompletion("unknown error".to_string()), "bodhi_server_chat_completion: unknown error")]
  #[case::server_stop(&LlamaCppError::BodhiServerStop("unknown error".to_string()), "bodhi_server_stop: unknown error")]
  #[case::nul_error(&LlamaCppError::BodhiServerNullError(CString::new("nul\0error").unwrap_err()), "invalid cstring: nul byte found in provided data at position: 3")]
  #[case::llamacppsys(&LlamaCppError::LlamaCppSys(LlamaCppSysError::LibraryNotLoaded), "llama_cpp_sys: Library not loaded")]
  fn test_error_messages_llama_cpp_rs(
    #[from(setup_l10n)] localization_service: &Arc<FluentLocalizationService>,
    #[case] error: &dyn AppError,
    #[case] expected: &str,
  ) {
    assert_error_message(localization_service, &error.code(), error.args(), expected);
  }
}
