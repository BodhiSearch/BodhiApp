mod objs;

use errmeta_derive::ErrorMeta;
use objs::ErrorMetas;
use rstest::rstest;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, strum::AsRefStr, strum::Display)]
#[strum(serialize_all = "snake_case")]
pub enum ErrorType {
  BadRequest,
  InternalServerError,
}

pub struct Status(u16);

impl From<Status> for i32 {
  fn from(status: Status) -> Self {
    status.0 as i32
  }
}

fn get_status() -> Status {
  Status(500)
}

fn get_error_type() -> &'static str {
  "dynamic_error_type"
}

#[derive(Debug, thiserror::Error, ErrorMeta)]
enum TestErrorExpr {
  #[error("error with expression status")]
  #[error_meta(status = get_status(), error_type = "expr_status_error")]
  ExprStatus,

  #[error("error with expression error type")]
  #[error_meta(status = 400, error_type = get_error_type())]
  ExprErrorType,

  #[error("error with expression error code")]
  #[error_meta(status = 400, error_type = "expr_error_code_error", code = self.get_code())]
  ExprErrorCode,

  #[error("error with enum error type")]
  #[error_meta(status = 500, error_type = ErrorType::InternalServerError)]
  EnumErrorType,

  #[error("error with both expressions")]
  #[error_meta(status = get_status(), error_type = get_error_type(), code = self.get_code())]
  AllExpr,

  #[error("error with enum and expression")]
  #[error_meta(status = get_status(), error_type = ErrorType::BadRequest)]
  EnumAndExpr,
}

impl TestErrorExpr {
  fn get_code(&self) -> String {
    self.to_string()
  }
}

impl From<&TestErrorExpr> for ErrorMetas {
  fn from(error: &TestErrorExpr) -> Self {
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
#[case::expr_status(TestErrorExpr::ExprStatus, ErrorMetas {
    message: "error with expression status".to_string(),
    status: 500,
    code: "test_error_expr-expr_status".to_string(),
    error_type: "expr_status_error".to_string(),
    args: HashMap::new(),
})]
#[case::expr_error_type(TestErrorExpr::ExprErrorType, ErrorMetas {
    message: "error with expression error type".to_string(),
    status: 400,
    code: "test_error_expr-expr_error_type".to_string(),
    error_type: "dynamic_error_type".to_string(),
    args: HashMap::new(),
})]
#[case::enum_error_type(TestErrorExpr::EnumErrorType, ErrorMetas {
    message: "error with enum error type".to_string(),
    status: 500,
    code: "test_error_expr-enum_error_type".to_string(),
    error_type: "internal_server_error".to_string(),
    args: HashMap::new(),
})]
#[case::expr_error_code(TestErrorExpr::ExprErrorCode, ErrorMetas {
    message: "error with expression error code".to_string(),
    status: 400,
    code: "error with expression error code".to_string(),
    error_type: "expr_error_code_error".to_string(),
    args: HashMap::new(),
})]
#[case::both_expr(TestErrorExpr::AllExpr, ErrorMetas {
    message: "error with both expressions".to_string(),
    status: 500,
    code: "error with both expressions".to_string(),
    error_type: "dynamic_error_type".to_string(),
    args: HashMap::new(),
})]
#[case::enum_and_expr(TestErrorExpr::EnumAndExpr, ErrorMetas {
    message: "error with enum and expression".to_string(),
    status: 500,
    code: "test_error_expr-enum_and_expr".to_string(),
    error_type: "bad_request".to_string(),
    args: HashMap::new(),
})]
fn test_error_metadata_expr(#[case] error: TestErrorExpr, #[case] expected: ErrorMetas) {
  let error_metas = ErrorMetas::from(&error);
  assert_eq!(error_metas, expected);
}
