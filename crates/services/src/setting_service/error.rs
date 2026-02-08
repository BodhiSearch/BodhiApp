use objs::{impl_error_from, AppError, ErrorType, IoError, SerdeYamlError};

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum SettingServiceError {
  #[error(transparent)]
  Io(#[from] IoError),
  #[error(transparent)]
  SerdeYaml(#[from] SerdeYamlError),
  #[error("Settings lock failed: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  LockError(String),
  #[error("Invalid settings source.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  InvalidSource,
}

impl_error_from!(::std::io::Error, SettingServiceError::Io, ::objs::IoError);
impl_error_from!(
  ::serde_yaml::Error,
  SettingServiceError::SerdeYaml,
  ::objs::SerdeYamlError
);

pub type Result<T> = std::result::Result<T, SettingServiceError>;
