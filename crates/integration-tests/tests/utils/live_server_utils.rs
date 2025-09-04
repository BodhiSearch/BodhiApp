#![allow(dead_code)]

use auth_middleware::{
  test_utils::{AuthServerConfigBuilder, AuthServerTestClient},
  SESSION_KEY_ACCESS_TOKEN, SESSION_KEY_REFRESH_TOKEN,
};
use fs_extra::dir::{copy, CopyOptions};
use lib_bodhiserver::{setup_app_dirs, AppOptionsBuilder, AppServiceBuilder};
use objs::{test_utils::setup_l10n, FluentLocalizationService};
use rand::Rng;
use rstest::fixture;
use serde_json::Value;
use server_app::{ServeCommand, ServerShutdownHandle};
use services::{
  hash_key,
  test_utils::{test_auth_service, OfflineHubService},
  AppRegInfoBuilder, AppService, AppStatus, DefaultSecretService, HfHubService, LocalDataService,
  SecretServiceExt, SettingService, BODHI_ENCRYPTION_KEY, BODHI_EXEC_LOOKUP_PATH, BODHI_HOME,
  BODHI_LOGS, HF_HOME,
};
use std::{path::Path, sync::Arc};
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
  let env_test_path = Path::new(env!("CARGO_MANIFEST_DIR"))
    .join("tests")
    .join("resources")
    .join(".env.test");
  if env_test_path.exists() {
    dotenv::from_filename(&env_test_path).ok();
  }
  let temp_dir = tempfile::tempdir()?;
  let cache_dir = temp_dir.path().join(".cache");
  std::fs::create_dir_all(&cache_dir)?;

  let bodhi_home = cache_dir.join("bodhi");
  let hf_home = dirs::home_dir().unwrap().join(".cache").join("huggingface");
  copy_test_dir("tests/data/live/bodhi", &bodhi_home);

  let bodhi_logs = bodhi_home.join("logs");
  let _hf_cache = hf_home.join("hub");
  let execs_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
    .join("..")
    .join("llama_server_proc")
    .join("bin")
    .canonicalize()?;

  // JWT validation fields no longer needed - using database-backed token integrity validation
  let auth_server_url = std::env::var("INTEG_TEST_AUTH_URL").expect("INTEG_TEST_AUTH_URL not set");
  let realm = std::env::var("INTEG_TEST_AUTH_REALM").expect("INTEG_TEST_AUTH_REALM not set");
  let test_username = std::env::var("INTEG_TEST_USERNAME").expect("INTEG_TEST_USERNAME not set");
  // Get OpenID configuration values from environment
  let config = AuthServerConfigBuilder::default()
    .auth_server_url(&auth_server_url)
    .realm(&realm)
    .dev_console_client_id(
      std::env::var("INTEG_TEST_DEV_CONSOLE_CLIENT_ID")
        .expect("INTEG_TEST_DEV_CONSOLE_CLIENT_ID not set"),
    )
    .dev_console_client_secret(
      std::env::var("INTEG_TEST_DEV_CONSOLE_CLIENT_SECRET")
        .expect("INTEG_TEST_DEV_CONSOLE_CLIENT_SECRET not set"),
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
    .make_first_resource_admin(&resource_token, &test_username)
    .await?;

  let options = AppOptionsBuilder::development()
    .auth_url(&auth_server_url)
    .auth_realm(&realm)
    .set_env("HOME", temp_dir.path().to_str().unwrap())
    .set_env(BODHI_HOME, &bodhi_home.display().to_string())
    .set_env(BODHI_LOGS, &bodhi_logs.display().to_string())
    .set_env(HF_HOME, &hf_home.display().to_string())
    .set_env(BODHI_EXEC_LOOKUP_PATH, &execs_dir.display().to_string())
    .set_env(BODHI_ENCRYPTION_KEY, "test-encryption-key")
    .build()?;
  let setting_service = Arc::new(setup_app_dirs(&options)?);

  // Create AppRegInfo with values from environment
  let app_reg_info = AppRegInfoBuilder::default()
    .client_id(resource_client.client_id)
    .client_secret(resource_client.client_secret.unwrap())
    .build()?;

  // Create secret service with test configuration
  let encryption_key = setting_service.encryption_key().unwrap();
  let encryption_key = hash_key(&encryption_key);
  let secret_service = DefaultSecretService::new(&encryption_key, &setting_service.secrets_path())?;
  secret_service.set_app_reg_info(&app_reg_info)?;
  secret_service.set_app_status(&AppStatus::Ready)?;

  // Create mock hub service for offline testing
  let hf_cache = setting_service.hf_cache();
  let hub_service = Arc::new(OfflineHubService::new(HfHubService::new(
    hf_cache, false, None,
  )));

  // Create mock auth service for testing
  let auth_service = test_auth_service(&auth_server_url);

  // Build app service with custom services for testing
  let service = AppServiceBuilder::new(setting_service)
    .hub_service(hub_service)?
    .auth_service(Arc::new(auth_service))?
    .secret_service(Arc::new(secret_service))?
    .localization_service(localization_service.clone())?
    .build()
    .await?;
  Ok((temp_dir, Arc::new(service)))
}

#[fixture]
#[awt]
pub async fn live_server(
  #[future] llama2_7b_setup: anyhow::Result<(TempDir, Arc<dyn AppService>)>,
) -> anyhow::Result<TestServerHandle> {
  let host = String::from("127.0.0.1");
  let port = rand::rng().random_range(2000..60000);
  let (temp_cache_dir, app_service) = llama2_7b_setup?;
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
