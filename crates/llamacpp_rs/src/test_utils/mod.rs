use rstest::fixture;
use std::path::PathBuf;

#[fixture]
pub fn llama2_7b() -> PathBuf {
  let mut model_path = dirs::home_dir().expect("Home directory not found");
  model_path.push(".cache/huggingface/hub/models--TheBloke--Llama-2-7b-Chat-GGUF/snapshots/191239b3e26b2882fb562ffccdd1cf0f65402adb/llama-2-7b-chat.Q4_K_M.gguf");
  assert!(
    model_path.exists(),
    "model file does not exist at: {}",
    model_path.display()
  );
  let model_path = model_path
    .canonicalize()
    .expect("error canonicalizing path for test LLM model file");
  model_path
}

#[fixture]
pub fn llama2_7b_str(llama2_7b: PathBuf) -> String {
  llama2_7b.to_string_lossy().into_owned()
}
