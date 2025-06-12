use objs::{ApiError, ErrorMessage, FluentLocalizationService, LocalizationService};
use services::{
  db::{DbPool, DbService, DefaultTimeService, SqliteDbService, TimeService},
  hash_key, AuthService, CacheService, DataService, DefaultAppService, DefaultSecretService,
  HfHubService, HubService, KeycloakAuthService, KeyringStore, LocalDataService, MokaCacheService,
  SecretService, SessionService, SettingService, SqliteSessionService, SystemKeyringStore,
};
use std::sync::Arc;

use crate::AppServiceBuilderError;

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
  localization_service: Option<Arc<dyn LocalizationService>>,
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
      localization_service: None,
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

  /// Sets the localization service.
  pub fn localization_service(
    mut self,
    service: Arc<dyn LocalizationService>,
  ) -> Result<Self, AppServiceBuilderError> {
    if self.localization_service.is_some() {
      return Err(AppServiceBuilderError::ServiceAlreadySet(
        "localization_service".to_string(),
      ));
    }
    self.localization_service = Some(service);
    Ok(self)
  }

  /// Builds the complete DefaultAppService, resolving all dependencies automatically.
  pub async fn build(mut self) -> Result<DefaultAppService, ErrorMessage> {
    // Build services in dependency order
    let localization_service = self.get_or_build_localization_service()?;
    let hub_service = self.get_or_build_hub_service();
    let data_service = self.get_or_build_data_service(hub_service.clone());
    let time_service = self.get_or_build_time_service();
    let db_service = self.get_or_build_db_service(time_service.clone()).await?;
    let session_service = self.get_or_build_session_service().await?;
    let secret_service = self.get_or_build_secret_service()?;
    let cache_service = self.get_or_build_cache_service();
    let auth_service = self.get_or_build_auth_service();

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
      localization_service,
      time_service,
    );

    Ok(app_service)
  }

  /// Gets or builds the hub service.
  fn get_or_build_hub_service(&mut self) -> Arc<dyn HubService> {
    if let Some(service) = self.hub_service.take() {
      return service;
    }

    let hf_cache = self.setting_service.hf_cache();
    Arc::new(HfHubService::new_from_hf_cache(hf_cache, true))
  }

  /// Gets or builds the data service, ensuring hub service dependency is resolved.
  fn get_or_build_data_service(
    &mut self,
    hub_service: Arc<dyn HubService>,
  ) -> Arc<dyn DataService> {
    if let Some(service) = self.data_service.take() {
      return service;
    }

    let bodhi_home = self.setting_service.bodhi_home();
    Arc::new(LocalDataService::new(bodhi_home, hub_service))
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
  ) -> Result<Arc<dyn DbService>, ApiError> {
    if let Some(service) = self.db_service.take() {
      return Ok(service);
    }

    let app_db_pool = DbPool::connect(&format!(
      "sqlite:{}",
      self.setting_service.app_db_path().display()
    ))
    .await?;
    let db_service = SqliteDbService::new(app_db_pool, time_service);
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
  fn get_or_build_secret_service(&mut self) -> Result<Arc<dyn SecretService>, ApiError> {
    if let Some(service) = self.secret_service.take() {
      return Ok(service);
    }

    let app_suffix = if self.setting_service.is_production() {
      ""
    } else {
      " - Dev"
    };
    let app_name = format!("Bodhi App{app_suffix}");
    let secrets_path = self.setting_service.secrets_path();
    let encryption_key = self.setting_service.encryption_key();
    let encryption_key = encryption_key
      .map(|key| Ok(hash_key(&key)))
      .unwrap_or_else(|| SystemKeyringStore::new(&app_name).get_or_generate(SECRET_KEY))?;

    let secret_service = DefaultSecretService::new(encryption_key, &secrets_path)?;
    Ok(Arc::new(secret_service))
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

  /// Gets or builds the localization service.
  fn get_or_build_localization_service(
    &mut self,
  ) -> Result<Arc<dyn LocalizationService>, ErrorMessage> {
    if let Some(service) = self.localization_service.take() {
      return Ok(service);
    }

    let localization_service = FluentLocalizationService::get_instance();
    load_all_localization_resources(&localization_service)?;
    Ok(localization_service)
  }
}

/// Builds a complete DefaultAppService from a settings service.
/// This function orchestrates the creation of all required services and their dependencies.
pub async fn build_app_service(
  setting_service: Arc<dyn SettingService>,
) -> Result<DefaultAppService, ErrorMessage> {
  AppServiceBuilder::new(setting_service).build().await
}

/// Loads all localization resources from all crates in the workspace.
/// This ensures that error messages and other localized content are available.
fn load_all_localization_resources(
  localization_service: &FluentLocalizationService,
) -> Result<(), ErrorMessage> {
  localization_service
    .load_resource(objs::l10n::L10N_RESOURCES)?
    .load_resource(objs::gguf::l10n::L10N_RESOURCES)?
    .load_resource(llama_server_proc::l10n::L10N_RESOURCES)?
    .load_resource(services::l10n::L10N_RESOURCES)?
    .load_resource(commands::l10n::L10N_RESOURCES)?
    .load_resource(server_core::l10n::L10N_RESOURCES)?
    .load_resource(auth_middleware::l10n::L10N_RESOURCES)?
    .load_resource(routes_oai::l10n::L10N_RESOURCES)?
    .load_resource(routes_app::l10n::L10N_RESOURCES)?
    .load_resource(routes_all::l10n::L10N_RESOURCES)?
    .load_resource(server_app::l10n::L10N_RESOURCES)?
    .load_resource(crate::l10n::L10N_RESOURCES)?;

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::{AppServiceBuilder, AppServiceBuilderError};
  use crate::{setup_app_dirs, AppOptionsBuilder};
  use objs::{test_utils::empty_bodhi_home, FluentLocalizationService};
  use rstest::rstest;
  use services::{test_utils::FrozenTimeService, AppService, MockSecretService, SettingService};
  use std::sync::Arc;
  use tempfile::TempDir;

  #[rstest]
  #[case(&AppServiceBuilderError::ServiceAlreadySet("test_service".to_string()), "Service already set: test_service")]
  fn test_error_messages_objs(#[case] error: &AppServiceBuilderError, #[case] expected: &str) {
    assert_eq!(expected, error.to_string());
  }

  #[rstest]
  #[tokio::test]
  async fn test_app_service_builder_new(empty_bodhi_home: TempDir) -> anyhow::Result<()> {
    let bodhi_home = empty_bodhi_home.path().join("bodhi");
    let bodhi_home_str = bodhi_home.display().to_string();

    let options = AppOptionsBuilder::with_bodhi_home(&bodhi_home_str).build()?;
    let setting_service = Arc::new(setup_app_dirs(options)?);

    let builder = AppServiceBuilder::new(setting_service.clone());

    // Verify builder is created with correct settings
    assert_eq!(builder.setting_service.bodhi_home(), bodhi_home);

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_app_service_builder_with_custom_services(
    empty_bodhi_home: TempDir,
    #[from(objs::test_utils::setup_l10n)] _localization_service: &Arc<
      objs::FluentLocalizationService,
    >,
  ) -> anyhow::Result<()> {
    let bodhi_home = empty_bodhi_home.path().join("bodhi");
    let bodhi_home_str = bodhi_home.display().to_string();

    let options = AppOptionsBuilder::with_bodhi_home(&bodhi_home_str).build()?;
    let setting_service = Arc::new(setup_app_dirs(options)?);

    // Create mock services
    let mut mock_secret_service = MockSecretService::new();
    mock_secret_service
      .expect_set_secret_string()
      .returning(|_, _| Ok(()));
    mock_secret_service
      .expect_get_secret_string()
      .returning(|_| Ok(Some("test_value".to_string())));

    let time_service = Arc::new(FrozenTimeService::default());
    let localization_service = FluentLocalizationService::get_instance();

    let app_service = AppServiceBuilder::new(setting_service.clone())
      .secret_service(Arc::new(mock_secret_service))?
      .time_service(time_service.clone())?
      .localization_service(localization_service.clone())?
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
    #[from(objs::test_utils::setup_l10n)] _localization_service: &Arc<
      objs::FluentLocalizationService,
    >,
  ) -> anyhow::Result<()> {
    let bodhi_home = empty_bodhi_home.path().join("bodhi");
    let bodhi_home_str = bodhi_home.display().to_string();

    let options = AppOptionsBuilder::with_bodhi_home(&bodhi_home_str).build()?;
    let setting_service = Arc::new(setup_app_dirs(options)?);

    // Create mock secret service
    let mut mock_secret_service = MockSecretService::new();
    mock_secret_service
      .expect_set_secret_string()
      .returning(|_, _| Ok(()));
    mock_secret_service
      .expect_get_secret_string()
      .returning(|_| Ok(Some("test_value".to_string())));

    // Create mock localization service (using a simple Arc wrapper)
    let mock_localization_service = FluentLocalizationService::get_instance();

    let app_service = AppServiceBuilder::new(setting_service.clone())
      .secret_service(Arc::new(mock_secret_service))?
      .localization_service(mock_localization_service)?
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
  async fn test_build_app_service_default(
    empty_bodhi_home: TempDir,
    #[from(objs::test_utils::setup_l10n)] _localization_service: &Arc<
      objs::FluentLocalizationService,
    >,
  ) -> anyhow::Result<()> {
    let bodhi_home = empty_bodhi_home.path().join("bodhi");
    let bodhi_home_str = bodhi_home.display().to_string();

    let options = AppOptionsBuilder::with_bodhi_home(&bodhi_home_str).build()?;
    let setting_service = Arc::new(setup_app_dirs(options)?);

    // Use mock secret service to avoid keyring access
    let mock_secret_service = MockSecretService::new();

    let app_service = AppServiceBuilder::new(setting_service.clone())
      .secret_service(Arc::new(mock_secret_service))?
      .localization_service(FluentLocalizationService::get_instance())?
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
  async fn test_builder_dependency_resolution(
    empty_bodhi_home: TempDir,
    #[from(objs::test_utils::setup_l10n)] _localization_service: &Arc<
      objs::FluentLocalizationService,
    >,
  ) -> anyhow::Result<()> {
    let bodhi_home = empty_bodhi_home.path().join("bodhi");
    let bodhi_home_str = bodhi_home.display().to_string();

    let options = AppOptionsBuilder::with_bodhi_home(&bodhi_home_str).build()?;
    let setting_service = Arc::new(setup_app_dirs(options)?);

    // Test that dependencies are resolved automatically with mock secret service
    let mock_secret_service = MockSecretService::new();
    let app_service = AppServiceBuilder::new(setting_service.clone())
      .secret_service(Arc::new(mock_secret_service))?
      .localization_service(FluentLocalizationService::get_instance())?
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
}
