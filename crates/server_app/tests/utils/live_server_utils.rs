#![allow(dead_code)]

use auth_middleware::{
  test_utils::{AuthServerConfigBuilder, AuthServerTestClient},
  SESSION_KEY_ACCESS_TOKEN, SESSION_KEY_REFRESH_TOKEN,
};
use cookie::Cookie;
use objs::{EnvType, Setting, SettingMetadata, SettingSource};
use rand::Rng;
use rstest::fixture;
use serde_json::Value;
use serde_yaml::Value as YamlValue;
use server_app::{ServeCommand, ServerShutdownHandle};
use services::{
  db::{DbCore, DefaultTimeService, SqliteDbService},
  hash_key,
  test_utils::{test_auth_service, OfflineHubService, StubQueue},
  AppRegInfoBuilder, AppService, AppStatus, DefaultAccessRequestService, DefaultAiApiService,
  DefaultAppService, DefaultEnvWrapper, DefaultExaService, DefaultSecretService,
  DefaultSettingService, DefaultToolService, EnvWrapper, HfHubService, LocalConcurrencyService,
  LocalDataService, MokaCacheService, SecretServiceExt, SettingService, SqliteSessionService,
  StubNetworkService, BODHI_AUTH_REALM, BODHI_AUTH_URL, BODHI_ENCRYPTION_KEY, BODHI_ENV_TYPE,
  BODHI_EXEC_LOOKUP_PATH, BODHI_HOME, BODHI_LOGS, BODHI_VERSION, HF_HOME, SETTINGS_YAML,
};
use sqlx::SqlitePool;
use std::{collections::HashMap, fs, path::Path, sync::Arc};
use tempfile::TempDir;
use time::{Duration, OffsetDateTime};
use tower_sessions::session::{Id, Record};
use tower_sessions::SessionStore;

/// Inline minimal setup without lib_bodhiserver dependency
async fn setup_minimal_app_service(temp_dir: &TempDir) -> anyhow::Result<Arc<dyn AppService>> {
  // Load environment variables from .env.test
  let env_test_path = Path::new(env!("CARGO_MANIFEST_DIR"))
    .join("tests")
    .join("resources")
    .join(".env.test");
  if env_test_path.exists() {
    dotenv::from_filename(&env_test_path).ok();
  }

  let cache_dir = temp_dir.path().join(".cache");
  let bodhi_home = cache_dir.join("bodhi");
  let logs_dir = bodhi_home.join("logs");
  fs::create_dir_all(&logs_dir)?;

  // Use real HuggingFace cache at ~/.cache/huggingface
  let hf_home = dirs::home_dir()
    .ok_or_else(|| anyhow::anyhow!("Failed to determine home directory"))?
    .join(".cache")
    .join("huggingface");
  fs::create_dir_all(&hf_home.join("hub"))?;

  // Build env wrapper with test environment
  let mut env_vars = HashMap::new();
  env_vars.insert(BODHI_HOME.to_string(), bodhi_home.display().to_string());
  env_vars.insert(BODHI_LOGS.to_string(), logs_dir.display().to_string());
  env_vars.insert(HF_HOME.to_string(), hf_home.display().to_string());

  // Point to llama_server_proc bin directory
  let execs_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
    .parent()
    .unwrap()
    .join("llama_server_proc")
    .join("bin")
    .canonicalize()?;
  env_vars.insert(
    BODHI_EXEC_LOOKUP_PATH.to_string(),
    execs_dir.display().to_string(),
  );
  env_vars.insert(
    BODHI_ENCRYPTION_KEY.to_string(),
    "test-encryption-key".to_string(),
  );

  // Get OAuth config from environment
  let auth_server_url = std::env::var("INTEG_TEST_AUTH_URL")
    .map_err(|_| anyhow::anyhow!("INTEG_TEST_AUTH_URL not set - required for live tests"))?;
  let realm = std::env::var("INTEG_TEST_AUTH_REALM")
    .map_err(|_| anyhow::anyhow!("INTEG_TEST_AUTH_REALM not set - required for live tests"))?;
  let test_user_id = std::env::var("INTEG_TEST_USERNAME_ID")
    .map_err(|_| anyhow::anyhow!("INTEG_TEST_USERNAME_ID not set - required for live tests"))?;

  env_vars.insert(BODHI_AUTH_URL.to_string(), auth_server_url.clone());
  env_vars.insert(BODHI_AUTH_REALM.to_string(), realm.clone());

  let mut env_wrapper_impl = DefaultEnvWrapper::default();
  for (key, value) in &env_vars {
    env_wrapper_impl.set_var(key, value);
  }
  let env_wrapper: Arc<dyn EnvWrapper> = Arc::new(env_wrapper_impl);

  // Build system settings
  let app_settings = vec![
    Setting {
      key: BODHI_ENV_TYPE.to_string(),
      value: YamlValue::String(EnvType::Development.to_string()),
      source: SettingSource::System,
      metadata: SettingMetadata::String,
    },
    Setting {
      key: BODHI_VERSION.to_string(),
      value: YamlValue::String("0.0.46-dev".to_string()),
      source: SettingSource::System,
      metadata: SettingMetadata::String,
    },
    Setting {
      key: BODHI_AUTH_URL.to_string(),
      value: YamlValue::String(auth_server_url.clone()),
      source: SettingSource::System,
      metadata: SettingMetadata::String,
    },
    Setting {
      key: BODHI_AUTH_REALM.to_string(),
      value: YamlValue::String(realm.clone()),
      source: SettingSource::System,
      metadata: SettingMetadata::String,
    },
  ];

  // Build settings service directly
  let settings_file = bodhi_home.join(SETTINGS_YAML);
  let setting_service = DefaultSettingService::new_with_defaults(
    env_wrapper.clone(),
    Setting {
      key: BODHI_HOME.to_string(),
      value: YamlValue::String(bodhi_home.display().to_string()),
      source: SettingSource::Environment,
      metadata: SettingMetadata::String,
    },
    app_settings,
    HashMap::new(), // empty file_defaults
    settings_file,
  );
  setting_service.load_default_env();

  // Create SQLite databases
  let app_db_path = setting_service.app_db_path();
  fs::File::create_new(&app_db_path)?;
  let session_db_path = setting_service.session_db_path();
  fs::File::create_new(&session_db_path)?;

  // Setup OAuth resource client
  let config = AuthServerConfigBuilder::default()
    .auth_server_url(&auth_server_url)
    .realm(&realm)
    .dev_console_client_id(
      std::env::var("INTEG_TEST_DEV_CONSOLE_CLIENT_ID")
        .map_err(|_| anyhow::anyhow!("INTEG_TEST_DEV_CONSOLE_CLIENT_ID not set"))?,
    )
    .dev_console_client_secret(
      std::env::var("INTEG_TEST_DEV_CONSOLE_CLIENT_SECRET")
        .map_err(|_| anyhow::anyhow!("INTEG_TEST_DEV_CONSOLE_CLIENT_SECRET not set"))?,
    )
    .build()?;
  let auth_client = AuthServerTestClient::new(config);
  let resource_client = auth_client
    .create_resource_client("integration_test")
    .await?;
  let resource_token = auth_client
    .get_resource_service_token(&resource_client)
    .await?;
  auth_client
    .make_first_resource_admin(&resource_token, &test_user_id)
    .await?;

  // Create secret service with app registration
  let encryption_key = setting_service.encryption_key().unwrap();
  let encryption_key = hash_key(&encryption_key);
  let secret_service = DefaultSecretService::new(&encryption_key, &setting_service.secrets_path())?;
  let app_reg_info = AppRegInfoBuilder::default()
    .client_id(resource_client.client_id)
    .client_secret(resource_client.client_secret.unwrap())
    .build()?;
  secret_service.set_app_reg_info(&app_reg_info)?;
  secret_service.set_app_status(&AppStatus::Ready)?;

  // Build time service first (no dependencies)
  let time_service = Arc::new(DefaultTimeService);

  // Build DB service with pool
  let app_db_url = format!("sqlite:{}", app_db_path.display());
  let app_pool = SqlitePool::connect(&app_db_url).await?;
  let encryption_key = hash_key(&setting_service.encryption_key().unwrap());
  let db_service = Arc::new(SqliteDbService::new(
    app_pool,
    time_service.clone(),
    encryption_key,
  ));
  db_service.migrate().await?;

  // Build session service with pool and run migrations
  let session_db_url = format!("sqlite:{}", session_db_path.display());
  let session_pool = SqlitePool::connect(&session_db_url).await?;
  let session_service = SqliteSessionService::new(session_pool);
  session_service.migrate().await?;
  let session_service = Arc::new(session_service);

  // Store setting service in Arc for sharing
  let setting_service = Arc::new(setting_service);

  // Build hub service (offline wrapper around real HfHubService)
  let hf_cache = setting_service.hf_cache();
  let hub_service = Arc::new(OfflineHubService::new(HfHubService::new(
    hf_cache, false, None,
  )));

  // Build data service
  let data_service = Arc::new(LocalDataService::new(
    hub_service.clone(),
    db_service.clone(),
  ));

  // Build auth service
  let auth_service = Arc::new(test_auth_service(&auth_server_url));

  // Build cache service
  let cache_service = Arc::new(MokaCacheService::default());

  // Build AI API service
  let ai_api_service = Arc::new(DefaultAiApiService::with_db_service(db_service.clone()));

  // Build concurrency service
  let concurrency_service = Arc::new(LocalConcurrencyService::default());

  // Build queue producer (StubQueue is a unit struct, no new() method)
  let queue_producer: Arc<dyn services::QueueProducer> = Arc::new(StubQueue);

  // Build ExaService (needed by ToolService)
  let exa_service = Arc::new(DefaultExaService::new());

  // Build tool service
  let tool_service = Arc::new(DefaultToolService::new(
    db_service.clone(),
    exa_service,
    time_service.clone(),
    false, // is_production
  ));
  let access_request_service = Arc::new(DefaultAccessRequestService::new(
    db_service.clone(),
    auth_service.clone(),
    tool_service.clone(),
    time_service.clone(),
    setting_service.public_server_url(),
  ));

  // Build network service (need to provide ip field for struct)
  let network_service = Arc::new(StubNetworkService {
    ip: Some("127.0.0.1".to_string()),
  });

  // Build DefaultAppService with all services in correct order
  let app_service = DefaultAppService::new(
    setting_service,
    hub_service,
    data_service,
    auth_service,
    db_service,
    session_service,
    Arc::new(secret_service),
    cache_service,
    time_service,
    ai_api_service,
    concurrency_service,
    queue_producer,
    tool_service,
    network_service,
    access_request_service,
  );

  Ok(Arc::new(app_service))
}

#[fixture]
pub async fn app_service_setup() -> anyhow::Result<(TempDir, Arc<dyn AppService>)> {
  let temp_dir = tempfile::tempdir()?;
  let app_service = setup_minimal_app_service(&temp_dir).await?;
  Ok((temp_dir, app_service))
}

#[fixture]
#[awt]
pub async fn live_server(
  #[future] app_service_setup: anyhow::Result<(TempDir, Arc<dyn AppService>)>,
) -> anyhow::Result<TestServerHandle> {
  let host = String::from("127.0.0.1");
  let port = rand::rng().random_range(2000..60000);
  let (temp_cache_dir, app_service) = app_service_setup?;
  let serve_command = ServeCommand::ByParams {
    host: host.clone(),
    port,
  };
  let handle = serve_command
    .get_server_handle(app_service.clone(), None)
    .await?;
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
    .same_site(cookie::SameSite::Strict)
    .build()
}
