use errmeta_derive::ErrorMeta;
use rstest::rstest;
use std::collections::HashMap;

#[derive(Debug, thiserror::Error, ErrorMeta)]
#[error("inner error")]
#[error_meta(code = "inner_error_code", error_type = "inner_error_type")]
pub struct InnerError {
  field1: String,
}

#[derive(Debug, thiserror::Error, ErrorMeta)]
#[error("inner error dup")]
#[error_meta(code = "inner_error_code", error_type = "inner_error_type")]
pub struct InnerErrorDup {
  field1: String,
}

#[derive(Debug, thiserror::Error, ErrorMeta)]
enum TestArgsDelegate {
  #[error(transparent)]
  #[error_meta(
    code = "test_error_code",
    error_type = "test_error_type",
    args_delegate = true
  )]
  TestError(#[from] InnerError),
  #[error(transparent)]
  #[error_meta(
    code = "test_error_code",
    error_type = "test_error_type",
    args_delegate = false
  )]
  TestErrorDelegateFalse(#[from] InnerErrorDup),
}

#[allow(dead_code)]
#[derive(Debug, thiserror::Error, ErrorMeta)]
enum TestArgsDelegateFalse {
  #[error(transparent)]
  #[error_meta(
    code = "test_error_code",
    error_type = "test_error_type",
    args_delegate = false
  )]
  TestError(#[from] InnerError),
}

#[rstest]
#[case(TestArgsDelegate::TestError(InnerError {
  field1: "value".to_string(),
}), HashMap::from([
  ("field1".to_string(), "value".to_string()),
]))]
#[case(TestArgsDelegate::TestErrorDelegateFalse(InnerErrorDup {
  field1: "value".to_string(),
}), HashMap::from([
  ("error".to_string(), "inner error dup".to_string()),
]))]
fn test_args_delegate(#[case] error: TestArgsDelegate, #[case] args: HashMap<String, String>) {
  assert_eq!(args, error.args());
}
