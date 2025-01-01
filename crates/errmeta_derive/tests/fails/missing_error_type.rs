#[derive(Debug, errmeta_derive::ErrorMeta)]
pub enum TestError {
  #[error_meta(code = "bad_request")]
  BadRequest,
}

fn main() {}
