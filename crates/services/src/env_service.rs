use crate::{SettingService, SettingServiceError};
use objs::{
  impl_error_from, AppError, AppType, EnvType, ErrorType, IoDirCreateError, IoError, LogLevel,
  SerdeYamlError,
};
use std::{collections::HashMap, path::PathBuf, sync::Arc};

pub static PROD_DB: &str = "bodhi.sqlite";
pub static ALIASES_DIR: &str = "aliases";
pub static MODELS_YAML: &str = "models.yaml";

pub static LOGS_DIR: &str = "logs";
pub static DEFAULT_SCHEME: &str = "http";
pub static DEFAULT_HOST: &str = "localhost";
pub static DEFAULT_PORT: u16 = 1135;
pub static DEFAULT_PORT_STR: &str = "1135";
pub static DEFAULT_LOG_LEVEL: &str = "warn";

pub static BODHI_HOME: &str = "BODHI_HOME";
pub static BODHI_ENV_TYPE: &str = "BODHI_ENV_TYPE";
pub static BODHI_APP_TYPE: &str = "BODHI_APP_TYPE";
pub static BODHI_VERSION: &str = "BODHI_VERSION";
pub static BODHI_AUTH_URL: &str = "BODHI_AUTH_URL";
pub static BODHI_AUTH_REALM: &str = "BODHI_AUTH_REALM";

pub static HF_HOME: &str = "HF_HOME";
pub static BODHI_LOGS: &str = "BODHI_LOGS";
pub static BODHI_LOG_LEVEL: &str = "BODHI_LOG_LEVEL";
pub static BODHI_LOG_STDOUT: &str = "BODHI_LOG_STDOUT";
pub static BODHI_SCHEME: &str = "BODHI_SCHEME";
pub static BODHI_HOST: &str = "BODHI_HOST";
pub static BODHI_PORT: &str = "BODHI_PORT";
pub static BODHI_FRONTEND_URL: &str = "BODHI_FRONTEND_URL";
pub static BODHI_EXEC_PATH: &str = "BODHI_EXEC_PATH";
pub static BODHI_EXEC_LOOKUP_PATH: &str = "BODHI_EXEC_LOOKUP_PATH";
pub static BODHI_ENCRYPTION_KEY: &str = "BODHI_ENCRYPTION_KEY";
pub static BODHI_DEV_PROXY_UI: &str = "BODHI_DEV_PROXY_UI";

pub static SETTINGS_YAML: &str = "settings.yaml";

pub static SETTING_VARS: &[&str] = &[
  BODHI_LOGS,
  BODHI_LOG_LEVEL,
  BODHI_LOG_STDOUT,
  HF_HOME,
  BODHI_SCHEME,
  BODHI_HOST,
  BODHI_PORT,
  BODHI_FRONTEND_URL,
  BODHI_EXEC_PATH,
  BODHI_EXEC_LOOKUP_PATH,
];

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

  fn set_setting(&self, key: &str, value: &str) -> Result<(), EnvServiceError>;

  fn get_setting(&self, key: &str) -> Option<String>;

  fn get_env(&self, key: &str) -> Option<String>;

  fn list(&self) -> HashMap<String, String>;

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
  fn get_dev_env(&self) -> Option<String> {
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
    self
      .setting_service
      .get_setting_or_default(BODHI_SCHEME, DEFAULT_SCHEME)
  }

  fn host(&self) -> String {
    self
      .setting_service
      .get_setting_or_default(BODHI_HOST, DEFAULT_HOST)
  }

  fn port(&self) -> u16 {
    self
      .setting_service
      .get_setting_or_default(BODHI_PORT, DEFAULT_PORT_STR)
      .parse::<u16>()
      .unwrap_or(DEFAULT_PORT)
  }

  fn frontend_url(&self) -> String {
    self
      .setting_service
      .get_setting_or_default(BODHI_FRONTEND_URL, &self.server_url())
  }

  fn db_path(&self) -> PathBuf {
    self.bodhi_home().join(PROD_DB)
  }

  fn log_level(&self) -> LogLevel {
    let log_level = self
      .setting_service
      .get_setting_or_default(BODHI_LOG_LEVEL, DEFAULT_LOG_LEVEL);
    LogLevel::try_from(log_level.as_str()).unwrap_or(LogLevel::Warn)
  }

  fn exec_lookup_path(&self) -> String {
    let lookup_path = self.setting_service.get_setting(BODHI_EXEC_LOOKUP_PATH);
    match lookup_path {
      Some(lookup_path) => lookup_path,
      None => std::env::current_dir()
        .unwrap_or_else(|err| {
          tracing::warn!("failed to get current directory. err: {err}");
          PathBuf::from(".")
            .canonicalize()
            .expect("failed to canonicalize current directory")
        })
        .display()
        .to_string(),
    }
  }

  fn exec_path(&self) -> String {
    let exec_path = self.setting_service.get_setting_or_default(
      BODHI_EXEC_PATH,
      &format!(
        "{}/{}/{}",
        llama_server_proc::BUILD_TARGET,
        llama_server_proc::DEFAULT_VARIANT,
        llama_server_proc::EXEC_NAME
      ),
    );
    exec_path.replace('/', std::path::MAIN_SEPARATOR_STR)
  }

  fn set_setting(&self, key: &str, value: &str) -> Result<(), EnvServiceError> {
    if !SETTING_VARS.contains(&key) {
      return Err(EnvServiceError::InvalidSettingKey(key.to_string()));
    }
    self.setting_service.set_setting(key, value)?;
    Ok(())
  }

  fn get_setting(&self, key: &str) -> Option<String> {
    if !SETTING_VARS.contains(&key) {
      return None;
    }
    self.setting_service.get_setting(key)
  }

  fn get_env(&self, key: &str) -> Option<String> {
    self.setting_service.get_env(key)
  }

  fn list(&self) -> HashMap<String, String> {
    let mut result = HashMap::<String, String>::new();
    result.insert(
      BODHI_HOME.to_string(),
      self.bodhi_home().display().to_string(),
    );
    result.insert(BODHI_ENV_TYPE.to_string(), self.env_type().to_string());
    result.insert(BODHI_APP_TYPE.to_string(), self.app_type().to_string());
    result.insert(BODHI_VERSION.to_string(), self.version());
    result.insert(BODHI_AUTH_URL.to_string(), self.auth_url());
    result.insert(BODHI_AUTH_REALM.to_string(), self.auth_realm());

    for key in SETTING_VARS {
      result.insert(
        key.to_string(),
        self
          .setting_service
          .get_setting(key)
          .unwrap_or_else(|| "<not-set>".to_string()),
      );
    }
    result
  }

  fn encryption_key(&self) -> Option<String> {
    self.setting_service.get_env(BODHI_ENCRYPTION_KEY)
  }
}

#[cfg(test)]
mod tests {
  use crate::EnvServiceError;
  use objs::AppError;
  use objs::{
    test_utils::{assert_error_message, setup_l10n},
    FluentLocalizationService,
  };
  use rstest::rstest;
  use std::sync::Arc;

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
}
