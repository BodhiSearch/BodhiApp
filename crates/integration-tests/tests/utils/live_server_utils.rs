use fs_extra::dir::{copy, CopyOptions};
use objs::{test_utils::setup_l10n, EnvType, FluentLocalizationService, SettingSource};
use rand::Rng;
use rstest::fixture;
use server_app::{ServeCommand, ServerShutdownHandle};
use services::{
  db::{DefaultTimeService, SqliteDbService},
  test_utils::{
    bodhi_home_setting, test_auth_service, EnvWrapperStub, OfflineHubService, SecretServiceStub,
  },
  AppService, DefaultAppService, DefaultSettingService, HfHubService, InitService,
  LocalDataService, MokaCacheService, SqliteSessionService, BODHI_ENV_TYPE, BODHI_EXEC_LOOKUP_PATH,
  BODHI_HOME, BODHI_LOGS, HF_HOME,
};
use sqlx::SqlitePool;
use std::{collections::HashMap, path::Path, sync::Arc};
use tempfile::TempDir;

static COPY_OPTIONS: CopyOptions = CopyOptions {
  overwrite: true,
  skip_exist: false,
  copy_inside: true,
  content_only: false,
  buffer_size: 64000,
  depth: 0,
};

pub fn copy_test_dir(src: &str, dst_path: &Path) {
  let src_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(src);
  copy(src_path, dst_path, &COPY_OPTIONS).unwrap();
}

#[fixture]
pub fn llama2_7b_setup(
  #[from(setup_l10n)] localization_service: &Arc<FluentLocalizationService>,
) -> (TempDir, Arc<dyn AppService>) {
  let temp_dir = tempfile::tempdir().unwrap();
  let cache_dir = temp_dir.path().join(".cache");
  std::fs::create_dir_all(&cache_dir).unwrap();

  let bodhi_home = cache_dir.join("bodhi");
  let hf_home = dirs::home_dir().unwrap().join(".cache").join("huggingface");
  copy_test_dir("tests/data/live/bodhi", &bodhi_home);

  let bodhi_logs = bodhi_home.join("logs");
  let hf_cache = hf_home.join("hub");
  let execs_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
    .join("..")
    .join("llama_server_proc")
    .join("bin")
    .canonicalize()
    .unwrap();
  let envs = HashMap::from([
    (
      String::from("HOME"),
      temp_dir.path().to_str().unwrap().to_string(),
    ),
    (
      BODHI_HOME.to_string(),
      bodhi_home.to_str().unwrap().to_string(),
    ),
    (
      BODHI_LOGS.to_string(),
      bodhi_logs.to_str().unwrap().to_string(),
    ),
    (HF_HOME.to_string(), hf_home.to_str().unwrap().to_string()),
    (
      BODHI_EXEC_LOOKUP_PATH.to_string(),
      execs_dir.display().to_string(),
    ),
    (BODHI_ENV_TYPE.to_string(), EnvType::Development.to_string()),
  ]);
  let env_wrapper = Arc::new(EnvWrapperStub::new(envs));
  InitService::new(env_wrapper.clone(), EnvType::Development)
    .setup_bodhi_home_dir()
    .unwrap();
  let setting_service = DefaultSettingService::new_with_defaults(
    env_wrapper,
    bodhi_home_setting(&bodhi_home, SettingSource::Environment),
    vec![],
    bodhi_home.join("settings.yaml"),
  )
  .expect("failed to setup setting service");
  let hub_service = Arc::new(OfflineHubService::new(HfHubService::new(
    hf_cache, false, None,
  )));
  let data_service = LocalDataService::new(bodhi_home.clone(), hub_service.clone());
  let auth_service = test_auth_service("http://id.localhost:8080");
  let pool = SqlitePool::connect_lazy("sqlite::memory:").unwrap();
  let time_service = Arc::new(DefaultTimeService);
  let db_service = SqliteDbService::new(pool.clone(), time_service.clone());
  let secret_service = SecretServiceStub::new()
    .with_authz_disabled()
    .with_app_status_ready();
  let session_service = SqliteSessionService::new(pool);
  let cache_service = MokaCacheService::default();
  let service = DefaultAppService::new(
    Arc::new(setting_service),
    hub_service,
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
#[awt]
pub async fn live_server(
  llama2_7b_setup: (TempDir, Arc<dyn AppService>),
) -> anyhow::Result<TestServerHandle> {
  let host = String::from("127.0.0.1");
  let port = rand::thread_rng().gen_range(2000..60000);
  let (temp_cache_dir, app_service) = llama2_7b_setup;
  let serve_command = ServeCommand::ByParams {
    host: host.clone(),
    port,
  };
  let handle = serve_command
    .get_server_handle(app_service.clone(), None)
    .await
    .unwrap();
  Ok(TestServerHandle {
    temp_cache_dir,
    host,
    port,
    handle,
  })
}

pub struct TestServerHandle {
  pub temp_cache_dir: TempDir,
  pub host: String,
  pub port: u16,
  pub handle: ServerShutdownHandle,
}
