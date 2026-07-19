#![allow(dead_code)]

use cookie::Cookie;
use routes_app::middleware::{
  access_token_key, refresh_token_key, SESSION_KEY_ACTIVE_CLIENT_ID, SESSION_KEY_USER_ID,
};
use rstest::fixture;
use serde_json::Value;
use serde_yaml::Value as YamlValue;
use server_app::{ServeCommand, ServerShutdownHandle};
use server_core::{DefaultSharedContext, LocalLlamaImpl};
use services::test_utils::TEST_CLIENT_ID;
use services::{
  db::{DbCore, DefaultDbService, DefaultTimeService},
  extract_claims, hash_key,
  inference::LocalLlama,
  test_utils::{
    access_token_claims, build_token, test_auth_service, OfflineHubService, StubNetworkService,
    StubQueue,
  },
  AppService, AppStatus, DefaultAccessRequestService, DefaultAiApiClientFactory, DefaultAppService,
  DefaultEnvWrapper, DefaultMcpService, DefaultSessionService, DefaultSettingService,
  DefaultTenantService, EnvWrapper, HfHubService, LocalConcurrencyService, LocalDataService,
  MokaCacheService, SettingService, TenantService, UserIdClaims, BODHI_AUTH_REALM, BODHI_AUTH_URL,
  BODHI_DEPLOYMENT, BODHI_ENCRYPTION_KEY, BODHI_ENV_TYPE, BODHI_EXEC_LOOKUP_PATH, BODHI_HOME,
  BODHI_HOST, BODHI_LOGS, BODHI_MULTITENANT_CLIENT_ID, BODHI_MULTITENANT_CLIENT_SECRET, BODHI_PORT,
  BODHI_VERSION, HF_HOME, SETTINGS_YAML,
};
use services::{EnvType, KeycloakAuthService, Setting, SettingMetadata, SettingSource};
use std::{collections::HashMap, fs, path::Path, sync::Arc};
use tempfile::TempDir;
use time::{Duration, OffsetDateTime};
use tower_sessions::session::{Id, Record};
use tower_sessions::SessionStore;

/// Inline minimal setup without lib_bodhiserver dependency
async fn setup_minimal_app_service(temp_dir: &TempDir) -> anyhow::Result<Arc<dyn AppService>> {
  let time_service: Arc<dyn services::TimeService> = Arc::new(DefaultTimeService);
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

  // Real HuggingFace cache at ~/.cache/huggingface (live tests use downloaded models).
  let hf_home = dirs::home_dir()
    .ok_or_else(|| anyhow::anyhow!("Failed to determine home directory"))?
    .join(".cache")
    .join("huggingface");
  fs::create_dir_all(hf_home.join("hub"))?;

  let mut env_vars = HashMap::new();
  env_vars.insert(BODHI_HOME.to_string(), bodhi_home.display().to_string());
  env_vars.insert(BODHI_LOGS.to_string(), logs_dir.display().to_string());
  env_vars.insert(HF_HOME.to_string(), hf_home.display().to_string());

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

  let auth_server_url = std::env::var("INTEG_TEST_AUTH_URL")
    .map_err(|_| anyhow::anyhow!("INTEG_TEST_AUTH_URL not set - required for live tests"))?;
  let realm = std::env::var("INTEG_TEST_AUTH_REALM")
    .map_err(|_| anyhow::anyhow!("INTEG_TEST_AUTH_REALM not set - required for live tests"))?;
  env_vars.insert(BODHI_AUTH_URL.to_string(), auth_server_url.clone());
  env_vars.insert(BODHI_AUTH_REALM.to_string(), realm.clone());
  env_vars.insert(BODHI_HOST.to_string(), "127.0.0.1".to_string());
  env_vars.insert(BODHI_PORT.to_string(), "51135".to_string());

  let mut env_wrapper_impl = DefaultEnvWrapper::default();
  for (key, value) in &env_vars {
    env_wrapper_impl.set_var(key, value);
  }
  let env_wrapper: Arc<dyn EnvWrapper> = Arc::new(env_wrapper_impl);

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
    Setting {
      key: BODHI_DEPLOYMENT.to_string(),
      value: YamlValue::String("standalone".to_string()),
      source: SettingSource::System,
      metadata: SettingMetadata::String,
    },
  ];

  // Create SQLite databases early (before setting_service, which needs db_service)
  let app_db_path = bodhi_home.join("bodhi.sqlite");
  fs::File::create_new(&app_db_path)?;
  let session_db_path = bodhi_home.join("session.sqlite");
  fs::File::create_new(&session_db_path)?;

  // Build DB service with pool (needed by setting_service)
  let encryption_key_raw = env_wrapper.var(BODHI_ENCRYPTION_KEY).unwrap();
  let encryption_key = hash_key(&encryption_key_raw);
  let app_db_url = format!("sqlite:{}?mode=rwc", app_db_path.display());
  let db = sea_orm::Database::connect(&app_db_url).await?;
  let db_service = Arc::new(DefaultDbService::new(
    db,
    time_service.clone(),
    encryption_key.clone(),
  ));
  db_service.migrate().await?;

  // Build settings service directly
  let settings_file = bodhi_home.join(SETTINGS_YAML);
  let mut system_settings = app_settings;
  system_settings.push(Setting {
    key: BODHI_HOME.to_string(),
    value: YamlValue::String(bodhi_home.display().to_string()),
    source: SettingSource::Environment,
    metadata: SettingMetadata::String,
  });
  let setting_service = DefaultSettingService::from_parts(
    services::BootstrapParts {
      env_wrapper: env_wrapper.clone(),
      settings_file,
      system_settings,
      file_defaults: HashMap::new(),
      app_settings: HashMap::new(),
      app_command: services::AppCommand::Default,
      bodhi_home: bodhi_home.clone(),
    },
    db_service.clone(),
  );

  let resource_client_id = std::env::var("INTEG_TEST_RESOURCE_CLIENT_ID")
    .map_err(|_| anyhow::anyhow!("INTEG_TEST_RESOURCE_CLIENT_ID not set"))?;
  let resource_client_secret = std::env::var("INTEG_TEST_RESOURCE_CLIENT_SECRET")
    .map_err(|_| anyhow::anyhow!("INTEG_TEST_RESOURCE_CLIENT_SECRET not set"))?;

  let tenant_service = DefaultTenantService::new(db_service.clone());
  tenant_service
    .create_tenant(
      &resource_client_id,
      &resource_client_secret,
      "Integration Test App",
      None,
      AppStatus::Ready,
      Some("integration-test-user".to_string()),
    )
    .await?;

  // Build session service using DefaultSessionService (auto-detects backend from URL)
  let session_db_url = format!("sqlite:{}", session_db_path.display());
  let session_service = DefaultSessionService::connect(&session_db_url).await?;
  let session_service = Arc::new(session_service);

  let setting_service = Arc::new(setting_service);

  let hf_cache = setting_service.hf_cache().await;
  let hub_service = Arc::new(OfflineHubService::new(HfHubService::new(
    hf_cache, false, None,
  )));

  let data_service = Arc::new(LocalDataService::new(
    hub_service.clone(),
    db_service.clone(),
  ));

  let auth_service = Arc::new(test_auth_service(&auth_server_url));
  let cache_service = Arc::new(MokaCacheService::default());
  let concurrency_service = Arc::new(LocalConcurrencyService::default());
  let queue_producer: Arc<dyn services::QueueProducer> = Arc::new(StubQueue);

  let tenant_service: Arc<dyn TenantService> = Arc::new(tenant_service);
  let access_request_service = Arc::new(DefaultAccessRequestService::new(
    db_service.clone(),
    auth_service.clone(),
    time_service.clone(),
    setting_service.clone(),
    Arc::new(StubNetworkService {
      ip: Some("127.0.0.1".to_string()),
    }),
  ));

  let network_service = Arc::new(StubNetworkService {
    ip: Some("127.0.0.1".to_string()),
  });

  let mcp_client = Arc::new(mcp_client::DefaultMcpClient::new());
  let mcp_service = Arc::new(DefaultMcpService::new(
    db_service.clone(),
    mcp_client,
    time_service.clone(),
  )?);

  let token_service: Arc<dyn services::TokenService> = Arc::new(
    services::DefaultTokenService::new(db_service.clone(), time_service.clone()),
  );
  let ctx = Arc::new(DefaultSharedContext::new(hub_service.clone(), setting_service.clone()).await);
  let keep_alive_secs = setting_service.keep_alive().await;
  let local_llama: Arc<dyn LocalLlama> = Arc::new(LocalLlamaImpl::new(ctx, keep_alive_secs));
  let ai_api_client_factory =
    Arc::new(DefaultAiApiClientFactory::new()?.with_local_llama(local_llama.clone()));
  let api_model_service: Arc<dyn services::ApiModelService> =
    Arc::new(services::DefaultApiModelService::new(
      db_service.clone(),
      time_service.clone(),
      ai_api_client_factory.clone(),
    ));
  let model_router_service: Arc<dyn services::ModelRouterService> =
    Arc::new(services::DefaultModelRouterService::new(
      db_service.clone(),
      data_service.clone(),
      time_service.clone(),
    ));
  let download_service: Arc<dyn services::DownloadService> = Arc::new(
    services::DefaultDownloadService::new(db_service.clone(), time_service.clone()),
  );
  let health_registry: Arc<dyn services::HealthRegistry> =
    Arc::new(services::DefaultHealthRegistry::default());
  let app_service = DefaultAppService::new(
    setting_service,
    hub_service,
    data_service,
    auth_service,
    db_service,
    session_service,
    tenant_service,
    cache_service,
    time_service,
    ai_api_client_factory,
    concurrency_service,
    queue_producer,
    network_service,
    access_request_service,
    mcp_service,
    token_service,
    Some(local_llama),
    api_model_service,
    model_router_service,
    health_registry,
    download_service,
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
  let port: u16 = 51135;
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
  let auth_url = setting_service.auth_url().await;
  let realm = setting_service.auth_realm().await;
  let instance = app_service
    .tenant_service()
    .get_standalone_app()
    .await?
    .expect("Tenant is not set");
  let client_id = instance.client_id;
  let client_secret = instance.client_secret;
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

/// Create a session with OAuth tokens and return session ID.
///
/// Stores tokens under multi-tenant namespaced session keys:
/// - `active_client_id` -> tenant client_id
/// - `user_id` -> JWT sub claim
/// - `access_token:{client_id}` -> access token
/// - `refresh_token:{client_id}` -> refresh token
pub async fn create_authenticated_session(
  app_service: &Arc<dyn AppService>,
  access_token: &str,
  refresh_token: &str,
) -> anyhow::Result<String> {
  let session_service = app_service.session_service();

  let instance = app_service
    .tenant_service()
    .get_standalone_app()
    .await?
    .expect("Tenant is not set");
  let client_id = instance.client_id;

  let claims = extract_claims::<UserIdClaims>(access_token)?;
  let user_id = claims.sub;

  let session_id = Id::default();
  let session_data = maplit::hashmap! {
    SESSION_KEY_ACTIVE_CLIENT_ID.to_string() => Value::String(client_id.clone()),
    SESSION_KEY_USER_ID.to_string() => Value::String(user_id),
    access_token_key(&client_id) => Value::String(access_token.to_string()),
    refresh_token_key(&client_id) => Value::String(refresh_token.to_string()),
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
pub fn create_session_cookie(session_id: &str) -> Cookie<'_> {
  Cookie::build(("bodhiapp_session_id", session_id))
    .path("/")
    .http_only(true)
    .same_site(cookie::SameSite::Strict)
    .build()
}

// =============================================================================
// OAuth test infrastructure (no Keycloak dependency)
// =============================================================================

/// Builds a `DefaultAppService` with real services but NO Keycloak dependency.
///
/// Mirrors `setup_minimal_app_service()` but replaces all `INTEG_TEST_*` env vars
/// with test defaults. Uses `test_auth_service()` with a fake URL since no real
/// Keycloak calls will be made (ExternalTokenSimulator seeds the cache directly).
pub async fn setup_test_app_service(temp_dir: &TempDir) -> anyhow::Result<Arc<dyn AppService>> {
  setup_test_app_service_with_time(temp_dir, Arc::new(DefaultTimeService)).await
}

/// Like `setup_test_app_service` but with an injected clock, so tests that need to
/// cross a time boundary (e.g. expiring a router cooldown) can advance it without
/// sleeping. Pass a `services::test_utils::TestTimeService` and keep a clone.
pub async fn setup_test_app_service_with_time(
  temp_dir: &TempDir,
  time_service: Arc<dyn services::TimeService>,
) -> anyhow::Result<Arc<dyn AppService>> {
  let cache_dir = temp_dir.path().join(".cache");
  let bodhi_home = cache_dir.join("bodhi");
  let logs_dir = bodhi_home.join("logs");
  fs::create_dir_all(&logs_dir)?;

  let hf_home = dirs::home_dir()
    .ok_or_else(|| anyhow::anyhow!("Failed to determine home directory"))?
    .join(".cache")
    .join("huggingface");
  fs::create_dir_all(hf_home.join("hub"))?;

  let mut env_vars = HashMap::new();
  env_vars.insert(BODHI_HOME.to_string(), bodhi_home.display().to_string());
  env_vars.insert(BODHI_LOGS.to_string(), logs_dir.display().to_string());
  env_vars.insert(HF_HOME.to_string(), hf_home.display().to_string());

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

  // Fake auth URL — no real Keycloak calls (ExternalTokenSimulator seeds cache).
  let auth_server_url = "https://test-id.getbodhi.app".to_string();
  let realm = "bodhi".to_string();
  env_vars.insert(BODHI_AUTH_URL.to_string(), auth_server_url.clone());
  env_vars.insert(BODHI_AUTH_REALM.to_string(), realm.clone());
  env_vars.insert(BODHI_HOST.to_string(), "127.0.0.1".to_string());
  env_vars.insert(BODHI_PORT.to_string(), "51135".to_string());

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
    Setting {
      key: BODHI_DEPLOYMENT.to_string(),
      value: YamlValue::String("standalone".to_string()),
      source: SettingSource::System,
      metadata: SettingMetadata::String,
    },
  ];

  // Create SQLite databases early (before setting_service, which needs db_service)
  let app_db_path = bodhi_home.join("bodhi.sqlite");
  fs::File::create_new(&app_db_path)?;
  let session_db_path = bodhi_home.join("session.sqlite");
  fs::File::create_new(&session_db_path)?;

  // Build DB service with pool (needed by setting_service)
  let encryption_key_raw = env_wrapper.var(BODHI_ENCRYPTION_KEY).unwrap();
  let encryption_key = hash_key(&encryption_key_raw);
  let app_db_url = format!("sqlite:{}?mode=rwc", app_db_path.display());
  let db = sea_orm::Database::connect(&app_db_url).await?;
  let db_service = Arc::new(DefaultDbService::new(
    db,
    time_service.clone(),
    encryption_key.clone(),
  ));
  db_service.migrate().await?;

  // Build settings service directly
  let settings_file = bodhi_home.join(SETTINGS_YAML);
  let mut system_settings = app_settings;
  system_settings.push(Setting {
    key: BODHI_HOME.to_string(),
    value: YamlValue::String(bodhi_home.display().to_string()),
    source: SettingSource::Environment,
    metadata: SettingMetadata::String,
  });
  let setting_service = DefaultSettingService::from_parts(
    services::BootstrapParts {
      env_wrapper: env_wrapper.clone(),
      settings_file,
      system_settings,
      file_defaults: HashMap::new(),
      app_settings: HashMap::new(),
      app_command: services::AppCommand::Default,
      bodhi_home: bodhi_home.clone(),
    },
    db_service.clone(),
  );

  let tenant_service = DefaultTenantService::new(db_service.clone());
  tenant_service
    .create_tenant(
      "test-resource-client",
      "test-resource-secret",
      "Test App",
      None,
      AppStatus::Ready,
      Some("test-user".to_string()),
    )
    .await?;

  // Build session service using DefaultSessionService (auto-detects backend from URL)
  let session_db_url = format!("sqlite:{}", session_db_path.display());
  let session_service = DefaultSessionService::connect(&session_db_url).await?;
  let session_service = Arc::new(session_service);

  let setting_service = Arc::new(setting_service);

  let hf_cache = setting_service.hf_cache().await;
  let hub_service = Arc::new(OfflineHubService::new(HfHubService::new(
    hf_cache, false, None,
  )));

  let data_service = Arc::new(LocalDataService::new(
    hub_service.clone(),
    db_service.clone(),
  ));

  // Fake URL — never called; cache is seeded by ExternalTokenSimulator.
  let auth_service = Arc::new(test_auth_service(&auth_server_url));
  let cache_service = Arc::new(MokaCacheService::default());
  let concurrency_service = Arc::new(LocalConcurrencyService::default());
  let queue_producer: Arc<dyn services::QueueProducer> = Arc::new(StubQueue);
  let tenant_service: Arc<dyn TenantService> = Arc::new(tenant_service);
  let access_request_service = Arc::new(DefaultAccessRequestService::new(
    db_service.clone(),
    auth_service.clone(),
    time_service.clone(),
    setting_service.clone(),
    Arc::new(StubNetworkService {
      ip: Some("127.0.0.1".to_string()),
    }),
  ));
  let network_service = Arc::new(StubNetworkService {
    ip: Some("127.0.0.1".to_string()),
  });

  let mcp_client = Arc::new(mcp_client::DefaultMcpClient::new());
  let mcp_service = Arc::new(DefaultMcpService::new(
    db_service.clone(),
    mcp_client,
    time_service.clone(),
  )?);

  let token_service: Arc<dyn services::TokenService> = Arc::new(
    services::DefaultTokenService::new(db_service.clone(), time_service.clone()),
  );
  let ctx = Arc::new(DefaultSharedContext::new(hub_service.clone(), setting_service.clone()).await);
  let keep_alive_secs = setting_service.keep_alive().await;
  let local_llama: Arc<dyn LocalLlama> = Arc::new(LocalLlamaImpl::new(ctx, keep_alive_secs));
  let ai_api_client_factory =
    Arc::new(DefaultAiApiClientFactory::new()?.with_local_llama(local_llama.clone()));
  let api_model_service: Arc<dyn services::ApiModelService> =
    Arc::new(services::DefaultApiModelService::new(
      db_service.clone(),
      time_service.clone(),
      ai_api_client_factory.clone(),
    ));
  let model_router_service: Arc<dyn services::ModelRouterService> =
    Arc::new(services::DefaultModelRouterService::new(
      db_service.clone(),
      data_service.clone(),
      time_service.clone(),
    ));
  let download_service: Arc<dyn services::DownloadService> = Arc::new(
    services::DefaultDownloadService::new(db_service.clone(), time_service.clone()),
  );
  let health_registry: Arc<dyn services::HealthRegistry> =
    Arc::new(services::DefaultHealthRegistry::default());
  let app_service = DefaultAppService::new(
    setting_service,
    hub_service,
    data_service,
    auth_service,
    db_service,
    session_service,
    tenant_service,
    cache_service,
    time_service,
    ai_api_client_factory,
    concurrency_service,
    queue_producer,
    network_service,
    access_request_service,
    mcp_service,
    token_service,
    Some(local_llama),
    api_model_service,
    model_router_service,
    health_registry,
    download_service,
  );

  Ok(Arc::new(app_service))
}

/// Live test server handle for OAuth tests (no Keycloak dependency).
pub struct TestLiveServer {
  pub _temp_dir: TempDir,
  pub base_url: String,
  pub app_service: Arc<dyn AppService>,
  pub handle: ServerShutdownHandle,
}

/// Starts a live HTTP server on port 51135 with real services but no Keycloak.
///
/// Uses `setup_test_app_service()` for service bootstrap and `ServeCommand::ByParams`
/// to bind a TCP listener. Fails if port 51135 is unavailable.
pub async fn start_test_live_server() -> anyhow::Result<TestLiveServer> {
  let temp_dir = tempfile::tempdir()?;
  let app_service = setup_test_app_service(&temp_dir).await?;

  let host = String::from("127.0.0.1");
  let port: u16 = 51135;
  let serve_command = ServeCommand::ByParams {
    host: host.clone(),
    port,
  };
  let handle = serve_command
    .get_server_handle(app_service.clone(), None)
    .await?;

  let base_url = format!("http://{}:{}", host, port);
  Ok(TestLiveServer {
    _temp_dir: temp_dir,
    base_url,
    app_service,
    handle,
  })
}

/// Like `start_test_live_server` but with a settable clock the test can advance to
/// expire router cooldowns deterministically. Returns the server and the shared
/// `TestTimeService` (a clone of the one the server uses — `advance`/`set` are visible
/// to the running server).
pub async fn start_test_live_server_with_time(
) -> anyhow::Result<(TestLiveServer, services::test_utils::TestTimeService)> {
  let temp_dir = tempfile::tempdir()?;
  let time = services::test_utils::TestTimeService::default();
  let app_service = setup_test_app_service_with_time(&temp_dir, Arc::new(time.clone())).await?;

  let host = String::from("127.0.0.1");
  let port: u16 = 51135;
  let serve_command = ServeCommand::ByParams {
    host: host.clone(),
    port,
  };
  let handle = serve_command
    .get_server_handle(app_service.clone(), None)
    .await?;

  let base_url = format!("http://{}:{}", host, port);
  Ok((
    TestLiveServer {
      _temp_dir: temp_dir,
      base_url,
      app_service,
      handle,
    },
    time,
  ))
}

/// Creates an authenticated session for live server tests by minting a JWT
/// with the specified roles and storing it in the session DB.
///
/// Returns `(session_cookie, user_id)` where:
/// - `session_cookie` is the `Cookie` header value for HTTP requests
/// - `user_id` is the UUID from the JWT `sub` claim, used to coordinate with
///   `ExternalTokenSimulator::create_token_with_scope_and_user()`
pub async fn create_test_session_for_live_server(
  app_service: &Arc<dyn AppService>,
  roles: &[&str],
) -> anyhow::Result<(String, String)> {
  let actual_client_id = app_service
    .tenant_service()
    .get_standalone_app()
    .await?
    .map(|inst| inst.client_id)
    .unwrap_or_else(|| TEST_CLIENT_ID.to_string());

  let mut claims = access_token_claims();
  // azp must be the actual tenant client_id so middleware can resolve the tenant.
  claims["azp"] = serde_json::json!(&actual_client_id);
  claims["resource_access"][&actual_client_id]["roles"] = serde_json::json!(roles);
  // Also keep roles under TEST_CLIENT_ID for backward compat.
  claims["resource_access"][TEST_CLIENT_ID]["roles"] = serde_json::json!(roles);

  let user_id = claims["sub"]
    .as_str()
    .expect("access_token_claims must have sub")
    .to_string();

  let (token, _) = build_token(claims)?;

  let session_id = Id::default();
  let mut data = HashMap::new();
  data.insert(
    SESSION_KEY_ACTIVE_CLIENT_ID.to_string(),
    Value::String(actual_client_id.clone()),
  );
  data.insert(
    SESSION_KEY_USER_ID.to_string(),
    Value::String(user_id.clone()),
  );
  data.insert(access_token_key(&actual_client_id), Value::String(token));

  let record = Record {
    id: session_id,
    data,
    expiry_date: OffsetDateTime::now_utc() + Duration::hours(1),
  };

  let session_service = app_service.session_service();
  let store = session_service.get_session_store();
  store.save(&record).await?;

  let session_cookie = format!("bodhiapp_session_id={}", session_id);
  Ok((session_cookie, user_id))
}

// =============================================================================
// Multi-tenant live server infrastructure (requires real Keycloak)
// =============================================================================

/// Inline minimal setup for multi-tenant mode without lib_bodhiserver dependency.
///
/// Mirrors `setup_minimal_app_service()` but:
/// - Sets `BODHI_DEPLOYMENT=multi-tenant` in env vars and system settings
/// - Reads `INTEG_TEST_MULTI_TENANT_CLIENT_ID` and `INTEG_TEST_MULTI_TENANT_CLIENT_SECRET` from env
/// - Sets `BODHI_MULTITENANT_CLIENT_ID` and `BODHI_MULTITENANT_CLIENT_SECRET` env vars/settings
/// - Does NOT register a standalone tenant (multi-tenant starts clean)
pub async fn setup_multitenant_app_service(
  temp_dir: &TempDir,
) -> anyhow::Result<Arc<dyn AppService>> {
  let time_service: Arc<dyn services::TimeService> = Arc::new(DefaultTimeService);
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

  let hf_home = dirs::home_dir()
    .ok_or_else(|| anyhow::anyhow!("Failed to determine home directory"))?
    .join(".cache")
    .join("huggingface");
  fs::create_dir_all(hf_home.join("hub"))?;

  let mut env_vars = HashMap::new();
  env_vars.insert(BODHI_HOME.to_string(), bodhi_home.display().to_string());
  env_vars.insert(BODHI_LOGS.to_string(), logs_dir.display().to_string());
  env_vars.insert(HF_HOME.to_string(), hf_home.display().to_string());

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

  let auth_server_url = std::env::var("INTEG_TEST_AUTH_URL")
    .map_err(|_| anyhow::anyhow!("INTEG_TEST_AUTH_URL not set - required for live tests"))?;
  let realm = std::env::var("INTEG_TEST_AUTH_REALM")
    .map_err(|_| anyhow::anyhow!("INTEG_TEST_AUTH_REALM not set - required for live tests"))?;
  env_vars.insert(BODHI_AUTH_URL.to_string(), auth_server_url.clone());
  env_vars.insert(BODHI_AUTH_REALM.to_string(), realm.clone());
  env_vars.insert(BODHI_HOST.to_string(), "127.0.0.1".to_string());
  env_vars.insert(BODHI_PORT.to_string(), "51135".to_string());

  let mt_client_id = std::env::var("INTEG_TEST_MULTI_TENANT_CLIENT_ID")
    .map_err(|_| anyhow::anyhow!("INTEG_TEST_MULTI_TENANT_CLIENT_ID not set"))?;
  let mt_client_secret = std::env::var("INTEG_TEST_MULTI_TENANT_CLIENT_SECRET")
    .map_err(|_| anyhow::anyhow!("INTEG_TEST_MULTI_TENANT_CLIENT_SECRET not set"))?;
  env_vars.insert(
    BODHI_MULTITENANT_CLIENT_ID.to_string(),
    mt_client_id.clone(),
  );
  env_vars.insert(
    BODHI_MULTITENANT_CLIENT_SECRET.to_string(),
    mt_client_secret.clone(),
  );

  let mut env_wrapper_impl = DefaultEnvWrapper::default();
  for (key, value) in &env_vars {
    env_wrapper_impl.set_var(key, value);
  }
  let env_wrapper: Arc<dyn EnvWrapper> = Arc::new(env_wrapper_impl);

  // Build system settings (includes multi-tenant deployment mode)
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
    Setting {
      key: BODHI_DEPLOYMENT.to_string(),
      value: YamlValue::String("multi_tenant".to_string()),
      source: SettingSource::System,
      metadata: SettingMetadata::String,
    },
    Setting {
      key: BODHI_MULTITENANT_CLIENT_ID.to_string(),
      value: YamlValue::String(mt_client_id),
      source: SettingSource::System,
      metadata: SettingMetadata::String,
    },
  ];

  // Create SQLite databases early (before setting_service, which needs db_service)
  let app_db_path = bodhi_home.join("bodhi.sqlite");
  fs::File::create_new(&app_db_path)?;
  let session_db_path = bodhi_home.join("session.sqlite");
  fs::File::create_new(&session_db_path)?;

  // Build DB service with pool (needed by setting_service)
  let encryption_key_raw = env_wrapper.var(BODHI_ENCRYPTION_KEY).unwrap();
  let encryption_key = hash_key(&encryption_key_raw);
  let app_db_url = format!("sqlite:{}?mode=rwc", app_db_path.display());
  let db = sea_orm::Database::connect(&app_db_url).await?;
  let db_service = Arc::new(DefaultDbService::new(
    db,
    time_service.clone(),
    encryption_key.clone(),
  ));
  db_service.migrate().await?;

  // Build settings service directly
  let settings_file = bodhi_home.join(SETTINGS_YAML);
  let mut system_settings = app_settings;
  system_settings.push(Setting {
    key: BODHI_HOME.to_string(),
    value: YamlValue::String(bodhi_home.display().to_string()),
    source: SettingSource::Environment,
    metadata: SettingMetadata::String,
  });
  let setting_service = DefaultSettingService::from_parts(
    services::BootstrapParts {
      env_wrapper: env_wrapper.clone(),
      settings_file,
      system_settings,
      file_defaults: HashMap::new(),
      app_settings: HashMap::new(),
      app_command: services::AppCommand::Default,
      bodhi_home: bodhi_home.clone(),
    },
    db_service.clone(),
  );

  // Create tenant service — NO standalone tenant registration for multi-tenant
  let tenant_service = DefaultTenantService::new(db_service.clone());

  // Build session service using DefaultSessionService (auto-detects backend from URL)
  let session_db_url = format!("sqlite:{}", session_db_path.display());
  let session_service = DefaultSessionService::connect(&session_db_url).await?;
  let session_service = Arc::new(session_service);

  let setting_service = Arc::new(setting_service);

  let hf_cache = setting_service.hf_cache().await;
  let hub_service = Arc::new(OfflineHubService::new(HfHubService::new(
    hf_cache, false, None,
  )));

  let data_service = Arc::new(LocalDataService::new(
    hub_service.clone(),
    db_service.clone(),
  ));

  let auth_service = Arc::new(KeycloakAuthService::new(
    "test-version",
    auth_server_url.clone(),
    realm.clone(),
  ));

  let cache_service = Arc::new(MokaCacheService::default());
  let concurrency_service = Arc::new(LocalConcurrencyService::default());
  let queue_producer: Arc<dyn services::QueueProducer> = Arc::new(StubQueue);

  let tenant_service: Arc<dyn TenantService> = Arc::new(tenant_service);
  let access_request_service = Arc::new(DefaultAccessRequestService::new(
    db_service.clone(),
    auth_service.clone(),
    time_service.clone(),
    setting_service.clone(),
    Arc::new(StubNetworkService {
      ip: Some("127.0.0.1".to_string()),
    }),
  ));

  let network_service = Arc::new(StubNetworkService {
    ip: Some("127.0.0.1".to_string()),
  });

  let mcp_client = Arc::new(mcp_client::DefaultMcpClient::new());
  let mcp_service = Arc::new(DefaultMcpService::new(
    db_service.clone(),
    mcp_client,
    time_service.clone(),
  )?);

  let token_service: Arc<dyn services::TokenService> = Arc::new(
    services::DefaultTokenService::new(db_service.clone(), time_service.clone()),
  );
  // Multi-tenant: no local llama runtime — Box::None on AppService.
  let ai_api_client_factory = Arc::new(DefaultAiApiClientFactory::new()?);
  let api_model_service: Arc<dyn services::ApiModelService> =
    Arc::new(services::DefaultApiModelService::new(
      db_service.clone(),
      time_service.clone(),
      ai_api_client_factory.clone(),
    ));
  let model_router_service: Arc<dyn services::ModelRouterService> =
    Arc::new(services::DefaultModelRouterService::new(
      db_service.clone(),
      data_service.clone(),
      time_service.clone(),
    ));
  let download_service: Arc<dyn services::DownloadService> = Arc::new(
    services::DefaultDownloadService::new(db_service.clone(), time_service.clone()),
  );
  let health_registry: Arc<dyn services::HealthRegistry> =
    Arc::new(services::DefaultHealthRegistry::default());
  let app_service = DefaultAppService::new(
    setting_service,
    hub_service,
    data_service,
    auth_service,
    db_service,
    session_service,
    tenant_service,
    cache_service,
    time_service,
    ai_api_client_factory,
    concurrency_service,
    queue_producer,
    network_service,
    access_request_service,
    mcp_service,
    token_service,
    None,
    api_model_service,
    model_router_service,
    health_registry,
    download_service,
  );

  Ok(Arc::new(app_service))
}

/// Starts a live multi-tenant HTTP server on port 51135 with real Keycloak.
///
/// Uses `setup_multitenant_app_service()` for service bootstrap. Fails if port 51135
/// is unavailable.
pub async fn start_multitenant_live_server() -> anyhow::Result<TestLiveServer> {
  let temp_dir = tempfile::tempdir()?;
  let app_service = setup_multitenant_app_service(&temp_dir).await?;

  let host = String::from("127.0.0.1");
  let port: u16 = 51135;
  let serve_command = ServeCommand::ByParams {
    host: host.clone(),
    port,
  };
  let handle = serve_command
    .get_server_handle(app_service.clone(), None)
    .await?;

  let base_url = format!("http://{}:{}", host, port);
  Ok(TestLiveServer {
    _temp_dir: temp_dir,
    base_url,
    app_service,
    handle,
  })
}

/// Create a dashboard session by injecting the dashboard access token into the session store.
///
/// Stores token under `dashboard:access_token` key.
/// Returns the session cookie string: `bodhiapp_session_id={id}`.
pub async fn create_dashboard_session(
  app_service: &Arc<dyn AppService>,
  dashboard_access_token: &str,
) -> anyhow::Result<(String, Id)> {
  let session_service = app_service.session_service();
  let session_id = Id::default();
  let session_data = maplit::hashmap! {
    "dashboard:access_token".to_string() => Value::String(dashboard_access_token.to_string()),
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
  let cookie = format!("bodhiapp_session_id={}", session_id);
  Ok((cookie, session_id))
}

/// Update an existing session by adding resource token keys and active_client_id.
///
/// Loads the session record by ID, adds the new keys, and saves it back.
pub async fn add_resource_token_to_session(
  app_service: &Arc<dyn AppService>,
  session_id: Id,
  client_id: &str,
  resource_access_token: &str,
) -> anyhow::Result<()> {
  let session_service = app_service.session_service();
  let store = session_service.get_session_store();

  let mut record = store
    .load(&session_id)
    .await?
    .ok_or_else(|| anyhow::anyhow!("Session not found: {}", session_id))?;

  record.data.insert(
    SESSION_KEY_ACTIVE_CLIENT_ID.to_string(),
    Value::String(client_id.to_string()),
  );
  record.data.insert(
    access_token_key(client_id),
    Value::String(resource_access_token.to_string()),
  );

  store.save(&record).await?;
  Ok(())
}

/// Get a dashboard token via password grant against the multi-tenant client.
pub async fn get_dashboard_token_via_password_grant(
  auth_url: &str,
  realm: &str,
  mt_client_id: &str,
  mt_client_secret: &str,
  username: &str,
  password: &str,
) -> anyhow::Result<String> {
  let token_url = format!(
    "{}/realms/{}/protocol/openid-connect/token",
    auth_url.trim_end_matches('/'),
    realm
  );

  let params = [
    ("grant_type", "password"),
    ("client_id", mt_client_id),
    ("client_secret", mt_client_secret),
    ("username", username),
    ("password", password),
    ("scope", "openid email profile"),
  ];

  let client = reqwest::Client::new();
  let response = client.post(&token_url).form(&params).send().await?;
  if response.status() != reqwest::StatusCode::OK {
    let body = response
      .text()
      .await
      .unwrap_or_else(|_| "Unable to read response body".to_string());
    anyhow::bail!("Dashboard token request failed: {}", body);
  }
  let token_data: Value = response.json().await?;
  let access_token = token_data["access_token"]
    .as_str()
    .ok_or_else(|| anyhow::anyhow!("Missing access_token in response"))?;
  Ok(access_token.to_string())
}

/// Get a resource token via password grant against a resource client (with client_secret).
pub async fn get_resource_token_via_password_grant(
  auth_url: &str,
  realm: &str,
  client_id: &str,
  client_secret: &str,
  username: &str,
  password: &str,
) -> anyhow::Result<String> {
  let token_url = format!(
    "{}/realms/{}/protocol/openid-connect/token",
    auth_url.trim_end_matches('/'),
    realm
  );

  let params = [
    ("grant_type", "password"),
    ("client_id", client_id),
    ("client_secret", client_secret),
    ("username", username),
    ("password", password),
    ("scope", "openid email profile roles"),
  ];

  let client = reqwest::Client::new();
  let response = client.post(&token_url).form(&params).send().await?;
  if response.status() != reqwest::StatusCode::OK {
    let body = response
      .text()
      .await
      .unwrap_or_else(|_| "Unable to read response body".to_string());
    anyhow::bail!("Resource token request failed: {}", body);
  }
  let token_data: Value = response.json().await?;
  let access_token = token_data["access_token"]
    .as_str()
    .ok_or_else(|| anyhow::anyhow!("Missing access_token in response"))?;
  Ok(access_token.to_string())
}
