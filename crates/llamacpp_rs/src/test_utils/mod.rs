use rstest::fixture;
use std::path::PathBuf;

#[fixture]
pub fn llama2_7b() -> PathBuf {
  let model_path = dirs::home_dir().unwrap().join(".cache/huggingface/hub/models--TheBloke--Llama-2-7B-Chat-GGUF/snapshots/191239b3e26b2882fb562ffccdd1cf0f65402adb/llama-2-7b-chat.Q4_K_M.gguf");
  assert!(
    model_path.exists(),
    "Model path does not exist: {}",
    model_path.display()
  );
  model_path
    .canonicalize()
    .unwrap()
    .to_str()
    .unwrap()
    .to_string()
}

#[fixture]
pub fn llama2_7b_str(llama2_7b: PathBuf) -> String {
  llama2_7b.to_string_lossy().into_owned()
}
