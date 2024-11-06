use libloading::Library;
use llamacpp_rs::CommonParams;
use llamacpp_sys::{BodhiServer, DynamicBodhiServer};
use rstest::{fixture, rstest};
use server_core::{DefaultServerContextFactory, DefaultSharedContextRw, SharedContextRw};
use std::path::PathBuf;

#[fixture]
fn lib_path() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("..")
    .join("..")
    .join("llamacpp-sys")
    .join("libs")
    .join(llamacpp_sys::BUILD_TARGET)
    .join(llamacpp_sys::DEFAULT_VARIANT)
    .join(llamacpp_sys::LIBRARY_NAME)
}

#[fixture]
fn tests_data() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("tests")
    .join("data")
}

#[rstest]
fn test_live_lib_llama_server_load_library_for_current_platform(lib_path: PathBuf) {
  let lib = unsafe { Library::new(&lib_path) };
  assert!(lib.is_ok(), "library loading failed with error: {:?}", lib);
}

#[rstest]
fn test_live_lib_llama_server_load_library_with_dynamic_bodhi_server(lib_path: PathBuf) {
  let server = DynamicBodhiServer::default();
  let result = server.load_library(&lib_path);
  assert!(
    result.is_ok(),
    "library loading failed with error: {:?}",
    result
  );
}

#[rstest]
#[tokio::test]
async fn test_live_lib_shared_rw_reload(lib_path: PathBuf) {
  let shared_rw =
    DefaultSharedContextRw::new(true, Box::new(DefaultServerContextFactory), Some(lib_path));
  let result = shared_rw.reload(None).await;
  assert!(
    result.is_ok(),
    "shared rw reload failed with error: {:?}",
    result
  );
}

#[rstest]
#[tokio::test]
async fn test_live_lib_shared_rw_reload_with_gpt_params(lib_path: PathBuf, tests_data: PathBuf) {
  let shared_rw =
    DefaultSharedContextRw::new(true, Box::new(DefaultServerContextFactory), Some(lib_path));
  let gpt_params = CommonParams {
    model: tests_data.join("live/huggingface/hub/models--afrideva--Llama-68M-Chat-v1-GGUF/snapshots/4bcbc666d2f0d2b04d06f046d6baccdab79eac61/llama-68m-chat-v1.q8_0.gguf").to_string_lossy().to_string(),
    ..Default::default()
  };
  let result = shared_rw.reload(Some(gpt_params)).await;
  assert!(
    result.is_ok(),
    "shared rw reload failed with error: {:?}",
    result
  );
}
