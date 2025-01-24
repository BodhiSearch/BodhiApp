use llama_server_proc::{exec_path_from, LlamaServer, LlamaServerArgsBuilder, Server};
use rstest::{fixture, rstest};
use server_core::{DefaultServerFactory, DefaultSharedContext, SharedContext};
use services::{test_utils::OfflineHubService, HfHubService};
use std::{path::PathBuf, sync::Arc};

#[fixture]
fn lookup_path() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("..")
    .join("llama_server_proc")
    .join("bin")
}

#[fixture]
fn tests_data() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("tests")
    .join("data")
}

#[rstest]
#[tokio::test]
async fn test_live_llama_server_load_exec_with_server(
  tests_data: PathBuf,
  lookup_path: PathBuf,
) -> anyhow::Result<()> {
  let llama_68m = tests_data.join("live/huggingface/hub/models--afrideva--Llama-68M-Chat-v1-GGUF/snapshots/4bcbc666d2f0d2b04d06f046d6baccdab79eac61/llama-68m-chat-v1.q8_0.gguf");
  let exec_path = exec_path_from(&lookup_path, llama_server_proc::DEFAULT_VARIANT);
  let server = LlamaServer::new(
    &exec_path,
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

#[rstest]
#[tokio::test]
async fn test_live_shared_rw_reload(
  lookup_path: PathBuf,
  tests_data: PathBuf,
) -> anyhow::Result<()> {
  let hub_service = OfflineHubService::new(HfHubService::new(
    tests_data.join("live/huggingface/hub"),
    false,
    None,
  ));
  let shared_rw = DefaultSharedContext::with_args(
    Arc::new(hub_service),
    Box::new(DefaultServerFactory),
    &lookup_path,
    llama_server_proc::DEFAULT_VARIANT,
  );
  let result = shared_rw.reload(None).await;
  shared_rw.stop().await?;
  assert!(
    result.is_ok(),
    "shared rw reload failed with error: {:?}",
    result
  );
  Ok(())
}

#[rstest]
#[tokio::test]
async fn test_live_shared_rw_reload_with_model_as_symlink(
  tests_data: PathBuf,
  lookup_path: PathBuf,
) -> anyhow::Result<()> {
  let llama_68m = tests_data.join("live/huggingface/hub/models--afrideva--Llama-68M-Chat-v1-GGUF/snapshots/4bcbc666d2f0d2b04d06f046d6baccdab79eac61/llama-68m-chat-v1.q8_0.gguf");
  let hub_service = OfflineHubService::new(HfHubService::new(
    tests_data.join("live/huggingface/hub"),
    false,
    None,
  ));
  let shared_rw = DefaultSharedContext::with_args(
    Arc::new(hub_service),
    Box::new(DefaultServerFactory),
    &lookup_path,
    llama_server_proc::DEFAULT_VARIANT,
  );
  let server_args = LlamaServerArgsBuilder::default()
    .alias("testalias")
    .model(llama_68m)
    .build()?;
  let result = shared_rw.reload(Some(server_args)).await;
  shared_rw.stop().await?;
  assert!(
    result.is_ok(),
    "shared rw reload failed with error: {:?}",
    result
  );
  Ok(())
}

#[rstest]
#[tokio::test]
async fn test_live_shared_rw_reload_with_actual_file(
  lookup_path: PathBuf,
  tests_data: PathBuf,
) -> anyhow::Result<()> {
  let hub_service = OfflineHubService::new(HfHubService::new(
    tests_data.join("live/huggingface/hub"),
    false,
    None,
  ));
  let shared_rw = DefaultSharedContext::with_args(
    Arc::new(hub_service),
    Box::new(DefaultServerFactory),
    &lookup_path,
    llama_server_proc::DEFAULT_VARIANT,
  );
  let server_params = LlamaServerArgsBuilder::default()
    .alias("testalias")
    .model(tests_data.join("live/huggingface/hub/models--afrideva--Llama-68M-Chat-v1-GGUF/blobs/cdd6bad08258f53c637c233309c3b41ccd91907359364aaa02e18df54c34b836"))
    .build()?;
  let result = shared_rw.reload(Some(server_params)).await;
  shared_rw.stop().await?;
  assert!(
    result.is_ok(),
    "shared rw reload failed with error: {:?}",
    result
  );
  Ok(())
}
