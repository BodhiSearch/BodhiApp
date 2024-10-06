#[derive(Debug, errmeta_derive::ErrorMeta)]
pub enum TestError {
  #[error_meta(code = "bad_request", status = 200)]
  BadRequest,
}

fn main() {}
