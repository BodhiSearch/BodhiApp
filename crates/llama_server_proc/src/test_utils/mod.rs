use http::response::Builder;
use reqwest::{Response, ResponseBuilderExt};
use rstest::fixture;
use std::path::PathBuf;
use url::Url;

#[fixture]
pub fn llama2_7b() -> PathBuf {
  let model_path = dirs::home_dir().unwrap().join(".cache/huggingface/hub/models--TheBloke--Llama-2-7B-Chat-GGUF/snapshots/191239b3e26b2882fb562ffccdd1cf0f65402adb/llama-2-7b-chat.Q4_K_M.gguf");
  assert!(
    model_path.exists(),
    "Model path does not exist: {}",
    model_path.display()
  );
  model_path.canonicalize().unwrap()
}

#[fixture]
pub fn llama2_7b_str(llama2_7b: PathBuf) -> String {
  llama2_7b.to_string_lossy().into_owned()
}

pub fn mock_response(body: impl Into<String>) -> Response {
  let url = Url::parse("http://127.0.0.1:8080").unwrap();
  let body: String = body.into();
  let hyper_response = Builder::new().url(url).status(200).body(body).unwrap();
  Response::from(hyper_response)
}
