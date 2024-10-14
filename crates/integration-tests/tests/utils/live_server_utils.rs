use dircpy::CopyBuilder;
use llama_server_bindings::{bindings::llama_server_disable_logging, disable_llama_log};
use mockall::predicate::eq;
use objs::test_utils::setup_l10n;
use objs::{EnvType, FluentLocalizationService};
use rstest::fixture;
use server_app::{ServeCommand, ServerShutdownHandle};
use services::{
  db::{DefaultTimeService, SqliteDbService},
  test_utils::EnvWrapperStub,
  AppService, DefaultAppService, DefaultEnvService, HfHubService, KeycloakAuthService,
  LocalDataService, MockSecretService, MokaCacheService, SqliteSessionService, KEY_APP_AUTHZ,
  KEY_APP_STATUS,
};
use sqlx::SqlitePool;
use std::{collections::HashMap, path::Path, sync::Arc};
use tempfile::TempDir;

pub fn copy_test_dir(src: &str, dst_path: &Path) {
  let src_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(src);
  CopyBuilder::new(src_path, dst_path)
    .overwrite(true)
    .with_include_filter("")
    .run()
    .unwrap();
}

#[allow(unused)]
pub fn disable_test_logging() {
  disable_llama_log();
  unsafe {
    llama_server_disable_logging();
  }
}

#[fixture]
#[once]
pub fn tinyllama(
  #[from(setup_l10n)] localization_service: &Arc<FluentLocalizationService>,
) -> (TempDir, Arc<dyn AppService>) {
  let temp_dir = tempfile::tempdir().unwrap();
  let cache_dir = temp_dir.path().join(".cache");
  std::fs::create_dir_all(&cache_dir).unwrap();

  copy_test_dir("tests/data/live", &cache_dir);

  let bodhi_home = cache_dir.join("bodhi");
  let hf_home = cache_dir.join("huggingface");
  let hf_cache = hf_home.join("hub");
  let envs = HashMap::from([
    (
      String::from("HOME"),
      temp_dir.path().to_str().unwrap().to_string(),
    ),
    (
      String::from("BODHI_HOME"),
      bodhi_home.to_str().unwrap().to_string(),
    ),
    (
      String::from("HF_HOME"),
      hf_home.to_str().unwrap().to_string(),
    ),
  ]);
  let env_wrapper = EnvWrapperStub::new(envs);
  let env_service = DefaultEnvService::new(
    EnvType::Development,
    "".to_string(),
    "".to_string(),
    Arc::new(env_wrapper),
  );
  env_service.create_home_dirs(&bodhi_home).unwrap();
  let data_service = LocalDataService::new(bodhi_home.clone());
  let hub_service = HfHubService::new(hf_cache, false, None);
  let auth_service = KeycloakAuthService::new(
    String::from("http://id.localhost:8080"),
    String::from("bodhi"),
  );
  let pool = SqlitePool::connect_lazy("sqlite::memory:").unwrap();
  let time_service = Arc::new(DefaultTimeService);
  let db_service = SqliteDbService::new(pool.clone(), time_service.clone());
  let mut secret_service = MockSecretService::default();
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
  let service = DefaultAppService::new(
    Arc::new(env_service),
    Arc::new(hub_service),
    Arc::new(data_service),
    Arc::new(auth_service),
    Arc::new(db_service),
    Arc::new(session_service),
    Arc::new(secret_service),
    Arc::new(cache_service),
    localization_service.clone(),
    time_service,
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
  tinyllama: &(TempDir, Arc<dyn AppService>),
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
