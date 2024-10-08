use derive_builder::UninitializedFieldError;
use std::collections::HashMap;
use validator::{ValidationError, ValidationErrors};

pub trait AppError: std::error::Error {
  fn error_type(&self) -> String;

  fn status(&self) -> i32;

  fn status_u16(&self) -> u16;

  fn code(&self) -> String;

  fn args(&self) -> HashMap<String, String>;
}

pub fn validation_errors(field: &'static str, error: ValidationError) -> ValidationErrors {
  let mut errs = ValidationErrors::new();
  errs.add(field, error);
  errs
}

#[derive(Debug, strum::Display, strum::AsRefStr)]
#[strum(serialize_all = "snake_case")]
pub enum ErrorType {
  #[strum(serialize = "validation_error")]
  Validation,
  #[strum(serialize = "bad_request_error")]
  BadRequest,
  #[strum(serialize = "invalid_app_state")]
  InvalidAppState,
  #[strum(serialize = "internal_server_error")]
  InternalServer,
  #[strum(serialize = "authentication_error")]
  Authentication,
  #[strum(serialize = "not_found_error")]
  NotFound,
}

#[derive(Debug, PartialEq, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum ObjValidationError {
  #[error("validation_errors")]
  #[error_meta(error_type = ErrorType::BadRequest, status = 400)]
  ValidationErrors(#[from] ValidationErrors),

  #[error("file_pattern_mismatch")]
  #[error_meta(error_type = ErrorType::InternalServer, status = 400)]
  FilePatternMismatch(String),
}

#[derive(Debug, PartialEq, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("bad_request_error")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::BadRequest, status = 400)]
pub struct BadRequestError {
  reason: String,
}

#[derive(Debug, PartialEq, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("internal_server_error")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::InternalServer, status = 500)]
pub struct InternalServerError {
  reason: String,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("io_error")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::InternalServer, status = 500)]
pub struct IoError {
  #[from]
  source: std::io::Error,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("io_with_path_error")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::InternalServer, status = 500)]
pub struct IoWithPathError {
  #[source]
  source: std::io::Error,
  path: String,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("io_dir_create_error")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::InternalServer, status = 500)]
pub struct IoDirCreateError {
  #[source]
  source: std::io::Error,
  path: String,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("io_file_read_failed")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::InternalServer, status = 500)]
pub struct IoFileReadError {
  #[source]
  source: std::io::Error,
  path: String,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("io_file_write_failed")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::InternalServer, status = 500)]
pub struct IoFileWriteError {
  #[source]
  source: std::io::Error,
  path: String,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("io_file_delete_failed")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::InternalServer, status = 500)]
pub struct IoFileDeleteError {
  #[source]
  source: std::io::Error,
  path: String,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("serde_json_error")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::InternalServer, status = 500)]
pub struct SerdeJsonError {
  #[from]
  source: serde_json::Error,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("serde_json_with_path_error")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::InternalServer, status = 500)]
pub struct SerdeJsonWithPathError {
  #[source]
  source: serde_json::Error,
  path: String,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("serde_yaml_error")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::InternalServer, status = 500)]
pub struct SerdeYamlError {
  #[from]
  source: serde_yaml::Error,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("serde_yaml_with_path_error")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::InternalServer, status = 500)]
pub struct SerdeYamlWithPathError {
  #[source]
  source: serde_yaml::Error,
  path: String,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error("internal_network_error")]
#[error_meta(trait_to_impl = AppError,
  error_type = "ErrorType::InternalServer",
  status = 500,
)]
pub struct ReqwestError {
  #[from]
  source: reqwest::Error,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, strum::Display)]
#[strum(serialize_all = "snake_case")]
#[error_meta(trait_to_impl = AppError)]
#[non_exhaustive]
pub enum BuilderError {
  #[error_meta(error_type = ErrorType::InternalServer, status = 500)]
  UninitializedField(&'static str),
  #[error_meta(error_type = ErrorType::InternalServer, status = 500)]
  ValidationError(String),
}

impl From<UninitializedFieldError> for BuilderError {
  fn from(s: UninitializedFieldError) -> Self {
    Self::UninitializedField(s.field_name())
  }
}

impl From<String> for BuilderError {
  fn from(s: String) -> Self {
    Self::ValidationError(s)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    test_utils::{assert_error_message, fluent_bundle},
    Repo,
  };
  use fluent::{FluentBundle, FluentResource};
  use rstest::rstest;
  use std::{
    borrow::Cow,
    io::{Error as StdIoError, ErrorKind},
  };
  use validator::ValidationErrorsKind;

  #[rstest]
  #[case(&Repo::try_from("invalid-repo").unwrap_err(), "validation_error: value: does not match the huggingface repo pattern 'username/repo'")]
  #[case(&IoWithPathError::new(StdIoError::new(ErrorKind::NotFound, "file not found"), "test.txt".to_string()), "io_error: path: test.txt, file not found")]
  #[case(&IoDirCreateError::new(StdIoError::new(ErrorKind::AlreadyExists, "already exists"), "model-home".to_string()), "io_error: failed to create directory $BODHI_HOME/model-home, error: already exists")]
  #[case(&IoFileWriteError::new(StdIoError::new(ErrorKind::NotFound, "file not found"), "test.txt".to_string()), "io_error: failed to update file $BODHI_HOME/test.txt, error: file not found")]
  #[case(&IoFileReadError::new(StdIoError::new(ErrorKind::NotFound, "file not found"), "test.txt".to_string()), "io_error: failed to read file $BODHI_HOME/test.txt, error: file not found")]
  #[case(&IoFileDeleteError::new(StdIoError::new(ErrorKind::NotFound, "file not found"), "test.txt".to_string()), "io_error: failed to delete file $BODHI_HOME/test.txt, error: file not found")]
  #[case(&SerdeJsonError::new(serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err()), "error serializing/deserializing json: expected value at line 1 column 1")]
  #[case(&SerdeJsonWithPathError::new(serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err(), "test.json".to_string()), "error serializing/deserializing json: path: test.json, expected value at line 1 column 1")]
  #[case(&SerdeYamlError::new(serde_yaml::from_str::<serde_yaml::Value>("invalid: foo\n\tbar").unwrap_err()), "error serializing/deserializing yaml: found a tab character that violates indentation at line 2 column 1, while scanning a plain scalar at line 1 column 10")]
  #[case(&SerdeYamlWithPathError::new(serde_yaml::from_str::<serde_yaml::Value>("invalid: foo\n\tbar").unwrap_err(), "test.yaml".to_string()), "error serializing/deserializing yaml: path: test.yaml, found a tab character that violates indentation at line 2 column 1, while scanning a plain scalar at line 1 column 10")]
  #[case(&BadRequestError::new("invalid input".to_string()), "invalid request, reason: invalid input")]
  #[case(&InternalServerError::new("unexpected server error".to_string()), "internal_server_error: unexpected server error")]
  #[case(&IoError::new(StdIoError::new(ErrorKind::PermissionDenied, "test io error")), "io_error: test io error")]
  #[case(&ObjValidationError::ValidationErrors(ValidationErrors(HashMap::from([("field", ValidationErrorsKind::Field(vec![validator::ValidationError::new("value").with_message(Cow::Borrowed("validation failed"))]))]))), "validation_error: field: validation failed")]
  #[case(&ObjValidationError::FilePatternMismatch("huggingface/hub/models--invalid-repo/snapshots/model.gguf".to_string()), "file pattern does not match huggingface repo pattern, path: huggingface/hub/models--invalid-repo/snapshots/model.gguf")]
  fn test_objs_error_messages(
    fluent_bundle: FluentBundle<FluentResource>,
    #[case] error: &dyn AppError,
    #[case] expected: &str,
  ) {
    assert_error_message(&fluent_bundle, &error.code(), error.args(), expected);
  }

  #[rstest]
  #[tokio::test]
  async fn test_reqwest_error(fluent_bundle: FluentBundle<FluentResource>) {
    let reqwest_error = reqwest::Client::new()
      .get("http://foobar.nohost/")
      .send()
      .await
      .unwrap_err();
    let error = ReqwestError {
      source: reqwest_error,
    };
    assert_error_message(
      &fluent_bundle,
      &error.code(),
      error.args(),
      "error connecting to internal service: error sending request for url (http://foobar.nohost/)",
    );
  }

  #[rstest]
  #[case::uninitialized_field(
    &BuilderError::UninitializedField("field_name"),
    "builder_error: uninitialized field: field_name"
  )]
  #[case::validation_error(&BuilderError::ValidationError("validation failed".to_string()), "builder_error: validation error: validation failed")]
  #[case::file_pattern_mismatch(&ObjValidationError::FilePatternMismatch("test.txt".to_string()), "file pattern does not match huggingface repo pattern, path: test.txt")]
  fn test_object_error(
    fluent_bundle: FluentBundle<FluentResource>,
    #[case] error: &dyn AppError,
    #[case] expected: &str,
  ) {
    assert_error_message(&fluent_bundle, &error.code(), error.args(), &expected);
  }
}
