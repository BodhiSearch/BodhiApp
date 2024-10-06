#[derive(errmeta_derive::ErrorMeta)]
enum TestError {
  #[error_meta(code = "bad_request", error_type = "invalid_request")]
  BadRequest,
}

fn main() {}
