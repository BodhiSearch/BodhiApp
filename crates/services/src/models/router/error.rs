use crate::db::DbError;
use crate::AiApiClientFactoryError;
use errmeta::{AppError, EntityError, ErrorType};

/// Errors for model-router validation (create/update) and request-time routing.
///
/// Upstream HTTP failures are NEVER wrapped here — they arrive as a committable
/// `Response` (with the upstream status) and are returned verbatim. This type is
/// reserved for structural problems: an empty active chain, dangling/nested/invalid
/// references, and (at create/update) validation failures.
#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum ModelRouterError {
  #[error("Model-router has no active (enabled) targets to route to.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  EmptyChain,

  #[error("Referenced alias '{alias}' does not exist.")]
  #[error_meta(error_type = ErrorType::NotFound)]
  ReferencedAliasNotFound { alias: String },

  #[error("Target '{alias}' references another model-router; nesting is not allowed.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  NestedRouterNotAllowed { alias: String },

  #[error("A target cannot reference the model-router being defined ('{alias}').")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  SelfReference { alias: String },

  #[error("Model '{model}' is not valid for referenced alias '{alias}'.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  InvalidPinnedModel { alias: String, model: String },

  #[error(
    "Referenced alias '{alias}' has format '{api_format}' which does not support chat completions."
  )]
  #[error_meta(error_type = ErrorType::BadRequest)]
  TargetFormatUnsupported { alias: String, api_format: String },

  #[error("A model-router named '{alias}' already exists.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  AliasExists { alias: String },

  #[error(transparent)]
  #[error_meta(error_type = ErrorType::NotFound, code = "model_router_error-not_found")]
  NotFound(#[from] EntityError),

  #[error(transparent)]
  #[error_meta(error_type = ErrorType::Authentication, code = "model_router_error-auth")]
  Auth(#[from] crate::auth::AuthContextError),

  #[error(transparent)]
  #[error_meta(args_delegate = false)]
  Db(#[from] DbError),

  #[error(transparent)]
  #[error_meta(args_delegate = false)]
  Forward(#[from] AiApiClientFactoryError),
}

impl ModelRouterError {
  /// A genuine transport failure (connection refused, timeout, upstream client
  /// error) — transient, so the target should be cooled. Distinguished from a
  /// *structural* skip (dangling/nested/unsupported reference), which is not
  /// transient and must NOT be cooled.
  pub fn is_transport_failure(&self) -> bool {
    matches!(self, ModelRouterError::Forward(_))
  }
}
