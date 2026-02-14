use llama_server_proc::{
  LlamaServer, LlamaServerArgsBuilder, Server, BUILD_TARGET, DEFAULT_VARIANT, EXEC_NAME,
};
use rstest::{fixture, rstest};
use std::path::PathBuf;

#[fixture]
fn lookup_path() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("bin")
}

#[fixture]
fn hf_cache() -> PathBuf {
  if let Ok(hf_home) = std::env::var("HF_HOME") {
    PathBuf::from(hf_home).join("hub")
  } else {
    dirs::home_dir()
      .unwrap()
      .join(".cache")
      .join("huggingface")
      .join("hub")
  }
}

#[rstest]
#[tokio::test]
async fn test_live_llama_server_load_exec_with_server(
  hf_cache: PathBuf,
  lookup_path: PathBuf,
) -> anyhow::Result<()> {
  let llama_68m = hf_cache.join("models--afrideva--Llama-68M-Chat-v1-GGUF/snapshots/4bcbc666d2f0d2b04d06f046d6baccdab79eac61/llama-68m-chat-v1.q8_0.gguf");
  let exec_path = &lookup_path
    .join(BUILD_TARGET)
    .join(DEFAULT_VARIANT)
    .join(EXEC_NAME);
  let server = LlamaServer::new(
    exec_path,
    LlamaServerArgsBuilder::default()
      .alias("testalias")
      .model(llama_68m)
      .build()
      .unwrap(),
  )?;
  let result = server.start().await;
  server.stop_unboxed().await?;
  assert!(
    result.is_ok(),
    "server start failed with error: {:?}",
    result
  );
  Ok(())
}
