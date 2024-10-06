use errmeta_derive::ErrorMeta;
use rstest::rstest;
use std::collections::HashMap;

#[derive(Debug, thiserror::Error, ErrorMeta)]
#[error("inner error")]
#[error_meta(
  status = 400,
  code = "inner_error_code",
  error_type = "inner_error_type"
)]
pub struct InnerError {
  field1: String,
  status: i32,
}

#[derive(Debug, thiserror::Error, ErrorMeta)]
#[error("inner error dup")]
#[error_meta(
  status = 400,
  code = "inner_error_code",
  error_type = "inner_error_type"
)]
pub struct InnerErrorDup {
  field1: String,
  status: i32,
}

#[derive(Debug, thiserror::Error, ErrorMeta)]
enum TestArgsDelegate {
  #[error(transparent)]
  #[error_meta(
    status = 500,
    code = "test_error_code",
    error_type = "test_error_type",
    args_delegate = true
  )]
  TestError(#[from] InnerError),
  #[error(transparent)]
  #[error_meta(
    status = 500,
    code = "test_error_code",
    error_type = "test_error_type",
    args_delegate = false
  )]
  TestErrorDelegateFalse(#[from] InnerErrorDup),
}

#[derive(Debug, thiserror::Error, ErrorMeta)]
enum TestArgsDelegateFalse {
  #[error(transparent)]
  #[error_meta(
    status = 500,
    code = "test_error_code",
    error_type = "test_error_type",
    args_delegate = false
  )]
  TestError(#[from] InnerError),
}

#[rstest]
#[case(TestArgsDelegate::TestError(InnerError {
  field1: "value".to_string(),
  status: 400,
}), HashMap::from([
  ("field1".to_string(), "value".to_string()),
  ("status".to_string(), "400".to_string())
]))]
#[case(TestArgsDelegate::TestErrorDelegateFalse(InnerErrorDup {
  field1: "value".to_string(),
  status: 400,
}), HashMap::from([
  ("error".to_string(), "inner error dup".to_string()),
]))]
fn test_args_delegate(#[case] error: TestArgsDelegate, #[case] args: HashMap<String, String>) {
  assert_eq!(error.args(), args);
}
