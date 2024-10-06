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
  use crate::Repo;

  use super::*;
  use fluent::{FluentBundle, FluentResource};
  use rstest::{fixture, rstest};
  use std::{collections::HashMap, fs};

  #[fixture]
  fn fluent_bundle() -> FluentBundle<FluentResource> {
    let ftl_string =
      fs::read_to_string("tests/messages/test.ftl").expect("Failed to read FTL file");
    let res = FluentResource::try_new(ftl_string).expect("Failed to parse FTL resource");
    let mut bundle = FluentBundle::default();
    bundle
      .add_resource(res)
      .expect("Failed to add FTL resource to bundle");
    bundle
  }

  // Test helper function
  fn assert_error_message(
    bundle: &FluentBundle<FluentResource>,
    code: &str,
    args: HashMap<String, String>,
    expected: &str,
  ) {
    let message = bundle
      .get_message(code)
      .expect(&format!("Message not found, code: {code}"))
      .value()
      .expect(&format!("Message has no value, code: {code}"));
    let mut errors = Vec::new();
    let args = args
      .into_iter()
      .map(|(k, v)| (k.to_string(), v.to_string()))
      .collect();
    let formatted = bundle.format_pattern(message, Some(&args), &mut errors);
    assert_eq!(
      errors
        .first()
        .map(|err| err.to_string())
        .unwrap_or_default(),
      "",
      "formatting errors occurred"
    );
    assert!(errors.is_empty(), "formatting errors occurred");
    assert_eq!(formatted.to_string(), expected);
  }

  #[rstest]
  fn test_validation_error(fluent_bundle: FluentBundle<FluentResource>) {
    let validation_error = Repo::try_from("invalid-repo").unwrap_err();
    assert_error_message(
      &fluent_bundle,
      &validation_error.code(),
      validation_error.args(),
      "validation_error: \u{2068}value: does not match the huggingface repo pattern 'username/repo'\u{2069}",
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
      "io_error: path: \u{2068}test.txt\u{2069}, \u{2068}file not found\u{2069}",
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
      "io_error: failed to create directory $BODHI_HOME/\u{2068}model-home\u{2069}, error: \u{2068}already exists\u{2069}",
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
      "io_error: failed to update file $BODHI_HOME/\u{2068}test.txt\u{2069}, error: \u{2068}file not found\u{2069}",
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
      "io_error: failed to read file $BODHI_HOME/\u{2068}test.txt\u{2069}, error: \u{2068}file not found\u{2069}",
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
      "io_error: failed to delete file $BODHI_HOME/\u{2068}test.txt\u{2069}, error: \u{2068}file not found\u{2069}",
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
      "error serializing/deserializing json: \u{2068}expected value at line 1 column 1\u{2069}",
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
      "error serializing/deserializing json: path: \u{2068}test.json\u{2069}, \u{2068}expected value at line 1 column 1\u{2069}",
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
      "error serializing/deserializing yaml: \u{2068}found a tab character that violates indentation at line 2 column 1, while scanning a plain scalar at line 1 column 10\u{2069}",
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
      "error serializing/deserializing yaml: path: \u{2068}test.yaml\u{2069}, \u{2068}found a tab character that violates indentation at line 2 column 1, while scanning a plain scalar at line 1 column 10\u{2069}",
    );
  }

  #[rstest]
  fn test_bad_request_error(fluent_bundle: FluentBundle<FluentResource>) {
    let error = BadRequestError::new("invalid input".to_string());
    assert_error_message(
      &fluent_bundle,
      &error.code(),
      error.args(),
      "invalid request, reason: \u{2068}invalid input\u{2069}",
    );
  }

  #[rstest]
  fn test_internal_server_error(fluent_bundle: FluentBundle<FluentResource>) {
    let error = InternalServerError::new("unexpected server error".to_string());
    assert_error_message(
      &fluent_bundle,
      &error.code(),
      error.args(),
      "internal_server_error: \u{2068}unexpected server error\u{2069}",
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
      "io_error: \u{2068}test io error\u{2069}",
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
      "error connecting to internal service: \u{2068}error sending request for url (http://foobar.nohost/)\u{2069}",
    );
  }

  #[rstest]
  #[case::uninitialized_field(
    ObjError::Builder(BuilderError::UninitializedField("field_name")),
    "builder_error: uninitialized field: \u{2068}field_name\u{2069}"
  )]
  #[case::validation_error(ObjError::Builder(BuilderError::ValidationError("validation failed".to_string())), "builder_error: validation error: \u{2068}validation failed\u{2069}")]
  #[case(ObjError::FilePatternMismatch("test.txt".to_string()), "file pattern does not match huggingface repo pattern, path: \u{2068}test.txt\u{2069}")]
  fn test_object_error(
    fluent_bundle: FluentBundle<FluentResource>,
    #[case] error: ObjError,
    #[case] expected: &str,
  ) {
    assert_error_message(&fluent_bundle, &error.code(), error.args(), &expected);
  }
}
