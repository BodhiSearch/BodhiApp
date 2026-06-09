use crate::auth::AuthContextError;
use crate::inference::LocalLlamaError;
use crate::{ReqwestError, UrlValidationError};
use errmeta::{impl_error_from, AppError, ErrorType};

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum AiApiClientFactoryError {
  #[error(transparent)]
  Reqwest(#[from] ReqwestError),

  #[error(transparent)]
  Auth(#[from] AuthContextError),

  #[error("API error: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  ApiError(String),

  #[error("LlmLibertyOauth aliases must be constructed via for_liberty.")]
  #[error_meta(error_type = ErrorType::InternalServer, code = "ai_api_client_factory_error-liberty_requires_credentials")]
  LibertyRequiresCredentials,

  #[error("Local inference is not supported in cluster deployment mode.")]
  #[error_meta(error_type = ErrorType::BadRequest, code = "ai_api_client_factory_error-local_not_supported_in_cluster")]
  LocalNotSupportedInCluster,

  #[error("Model-router aliases are routed through their targets, not forwarded directly.")]
  #[error_meta(error_type = ErrorType::InternalServer, code = "ai_api_client_factory_error-model_router_not_forwardable")]
  ModelRouterNotForwardable,

  #[error("LLM Liberty provider '{0}' is not supported.")]
  #[error_meta(error_type = ErrorType::BadRequest, code = "ai_api_client_factory_error-liberty_provider_unsupported")]
  LibertyProviderUnsupported(String),

  #[error("API authentication failed: {0}.")]
  #[error_meta(error_type = ErrorType::Authentication)]
  Unauthorized(String),

  #[error("Resource not found: {0}.")]
  #[error_meta(error_type = ErrorType::NotFound)]
  NotFound(String),

  #[error("Too many requests to API. Please wait and try again.")]
  #[error_meta(error_type = ErrorType::ServiceUnavailable)]
  RateLimit(String),

  #[error("Message too long. Maximum length is {max_length} but received {actual_length}.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  PromptTooLong {
    max_length: usize,
    actual_length: usize,
  },

  #[error("API model '{0}' not found.")]
  #[error_meta(error_type = ErrorType::NotFound)]
  ModelNotFound(String),

  #[error(transparent)]
  #[error_meta(error_type = ErrorType::BadRequest, args_delegate = false)]
  UrlValidation(#[from] UrlValidationError),
}

impl AiApiClientFactoryError {
  pub fn status_to_error(status: reqwest::StatusCode, body: String) -> Self {
    match status {
      reqwest::StatusCode::UNAUTHORIZED => Self::Unauthorized(body),
      reqwest::StatusCode::NOT_FOUND => Self::NotFound(body),
      reqwest::StatusCode::TOO_MANY_REQUESTS => Self::RateLimit(body),
      _ => Self::ApiError(format!("Status {}: {}", status, body)),
    }
  }
}

impl From<LocalLlamaError> for AiApiClientFactoryError {
  fn from(e: LocalLlamaError) -> Self {
    match e {
      LocalLlamaError::ModelNotFound(m) => AiApiClientFactoryError::NotFound(m),
      LocalLlamaError::ExecNotFound(p) => AiApiClientFactoryError::ApiError(p),
      LocalLlamaError::Internal(s) => AiApiClientFactoryError::ApiError(s),
    }
  }
}

impl_error_from!(
  reqwest::Error,
  AiApiClientFactoryError::Reqwest,
  crate::ReqwestError
);

pub(crate) type Result<T> = std::result::Result<T, AiApiClientFactoryError>;
