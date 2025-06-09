#![allow(dead_code)]

use fs_extra::dir::{copy, CopyOptions};
use objs::{test_utils::setup_l10n, EnvType, FluentLocalizationService, SettingSource};
use rand::Rng;
use routes_app::{SESSION_KEY_ACCESS_TOKEN, SESSION_KEY_REFRESH_TOKEN};
use rstest::fixture;
use serde_json::Value;
use server_app::{ServeCommand, ServerShutdownHandle};
use services::{
  db::{DbService, DefaultTimeService, SqliteDbService},
  hash_key,
  test_utils::{bodhi_home_setting, test_auth_service, EnvWrapperStub, OfflineHubService},
  AppRegInfoBuilder, AppService, AppStatus, DefaultAppService, DefaultSecretService,
  DefaultSettingService, HfHubService, InitService, LocalDataService, MokaCacheService,
  SecretServiceExt, SettingService, SqliteSessionService, BODHI_AUTH_REALM, BODHI_AUTH_URL,
  BODHI_ENCRYPTION_KEY, BODHI_ENV_TYPE, BODHI_EXEC_LOOKUP_PATH, BODHI_HOME, BODHI_LOGS, HF_HOME,
};
use sqlx::SqlitePool;
use std::{collections::HashMap, path::Path, sync::Arc};
use tempfile::TempDir;
use tower_sessions::session::{Id, Record};
use tower_sessions::SessionStore;

use cookie::Cookie;
use time::{Duration, OffsetDateTime};

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
pub async fn llama2_7b_setup(
  #[from(setup_l10n)] localization_service: &Arc<FluentLocalizationService>,
) -> anyhow::Result<(TempDir, Arc<dyn AppService>)> {
  // Load environment variables from .env.test
  let env_test_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
    .join("tests")
    .join("resources")
    .join(".env.test");
  assert!(env_test_path.exists(), "Failed to find .env.test file");
  dotenv::from_filename(&env_test_path).ok();
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
    (
      BODHI_AUTH_URL.to_string(),
      std::env::var("INTEG_TEST_AUTH_URL").unwrap(),
    ),
    (
      BODHI_AUTH_REALM.to_string(),
      std::env::var("INTEG_TEST_AUTH_REALM").unwrap(),
    ),
    (
      BODHI_ENCRYPTION_KEY.to_string(),
      "test-encryption-key".to_string(),
    ),
  ]);

  // Get OpenID configuration values from environment
  let client_id = std::env::var("INTEG_TEST_CLIENT_ID").expect("INTEG_TEST_CLIENT_ID not set");
  let client_secret =
    std::env::var("INTEG_TEST_CLIENT_SECRET").expect("INTEG_TEST_CLIENT_SECRET not set");
  let issuer = std::env::var("INTEG_TEST_ISSUER").expect("INTEG_TEST_ISSUER not set");
  let kid = std::env::var("INTEG_TEST_KID").expect("INTEG_TEST_KID not set");
  let public_key = std::env::var("INTEG_TEST_PUBLIC_KEY").expect("INTEG_TEST_PUBLIC_KEY not set");

  let env_wrapper = Arc::new(EnvWrapperStub::new(envs));
  let init_service = InitService::new(env_wrapper.clone(), EnvType::Development);
  init_service.setup_bodhi_home_dir()?;
  let setting_service = DefaultSettingService::new_with_defaults(
    env_wrapper,
    bodhi_home_setting(&bodhi_home, SettingSource::Environment),
    vec![],
    bodhi_home.join("settings.yaml"),
  )?;
  init_service.set_bodhi_home(&setting_service)?;
  let hub_service = Arc::new(OfflineHubService::new(HfHubService::new(
    hf_cache, false, None,
  )));
  let data_service = LocalDataService::new(bodhi_home.clone(), hub_service.clone());
  let auth_service = test_auth_service("http://id.localhost:8080");

  let app_db_path = setting_service.app_db_path();
  let session_db_path = setting_service.session_db_path();

  let app_pool = SqlitePool::connect_lazy(&format!("sqlite:{}", app_db_path.display())).unwrap();
  let session_pool =
    SqlitePool::connect_lazy(&format!("sqlite:{}", session_db_path.display())).unwrap();

  let time_service = Arc::new(DefaultTimeService);
  let db_service = SqliteDbService::new(app_pool, time_service.clone());
  // Migrate databases to ensure they exist
  db_service.migrate().await.unwrap();
  let session_service = SqliteSessionService::new(session_pool);
  session_service.migrate().await.unwrap();
  // Create AppRegInfo with values from environment
  let app_reg_info = AppRegInfoBuilder::test_default()
    .client_id(client_id)
    .client_secret(client_secret)
    .issuer(issuer)
    .kid(kid)
    .public_key(public_key)
    .alg(jsonwebtoken::Algorithm::RS256)
    .build()
    .unwrap();

  let encryption_key = setting_service.encryption_key().unwrap();
  let encryption_key = hash_key(&encryption_key);
  let secret_service = DefaultSecretService::new(&encryption_key, &setting_service.secrets_path())?;
  secret_service.set_app_reg_info(&app_reg_info)?;
  secret_service.set_app_status(&AppStatus::Ready)?;

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
  Ok((temp_dir, Arc::new(service)))
}

#[fixture]
#[awt]
pub async fn live_server(
  #[future] llama2_7b_setup: anyhow::Result<(TempDir, Arc<dyn AppService>)>,
) -> anyhow::Result<TestServerHandle> {
  let host = String::from("127.0.0.1");
  let port = rand::thread_rng().gen_range(2000..60000);
  let (temp_cache_dir, app_service) = llama2_7b_setup?;
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
    app_service,
  })
}

pub struct TestServerHandle {
  pub temp_cache_dir: TempDir,
  pub host: String,
  pub port: u16,
  pub handle: ServerShutdownHandle,
  pub app_service: Arc<dyn AppService>,
}

pub async fn get_oauth_tokens(app_service: &dyn AppService) -> anyhow::Result<(String, String)> {
  let setting_service = app_service.setting_service();
  let auth_url = setting_service.auth_url();
  let realm = setting_service.auth_realm();
  let app_reg_info = app_service
    .secret_service()
    .app_reg_info()?
    .expect("AppRegInfo is not set");
  let client_id = app_reg_info.client_id;
  let client_secret = app_reg_info.client_secret;
  let username = std::env::var("INTEG_TEST_USERNAME")
    .map_err(|_| anyhow::anyhow!("INTEG_TEST_USERNAME not set"))?;
  let password = std::env::var("INTEG_TEST_PASSWORD")
    .map_err(|_| anyhow::anyhow!("INTEG_TEST_PASSWORD not set"))?;

  let token_url = format!(
    "{}/realms/{}/protocol/openid-connect/token",
    auth_url.trim_end_matches('/'),
    realm
  );

  let params = [
    ("grant_type", "password"),
    ("client_id", &client_id),
    ("client_secret", &client_secret),
    ("username", &username),
    ("password", &password),
    ("scope", &["openid", "email", "profile", "roles"].join(" ")),
  ];

  let client = reqwest::Client::new();
  let response = client.post(&token_url).form(&params).send().await?;
  assert_eq!(200, response.status());
  let token_data: Value = response.json().await?;
  let access_token = token_data["access_token"]
    .as_str()
    .ok_or_else(|| anyhow::anyhow!("Missing access_token in response"))?;
  let refresh_token = token_data["refresh_token"]
    .as_str()
    .ok_or_else(|| anyhow::anyhow!("Missing refresh_token in response"))?;

  Ok((access_token.to_string(), refresh_token.to_string()))
}

/// Create a session with OAuth tokens and return session ID
pub async fn create_authenticated_session(
  app_service: &Arc<dyn AppService>,
  access_token: &str,
  refresh_token: &str,
) -> anyhow::Result<String> {
  let session_service = app_service.session_service();

  let session_id = Id::default();
  let session_data = maplit::hashmap! {
    SESSION_KEY_ACCESS_TOKEN.to_string() => Value::String(access_token.to_string()),
    SESSION_KEY_REFRESH_TOKEN.to_string() => Value::String(refresh_token.to_string()),
  };

  let mut record = Record {
    id: session_id,
    data: session_data,
    expiry_date: OffsetDateTime::now_utc() + Duration::days(1),
  };

  session_service
    .get_session_store()
    .create(&mut record)
    .await?;
  Ok(session_id.to_string())
}

/// Create a session cookie for the given session ID
pub fn create_session_cookie(session_id: &str) -> Cookie {
  Cookie::build(("bodhiapp_session_id", session_id))
    .path("/")
    .http_only(true)
    .same_site(cookie::SameSite::Lax)
    .build()
}
