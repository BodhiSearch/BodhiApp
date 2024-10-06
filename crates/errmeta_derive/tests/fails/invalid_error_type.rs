pub struct InvalidErrorType;

#[derive(Debug, errmeta_derive::ErrorMeta)]
enum TestError {
  #[error_meta(code = "bad_request", status = 200, error_type = InvalidErrorType)]
  BadRequest,
}

fn main() {}
