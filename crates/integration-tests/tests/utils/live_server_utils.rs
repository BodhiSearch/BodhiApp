use fs_extra::dir::{copy, CopyOptions};
use mockall::predicate::eq;
use objs::{test_utils::setup_l10n, AppType, EnvType, FluentLocalizationService};
use rstest::fixture;
use server_app::{ServeCommand, ServerShutdownHandle};
use services::{
  db::{DefaultTimeService, SqliteDbService},
  test_utils::EnvWrapperStub,
  AppService, DefaultAppService, DefaultEnvService, DefaultSettingService, HfHubService,
  InitService, KeycloakAuthService, LocalDataService, MockSecretService, MokaCacheService,
  SqliteSessionService, KEY_APP_AUTHZ, KEY_APP_STATUS,
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
  let libs_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
    .join("..")
    .join("..")
    .join("llamacpp-sys")
    .join("libs")
    .canonicalize()
    .unwrap();
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
      String::from("BODHI_LOGS"),
      bodhi_logs.to_str().unwrap().to_string(),
    ),
    (
      String::from("HF_HOME"),
      hf_home.to_str().unwrap().to_string(),
    ),
    (
      String::from("BODHI_LIBRARY_LOOKUP_PATH"),
      libs_dir.display().to_string(),
    ),
  ]);
  let env_wrapper = EnvWrapperStub::new(envs);
  InitService::new(&env_wrapper, &EnvType::Development)
    .setup_bodhi_home()
    .unwrap();
  let setting_service =
    DefaultSettingService::new(Arc::new(env_wrapper), bodhi_home.join("settings.yaml"));
  let env_service = DefaultEnvService::new(
    bodhi_home.clone(),
    EnvType::Development,
    AppType::Container,
    "".to_string(),
    "".to_string(),
    Arc::new(setting_service),
  )
  .unwrap();
  // TODO: fix this
  // env_service.set_library_path(library_path().display().to_string());
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
#[awt]
pub async fn live_server(
  llama2_7b_setup: (TempDir, Arc<dyn AppService>),
) -> anyhow::Result<TestServerHandle> {
  let host = String::from("127.0.0.1");
  let port = rand::random::<u16>();
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
