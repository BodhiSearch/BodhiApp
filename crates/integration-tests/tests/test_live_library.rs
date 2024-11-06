use libloading::Library;
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

#[rstest]
fn test_live_llama_server_load_library_for_current_platform(lib_path: PathBuf) {
  let lib = unsafe { Library::new(&lib_path) };
  assert!(lib.is_ok(), "library loading failed with error: {:?}", lib);
}

#[rstest]
fn test_live_llama_server_load_library_with_dynamic_bodhi_server(lib_path: PathBuf) {
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
async fn test_live_shared_rw_reload(lib_path: PathBuf) {
  let shared_rw =
    DefaultSharedContextRw::new(true, Box::new(DefaultServerContextFactory), Some(lib_path));
  let result = shared_rw.reload(None).await;
  assert!(
    result.is_ok(),
    "shared rw reload failed with error: {:?}",
    result
  );
}
