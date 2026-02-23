use llama_server_proc::LlamaServerArgsBuilder;
use rstest::{fixture, rstest};
use server_core::{DefaultServerFactory, DefaultSharedContext, SharedContext};
use services::{test_utils::OfflineHubService, HfHubService, MockSettingService};
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
async fn test_live_shared_rw_reload(
  lookup_path: PathBuf,
  tests_data: PathBuf,
) -> anyhow::Result<()> {
  let hub_service = OfflineHubService::new(HfHubService::new(
    tests_data.join("live/huggingface/hub"),
    false,
    None,
  ));

  let exec_path = lookup_path
    .join(llama_server_proc::BUILD_TARGET)
    .join(llama_server_proc::DEFAULT_VARIANT)
    .join(llama_server_proc::EXEC_NAME);
  let mut mock_setting_service = MockSettingService::new();
  mock_setting_service
    .expect_exec_path_from()
    .return_const(exec_path);
  mock_setting_service
    .expect_exec_variant()
    .return_const(llama_server_proc::DEFAULT_VARIANT.to_string());

  let shared_rw = DefaultSharedContext::with_args(
    Arc::new(hub_service),
    Arc::new(mock_setting_service),
    Box::new(DefaultServerFactory),
  )
  .await;
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

  let exec_path = lookup_path
    .join(llama_server_proc::BUILD_TARGET)
    .join(llama_server_proc::DEFAULT_VARIANT)
    .join(llama_server_proc::EXEC_NAME);
  let mut mock_setting_service = MockSettingService::new();
  mock_setting_service
    .expect_exec_path_from()
    .return_const(exec_path);
  mock_setting_service
    .expect_exec_variant()
    .return_const(llama_server_proc::DEFAULT_VARIANT.to_string());

  let shared_rw = DefaultSharedContext::with_args(
    Arc::new(hub_service),
    Arc::new(mock_setting_service),
    Box::new(DefaultServerFactory),
  )
  .await;
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

  let exec_path = lookup_path
    .join(llama_server_proc::BUILD_TARGET)
    .join(llama_server_proc::DEFAULT_VARIANT)
    .join(llama_server_proc::EXEC_NAME);
  let mut mock_setting_service = MockSettingService::new();
  mock_setting_service
    .expect_exec_path_from()
    .return_const(exec_path);
  mock_setting_service
    .expect_exec_variant()
    .return_const(llama_server_proc::DEFAULT_VARIANT.to_string());

  let shared_rw = DefaultSharedContext::with_args(
    Arc::new(hub_service),
    Arc::new(mock_setting_service),
    Box::new(DefaultServerFactory),
  )
  .await;
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
