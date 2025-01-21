use crate::{
  SettingService, SettingServiceError, BODHI_EXEC_LOOKUP_PATH, BODHI_EXEC_PATH, BODHI_HOST,
  BODHI_LOGS, BODHI_LOG_LEVEL, BODHI_PORT, BODHI_SCHEME, DEFAULT_PORT, HF_HOME,
};
use objs::{
  impl_error_from, AppError, AppType, EnvType, ErrorType, IoDirCreateError, IoError, LogLevel,
  SerdeYamlError, SettingInfo, SettingMetadata, SettingSource,
};
use std::{path::PathBuf, sync::Arc};

pub const PROD_DB: &str = "bodhi.sqlite";
pub const ALIASES_DIR: &str = "aliases";
pub const MODELS_YAML: &str = "models.yaml";

pub const LOGS_DIR: &str = "logs";

pub const BODHI_HOME: &str = "BODHI_HOME";
pub const BODHI_ENV_TYPE: &str = "BODHI_ENV_TYPE";
pub const BODHI_APP_TYPE: &str = "BODHI_APP_TYPE";
pub const BODHI_VERSION: &str = "BODHI_VERSION";
pub const BODHI_AUTH_URL: &str = "BODHI_AUTH_URL";
pub const BODHI_AUTH_REALM: &str = "BODHI_AUTH_REALM";

pub const BODHI_FRONTEND_URL: &str = "BODHI_FRONTEND_URL";
pub const BODHI_ENCRYPTION_KEY: &str = "BODHI_ENCRYPTION_KEY";
pub const BODHI_DEV_PROXY_UI: &str = "BODHI_DEV_PROXY_UI";

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl= AppError)]
pub enum EnvServiceError {
  #[error("bodhi_home_not_exists")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  BodhiHomeNotExists(String),
  #[error("invalid_setting_key")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  InvalidSettingKey(String),
  #[error(transparent)]
  DirCreate(#[from] IoDirCreateError),
  #[error("settings_update_error")]
  #[error_meta(error_type=ErrorType::InternalServer)]
  SettingsUpdateError(String),
  #[error(transparent)]
  IoError(#[from] IoError),
  #[error(transparent)]
  SerdeYamlError(#[from] SerdeYamlError),
  #[error(transparent)]
  SettingService(#[from] SettingServiceError),
}

impl_error_from!(::std::io::Error, EnvServiceError::IoError, ::objs::IoError);
impl_error_from!(
  ::serde_yaml::Error,
  EnvServiceError::SerdeYamlError,
  ::objs::SerdeYamlError
);

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait EnvService: Send + Sync + std::fmt::Debug {
  fn bodhi_home(&self) -> PathBuf;

  fn env_type(&self) -> EnvType;

  fn is_production(&self) -> bool {
    self.env_type().is_production()
  }

  fn app_type(&self) -> AppType;

  fn is_native(&self) -> bool {
    self.app_type().is_native()
  }

  fn version(&self) -> String;

  fn auth_url(&self) -> String;

  fn auth_realm(&self) -> String;

  fn hf_home(&self) -> PathBuf;

  fn logs_dir(&self) -> PathBuf;

  fn scheme(&self) -> String;

  fn host(&self) -> String;

  fn port(&self) -> u16;

  fn server_url(&self) -> String {
    format!("{}://{}:{}", self.scheme(), self.host(), self.port())
  }

  fn frontend_url(&self) -> String;

  fn db_path(&self) -> PathBuf;

  fn log_level(&self) -> LogLevel;

  fn exec_lookup_path(&self) -> String;

  fn exec_path(&self) -> String;

  fn setting_service(&self) -> Arc<dyn SettingService>;

  fn get_setting(&self, key: &str) -> Option<String>;

  fn get_env(&self, key: &str) -> Option<String>;

  fn list(&self) -> Vec<SettingInfo>;

  fn hf_cache(&self) -> PathBuf {
    self.hf_home().join("hub")
  }

  fn aliases_dir(&self) -> PathBuf {
    self.bodhi_home().join("aliases")
  }

  fn login_url(&self) -> String {
    format!(
      "{}/realms/{}/protocol/openid-connect/auth",
      self.auth_url(),
      self.auth_realm()
    )
  }

  fn token_url(&self) -> String {
    format!(
      "{}/realms/{}/protocol/openid-connect/token",
      self.auth_url(),
      self.auth_realm()
    )
  }

  fn login_callback_url(&self) -> String {
    format!(
      "{}://{}:{}/app/login/callback",
      self.scheme(),
      self.host(),
      self.port()
    )
  }

  fn secrets_path(&self) -> PathBuf {
    self.bodhi_home().join("secrets.yaml")
  }

  fn encryption_key(&self) -> Option<String>;

  #[cfg(not(debug_assertions))]
  fn get_dev_env(&self, _key: &str) -> Option<String> {
    None
  }

  #[cfg(debug_assertions)]
  fn get_dev_env(&self, key: &str) -> Option<String> {
    self.get_env(key)
  }
}

#[derive(Debug, Clone)]
pub struct DefaultEnvService {
  bodhi_home: PathBuf,
  bodhi_home_source: SettingSource,
  env_type: EnvType,
  app_type: AppType,
  version: String,
  auth_url: String,
  auth_realm: String,
  setting_service: Arc<dyn SettingService>,
}

#[allow(clippy::too_many_arguments)]
impl DefaultEnvService {
  #[allow(clippy::new_without_default)]
  pub fn new(
    bodhi_home: PathBuf,
    bodhi_home_source: SettingSource,
    env_type: EnvType,
    app_type: AppType,
    auth_url: String,
    auth_realm: String,
    setting_service: Arc<dyn SettingService>,
  ) -> Result<Self, EnvServiceError> {
    if !bodhi_home.exists() {
      return Err(EnvServiceError::BodhiHomeNotExists(
        bodhi_home.display().to_string(),
      ));
    }
    Ok(DefaultEnvService {
      bodhi_home,
      bodhi_home_source,
      env_type,
      app_type,
      version: env!("CARGO_PKG_VERSION").to_string(),
      setting_service,
      auth_url,
      auth_realm,
    })
  }
}

impl EnvService for DefaultEnvService {
  fn bodhi_home(&self) -> PathBuf {
    self.bodhi_home.clone()
  }

  fn env_type(&self) -> EnvType {
    self.env_type.clone()
  }

  fn app_type(&self) -> AppType {
    self.app_type.clone()
  }

  fn version(&self) -> String {
    self.version.clone()
  }

  fn auth_url(&self) -> String {
    self.auth_url.clone()
  }

  fn auth_realm(&self) -> String {
    self.auth_realm.clone()
  }

  fn hf_home(&self) -> PathBuf {
    PathBuf::from(
      self
        .setting_service
        .get_setting(HF_HOME)
        .expect("HF_HOME should be set"),
    )
  }

  fn logs_dir(&self) -> PathBuf {
    PathBuf::from(
      self
        .setting_service
        .get_setting(BODHI_LOGS)
        .expect("BODHI_LOGS should be set"),
    )
  }

  fn scheme(&self) -> String {
    self.setting_service.get_setting_or_default(BODHI_SCHEME)
  }

  fn host(&self) -> String {
    self.setting_service.get_setting_or_default(BODHI_HOST)
  }

  fn port(&self) -> u16 {
    self
      .setting_service
      .get_setting_or_default(BODHI_PORT)
      .parse::<u16>()
      .unwrap_or(DEFAULT_PORT)
  }

  fn frontend_url(&self) -> String {
    format!(
      "{}://{}:{}",
      self.setting_service.get_setting_or_default(BODHI_SCHEME),
      self.setting_service.get_setting_or_default(BODHI_HOST),
      self.setting_service.get_setting_or_default(BODHI_PORT)
    )
  }

  fn db_path(&self) -> PathBuf {
    self.bodhi_home().join(PROD_DB)
  }

  fn log_level(&self) -> LogLevel {
    let log_level = self.setting_service.get_setting_or_default(BODHI_LOG_LEVEL);
    LogLevel::try_from(log_level.as_str()).unwrap_or(LogLevel::Warn)
  }

  fn exec_lookup_path(&self) -> String {
    self
      .setting_service
      .get_setting_or_default(BODHI_EXEC_LOOKUP_PATH)
  }

  fn exec_path(&self) -> String {
    self.setting_service.get_setting_or_default(BODHI_EXEC_PATH)
  }

  fn setting_service(&self) -> Arc<dyn SettingService> {
    self.setting_service.clone()
  }

  fn get_setting(&self, key: &str) -> Option<String> {
    self.setting_service.get_setting(key)
  }

  fn get_env(&self, key: &str) -> Option<String> {
    self.setting_service.get_env(key)
  }

  fn list(&self) -> Vec<SettingInfo> {
    let mut settings = Vec::new();
    let default_home = self.setting_service.get_default_value(BODHI_HOME);
    // Add system settings
    settings.push(SettingInfo {
      key: BODHI_HOME.to_string(),
      current_value: serde_yaml::Value::String(self.bodhi_home().display().to_string()),
      default_value: default_home,
      source: self.bodhi_home_source.clone(),
      metadata: SettingMetadata::String,
    });

    // Add configurable settings
    settings.extend(self.setting_service.list());
    settings
  }

  fn encryption_key(&self) -> Option<String> {
    self.setting_service.get_env(BODHI_ENCRYPTION_KEY)
  }

  fn get_dev_env(&self, key: &str) -> Option<String> {
    self.get_env(key)
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    test_utils::EnvWrapperStub, DefaultEnvService, DefaultSettingService, EnvService,
    EnvServiceError, BODHI_HOME, BODHI_HOST, BODHI_LOGS, BODHI_LOG_LEVEL, BODHI_LOG_STDOUT,
    BODHI_PORT, BODHI_SCHEME, DEFAULT_LOG_LEVEL, DEFAULT_PORT, DEFAULT_SCHEME, HF_HOME,
  };
  use anyhow_trace::anyhow_trace;
  use objs::{
    test_utils::{assert_error_message, setup_l10n, temp_dir},
    AppError, AppType, EnvType, FluentLocalizationService, SettingInfo, SettingMetadata,
    SettingSource,
  };
  use pretty_assertions::assert_eq;
  use rstest::rstest;
  use std::{collections::HashMap, fs, sync::Arc};
  use tempfile::TempDir;

  #[rstest]
  #[case(&EnvServiceError::BodhiHomeNotExists("/path/to/home".to_string()),
    "BODHI_HOME does not exists: /path/to/home")]
  #[case(&EnvServiceError::InvalidSettingKey("invalid_key".to_string()),
    "Setting key is invalid: invalid_key")]
  #[case(&EnvServiceError::SettingsUpdateError("update failed".to_string()),
    "failed to update settings: update failed")]
  fn test_env_service_error_messages(
    #[from(setup_l10n)] localization_service: &Arc<FluentLocalizationService>,
    #[case] error: &dyn AppError,
    #[case] message: &str,
  ) {
    assert_error_message(localization_service, &error.code(), error.args(), message);
  }

  #[anyhow_trace]
  #[rstest]
  fn test_env_service_list(
    temp_dir: TempDir,
    #[from(temp_dir)] bodhi_home: TempDir,
  ) -> anyhow::Result<()> {
    // GIVEN
    // BODHI_LOGS - from env
    // BODHI_LOG_LEVEL - from env
    // BODHI_LOG_STDOUT - from env
    // HF_HOME - from env
    // BODHI_SCHEME - default=http
    // BODHI_HOST - from setting.yaml
    // BODHI_PORT - from setting.yaml
    // BODHI_FRONTEND_URL - derived
    // BODHI_EXEC_PATH - from setting.yaml
    // BODHI_EXEC_LOOKUP_PATH - from setting.yaml

    let env_wrapper = EnvWrapperStub::new(maplit::hashmap! {
      "HOME".to_owned() => "/test/home".to_string(),
      BODHI_LOGS.to_owned() => "/test/logs".to_string(),
      BODHI_LOG_LEVEL.to_owned() => "debug".to_string(),
      BODHI_LOG_STDOUT.to_owned() => "true".to_string(),
      HF_HOME.to_owned() => "/test/hf/home".to_string(),
    });

    let settings_file = temp_dir.path().join("settings.yaml");
    fs::write(
      &settings_file,
      r#"
BODHI_HOST: test.host
BODHI_PORT: 8080
BODHI_EXEC_PATH: test_server
BODHI_EXEC_LOOKUP_PATH: /test/exec/lookup
"#,
    )?;

    let setting_service = DefaultSettingService::new(Arc::new(env_wrapper), settings_file.clone());
    let env_service = DefaultEnvService::new(
      bodhi_home.path().to_path_buf(),
      SettingSource::Default,
      EnvType::Production,
      AppType::Native,
      "http://auth.test".to_string(),
      "test-realm".to_string(),
      Arc::new(setting_service),
    )?;

    // WHEN
    let settings = env_service
      .list()
      .into_iter()
      .map(|setting| (setting.key.clone(), setting))
      .collect::<HashMap<String, SettingInfo>>();

    // THEN
    // System settings
    let expected_bodhi_home = SettingInfo {
      key: BODHI_HOME.to_string(),
      current_value: serde_yaml::Value::String(bodhi_home.path().display().to_string()),
      default_value: serde_yaml::Value::String("/test/home/.cache/bodhi".to_string()),
      source: SettingSource::Default,
      metadata: SettingMetadata::String,
    };
    assert_eq!(
      expected_bodhi_home,
      settings.get(BODHI_HOME).unwrap().clone()
    );

    // Environment variable settings
    let expected_log_level = SettingInfo {
      key: BODHI_LOG_LEVEL.to_string(),
      current_value: serde_yaml::Value::String("debug".to_string()),
      default_value: serde_yaml::Value::String(DEFAULT_LOG_LEVEL.to_string()),
      source: SettingSource::Environment,
      metadata: SettingMetadata::option(&["error", "warn", "info", "debug", "trace"]),
    };
    assert_eq!(
      expected_log_level,
      settings.get(BODHI_LOG_LEVEL).unwrap().clone()
    );

    // Settings file settings
    let expected_port = SettingInfo {
      key: BODHI_PORT.to_string(),
      current_value: serde_yaml::Value::Number(8080.into()),
      default_value: serde_yaml::Value::Number(DEFAULT_PORT.into()),
      source: SettingSource::SettingsFile,
      metadata: SettingMetadata::Number {
        min: 1025,
        max: 65535,
      },
    };
    assert_eq!(expected_port, settings.get(BODHI_PORT).unwrap().clone());

    // Boolean setting
    let expected_stdout = SettingInfo {
      key: BODHI_LOG_STDOUT.to_string(),
      current_value: serde_yaml::Value::Bool(true),
      default_value: serde_yaml::Value::Bool(false),
      source: SettingSource::Environment,
      metadata: SettingMetadata::Boolean,
    };
    assert_eq!(
      expected_stdout,
      settings.get(BODHI_LOG_STDOUT).unwrap().clone()
    );

    // Default value setting
    let expected_scheme = SettingInfo {
      key: BODHI_SCHEME.to_string(),
      current_value: serde_yaml::Value::String(DEFAULT_SCHEME.to_string()),
      default_value: serde_yaml::Value::String(DEFAULT_SCHEME.to_string()),
      source: SettingSource::Default,
      metadata: SettingMetadata::String,
    };
    assert_eq!(expected_scheme, settings.get(BODHI_SCHEME).unwrap().clone());

    let expected_host = SettingInfo {
      key: BODHI_HOST.to_string(),
      current_value: serde_yaml::Value::String("test.host".to_string()),
      default_value: serde_yaml::Value::String("localhost".to_string()),
      source: SettingSource::SettingsFile,
      metadata: SettingMetadata::String,
    };
    assert_eq!(expected_host, settings.get(BODHI_HOST).unwrap().clone());
    Ok(())
  }
}
