use bodhicore::{
  bindings::{disable_llama_log, llama_server_disable_logging},
  service::{
    env_wrapper::EnvWrapper, AppService, AppServiceFn, EnvService, HfHubService, LocalDataService,
  },
  ServeCommand, ServerShutdownHandle,
};
use dircpy::CopyBuilder;
use rstest::fixture;
use std::{path::Path, sync::Arc};
use tempfile::TempDir;

pub fn copy_test_dir(src: &str, dst_path: &Path) {
  let src_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(src);
  CopyBuilder::new(src_path, dst_path)
    .overwrite(true)
    .with_include_filter("")
    .run()
    .unwrap();
}

#[fixture]
#[once]
pub fn tinyllama() -> (TempDir, Arc<dyn AppServiceFn>) {
  let temp_dir = tempfile::tempdir().unwrap();
  let cache_dir = temp_dir.path().join(".cache");
  std::fs::create_dir_all(&cache_dir).unwrap();

  copy_test_dir("tests/data/live", &cache_dir);

  let bodhi_home = cache_dir.join("bodhi");
  let hf_home = cache_dir.join("huggingface");
  let hf_cache = hf_home.join("hub");
  let env_service = EnvService::new_with_args(EnvWrapper::default(), bodhi_home.clone(), hf_home);
  env_service.create_home_dirs(&bodhi_home).unwrap();
  let data_service = LocalDataService::new(bodhi_home.clone());
  let hub_service = HfHubService::new(hf_cache, false, None);
  let app_service = AppService::new(Arc::new(env_service), hub_service, data_service);
  (temp_dir, Arc::new(app_service))
}

#[fixture]
pub fn setup_logs() {
  disable_llama_log();
  unsafe {
    llama_server_disable_logging();
  }
}

#[fixture]
pub fn setup(#[from(setup_logs)] _setup_logs: ()) {}

#[fixture]
#[awt]
pub async fn live_server(
  #[from(setup)] _setup: (),
  tinyllama: &(TempDir, Arc<dyn AppServiceFn>),
) -> anyhow::Result<TestServerHandle> {
  let host = String::from("127.0.0.1");
  let port = rand::random::<u16>();
  let (_temp_cache_dir, app_service) = tinyllama;
  let serve_command = ServeCommand::ByParams {
    host: host.clone(),
    port,
  };
  let handle = serve_command.aexecute(app_service.clone(), None).await?;
  Ok(TestServerHandle { host, port, handle })
}

pub struct TestServerHandle {
  pub host: String,
  pub port: u16,
  pub handle: ServerShutdownHandle,
}
