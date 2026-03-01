use crate::db::DbError;
use crate::SerdeYamlError;
use errmeta::{impl_error_from, AppError, ErrorType, IoError};

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum SettingsMetadataError {
  #[error("cannot parse {1} as {0}")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  InvalidValueType(super::setting_objs::SettingMetadata, serde_json::Value),
  #[error("passed value is not a valid value: {0}")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  InvalidValue(serde_json::Value),
  #[error("value is null")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  NullValue,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum SettingServiceError {
  #[error(transparent)]
  Io(#[from] IoError),
  #[error(transparent)]
  SerdeYaml(#[from] SerdeYamlError),
  #[error(transparent)]
  Db(#[from] DbError),
  #[error("Settings lock failed: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  LockError(String),
  #[error("Invalid settings source.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  InvalidSource,
  #[error("Setting key '{0}' cannot be updated via database.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  InvalidKey(String),
}

impl_error_from!(
  ::std::io::Error,
  SettingServiceError::Io,
  ::errmeta::IoError
);
impl_error_from!(
  ::serde_yaml::Error,
  SettingServiceError::SerdeYaml,
  crate::SerdeYamlError
);

pub(crate) type Result<T> = std::result::Result<T, SettingServiceError>;
