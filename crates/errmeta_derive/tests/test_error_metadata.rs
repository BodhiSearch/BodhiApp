mod objs;

use crate::objs::ErrorMetas;
use errmeta_derive::ErrorMeta;
use rstest::rstest;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, strum::AsRefStr, strum::Display)]
#[strum(serialize_all = "snake_case")]
pub enum ErrorType {
  InternalServerError,
}

#[derive(Debug, thiserror::Error, ErrorMeta)]
#[error("inner error")]
#[error_meta(
  status = 500,
  code = "inner_error_code",
  error_type = "inner_error_type"
)]
pub struct InnerError;

#[derive(Debug, thiserror::Error, ErrorMeta)]
#[error("inner error fields")]
#[error_meta(
  status = 500,
  code = "inner_error_code",
  error_type = "inner_error_type"
)]
pub struct InnerErrorFields {
  msg: String,
  status: i32,
}

#[derive(Debug, thiserror::Error, ErrorMeta)]
#[error("inner error fields dup")]
#[error_meta(
  status = 500,
  code = "inner_error_code",
  error_type = "inner_error_type"
)]
pub struct InnerErrorFieldsDup {
  msg: String,
  status: i32,
}

#[derive(Debug, thiserror::Error, ErrorMeta)]
#[error("error with source")]
#[error_meta(
  status = 500,
  code = "error_with_source_code",
  error_type = "error_with_source_type"
)]
pub struct ErrorWithSource {
  #[source]
  inner: InnerErrorFields,
  path: String,
}

#[derive(Debug, thiserror::Error, ErrorMeta)]
enum TestError {
  #[error("test error message")]
  #[error_meta(status = 500, code = "test_error_code", error_type = "test_error_type")]
  TestError,
  #[error("error default code")]
  #[error_meta(status = 500, error_type = "test_error_default_code_type")]
  TestErrorDefaultCode,
  #[error("error type asref str")]
  #[error_meta(status = 500, error_type = ErrorType::InternalServerError)]
  WithErrorTypeAsRefStr,
  #[error("error with fields")]
  #[error_meta(status = 500, code = "test_error_code", error_type = "test_error_type")]
  WithFields { field1: String, field2: i32 },
  #[error("error with tuples")]
  #[error_meta(status = 500, code = "test_error_code", error_type = "test_error_type")]
  WithTuples(String, i32),
  #[error(transparent)]
  Transparent(#[from] InnerError),
  #[error(transparent)]
  TransparentFields(#[from] InnerErrorFields),
  #[error(transparent)]
  #[error_meta(status = 400, code = "override_code", error_type = "override_type")]
  TransparentOverride(#[from] InnerErrorFieldsDup),
  #[error(transparent)]
  TransparentSource(#[from] ErrorWithSource),
}

impl From<&TestError> for ErrorMetas {
  fn from(error: &TestError) -> Self {
    let error_type = error.error_type();
    let status = error.status();
    let code = error.code();
    let args = error.args();
    Self {
      message: error.to_string(),
      status,
      code,
      error_type,
      args,
    }
  }
}

#[rstest]
#[case::default(TestError::TestError, ErrorMetas {
  message: "test error message".to_string(),
  status: 500,
  code: "test_error_code".to_string(),
  error_type: "test_error_type".to_string(),
  args: HashMap::new(),
})]
#[case::default_code(TestError::TestErrorDefaultCode, ErrorMetas {
  message: "error default code".to_string(),
  status: 500,
  code: "test_error-test_error_default_code".to_string(),
  error_type: "test_error_default_code_type".to_string(),
  args: HashMap::new(),
})]
#[case::error_type_asref_str(TestError::WithErrorTypeAsRefStr, ErrorMetas {
  message: "error type asref str".to_string(),
  status: 500,
  code: "test_error-with_error_type_as_ref_str".to_string(),
  error_type: "internal_server_error".to_string(),
  args: HashMap::new(),
})]
#[case::with_fields(TestError::WithFields { field1: "value1".to_string(), field2: 200 }, ErrorMetas {
  message: "error with fields".to_string(),
  status: 500,
  code: "test_error_code".to_string(),
  error_type: "test_error_type".to_string(),
  args: HashMap::from([("field1".to_string(), "value1".to_string()), ("field2".to_string(), "200".to_string())]),
})]
#[case::with_tuples(TestError::WithTuples("value1".to_string(), 200), ErrorMetas {
  message: "error with tuples".to_string(),
  status: 500,
  code: "test_error_code".to_string(),
  error_type: "test_error_type".to_string(),
  args: HashMap::from([("var_0".to_string(), "value1".to_string()), ("var_1".to_string(), "200".to_string())]),
})]
#[case::transparent(TestError::Transparent(InnerError {}), ErrorMetas {
  message: "inner error".to_string(),
  status: 500,
  code: "inner_error_code".to_string(),
  error_type: "inner_error_type".to_string(),
  args: HashMap::new(),
})]
#[case::transparent_fields(TestError::TransparentFields(InnerErrorFields { msg: "value1".to_string(), status: 200 }), ErrorMetas {
  message: "inner error fields".to_string(),
  status: 500,
  code: "inner_error_code".to_string(),
  error_type: "inner_error_type".to_string(),
  args: HashMap::from([("msg".to_string(), "value1".to_string()), ("status".to_string(), "200".to_string())]),
})]
#[case::transparent_fields_override(TestError::TransparentOverride(InnerErrorFieldsDup { msg: "value1".to_string(), status: 200 }), ErrorMetas {
  message: "inner error fields dup".to_string(),
  status: 400,
  code: "override_code".to_string(),
  error_type: "override_type".to_string(),
  args: HashMap::from([("msg".to_string(), "value1".to_string()), ("status".to_string(), "200".to_string())]),
})]
#[case::transparent_source(
  TestError::TransparentSource(
    ErrorWithSource {
      inner: InnerErrorFields {
        msg: "inner message".to_string(), 
        status: 318
      },
      path: "value1".to_string()
    }
  ),
  ErrorMetas {
    message: "error with source".to_string(),
    status: 500,
    code: "error_with_source_code".to_string(),
    error_type: "error_with_source_type".to_string(),
    args: HashMap::from([
      ("inner".to_string(), "inner error fields".to_string()),
      ("path".to_string(), "value1".to_string()),
    ]),
  }
)]
fn test_error_metadata(#[case] error: TestError, #[case] expected: ErrorMetas) {
  let error_metas = ErrorMetas::from(&error);
  assert_eq!(error_metas, expected);
}
