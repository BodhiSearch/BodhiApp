use crate::{DataServiceError, EnvWrapper};
use objs::{
  ApiError, AppError, AppType, EnvType, ErrorType, IoError, SerdeYamlError, SerdeYamlWithPathError,
  Settings,
};
use serde::Serialize;
use std::{
  collections::HashMap,
  path::{Path, PathBuf},
  sync::{Arc, Mutex},
};

pub static PROD_DB: &str = "bodhi.sqlite";
pub static ALIASES_DIR: &str = "aliases";
pub static MODELS_YAML: &str = "models.yaml";

pub static LOGS_DIR: &str = "logs";
pub static DEFAULT_SCHEME: &str = "http";
pub static DEFAULT_HOST: &str = "localhost";
pub static DEFAULT_PORT: u16 = 1135;
pub static DEFAULT_PORT_STR: &str = "1135";

pub static BODHI_HOME: &str = "BODHI_HOME";
pub static BODHI_SCHEME: &str = "BODHI_SCHEME";
pub static BODHI_FRONTEND_URL: &str = "BODHI_FRONTEND_URL";
pub static BODHI_HOST: &str = "BODHI_HOST";
pub static BODHI_PORT: &str = "BODHI_PORT";
pub static BODHI_LOGS: &str = "BODHI_LOGS";
pub static HF_HOME: &str = "HF_HOME";

pub static BODHI_AUTH_URL: &str = "BODHI_AUTH_URL";
pub static BODHI_AUTH_REALM: &str = "BODHI_AUTH_REALM";
pub static BODHI_APP_TYPE: &str = "BODHI_APP_TYPE";
pub static BODHI_LIBRARY_PATH: &str = "BODHI_LIBRARY_PATH";
pub static BODHI_LIBRARY_LOOKUP_PATH: &str = "BODHI_LIBRARY_LOOKUP_PATH";

pub static SETTINGS_YAML: &str = "settings.yaml";

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl= AppError)]
pub enum EnvServiceError {
  #[error("settings_error")]
  #[error_meta(error_type=ErrorType::InternalServer, status=500)]
  SettingsUpdateError(String),
  #[error(transparent)]
  IoError(#[from] IoError),
  #[error(transparent)]
  SerdeYamlError(#[from] SerdeYamlError),
}

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait EnvService: Send + Sync + std::fmt::Debug {
  fn env_type(&self) -> EnvType;

  fn is_production(&self) -> bool {
    self.env_type().is_production()
  }

  fn app_type(&self) -> AppType;

  fn is_native(&self) -> bool {
    self.app_type().is_native()
  }

  fn version(&self) -> String;

  fn frontend_url(&self) -> String;

  fn bodhi_home(&self) -> PathBuf;

  fn hf_home(&self) -> PathBuf;

  fn logs_dir(&self) -> PathBuf;

  fn scheme(&self) -> String;

  fn host(&self) -> String;

  fn port(&self) -> u16;

  fn db_path(&self) -> PathBuf;

  fn list(&self) -> HashMap<String, String>;

  fn auth_url(&self) -> String;

  fn auth_realm(&self) -> String;

  fn library_lookup_path(&self) -> String;

  fn library_path(&self) -> Option<String>;

  fn set_library_path(&mut self, library_path: String);

  fn hf_cache(&self) -> PathBuf {
    self.hf_home().join("hub")
  }

  fn server_url(&self) -> String {
    format!("{}://{}:{}", self.scheme(), self.host(), self.port())
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
}

#[derive(Debug, Clone)]
pub struct DefaultEnvService {
  bodhi_home: PathBuf,
  hf_home: PathBuf,
  logs_dir: PathBuf,
  env_type: EnvType,
  app_type: AppType,
  auth_url: String,
  auth_realm: String,
  settings: Settings,
  version: String,
  env_wrapper: Arc<dyn EnvWrapper>,
  host: Arc<Mutex<Option<String>>>,
  port: Arc<Mutex<Option<u16>>>,
}

impl DefaultEnvService {
  pub fn set_host(&self, host: &str) {
    *self.host.lock().unwrap() = Some(host.to_string());
  }

  pub fn set_port(&self, port: u16) {
    *self.port.lock().unwrap() = Some(port);
  }

  fn save_yaml<T: Serialize>(&self, path: &Path, value: &T) -> Result<(), EnvServiceError> {
    let contents = serde_yaml::to_string(value).map_err(SerdeYamlError::from)?;
    std::fs::write(path, contents).map_err(IoError::from)?;
    Ok(())
  }
}

impl EnvService for DefaultEnvService {
  fn env_type(&self) -> EnvType {
    self.env_type.clone()
  }

  fn app_type(&self) -> AppType {
    self.app_type.clone()
  }

  fn version(&self) -> String {
    self.version.clone()
  }

  fn bodhi_home(&self) -> PathBuf {
    self.bodhi_home.clone()
  }

  fn hf_home(&self) -> PathBuf {
    self.hf_home.clone()
  }

  fn logs_dir(&self) -> PathBuf {
    self.logs_dir.clone()
  }

  fn frontend_url(&self) -> String {
    match self.env_wrapper.var(BODHI_FRONTEND_URL) {
      Ok(value) => value,
      Err(_) => self.server_url(),
    }
  }

  fn scheme(&self) -> String {
    match self.env_wrapper.var(BODHI_SCHEME) {
      Ok(value) => value,
      Err(_) => DEFAULT_SCHEME.to_string(),
    }
  }

  fn host(&self) -> String {
    match self.host.lock().unwrap().as_ref() {
      Some(host) => host.clone(),
      None => match self.env_wrapper.var(BODHI_HOST) {
        Ok(value) => value,
        Err(_) => DEFAULT_HOST.to_string(),
      },
    }
  }

  fn port(&self) -> u16 {
    match self.port.lock().unwrap().as_ref() {
      Some(port) => *port,
      None => match self.env_wrapper.var(BODHI_PORT) {
        Ok(value) => match value.parse::<u16>() {
          Ok(port) => port,
          Err(_) => DEFAULT_PORT,
        },
        Err(_) => DEFAULT_PORT,
      },
    }
  }

  fn db_path(&self) -> PathBuf {
    self.bodhi_home().join(PROD_DB)
  }

  fn list(&self) -> HashMap<String, String> {
    let mut result = HashMap::<String, String>::new();
    result.insert(
      BODHI_HOME.to_string(),
      self.bodhi_home().display().to_string(),
    );
    result.insert(HF_HOME.to_string(), self.hf_home().display().to_string());
    result.insert(
      BODHI_LOGS.to_string(),
      self.logs_dir().display().to_string(),
    );
    result.insert(BODHI_HOST.to_string(), self.host());
    result.insert(BODHI_PORT.to_string(), self.port().to_string());
    result
  }

  fn auth_url(&self) -> String {
    self.auth_url.clone()
  }

  fn auth_realm(&self) -> String {
    self.auth_realm.clone()
  }

  fn library_lookup_path(&self) -> String {
    let lookup_path = self.env_wrapper.var(BODHI_LIBRARY_LOOKUP_PATH);
    match lookup_path {
      Ok(lookup_path) => lookup_path,
      Err(_) => std::env::current_dir()
        .unwrap_or_else(|err| {
          tracing::warn!("failed to get current directory. err: {err}");
          PathBuf::from(".")
        })
        .display()
        .to_string(),
    }
  }

  fn library_path(&self) -> Option<String> {
    self.settings.library_path.clone()
  }

  fn set_library_path(&mut self, library_path: String) {
    self.settings.library_path = Some(library_path);
    let result = self.save_yaml(&self.bodhi_home().join(SETTINGS_YAML), &self.settings);
    if result.is_err() {
      let api_error: ApiError = result.unwrap_err().into();
      tracing::warn!("failed to persist settings.yaml. err: {api_error}");
    }
  }
}

#[allow(clippy::too_many_arguments)]
impl DefaultEnvService {
  #[allow(clippy::new_without_default)]
  pub fn new(
    bodhi_home: PathBuf,
    hf_home: PathBuf,
    logs_dir: PathBuf,
    env_type: EnvType,
    app_type: AppType,
    auth_url: String,
    auth_realm: String,
    env_wrapper: Arc<dyn EnvWrapper>,
  ) -> Result<Self, DataServiceError> {
    if !bodhi_home.exists() {
      return Err(DataServiceError::BodhiHomeNotExists(
        bodhi_home.display().to_string(),
      ));
    }
    if !hf_home.exists() {
      return Err(DataServiceError::HfHomeNotExists(
        hf_home.display().to_string(),
      ));
    }
    if !logs_dir.exists() {
      return Err(DataServiceError::LogsDirNotExists(
        logs_dir.display().to_string(),
      ));
    }
    let settings_yaml = bodhi_home.join(SETTINGS_YAML);
    let settings: Settings = if settings_yaml.exists() {
      let contents = std::fs::read_to_string(&settings_yaml)?;
      serde_yaml::from_str(&contents)
        .map_err(|err| SerdeYamlWithPathError::new(err, settings_yaml.display().to_string()))?
    } else {
      Settings::app_default()
    };
    Ok(DefaultEnvService {
      bodhi_home,
      hf_home,
      logs_dir,
      env_type,
      app_type,
      auth_url,
      auth_realm,
      settings,
      version: env!("CARGO_PKG_VERSION").to_string(),
      env_wrapper,
      host: Arc::new(Mutex::new(None)),
      port: Arc::new(Mutex::new(None)),
    })
  }
}

#[cfg(test)]
mod test {
  use crate::InitService;
  use crate::{
    test_utils::EnvWrapperStub, DefaultEnvService, EnvService, MockEnvWrapper, BODHI_HOST,
  };
  use mockall::predicate::eq;
  use objs::{AppType, EnvType};
  use rstest::rstest;
  use std::collections::HashMap;
  use std::{env::VarError, sync::Arc};

  #[rstest]
  #[case(BODHI_HOST, "localhost", DefaultEnvService::host)]
  fn test_env_service_host_from_env_var(
    #[case] key: &str,
    #[case] value: String,
    #[case] func: for<'a> fn(&'a DefaultEnvService) -> String,
  ) -> anyhow::Result<()> {
    let env_wrapper = EnvWrapperStub::new(HashMap::new());
    let (bodhi_home, hf_home, logs_dir) = InitService::new(&env_wrapper, &EnvType::Development)
      .setup()
      .unwrap();
    let mut mock = MockEnvWrapper::default();
    let expected = value.clone();
    mock
      .expect_var()
      .with(eq(key.to_string()))
      .return_once(move |_| Ok(value));
    let env_service = DefaultEnvService::new(
      bodhi_home,
      hf_home,
      logs_dir,
      EnvType::Development,
      AppType::Container,
      "".to_string(),
      "".to_string(),
      Arc::new(mock),
    )
    .unwrap();
    let result = func(&env_service);
    assert_eq!(expected, result);
    Ok(())
  }

  #[rstest]
  fn test_env_service_host_from_fallback() -> anyhow::Result<()> {
    let env_wrapper = EnvWrapperStub::new(HashMap::new());
    let (bodhi_home, hf_home, logs_dir) = InitService::new(&env_wrapper, &EnvType::Development)
      .setup()
      .unwrap();
    let mut mock = MockEnvWrapper::default();
    mock
      .expect_var()
      .with(eq(BODHI_HOST))
      .return_once(move |_| Err(VarError::NotPresent));
    let env_service = DefaultEnvService::new(
      bodhi_home,
      hf_home,
      logs_dir,
      EnvType::Development,
      AppType::Container,
      "".to_string(),
      "".to_string(),
      Arc::new(mock),
    )
    .unwrap();
    let result = env_service.host();
    assert_eq!("localhost", result);
    Ok(())
  }
}
