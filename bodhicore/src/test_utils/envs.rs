pub fn hf_test_token_allowed() -> Option<String> {
  dotenv::from_filename(".env.test").ok().unwrap();
  Some(std::env::var("HF_TEST_TOKEN_ALLOWED").unwrap())
}

pub fn hf_test_token_public() -> Option<String> {
  dotenv::from_filename(".env.test").ok().unwrap();
  Some(std::env::var("HF_TEST_TOKEN_PUBLIC").unwrap())
}
