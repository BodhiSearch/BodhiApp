pub struct InvalidErrorType;

#[derive(Debug, errmeta_derive::ErrorMeta)]
enum TestError {
  #[error_meta(code = "bad_request", error_type = InvalidErrorType)]
  BadRequest,
}

fn main() {}
