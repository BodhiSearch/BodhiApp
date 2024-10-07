use derive_builder::UninitializedFieldError;
use validator::{ValidationError, ValidationErrors};

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
pub enum ObjError {
  #[error(transparent)]
  #[error_meta(error_type = ErrorType::Validation, status = 400, code="obj_error-validation", args_delegate = false)]
  Validation(#[from] ValidationErrors),

  #[error("file_pattern_mismatch")]
  #[error_meta(error_type = ErrorType::InternalServer, status = 400)]
  FilePatternMismatch(String),

  #[error(transparent)]
  IoWithPathError(#[from] IoWithPathError),

  #[error(transparent)]
  SerdeJsonError(#[from] SerdeJsonError),

  #[error(transparent)]
  Builder(#[from] BuilderError),
}

#[allow(unused)]
pub type Result<T> = std::result::Result<T, ObjError>;

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

#[derive(Debug, PartialEq, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("bad_request_error")]
#[error_meta(error_type = ErrorType::BadRequest, status = 400)]
pub struct BadRequestError {
  reason: String,
}

#[derive(Debug, PartialEq, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("internal_server_error")]
#[error_meta(error_type = ErrorType::InternalServer, status = 500)]
pub struct InternalServerError {
  reason: String,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("io_error")]
#[error_meta(error_type = ErrorType::InternalServer, status = 500)]
pub struct IoError {
  #[from]
  source: std::io::Error,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("io_with_path_error")]
#[error_meta(error_type = ErrorType::InternalServer, status = 500)]
pub struct IoWithPathError {
  #[source]
  source: std::io::Error,
  path: String,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("io_dir_create_error")]
#[error_meta(error_type = ErrorType::InternalServer, status = 500)]
pub struct IoDirCreateError {
  #[source]
  source: std::io::Error,
  path: String,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("io_file_read_failed")]
#[error_meta(error_type = ErrorType::InternalServer, status = 500)]
pub struct IoFileReadError {
  #[source]
  source: std::io::Error,
  path: String,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("io_file_write_failed")]
#[error_meta(error_type = ErrorType::InternalServer, status = 500)]
pub struct IoFileWriteError {
  #[source]
  source: std::io::Error,
  path: String,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("io_file_delete_failed")]
#[error_meta(error_type = ErrorType::InternalServer, status = 500)]
pub struct IoFileDeleteError {
  #[source]
  source: std::io::Error,
  path: String,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("serde_json_error")]
#[error_meta(error_type = ErrorType::InternalServer, status = 500)]
pub struct SerdeJsonError {
  #[from]
  source: serde_json::Error,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("serde_json_with_path_error")]
#[error_meta(error_type = ErrorType::InternalServer, status = 500)]
pub struct SerdeJsonWithPathError {
  #[source]
  source: serde_json::Error,
  path: String,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("serde_yaml_error")]
#[error_meta(error_type = ErrorType::InternalServer, status = 500)]
pub struct SerdeYamlError {
  #[from]
  source: serde_yaml::Error,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("serde_yaml_with_path_error")]
#[error_meta(error_type = ErrorType::InternalServer, status = 500)]
pub struct SerdeYamlWithPathError {
  #[source]
  source: serde_yaml::Error,
  path: String,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error("internal_network_error")]
#[error_meta(error_type = ErrorType::InternalServer, status = 500, args_delegate=false)]
pub struct ReqwestError {
  #[from]
  source: reqwest::Error,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, strum::Display)]
#[strum(serialize_all = "snake_case")]
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

  #[rstest]
  fn test_validation_error(fluent_bundle: FluentBundle<FluentResource>) {
    let validation_error = Repo::try_from("invalid-repo").unwrap_err();
    assert_error_message(
      &fluent_bundle,
      &validation_error.code(),
      validation_error.args(),
      "validation_error: value: does not match the huggingface repo pattern 'username/repo'",
    );
  }

  #[rstest]
  fn test_io_with_path_error(fluent_bundle: FluentBundle<FluentResource>) {
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let error = IoWithPathError::new(io_error, "test.txt".to_string());
    assert_error_message(
      &fluent_bundle,
      &error.code(),
      error.args(),
      "io_error: path: test.txt, file not found",
    );
  }

  #[rstest]
  fn test_io_dir_create_error(fluent_bundle: FluentBundle<FluentResource>) {
    let io_error = std::io::Error::new(std::io::ErrorKind::AlreadyExists, "already exists");
    let error = IoDirCreateError::new(io_error, "model-home".to_string());
    assert_error_message(
      &fluent_bundle,
      &error.code(),
      error.args(),
      "io_error: failed to create directory $BODHI_HOME/model-home, error: already exists",
    );
  }

  #[rstest]
  fn test_io_file_read_error(fluent_bundle: FluentBundle<FluentResource>) {
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let error = IoFileWriteError::new(io_error, "test.txt".to_string());
    assert_error_message(
      &fluent_bundle,
      &error.code(),
      error.args(),
      "io_error: failed to update file $BODHI_HOME/test.txt, error: file not found",
    );
  }

  #[rstest]
  fn test_io_file_write_error(fluent_bundle: FluentBundle<FluentResource>) {
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let error = IoFileReadError::new(io_error, "test.txt".to_string());
    assert_error_message(
      &fluent_bundle,
      &error.code(),
      error.args(),
      "io_error: failed to read file $BODHI_HOME/test.txt, error: file not found",
    );
  }

  #[rstest]
  fn test_io_file_delete_error(fluent_bundle: FluentBundle<FluentResource>) {
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let error = IoFileDeleteError::new(io_error, "test.txt".to_string());
    assert_error_message(
      &fluent_bundle,
      &error.code(),
      error.args(),
      "io_error: failed to delete file $BODHI_HOME/test.txt, error: file not found",
    );
  }

  #[rstest]
  fn test_serde_json_error(fluent_bundle: FluentBundle<FluentResource>) {
    let json_error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
    let error = SerdeJsonError::new(json_error);
    assert_error_message(
      &fluent_bundle,
      &error.code(),
      error.args(),
      "error serializing/deserializing json: expected value at line 1 column 1",
    );
  }

  #[rstest]
  fn test_serde_json_with_path_error(fluent_bundle: FluentBundle<FluentResource>) {
    let json_error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
    let error = SerdeJsonWithPathError::new(json_error, "test.json".to_string());
    assert_error_message(
      &fluent_bundle,
      &error.code(),
      error.args(),
      "error serializing/deserializing json: path: test.json, expected value at line 1 column 1",
    );
  }

  #[rstest]
  fn test_yaml_serialization_error(fluent_bundle: FluentBundle<FluentResource>) {
    let yaml_error = serde_yaml::from_str::<serde_yaml::Value>("invalid: foo\n\tbar").unwrap_err();
    let error = SerdeYamlError::new(yaml_error);
    assert_error_message(
      &fluent_bundle,
      &error.code(),
      error.args(),
      "error serializing/deserializing yaml: found a tab character that violates indentation at line 2 column 1, while scanning a plain scalar at line 1 column 10",
    );
  }

  #[rstest]
  fn test_yaml_serialization_with_path_error(fluent_bundle: FluentBundle<FluentResource>) {
    let yaml_error = serde_yaml::from_str::<serde_yaml::Value>("invalid: foo\n\tbar").unwrap_err();
    let error = SerdeYamlWithPathError::new(yaml_error, "test.yaml".to_string());
    assert_error_message(
      &fluent_bundle,
      &error.code(),
      error.args(),
      "error serializing/deserializing yaml: path: test.yaml, found a tab character that violates indentation at line 2 column 1, while scanning a plain scalar at line 1 column 10",
    );
  }

  #[rstest]
  fn test_bad_request_error(fluent_bundle: FluentBundle<FluentResource>) {
    let error = BadRequestError::new("invalid input".to_string());
    assert_error_message(
      &fluent_bundle,
      &error.code(),
      error.args(),
      "invalid request, reason: invalid input",
    );
  }

  #[rstest]
  fn test_internal_server_error(fluent_bundle: FluentBundle<FluentResource>) {
    let error = InternalServerError::new("unexpected server error".to_string());
    assert_error_message(
      &fluent_bundle,
      &error.code(),
      error.args(),
      "internal_server_error: unexpected server error",
    );
  }

  #[rstest]
  fn test_io_error(fluent_bundle: FluentBundle<FluentResource>) {
    let io_error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "test io error");
    let error = IoError::new(io_error);
    assert_error_message(
      &fluent_bundle,
      &error.code(),
      error.args(),
      "io_error: test io error",
    );
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
    ObjError::Builder(BuilderError::UninitializedField("field_name")),
    "builder_error: uninitialized field: field_name"
  )]
  #[case::validation_error(ObjError::Builder(BuilderError::ValidationError("validation failed".to_string())), "builder_error: validation error: validation failed")]
  #[case(ObjError::FilePatternMismatch("test.txt".to_string()), "file pattern does not match huggingface repo pattern, path: test.txt")]
  fn test_object_error(
    fluent_bundle: FluentBundle<FluentResource>,
    #[case] error: ObjError,
    #[case] expected: &str,
  ) {
    assert_error_message(&fluent_bundle, &error.code(), error.args(), &expected);
  }
}
