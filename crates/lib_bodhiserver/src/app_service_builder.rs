use crate::{AppServiceBuilderError, AppStateOption};
use objs::{ApiError, ErrorMessage};
use services::{
  db::{DbCore, DbPool, DbService, DefaultTimeService, SqliteDbService, TimeService},
  hash_key, AiApiService, AppService, AuthService, CacheService, DataService, DefaultAiApiService,
  DefaultAppService, DefaultExaService, DefaultSecretService, DefaultToolService, ExaService,
  HfHubService, HubService, InMemoryQueue, KeycloakAuthService, KeyringStore,
  LocalConcurrencyService, LocalDataService, MokaCacheService, QueueConsumer, QueueProducer,
  RefreshWorker, SecretService, SecretServiceExt, SessionService, SettingService,
  SqliteSessionService, SystemKeyringStore, ToolService, HF_TOKEN,
};
use std::sync::Arc;

const SECRET_KEY: &str = "secret_key";

/// A comprehensive service builder that handles dependency injection and automatic dependency resolution.
/// Services can be provided externally or built automatically from settings.
/// Prevents duplicate builds and provides clear error messages for dependency issues.
#[derive(Debug)]
pub struct AppServiceBuilder {
  setting_service: Arc<dyn SettingService>,

  // Core services
  hub_service: Option<Arc<dyn HubService>>,
  data_service: Option<Arc<dyn DataService>>,
  time_service: Option<Arc<dyn TimeService>>,

  // Database services
  db_service: Option<Arc<dyn DbService>>,
  session_service: Option<Arc<dyn SessionService>>,

  // Other services
  secret_service: Option<Arc<dyn SecretService>>,
  cache_service: Option<Arc<dyn CacheService>>,
  auth_service: Option<Arc<dyn AuthService>>,
  ai_api_service: Option<Arc<dyn AiApiService>>,
  encryption_key: Option<Vec<u8>>,
}

impl AppServiceBuilder {
  /// Creates a new builder with the given settings service.
  pub fn new(setting_service: Arc<dyn SettingService>) -> Self {
    Self {
      setting_service,
      hub_service: None,
      data_service: None,
      time_service: None,
      db_service: None,
      session_service: None,
      secret_service: None,
      cache_service: None,
      auth_service: None,
      ai_api_service: None,
      encryption_key: None,
    }
  }

  /// Sets the hub service. If data service depends on hub service and is not set,
  /// it will be built automatically when needed.
  pub fn hub_service(
    mut self,
    service: Arc<dyn HubService>,
  ) -> Result<Self, AppServiceBuilderError> {
    if self.hub_service.is_some() {
      return Err(AppServiceBuilderError::ServiceAlreadySet(
        "hub_service".to_string(),
      ));
    }
    self.hub_service = Some(service);
    Ok(self)
  }

  /// Sets the data service.
  pub fn data_service(
    mut self,
    service: Arc<dyn DataService>,
  ) -> Result<Self, AppServiceBuilderError> {
    if self.data_service.is_some() {
      return Err(AppServiceBuilderError::ServiceAlreadySet(
        "data_service".to_string(),
      ));
    }
    self.data_service = Some(service);
    Ok(self)
  }

  /// Sets the time service.
  pub fn time_service(
    mut self,
    service: Arc<dyn TimeService>,
  ) -> Result<Self, AppServiceBuilderError> {
    if self.time_service.is_some() {
      return Err(AppServiceBuilderError::ServiceAlreadySet(
        "time_service".to_string(),
      ));
    }
    self.time_service = Some(service);
    Ok(self)
  }

  /// Sets the database service.
  pub fn db_service(mut self, service: Arc<dyn DbService>) -> Result<Self, AppServiceBuilderError> {
    if self.db_service.is_some() {
      return Err(AppServiceBuilderError::ServiceAlreadySet(
        "db_service".to_string(),
      ));
    }
    self.db_service = Some(service);
    Ok(self)
  }

  /// Sets the session service.
  pub fn session_service(
    mut self,
    service: Arc<dyn SessionService>,
  ) -> Result<Self, AppServiceBuilderError> {
    if self.session_service.is_some() {
      return Err(AppServiceBuilderError::ServiceAlreadySet(
        "session_service".to_string(),
      ));
    }
    self.session_service = Some(service);
    Ok(self)
  }

  /// Sets the secret service.
  pub fn secret_service(
    mut self,
    service: Arc<dyn SecretService>,
  ) -> Result<Self, AppServiceBuilderError> {
    if self.secret_service.is_some() {
      return Err(AppServiceBuilderError::ServiceAlreadySet(
        "secret_service".to_string(),
      ));
    }
    self.secret_service = Some(service);
    Ok(self)
  }

  /// Sets the cache service.
  pub fn cache_service(
    mut self,
    service: Arc<dyn CacheService>,
  ) -> Result<Self, AppServiceBuilderError> {
    if self.cache_service.is_some() {
      return Err(AppServiceBuilderError::ServiceAlreadySet(
        "cache_service".to_string(),
      ));
    }
    self.cache_service = Some(service);
    Ok(self)
  }

  /// Sets the auth service.
  pub fn auth_service(
    mut self,
    service: Arc<dyn AuthService>,
  ) -> Result<Self, AppServiceBuilderError> {
    if self.auth_service.is_some() {
      return Err(AppServiceBuilderError::ServiceAlreadySet(
        "auth_service".to_string(),
      ));
    }
    self.auth_service = Some(service);
    Ok(self)
  }

  /// Sets the AI API service.
  pub fn ai_api_service(
    mut self,
    service: Arc<dyn AiApiService>,
  ) -> Result<Self, AppServiceBuilderError> {
    if self.ai_api_service.is_some() {
      return Err(AppServiceBuilderError::ServiceAlreadySet(
        "ai_api_service".to_string(),
      ));
    }
    self.ai_api_service = Some(service);
    Ok(self)
  }

  /// Builds the complete DefaultAppService, resolving all dependencies automatically.
  pub async fn build(mut self) -> Result<DefaultAppService, ErrorMessage> {
    // Build services in dependency order
    let hub_service = self.get_or_build_hub_service();
    let time_service = self.get_or_build_time_service();
    let encryption_key = self.get_or_build_encryption_key()?;
    let secret_service = self.get_or_build_secret_service(encryption_key.clone())?;
    let db_service = self
      .get_or_build_db_service(time_service.clone(), encryption_key)
      .await?;
    let data_service = self.get_or_build_data_service(hub_service.clone(), db_service.clone());
    let session_service = self.get_or_build_session_service().await?;
    let cache_service = self.get_or_build_cache_service();
    let auth_service = self.get_or_build_auth_service();
    let ai_api_service = self.get_or_build_ai_api_service(db_service.clone());
    let concurrency_service = self.get_or_build_concurrency_service();
    let tool_service = self.get_or_build_tool_service(db_service.clone(), time_service.clone());

    // Create queue and spawn refresh worker
    let queue = Arc::new(InMemoryQueue::new());
    let is_processing = queue.get_is_processing();
    let queue_producer: Arc<dyn QueueProducer> = queue.clone();
    let queue_consumer: Arc<dyn QueueConsumer> = queue;

    // Spawn refresh worker in background
    let worker = RefreshWorker::new(
      queue_consumer,
      hub_service.clone(),
      data_service.clone(),
      db_service.clone(),
      is_processing,
    );
    tokio::spawn(async move {
      worker.run().await;
    });

    // Build and return the complete app service
    let app_service = DefaultAppService::new(
      self.setting_service,
      hub_service,
      data_service,
      auth_service,
      db_service,
      session_service,
      secret_service,
      cache_service,
      time_service,
      ai_api_service,
      concurrency_service,
      queue_producer,
      tool_service,
    );
    Ok(app_service)
  }

  /// Gets or builds the hub service.
  fn get_or_build_hub_service(&mut self) -> Arc<dyn HubService> {
    if let Some(service) = self.hub_service.take() {
      return service;
    }

    let hf_cache = self.setting_service.hf_cache();
    let hf_token = self.setting_service.get_env(HF_TOKEN);
    Arc::new(HfHubService::new_from_hf_cache(hf_cache, hf_token, true))
  }

  /// Gets or builds the data service, ensuring hub service and db service dependencies are resolved.
  fn get_or_build_data_service(
    &mut self,
    hub_service: Arc<dyn HubService>,
    db_service: Arc<dyn DbService>,
  ) -> Arc<dyn DataService> {
    if let Some(service) = self.data_service.take() {
      return service;
    }

    Arc::new(LocalDataService::new(hub_service, db_service))
  }

  /// Gets or builds the time service.
  fn get_or_build_time_service(&mut self) -> Arc<dyn TimeService> {
    if let Some(service) = self.time_service.take() {
      return service;
    }

    Arc::new(DefaultTimeService)
  }

  /// Gets or builds the database service, ensuring time service dependency is resolved.
  async fn get_or_build_db_service(
    &mut self,
    time_service: Arc<dyn TimeService>,
    encryption_key: Vec<u8>,
  ) -> Result<Arc<dyn DbService>, ApiError> {
    if let Some(service) = self.db_service.take() {
      return Ok(service);
    }

    let app_db_pool = DbPool::connect(&format!(
      "sqlite:{}",
      self.setting_service.app_db_path().display()
    ))
    .await?;
    let is_production = self.setting_service.is_production();
    let db_service = SqliteDbService::new(app_db_pool, time_service, encryption_key, is_production);
    db_service.migrate().await?;
    Ok(Arc::new(db_service))
  }

  /// Gets or builds the session service.
  async fn get_or_build_session_service(&mut self) -> Result<Arc<dyn SessionService>, ApiError> {
    if let Some(service) = self.session_service.take() {
      return Ok(service);
    }

    let session_db_pool = DbPool::connect(&format!(
      "sqlite:{}",
      self.setting_service.session_db_path().display()
    ))
    .await?;
    let session_service = SqliteSessionService::new(session_db_pool);
    session_service.migrate().await?;
    Ok(Arc::new(session_service))
  }

  /// Gets or builds the secret service.
  #[allow(clippy::result_large_err)]
  fn get_or_build_secret_service(
    &mut self,
    encryption_key: Vec<u8>,
  ) -> Result<Arc<dyn SecretService>, ApiError> {
    if let Some(service) = self.secret_service.take() {
      return Ok(service);
    }
    let secrets_path = self.setting_service.secrets_path();
    let secret_service = DefaultSecretService::new(encryption_key, &secrets_path)?;
    Ok(Arc::new(secret_service))
  }

  fn get_app_name(&mut self) -> String {
    let app_suffix = if self.setting_service.is_production() {
      ""
    } else {
      " - Dev"
    };
    let app_name = format!("Bodhi App{app_suffix}");
    app_name
  }

  fn get_or_build_encryption_key(&mut self) -> Result<Vec<u8>, ApiError> {
    match self.encryption_key.as_ref() {
      Some(encryption_key) => return Ok(encryption_key.clone()),
      None => {
        let app_name = self.get_app_name();
        let encryption_key = self.setting_service.encryption_key();

        // Validate encryption key is not a placeholder value
        if let Some(ref key) = encryption_key {
          if key == "your-strong-encryption-key-here" {
            return Err(AppServiceBuilderError::PlaceholderValue(key.to_string()))?;
          }
        }

        let encryption_key = encryption_key
          .map(|key| Ok(hash_key(&key)))
          .unwrap_or_else(|| SystemKeyringStore::new(&app_name).get_or_generate(SECRET_KEY))?;
        self.encryption_key = Some(encryption_key);
        Ok(self.encryption_key.as_ref().unwrap().clone())
      }
    }
  }

  /// Gets or builds the cache service.
  fn get_or_build_cache_service(&mut self) -> Arc<dyn CacheService> {
    if let Some(service) = self.cache_service.take() {
      return service;
    }

    Arc::new(MokaCacheService::default())
  }

  /// Gets or builds the auth service.
  fn get_or_build_auth_service(&mut self) -> Arc<dyn AuthService> {
    if let Some(service) = self.auth_service.take() {
      return service;
    }

    let auth_url = self.setting_service.auth_url();
    let auth_realm = self.setting_service.auth_realm();
    Arc::new(KeycloakAuthService::new(
      &self.setting_service.version(),
      auth_url,
      auth_realm,
    ))
  }

  /// Gets or builds the AI API service.
  fn get_or_build_ai_api_service(
    &mut self,
    db_service: Arc<dyn DbService>,
  ) -> Arc<dyn AiApiService> {
    if let Some(service) = self.ai_api_service.take() {
      return service;
    }

    Arc::new(DefaultAiApiService::with_db_service(db_service))
  }

  /// Gets or builds the concurrency service.
  fn get_or_build_concurrency_service(&mut self) -> Arc<dyn services::ConcurrencyService> {
    Arc::new(LocalConcurrencyService::new())
  }

  /// Gets or builds the tool service.
  fn get_or_build_tool_service(
    &mut self,
    db_service: Arc<dyn DbService>,
    time_service: Arc<dyn TimeService>,
  ) -> Arc<dyn ToolService> {
    // Create Exa service
    let exa_service: Arc<dyn ExaService> = Arc::new(DefaultExaService::new());

    // Create tool service with dependencies
    let is_production = self.setting_service.is_production();
    Arc::new(DefaultToolService::new(
      db_service,
      exa_service,
      time_service,
      is_production,
    ))
  }
}

pub async fn build_app_service(
  setting_service: Arc<dyn SettingService>,
) -> Result<DefaultAppService, ErrorMessage> {
  AppServiceBuilder::new(setting_service).build().await
}

pub fn update_with_option(
  service: &Arc<dyn AppService>,
  option: AppStateOption,
) -> Result<(), ErrorMessage> {
  // Set app registration info if provided
  if let Some(app_reg_info) = option.app_reg_info {
    service
      .secret_service()
      .set_app_reg_info(&app_reg_info)
      .map_err(|e| {
        ErrorMessage::new(
          "secret_service_error".to_string(),
          "internal_server_error".to_string(),
          e.to_string(),
        )
      })?;
  }
  // Set app status if provided
  if let Some(app_status) = option.app_status {
    service
      .secret_service()
      .set_app_status(&app_status)
      .map_err(|e| {
        ErrorMessage::new(
          "secret_service_error".to_string(),
          "internal_server_error".to_string(),
          e.to_string(),
        )
      })?;
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::{AppServiceBuilder, AppServiceBuilderError};
  use crate::{setup_app_dirs, AppOptionsBuilder};
  use objs::test_utils::empty_bodhi_home;
  use rstest::rstest;
  use services::{
    test_utils::FrozenTimeService, AppService, MockCacheService, MockSecretService, SettingService,
  };
  use std::sync::Arc;
  use tempfile::TempDir;

  #[rstest]
  #[case(&AppServiceBuilderError::ServiceAlreadySet("test_service".to_string()), "test_service")]
  fn test_error_messages_objs(#[case] error: &AppServiceBuilderError, #[case] expected: &str) {
    assert_eq!(expected, error.to_string());
  }

  #[rstest]
  #[tokio::test]
  async fn test_app_service_builder_new(empty_bodhi_home: TempDir) -> anyhow::Result<()> {
    let bodhi_home = empty_bodhi_home.path().join("bodhi");

    let options = AppOptionsBuilder::with_bodhi_home(&bodhi_home.display().to_string()).build()?;
    let setting_service = Arc::new(setup_app_dirs(&options)?);

    let builder = AppServiceBuilder::new(setting_service);

    assert_eq!(builder.setting_service.bodhi_home(), bodhi_home);
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_app_service_builder_with_custom_services(
    empty_bodhi_home: TempDir,
  ) -> anyhow::Result<()> {
    let bodhi_home = empty_bodhi_home.path().join("bodhi");
    let options = AppOptionsBuilder::with_bodhi_home(&bodhi_home.display().to_string()).build()?;
    let setting_service = Arc::new(setup_app_dirs(&options)?);

    // Create mock services
    let mut mock_secret_service = MockSecretService::new();
    mock_secret_service
      .expect_set_secret_string()
      .returning(|_, _| Ok(()));
    mock_secret_service
      .expect_get_secret_string()
      .returning(|_| Ok(Some("test_value".to_string())));

    let time_service = Arc::new(FrozenTimeService::default());

    let app_service = AppServiceBuilder::new(setting_service.clone())
      .secret_service(Arc::new(mock_secret_service))?
      .time_service(time_service.clone())?
      .build()
      .await?;

    // Verify all services are properly initialized
    assert_eq!(app_service.setting_service().bodhi_home(), bodhi_home);
    assert!(setting_service.app_db_path().exists());
    assert!(setting_service.session_db_path().exists());

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_app_service_builder_with_multiple_services(
    empty_bodhi_home: TempDir,
  ) -> anyhow::Result<()> {
    let bodhi_home = empty_bodhi_home.path().join("bodhi");
    let bodhi_home_str = bodhi_home.display().to_string();

    let options = AppOptionsBuilder::with_bodhi_home(&bodhi_home_str).build()?;
    let setting_service = Arc::new(setup_app_dirs(&options)?);

    // Create mock secret service
    let mut mock_secret_service = MockSecretService::new();
    mock_secret_service
      .expect_set_secret_string()
      .returning(|_, _| Ok(()));
    mock_secret_service
      .expect_get_secret_string()
      .returning(|_| Ok(Some("test_value".to_string())));

    let app_service = AppServiceBuilder::new(setting_service.clone())
      .secret_service(Arc::new(mock_secret_service))?
      .cache_service(Arc::new(MockCacheService::new()))?
      .build()
      .await?;

    // Verify all services are properly initialized
    assert_eq!(app_service.setting_service().bodhi_home(), bodhi_home);
    assert!(setting_service.app_db_path().exists());
    assert!(setting_service.session_db_path().exists());

    Ok(())
  }

  #[rstest]
  fn test_service_already_set_errors() -> anyhow::Result<()> {
    let setting_service = Arc::new(services::test_utils::SettingServiceStub::default());
    let mock_secret_service = Arc::new(MockSecretService::new());

    let builder =
      AppServiceBuilder::new(setting_service).secret_service(mock_secret_service.clone())?;

    // This should return an error due to duplicate service setting
    let result = builder.secret_service(mock_secret_service);

    assert!(result.is_err());
    assert!(matches!(
      result.unwrap_err(),
      AppServiceBuilderError::ServiceAlreadySet(service) if service == *"secret_service"));

    Ok(())
  }

  #[rstest]
  fn test_setup_app_dirs_with_app_settings(empty_bodhi_home: TempDir) -> anyhow::Result<()> {
    use services::BODHI_PORT;

    let bodhi_home = empty_bodhi_home.path().join("bodhi_enhanced");
    let bodhi_home_str = bodhi_home.display().to_string();

    // Create options with app settings
    let options = AppOptionsBuilder::with_bodhi_home(&bodhi_home_str)
      .set_env("TEST_VAR", "test_value")
      .set_app_setting(BODHI_PORT, "9090")
      .set_system_setting(services::BODHI_ENV_TYPE, "development")?
      .build()?;

    let setting_service = setup_app_dirs(&options)?;

    // Verify configuration was applied
    assert_eq!(setting_service.get_setting(BODHI_PORT).unwrap(), "9090");

    // Verify the service is properly initialized
    assert!(!setting_service.is_production());
    assert!(setting_service.bodhi_home().exists());

    Ok(())
  }
}
