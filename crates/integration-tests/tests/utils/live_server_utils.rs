use dircpy::CopyBuilder;
use mockall::predicate::eq;
use rstest::fixture;
use server::{disable_llama_log, llama_server_disable_logging, ServeCommand, ServerShutdownHandle};
use services::{
  db::{SqliteDbService, TimeService},
  env_wrapper::DefaultEnvWrapper,
  AppService, AppServiceFn, EnvService, HfHubService, KeycloakAuthService, LocalDataService,
  MockISecretService, MokaCacheService, SqliteSessionService, KEY_APP_AUTHZ, KEY_APP_STATUS,
};
use sqlx::SqlitePool;
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
  let env_service = EnvService::new_with_args(
    Arc::new(DefaultEnvWrapper::default()),
    bodhi_home.clone(),
    hf_home,
  );
  env_service.create_home_dirs(&bodhi_home).unwrap();
  let data_service = LocalDataService::new(bodhi_home.clone());
  let hub_service = HfHubService::new(hf_cache, false, None);
  let auth_service = KeycloakAuthService::new(
    String::from("http://id.localhost:8080"),
    String::from("bodhi"),
  );
  let pool = SqlitePool::connect_lazy("sqlite::memory:").unwrap();
  let db_service = SqliteDbService::new(pool.clone(), Arc::new(TimeService));
  let mut secret_service = MockISecretService::default();
  secret_service
    .expect_get_secret_string()
    .with(eq(KEY_APP_AUTHZ))
    .returning(|_| Ok(Some("false".to_string())));
  secret_service
    .expect_get_secret_string()
    .with(eq(KEY_APP_STATUS))
    .returning(|_| Ok(Some("ready".to_string())));
  let session_service = SqliteSessionService::new(pool);
  let cache_service = MokaCacheService::default();
  let service = AppService::new(
    Arc::new(env_service),
    Arc::new(hub_service),
    Arc::new(data_service),
    Arc::new(auth_service),
    Arc::new(db_service),
    Arc::new(session_service),
    Arc::new(secret_service),
    Arc::new(cache_service),
  );
  (temp_dir, Arc::new(service))
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
  let handle = serve_command
    .get_server_handle(app_service.clone(), None)
    .await
    .unwrap();
  Ok(TestServerHandle { host, port, handle })
}

pub struct TestServerHandle {
  pub host: String,
  pub port: u16,
  pub handle: ServerShutdownHandle,
}
