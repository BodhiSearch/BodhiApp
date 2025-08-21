use crate::{asref_impl, EnvWrapper};
use objs::{
  impl_error_from, AppError, AppType, EnvType, ErrorType, IoError, LogLevel, SerdeYamlError,
  Setting, SettingInfo, SettingMetadata, SettingSource,
};
use serde_yaml::Value;
use std::{
  collections::HashMap,
  fs,
  path::{Path, PathBuf},
  sync::{Arc, RwLock},
};
use tracing::warn;

// System settings
pub const BODHI_HOME: &str = "BODHI_HOME";
pub const BODHI_ENV_TYPE: &str = "BODHI_ENV_TYPE";
pub const BODHI_APP_TYPE: &str = "BODHI_APP_TYPE";
pub const BODHI_VERSION: &str = "BODHI_VERSION";
pub const BODHI_AUTH_URL: &str = "BODHI_AUTH_URL";
pub const BODHI_AUTH_REALM: &str = "BODHI_AUTH_REALM";

// App Settings
pub const HF_HOME: &str = "HF_HOME";
pub const BODHI_LOGS: &str = "BODHI_LOGS";
pub const BODHI_LOG_LEVEL: &str = "BODHI_LOG_LEVEL";
pub const BODHI_LOG_STDOUT: &str = "BODHI_LOG_STDOUT";
pub const BODHI_SCHEME: &str = "BODHI_SCHEME";
pub const BODHI_HOST: &str = "BODHI_HOST";
pub const BODHI_PORT: &str = "BODHI_PORT";
pub const BODHI_EXEC_LOOKUP_PATH: &str = "BODHI_EXEC_LOOKUP_PATH";
pub const BODHI_EXEC_VARIANT: &str = "BODHI_EXEC_VARIANT";
pub const BODHI_EXEC_TARGET: &str = "BODHI_EXEC_TARGET";
pub const BODHI_EXEC_NAME: &str = "BODHI_EXEC_NAME";
pub const BODHI_EXEC_VARIANTS: &str = "BODHI_EXEC_VARIANTS";
pub const BODHI_KEEP_ALIVE_SECS: &str = "BODHI_KEEP_ALIVE_SECS";

// Public-facing host settings for Docker compatibility
pub const BODHI_PUBLIC_SCHEME: &str = "BODHI_PUBLIC_SCHEME";
pub const BODHI_PUBLIC_HOST: &str = "BODHI_PUBLIC_HOST";
pub const BODHI_PUBLIC_PORT: &str = "BODHI_PUBLIC_PORT";

pub const DEFAULT_SCHEME: &str = "http";
pub const DEFAULT_HOST: &str = "localhost";
pub const DEFAULT_PORT: u16 = 1135;
pub const DEFAULT_PORT_STR: &str = "1135";
pub const DEFAULT_LOG_LEVEL: &str = "warn";
pub const DEFAULT_LOG_STDOUT: bool = false;
pub const DEFAULT_KEEP_ALIVE_SECS: i64 = 300;

pub const SETTINGS_YAML: &str = "settings.yaml";

const PROD_DB: &str = "bodhi.sqlite";
const SESSION_DB: &str = "session.sqlite";

// TODO: remove the pub
pub const ALIASES_DIR: &str = "aliases";
pub const MODELS_YAML: &str = "models.yaml";

const LOGS_DIR: &str = "logs";

pub const BODHI_ENCRYPTION_KEY: &str = "BODHI_ENCRYPTION_KEY";
pub const BODHI_DEV_PROXY_UI: &str = "BODHI_DEV_PROXY_UI";

pub const SETTING_VARS: &[&str] = &[
  BODHI_LOGS,
  BODHI_LOG_LEVEL,
  BODHI_LOG_STDOUT,
  HF_HOME,
  BODHI_SCHEME,
  BODHI_HOST,
  BODHI_PORT,
  BODHI_PUBLIC_SCHEME,
  BODHI_PUBLIC_HOST,
  BODHI_PUBLIC_PORT,
  BODHI_EXEC_LOOKUP_PATH,
  BODHI_EXEC_VARIANT,
  BODHI_EXEC_TARGET,
  BODHI_EXEC_NAME,
  BODHI_EXEC_VARIANTS,
  BODHI_KEEP_ALIVE_SECS,
];

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait SettingsChangeListener: std::fmt::Debug + Send + Sync {
  fn on_change(
    &self,
    key: &str,
    prev_value: &Option<Value>,
    prev_source: &SettingSource,
    new_value: &Option<Value>,
    new_source: &SettingSource,
  );
}

impl SettingsChangeListener for Arc<dyn SettingsChangeListener> {
  fn on_change(
    &self,
    key: &str,
    prev_value: &Option<Value>,
    prev_source: &SettingSource,
    new_value: &Option<Value>,
    new_source: &SettingSource,
  ) {
    self
      .as_ref()
      .on_change(key, prev_value, prev_source, new_value, new_source)
  }
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum SettingServiceError {
  #[error(transparent)]
  Io(#[from] IoError),
  #[error(transparent)]
  SerdeYaml(#[from] SerdeYamlError),
  #[error("lock_error")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  LockError(String),
  #[error("invalid_source")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  InvalidSource,
}

impl_error_from!(::std::io::Error, SettingServiceError::Io, ::objs::IoError);
impl_error_from!(
  ::serde_yaml::Error,
  SettingServiceError::SerdeYaml,
  ::objs::SerdeYamlError
);

type Result<T> = std::result::Result<T, SettingServiceError>;

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait SettingService: std::fmt::Debug + Send + Sync {
  fn load(&self, path: &Path);

  fn load_default_env(&self) {
    let bodhi_home = self
      .get_setting(BODHI_HOME)
      .expect("BODHI_HOME should be set");
    let env_file = PathBuf::from(bodhi_home).join(".env");
    if env_file.exists() {
      self.load(&env_file);
    }
  }

  fn home_dir(&self) -> Option<PathBuf>;

  fn list(&self) -> Vec<SettingInfo>;

  fn get_default_value(&self, key: &str) -> Option<Value>;

  fn get_setting_metadata(&self, key: &str) -> SettingMetadata;

  fn get_env(&self, key: &str) -> Option<String>;

  fn get_setting(&self, key: &str) -> Option<String> {
    match self.get_setting_value(key) {
      Some(value) => match value {
        Value::String(s) => Some(s),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        Value::Null => None,
        _ => None,
      },
      None => None,
    }
  }

  fn get_setting_value(&self, key: &str) -> Option<Value> {
    self.get_setting_value_with_source(key).0
  }

  fn get_setting_value_with_source(&self, key: &str) -> (Option<Value>, SettingSource);

  fn set_setting_with_source(&self, key: &str, value: &Value, source: SettingSource);

  fn set_setting(&self, key: &str, value: &str) {
    self.set_setting_value(key, &Value::String(value.to_owned()))
  }

  fn set_setting_value(&self, key: &str, value: &Value) {
    self.set_setting_with_source(key, value, SettingSource::SettingsFile)
  }

  fn set_default(&self, key: &str, value: &Value) {
    self.set_setting_with_source(key, value, SettingSource::Default)
  }

  fn delete_setting(&self, key: &str) -> Result<()>;

  fn add_listener(&self, listener: Arc<dyn SettingsChangeListener>);

  fn bodhi_home(&self) -> PathBuf {
    let bodhi_home = self
      .get_setting(BODHI_HOME)
      .expect("BODHI_HOME should be set");
    PathBuf::from(bodhi_home)
  }

  fn models_yaml(&self) -> PathBuf {
    self.bodhi_home().join(MODELS_YAML)
  }

  fn env_type(&self) -> EnvType {
    self
      .get_setting(BODHI_ENV_TYPE)
      .expect("BODHI_ENV_TYPE should be set")
      .parse()
      .unwrap()
  }

  fn app_type(&self) -> AppType {
    self
      .get_setting(BODHI_APP_TYPE)
      .expect("BODHI_APP_TYPE should be set")
      .parse()
      .unwrap()
  }

  fn version(&self) -> String {
    self
      .get_setting(BODHI_VERSION)
      .expect("BODHI_VERSION should be set")
      .to_string()
  }

  fn auth_url(&self) -> String {
    self
      .get_setting(BODHI_AUTH_URL)
      .expect("BODHI_AUTH_URL should be set")
      .to_string()
  }

  fn auth_realm(&self) -> String {
    self
      .get_setting(BODHI_AUTH_REALM)
      .expect("BODHI_AUTH_REALM should be set")
      .to_string()
  }

  fn is_production(&self) -> bool {
    self.env_type().is_production()
  }

  fn is_native(&self) -> bool {
    self.app_type().is_native()
  }

  fn hf_home(&self) -> PathBuf {
    PathBuf::from(self.get_setting(HF_HOME).expect("HF_HOME should be set"))
  }

  fn logs_dir(&self) -> PathBuf {
    PathBuf::from(
      self
        .get_setting(BODHI_LOGS)
        .expect("BODHI_LOGS should be set"),
    )
  }

  fn scheme(&self) -> String {
    self
      .get_setting(BODHI_SCHEME)
      .expect("BODHI_SCHEME should be set")
  }

  fn host(&self) -> String {
    self
      .get_setting(BODHI_HOST)
      .expect("BODHI_HOST should be set")
  }

  fn port(&self) -> u16 {
    match self
      .get_setting_value(BODHI_PORT)
      .expect("BODHI_PORT should be set")
    {
      serde_yaml::Value::Number(n) => n.as_u64().expect("BODHI_PORT should be a number") as u16,
      serde_yaml::Value::String(s) => match s.parse() {
        Ok(port) => port,
        Err(_) => {
          warn!("BODHI_PORT is not a number: {}, falling back to default", s);
          DEFAULT_PORT
        }
      },
      _ => DEFAULT_PORT,
    }
  }

  fn frontend_default_url(&self) -> String {
    format!("{}/ui/chat", self.public_server_url())
  }

  fn app_db_path(&self) -> PathBuf {
    self.bodhi_home().join(PROD_DB)
  }

  fn session_db_path(&self) -> PathBuf {
    self.bodhi_home().join(SESSION_DB)
  }

  fn log_level(&self) -> LogLevel {
    let log_level = self
      .get_setting(BODHI_LOG_LEVEL)
      .expect("BODHI_LOG_LEVEL should be set");
    LogLevel::try_from(log_level.as_str()).unwrap_or(LogLevel::Warn)
  }

  fn exec_lookup_path(&self) -> String {
    self
      .get_setting(BODHI_EXEC_LOOKUP_PATH)
      .expect("BODHI_EXEC_LOOKUP_PATH should be set")
  }

  fn exec_variant(&self) -> String {
    self
      .get_setting(BODHI_EXEC_VARIANT)
      .expect("BODHI_EXEC_VARIANT should be set")
  }

  fn exec_target(&self) -> String {
    self
      .get_setting(BODHI_EXEC_TARGET)
      .expect("BODHI_EXEC_TARGET should be set")
  }

  fn exec_name(&self) -> String {
    self
      .get_setting(BODHI_EXEC_NAME)
      .expect("BODHI_EXEC_NAME should be set")
  }

  fn exec_variants(&self) -> Vec<String> {
    self
      .get_setting(BODHI_EXEC_VARIANTS)
      .expect("BODHI_EXEC_VARIANTS should be set")
      .split(',')
      .map(|s| s.trim().to_string())
      .collect()
  }

  fn exec_path_from(&self) -> PathBuf {
    let lookup_path = PathBuf::from(self.exec_lookup_path());
    let target = self.exec_target();
    let variant = self.exec_variant();
    let exec_name = self.exec_name();
    lookup_path.join(target).join(variant).join(exec_name)
  }

  fn public_scheme(&self) -> String {
    self
      .get_setting(BODHI_PUBLIC_SCHEME)
      .unwrap_or_else(|| self.scheme())
  }

  fn public_host(&self) -> String {
    self
      .get_setting(BODHI_PUBLIC_HOST)
      .unwrap_or_else(|| self.host())
  }

  fn get_public_host_explicit(&self) -> Option<String> {
    let (value, source) = self.get_setting_value_with_source(BODHI_PUBLIC_HOST);
    match source {
      SettingSource::Default => None,
      _ => value.and_then(|v| match v {
        Value::String(s) => Some(s),
        _ => None,
      }),
    }
  }

  fn public_port(&self) -> u16 {
    match self.get_setting_value(BODHI_PUBLIC_PORT) {
      Some(serde_yaml::Value::Number(n)) => n.as_u64().unwrap_or(self.port() as u64) as u16,
      Some(serde_yaml::Value::String(s)) => s.parse().unwrap_or(self.port()),
      _ => self.port(),
    }
  }

  fn public_server_url(&self) -> String {
    let scheme = self.public_scheme();
    let host = self.public_host();
    let port = self.public_port();
    match (scheme.as_str(), port) {
      ("http", 80) | ("https", 443) => format!("{}://{}", scheme, host),
      _ => format!("{}://{}:{}", scheme, host, port),
    }
  }

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

  fn auth_issuer(&self) -> String {
    format!("{}/realms/{}", self.auth_url(), self.auth_realm())
  }

  fn token_url(&self) -> String {
    format!(
      "{}/realms/{}/protocol/openid-connect/token",
      self.auth_url(),
      self.auth_realm()
    )
  }

  fn login_callback_url(&self) -> String {
    format!("{}/ui/auth/callback", self.public_server_url())
  }

  fn secrets_path(&self) -> PathBuf {
    self.bodhi_home().join("secrets.yaml")
  }

  fn encryption_key(&self) -> Option<String> {
    SettingService::get_env(self, BODHI_ENCRYPTION_KEY)
  }

  #[cfg(not(debug_assertions))]
  fn get_dev_env(&self, _key: &str) -> Option<String> {
    None
  }

  #[cfg(debug_assertions)]
  fn get_dev_env(&self, key: &str) -> Option<String> {
    SettingService::get_env(self, key)
  }

  fn keep_alive(&self) -> i64 {
    self
      .get_setting_value(BODHI_KEEP_ALIVE_SECS)
      .expect("BODHI_KEEP_ALIVE_SECS should be set")
      .as_i64()
      .expect("BODHI_KEEP_ALIVE_SECS should be a number")
  }
}

#[derive(Debug)]
pub struct DefaultSettingService {
  env_wrapper: Arc<dyn EnvWrapper>,
  path: PathBuf,
  system_settings: Vec<Setting>,
  settings_lock: RwLock<()>,
  defaults: RwLock<HashMap<String, Value>>,
  listeners: RwLock<Vec<Arc<dyn SettingsChangeListener>>>,
  cmd_lines: RwLock<HashMap<String, Value>>,
}

impl DefaultSettingService {
  fn new(
    env_wrapper: Arc<dyn EnvWrapper>,
    path: PathBuf,
    system_settings: Vec<Setting>,
  ) -> DefaultSettingService {
    Self {
      env_wrapper,
      path,
      system_settings,
      settings_lock: RwLock::new(()),
      defaults: RwLock::new(HashMap::new()),
      listeners: RwLock::new(Vec::new()),
      cmd_lines: RwLock::new(HashMap::new()),
    }
  }

  pub fn new_with_defaults(
    env_wrapper: Arc<dyn EnvWrapper>,
    bodhi_home: Setting,
    mut system_settings: Vec<Setting>,
    settings_file: PathBuf,
  ) -> Self {
    system_settings.push(bodhi_home.clone());
    let service = Self::new(env_wrapper, settings_file, system_settings);
    service.init_defaults(bodhi_home);
    service
  }

  fn init_defaults(&self, bodhi_home: Setting) {
    self.with_defaults_write_lock(|defaults| {
      if bodhi_home.source == SettingSource::Default {
        defaults.insert(BODHI_HOME.to_string(), bodhi_home.value.clone());
        defaults.insert(
          BODHI_LOGS.to_string(),
          Value::String(
            PathBuf::from(bodhi_home.value.as_str().unwrap())
              .join(LOGS_DIR)
              .display()
              .to_string(),
          ),
        );
      } else if let Some(home_dir) = self.home_dir() {
        let default_bodhi_home = home_dir.join(".cache").join("bodhi");
        defaults.insert(
          BODHI_HOME.to_string(),
          Value::String(default_bodhi_home.display().to_string()),
        );
        defaults.insert(
          BODHI_LOGS.to_string(),
          Value::String(default_bodhi_home.join("logs").display().to_string()),
        );
      }
      if let Some(home_dir) = self.home_dir() {
        defaults.insert(
          HF_HOME.to_string(),
          Value::String(
            home_dir
              .join(".cache")
              .join("huggingface")
              .display()
              .to_string(),
          ),
        );
      }
      defaults.insert(
        BODHI_SCHEME.to_string(),
        Value::String(DEFAULT_SCHEME.to_string()),
      );
      defaults.insert(
        BODHI_HOST.to_string(),
        Value::String(DEFAULT_HOST.to_string()),
      );
      defaults.insert(BODHI_PORT.to_string(), Value::Number(DEFAULT_PORT.into()));
      defaults.insert(
        BODHI_LOG_LEVEL.to_string(),
        Value::String(DEFAULT_LOG_LEVEL.to_string()),
      );
      defaults.insert(
        BODHI_LOG_STDOUT.to_string(),
        Value::Bool(DEFAULT_LOG_STDOUT),
      );
      defaults.insert(
        BODHI_EXEC_VARIANT.to_string(),
        Value::String(llama_server_proc::DEFAULT_VARIANT.to_string()),
      );
      defaults.insert(
        BODHI_EXEC_TARGET.to_string(),
        Value::String(llama_server_proc::BUILD_TARGET.to_string()),
      );
      defaults.insert(
        BODHI_EXEC_NAME.to_string(),
        Value::String(llama_server_proc::EXEC_NAME.to_string()),
      );
      defaults.insert(
        BODHI_EXEC_VARIANTS.to_string(),
        Value::String(
          llama_server_proc::BUILD_VARIANTS
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(","),
        ),
      );
      defaults.insert(
        BODHI_KEEP_ALIVE_SECS.to_string(),
        Value::Number(DEFAULT_KEEP_ALIVE_SECS.into()),
      );
    });
  }

  pub fn with_settings_read_lock<F, R>(&self, f: F) -> R
  where
    F: FnOnce(&serde_yaml::Mapping) -> R,
  {
    let _guard = self.settings_lock.read().unwrap();
    if !self.path.exists() {
      return f(&serde_yaml::Mapping::new());
    }
    let contents = fs::read_to_string(&self.path).unwrap_or_else(|_| String::new());
    let settings: serde_yaml::Mapping =
      serde_yaml::from_str(&contents).unwrap_or_else(|_| serde_yaml::Mapping::new());
    f(&settings)
  }

  pub fn with_settings_write_lock<F>(&self, f: F)
  where
    F: FnOnce(&mut serde_yaml::Mapping),
  {
    let _guard = self.settings_lock.write().unwrap();
    let mut settings = if !self.path.exists() {
      serde_yaml::Mapping::new()
    } else {
      let contents = fs::read_to_string(&self.path).unwrap_or_else(|_| String::new());
      serde_yaml::from_str(&contents).unwrap_or_else(|_| serde_yaml::Mapping::new())
    };
    f(&mut settings);
    let contents = serde_yaml::to_string(&settings).unwrap();
    fs::write(&self.path, contents).unwrap();
  }

  pub fn with_defaults_read_lock<F, R>(&self, f: F) -> R
  where
    F: FnOnce(&HashMap<String, Value>) -> R,
  {
    let defaults = self.defaults.read().unwrap();
    f(&defaults)
  }

  pub fn with_defaults_write_lock<F>(&self, f: F)
  where
    F: FnOnce(&mut HashMap<String, Value>),
  {
    let mut defaults = self.defaults.write().unwrap();
    f(&mut defaults);
  }

  pub fn with_cmd_lines_read_lock<F, R>(&self, f: F) -> R
  where
    F: FnOnce(&HashMap<String, Value>) -> R,
  {
    let cmd_lines = self.cmd_lines.read().unwrap();
    f(&cmd_lines)
  }

  pub fn with_cmd_lines_write_lock<F>(&self, f: F)
  where
    F: FnOnce(&mut HashMap<String, Value>),
  {
    let mut cmd_lines = self.cmd_lines.write().unwrap();
    f(&mut cmd_lines);
  }

  fn notify_listeners(
    &self,
    key: &str,
    prev_value: &Option<Value>,
    prev_source: &SettingSource,
    new_value: &Option<Value>,
    new_source: &SettingSource,
  ) {
    let lock = self.listeners.read().unwrap();
    for listener in lock.iter() {
      listener.on_change(key, prev_value, prev_source, new_value, new_source);
    }
  }
}

impl SettingService for DefaultSettingService {
  fn load(&self, path: &Path) {
    self.env_wrapper.load(path);
  }

  fn home_dir(&self) -> Option<PathBuf> {
    self.env_wrapper.home_dir()
  }

  fn get_env(&self, key: &str) -> Option<String> {
    self.env_wrapper.var(key).ok()
  }

  fn set_setting_with_source(&self, key: &str, value: &Value, source: SettingSource) {
    let (prev_value, prev_source) = self.get_setting_value_with_source(key);
    match source {
      SettingSource::CommandLine => {
        self.with_cmd_lines_write_lock(|cmd_lines| {
          cmd_lines.insert(key.to_string(), value.clone());
        });
      }
      SettingSource::Environment => {
        tracing::error!("SettingSource::Environment is not supported for override");
      }
      SettingSource::SettingsFile => {
        self.with_settings_write_lock(|settings| {
          settings.insert(key.into(), value.clone());
        });
        let (cur_value, cur_source) = self.get_setting_value_with_source(key);
        self.notify_listeners(key, &prev_value, &prev_source, &cur_value, &cur_source);
      }
      SettingSource::Default => {
        self.with_defaults_write_lock(|defaults| {
          defaults.insert(key.to_string(), value.clone());
        });
      }
      SettingSource::System => {
        tracing::error!("SettingSource::System is not supported for override");
      }
    }
  }

  fn delete_setting(&self, key: &str) -> Result<()> {
    let (prev_value, prev_source) = self.get_setting_value_with_source(key);
    self.with_settings_write_lock(|settings| {
      settings.remove(key);
    });
    let (cur_value, cur_source) = self.get_setting_value_with_source(key);
    self.notify_listeners(key, &prev_value, &prev_source, &cur_value, &cur_source);
    Ok(())
  }

  fn get_setting_value_with_source(&self, key: &str) -> (Option<Value>, SettingSource) {
    if let Some(setting) = self.system_settings.iter().find(|s| s.key == key) {
      return (Some(setting.value.clone()), SettingSource::System);
    }

    let metadata = self.get_setting_metadata(key);
    let result = self.with_cmd_lines_read_lock(|cmd_lines| cmd_lines.get(key).cloned());
    if let Some(value) = result {
      return (Some(value), SettingSource::CommandLine);
    }
    if let Ok(value) = self.env_wrapper.var(key) {
      let value = metadata.parse(Value::String(value));
      return (Some(value), SettingSource::Environment);
    }
    let result = self.with_settings_read_lock(|settings| settings.get(key).cloned());
    result
      .map(|value| (Some(metadata.parse(value)), SettingSource::SettingsFile))
      .unwrap_or((self.get_default_value(key), SettingSource::Default))
  }

  fn list(&self) -> Vec<SettingInfo> {
    let mut system_settings = self
      .system_settings
      .iter()
      .map(|s| SettingInfo {
        key: s.key.clone(),
        current_value: s.value.clone(),
        default_value: s.value.clone(),
        source: s.source.clone(),
        metadata: self.get_setting_metadata(&s.key),
      })
      .collect::<Vec<SettingInfo>>();
    let app_settings = SETTING_VARS
      .iter()
      .map(|key| {
        let (current_value, source) = self.get_setting_value_with_source(key);
        let metadata = self.get_setting_metadata(key);
        let current_value = current_value.map(|value| metadata.parse(value));

        SettingInfo {
          key: key.to_string(),
          current_value: current_value.unwrap_or(Value::Null),
          default_value: self.get_default_value(key).unwrap_or(Value::Null),
          source,
          metadata,
        }
      })
      .collect::<Vec<SettingInfo>>();
    system_settings.extend(app_settings);
    system_settings
  }

  fn get_setting_metadata(&self, key: &str) -> SettingMetadata {
    match key {
      BODHI_PORT | BODHI_PUBLIC_PORT => SettingMetadata::Number { min: 1, max: 65535 },
      BODHI_LOG_LEVEL => SettingMetadata::option(
        ["error", "warn", "info", "debug", "trace"]
          .iter()
          .map(|s| s.to_string())
          .collect(),
      ),
      BODHI_LOG_STDOUT => SettingMetadata::Boolean,
      BODHI_EXEC_VARIANT => {
        let variants = self.exec_variants();
        SettingMetadata::option(variants)
      }
      BODHI_EXEC_TARGET => SettingMetadata::String,
      BODHI_EXEC_NAME => SettingMetadata::String,
      BODHI_EXEC_VARIANTS => SettingMetadata::String,
      BODHI_KEEP_ALIVE_SECS => SettingMetadata::Number {
        min: 300,
        max: 86400,
      },
      _ => SettingMetadata::String,
    }
  }

  fn get_default_value(&self, key: &str) -> Option<Value> {
    self.with_defaults_read_lock(|defaults| match key {
      BODHI_HOME => match defaults.get(BODHI_HOME).cloned() {
        Some(value) => Some(value),
        None => {
          let result = self
            .home_dir()
            .map(|home_dir| home_dir.join(".cache").join("bodhi"))
            .map(|path| serde_yaml::Value::String(path.display().to_string()));
          result
        }
      },
      BODHI_PUBLIC_HOST => self.get_setting_value(BODHI_HOST),
      BODHI_PUBLIC_SCHEME => self.get_setting_value(BODHI_SCHEME),
      BODHI_PUBLIC_PORT => self.get_setting_value(BODHI_PORT),
      _ => defaults.get(key).cloned(),
    })
  }

  fn add_listener(&self, listener: Arc<dyn SettingsChangeListener>) {
    let mut listeners = self.listeners.write().unwrap();
    if !listeners
      .iter()
      .any(|existing| std::ptr::eq(existing.as_ref(), listener.as_ref()))
    {
      listeners.push(listener);
    }
  }
}

asref_impl!(SettingService, DefaultSettingService);

#[cfg(test)]
mod tests {
  use crate::{
    test_utils::{bodhi_home_setting, EnvWrapperStub},
    DefaultSettingService, MockEnvWrapper, MockSettingsChangeListener, SettingService,
    BODHI_EXEC_VARIANT, BODHI_HOME, BODHI_HOST, BODHI_LOGS, BODHI_LOG_LEVEL, BODHI_LOG_STDOUT,
    BODHI_PORT, BODHI_PUBLIC_HOST, BODHI_PUBLIC_PORT, BODHI_PUBLIC_SCHEME, BODHI_SCHEME,
    DEFAULT_HOST, DEFAULT_LOG_LEVEL, DEFAULT_LOG_STDOUT, DEFAULT_PORT, DEFAULT_SCHEME, HF_HOME,
  };
  use anyhow_trace::anyhow_trace;
  use mockall::predicate::eq;
  use objs::{test_utils::temp_dir, Setting, SettingInfo, SettingMetadata, SettingSource};
  use pretty_assertions::assert_eq;
  use rstest::rstest;
  use serde::{Deserialize, Serialize};
  use serde_yaml::Value;
  use std::{collections::HashMap, fs, fs::read_to_string, sync::Arc};
  use tempfile::TempDir;
  #[derive(Debug, PartialEq, Serialize, Deserialize)]
  struct TestConfig {
    name: String,
    value: i32,
  }

  #[rstest]
  fn test_setting_service_init_with_defaults(temp_dir: TempDir) -> anyhow::Result<()> {
    let path = temp_dir.path().join("settings.yaml");
    let home_dir = temp_dir.path().join("home");
    let env_wrapper =
      EnvWrapperStub::new(maplit::hashmap! {"HOME".to_string() => home_dir.display().to_string()});
    let service = DefaultSettingService::new_with_defaults(
      Arc::new(env_wrapper),
      bodhi_home_setting(temp_dir.path(), SettingSource::Environment),
      vec![],
      path.clone(),
    );
    for (key, expected) in [
      (
        BODHI_HOME,
        home_dir.join(".cache").join("bodhi").display().to_string(),
      ),
      (
        BODHI_LOGS,
        home_dir
          .join(".cache")
          .join("bodhi")
          .join("logs")
          .display()
          .to_string(),
      ),
      (
        HF_HOME,
        home_dir
          .join(".cache")
          .join("huggingface")
          .display()
          .to_string(),
      ),
      (BODHI_SCHEME, DEFAULT_SCHEME.to_string()),
      (BODHI_HOST, DEFAULT_HOST.to_string()),
      (BODHI_LOG_LEVEL, DEFAULT_LOG_LEVEL.to_string()),
      (
        BODHI_EXEC_VARIANT,
        llama_server_proc::DEFAULT_VARIANT.to_string(),
      ),
    ] {
      assert_eq!(
        expected,
        service.get_default_value(key).unwrap().as_str().unwrap()
      );
    }
    assert_eq!(
      DEFAULT_PORT as i64,
      service
        .get_default_value(BODHI_PORT)
        .unwrap()
        .as_i64()
        .unwrap()
    );
    assert_eq!(
      DEFAULT_LOG_STDOUT,
      service
        .get_default_value(BODHI_LOG_STDOUT)
        .unwrap()
        .as_bool()
        .unwrap()
    );
    Ok(())
  }

  #[rstest]
  fn test_setting_service_read_from_file_if_env_not_set(temp_dir: TempDir) -> anyhow::Result<()> {
    let path = temp_dir.path().join("settings.yaml");
    std::fs::write(
      &path,
      r#"
TEST_KEY: file_value
TEST_NUMBER: 8080
"#,
    )?;
    let env_wrapper = EnvWrapperStub::new(HashMap::new());
    let service = DefaultSettingService::new_with_defaults(
      Arc::new(env_wrapper),
      bodhi_home_setting(temp_dir.path(), SettingSource::Environment),
      vec![],
      path.clone(),
    );
    assert_eq!(
      Some("file_value".to_string()),
      service.get_setting("TEST_KEY"),
    );
    assert_eq!(Some("8080".to_string()), service.get_setting("TEST_NUMBER"),);
    Ok(())
  }

  #[rstest]
  fn test_setting_service_read_from_default_if_not_set_in_env_or_file(
    temp_dir: TempDir,
  ) -> anyhow::Result<()> {
    let path = temp_dir.path().join("settings.yaml");
    std::fs::write(
      &path,
      r#"
SOME_OTHER_KEY: value
"#,
    )?;
    let env_wrapper = EnvWrapperStub::new(HashMap::new());
    let service = DefaultSettingService::new(Arc::new(env_wrapper), path.clone(), vec![]);
    service.set_default("SOME_KEY", &Value::String("default_value".to_string()));
    assert_eq!(
      Some("default_value".to_string()),
      service.get_setting("SOME_KEY"),
    );
    Ok(())
  }

  #[rstest]
  fn test_setting_service_read_from_env_if_set(temp_dir: TempDir) -> anyhow::Result<()> {
    let path = temp_dir.path().join("settings.yaml");
    std::fs::write(&path, "TEST_KEY: file_value")?;
    let mut mock_env = MockEnvWrapper::new();
    mock_env
      .expect_var()
      .with(eq("TEST_KEY"))
      .times(1)
      .return_const(Ok("env_value".to_string()));
    let service = DefaultSettingService::new(Arc::new(mock_env), path, vec![]);
    assert_eq!(
      Some("env_value".to_string()),
      service.get_setting("TEST_KEY")
    );
    Ok(())
  }

  #[rstest]
  fn test_setting_service_read_from_cmd_line_if_set(temp_dir: TempDir) -> anyhow::Result<()> {
    let path = temp_dir.path().join("settings.yaml");
    std::fs::write(&path, "TEST_KEY: file_value")?;
    let mut mock_env = MockEnvWrapper::new();
    mock_env
      .expect_var()
      .with(eq("TEST_KEY"))
      .return_const(Ok("env_value".to_string()));
    let service = DefaultSettingService::new(Arc::new(mock_env), path, vec![]);
    service.set_setting_with_source(
      "TEST_KEY",
      &serde_yaml::Value::String("cmdline-value".to_string()),
      SettingSource::CommandLine,
    );
    assert_eq!(
      Some("cmdline-value".to_string()),
      service.get_setting("TEST_KEY")
    );
    Ok(())
  }

  #[rstest]
  fn test_setting_service_change_notification_when_overriding_settings(
    temp_dir: TempDir,
  ) -> anyhow::Result<()> {
    let mut mock_env = MockEnvWrapper::new();
    mock_env
      .expect_var()
      .with(eq("TEST_KEY"))
      .return_const(Err(std::env::VarError::NotPresent));
    let path = temp_dir.path().join("settings.yaml");
    std::fs::write(&path, "TEST_KEY: test_value")?;
    let service = DefaultSettingService::new(Arc::new(mock_env), path.clone(), vec![]);
    let mut mock_listener = MockSettingsChangeListener::default();
    mock_listener
      .expect_on_change()
      .with(
        eq("TEST_KEY"),
        eq(Some(Value::String("test_value".to_string()))),
        eq(SettingSource::SettingsFile),
        eq(Some(Value::String("new_value".to_string()))),
        eq(SettingSource::SettingsFile),
      )
      .times(1)
      .return_once(|_, _, _, _, _| ());
    service.add_listener(Arc::new(mock_listener));
    service.set_setting("TEST_KEY", "new_value");
    Ok(())
  }

  #[rstest]
  fn test_setting_service_change_notification_when_deleting_settings(
    temp_dir: TempDir,
  ) -> anyhow::Result<()> {
    let mut mock_env = MockEnvWrapper::default();
    mock_env
      .expect_var()
      .with(eq("TEST_KEY"))
      .return_const(Err(std::env::VarError::NotPresent));
    let path = temp_dir.path().join("settings.yaml");
    std::fs::write(&path, "TEST_KEY: test_value")?;
    let service = DefaultSettingService::new(Arc::new(mock_env), path.clone(), vec![]);
    service.set_default("TEST_KEY", &Value::String("default_value".to_string()));
    let mut mock_listener = MockSettingsChangeListener::default();
    mock_listener
      .expect_on_change()
      .with(
        eq("TEST_KEY"),
        eq(Some(Value::String("test_value".to_string()))),
        eq(SettingSource::SettingsFile),
        eq(Some(Value::String("default_value".to_string()))),
        eq(SettingSource::Default),
      )
      .times(1)
      .return_once(|_, _, _, _, _| ());
    service.add_listener(Arc::new(mock_listener));
    service.delete_setting("TEST_KEY")?;
    Ok(())
  }

  #[rstest]
  fn test_setting_service_change_notification_when_env_set(
    temp_dir: TempDir,
  ) -> anyhow::Result<()> {
    let mut mock_env = MockEnvWrapper::new();
    mock_env
      .expect_var()
      .with(eq("TEST_KEY"))
      .return_const(Ok("env_value".to_string()));
    let path = temp_dir.path().join("settings.yaml");
    std::fs::write(&path, "TEST_KEY: test_value")?;
    let service = DefaultSettingService::new(Arc::new(mock_env), path.clone(), vec![]);
    let mut mock_listener = MockSettingsChangeListener::default();
    mock_listener
      .expect_on_change()
      .with(
        eq("TEST_KEY"),
        eq(Some(Value::String("env_value".to_string()))),
        eq(SettingSource::Environment),
        eq(Some(Value::String("env_value".to_string()))),
        eq(SettingSource::Environment),
      )
      .times(1)
      .return_once(|_, _, _, _, _| ());
    service.add_listener(Arc::new(mock_listener));
    service.set_setting("TEST_KEY", "new_value");
    Ok(())
  }

  #[rstest]
  fn test_setting_service_change_notification_when_setting_defaults(
    temp_dir: TempDir,
  ) -> anyhow::Result<()> {
    let mut mock_env = MockEnvWrapper::new();
    mock_env
      .expect_var()
      .with(eq("TEST_KEY"))
      .times(1)
      .return_const(Err(std::env::VarError::NotPresent));
    let path = temp_dir.path().join("settings.yaml");
    std::fs::write(&path, "TEST_KEY: test_value")?;
    let service = DefaultSettingService::new(Arc::new(mock_env), path.clone(), vec![]);
    let mut mock_listener = MockSettingsChangeListener::default();
    mock_listener.expect_on_change().never();
    service.add_listener(Arc::new(mock_listener));
    service.set_default("TEST_KEY", &Value::String("default_value".to_string()));
    Ok(())
  }

  #[rstest]
  fn test_setting_service_persistence(temp_dir: TempDir) -> anyhow::Result<()> {
    let path = temp_dir.path().join("settings.yaml");
    let mut mock_env = MockEnvWrapper::new();
    mock_env
      .expect_var()
      .with(eq("TEST_KEY"))
      .return_const(Err(std::env::VarError::NotPresent));

    {
      let service = DefaultSettingService::new(Arc::new(mock_env), path.clone(), vec![]);
      service.set_setting("TEST_KEY", "test_value");
    }
    let contents = read_to_string(&path)?;
    assert_eq!("TEST_KEY: test_value\n", contents);

    let mut mock_env = MockEnvWrapper::new();
    mock_env
      .expect_var()
      .with(eq("TEST_KEY"))
      .return_const(Err(std::env::VarError::NotPresent));

    {
      let service = DefaultSettingService::new(Arc::new(mock_env), path.clone(), vec![]);
      assert_eq!(
        Some("test_value".to_string()),
        service.get_setting("TEST_KEY"),
      );
    }
    Ok(())
  }

  #[rstest]
  fn test_settings_service_delete(temp_dir: TempDir) -> anyhow::Result<()> {
    let path = temp_dir.path().join("settings.yaml");
    let mut mock_env = MockEnvWrapper::new();
    mock_env
      .expect_var()
      .with(eq("TEST_KEY"))
      .return_const(Err(std::env::VarError::NotPresent));
    mock_env
      .expect_var()
      .with(eq("NON_EXISTENT_KEY"))
      .return_const(Err(std::env::VarError::NotPresent));

    let service = DefaultSettingService::new(Arc::new(mock_env), path.clone(), vec![]);

    service.set_setting("TEST_KEY", "test_value");
    assert_eq!(
      Some("test_value".to_string()),
      service.get_setting("TEST_KEY")
    );

    service.delete_setting("TEST_KEY")?;
    assert_eq!(None, service.get_setting("TEST_KEY"));
    let contents = std::fs::read_to_string(path)?;
    assert_eq!("{}\n", contents);

    // Delete non-existent key should still succeed
    service.delete_setting("NON_EXISTENT_KEY")?;
    Ok(())
  }

  #[rstest]
  fn test_get_setting_with_app_settings_precedence(temp_dir: TempDir) -> anyhow::Result<()> {
    let path = temp_dir.path().join("settings.yaml");
    let home_dir = Some(temp_dir.path().to_path_buf());
    let mut mock_env = MockEnvWrapper::new();
    mock_env.expect_var().with(eq("TEST_KEY")).never();
    mock_env.expect_home_dir().return_const(home_dir);

    let app_settings = vec![Setting {
      key: "TEST_KEY".to_string(),
      value: Value::String("app_value".to_string()),
      source: SettingSource::Default,
      metadata: objs::SettingMetadata::String,
    }];

    let service = DefaultSettingService::new_with_defaults(
      Arc::new(mock_env),
      bodhi_home_setting(temp_dir.path(), SettingSource::Default),
      app_settings,
      path.clone(),
    );

    // Check that the value from app_settings is returned
    assert_eq!(
      Some("app_value".to_string()),
      service.get_setting("TEST_KEY"),
    );

    // Ensure that the original logic is still intact
    std::fs::write(&path, "TEST_KEY: file_value")?;
    assert_eq!(
      Some("app_value".to_string()),
      service.get_setting("TEST_KEY"),
    );

    Ok(())
  }

  #[anyhow_trace]
  #[rstest]
  fn test_setting_service_list(
    temp_dir: TempDir,
    #[from(temp_dir)] bodhi_home: TempDir,
  ) -> anyhow::Result<()> {
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
BODHI_EXEC_VARIANT: metal
BODHI_EXEC_LOOKUP_PATH: /test/exec/lookup
"#,
    )?;

    let setting_service = DefaultSettingService::new_with_defaults(
      Arc::new(env_wrapper),
      bodhi_home_setting(bodhi_home.path(), SettingSource::Default),
      vec![],
      settings_file.clone(),
    );
    let bodhi_home = bodhi_home.path().to_path_buf();
    // WHEN
    let settings = setting_service
      .list()
      .into_iter()
      .map(|setting| (setting.key.clone(), setting))
      .collect::<HashMap<String, SettingInfo>>();

    // THEN
    // System settings
    let expected_bodhi_home = SettingInfo {
      key: BODHI_HOME.to_string(),
      current_value: serde_yaml::Value::String(bodhi_home.display().to_string()),
      default_value: serde_yaml::Value::String(bodhi_home.display().to_string()),
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
      metadata: SettingMetadata::option(
        ["error", "warn", "info", "debug", "trace"]
          .iter()
          .map(|s| s.to_string())
          .collect(),
      ),
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
      metadata: SettingMetadata::Number { min: 1, max: 65535 },
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

  #[rstest]
  fn test_public_settings_fallback_behavior(temp_dir: TempDir) -> anyhow::Result<()> {
    let path = temp_dir.path().join("settings.yaml");
    let env_wrapper = EnvWrapperStub::new(HashMap::new());
    let service = DefaultSettingService::new_with_defaults(
      Arc::new(env_wrapper),
      bodhi_home_setting(temp_dir.path(), SettingSource::Environment),
      vec![],
      path.clone(),
    );

    // Test fallback behavior - public settings should use regular settings when not set
    assert_eq!(
      service.public_server_url(),
      "http://localhost:1135" // Uses defaults from regular settings
    );

    // Test that public setting defaults fallback to regular settings
    assert_eq!(
      service
        .get_default_value(BODHI_PUBLIC_HOST)
        .unwrap()
        .as_str()
        .unwrap(),
      DEFAULT_HOST
    );
    assert_eq!(
      service
        .get_default_value(BODHI_PUBLIC_PORT)
        .unwrap()
        .as_u64()
        .unwrap(),
      DEFAULT_PORT as u64
    );
    assert_eq!(
      service
        .get_default_value(BODHI_PUBLIC_SCHEME)
        .unwrap()
        .as_str()
        .unwrap(),
      DEFAULT_SCHEME
    );

    // Set regular settings and verify public URL uses them
    service.set_setting(BODHI_HOST, "example.com");
    service.set_setting(BODHI_PORT, "8080");
    service.set_setting(BODHI_SCHEME, "https");

    assert_eq!(service.public_server_url(), "https://example.com:8080");

    // Verify fallback still works after setting regular settings
    assert_eq!(
      service
        .get_default_value(BODHI_PUBLIC_HOST)
        .unwrap()
        .as_str()
        .unwrap(),
      "example.com"
    );
    assert_eq!(
      service
        .get_default_value(BODHI_PUBLIC_PORT)
        .unwrap()
        .as_u64()
        .unwrap(),
      8080
    );
    assert_eq!(
      service
        .get_default_value(BODHI_PUBLIC_SCHEME)
        .unwrap()
        .as_str()
        .unwrap(),
      "https"
    );

    Ok(())
  }

  #[rstest]
  fn test_public_settings_explicit_override(temp_dir: TempDir) -> anyhow::Result<()> {
    let path = temp_dir.path().join("settings.yaml");
    let env_wrapper = EnvWrapperStub::new(HashMap::new());
    let service = DefaultSettingService::new_with_defaults(
      Arc::new(env_wrapper),
      bodhi_home_setting(temp_dir.path(), SettingSource::Environment),
      vec![],
      path.clone(),
    );

    // Set regular settings first
    service.set_setting(BODHI_HOST, "internal.example.com");
    service.set_setting(BODHI_PORT, "8080");
    service.set_setting(BODHI_SCHEME, "http");

    assert_eq!(
      service.public_server_url(),
      "http://internal.example.com:8080"
    );

    // Override with explicit public settings
    service.set_setting(BODHI_PUBLIC_HOST, "public.example.com");
    service.set_setting(BODHI_PUBLIC_PORT, "443");
    service.set_setting(BODHI_PUBLIC_SCHEME, "https");

    // Should now use public settings and omit standard port
    assert_eq!(
      service.public_server_url(),
      "https://public.example.com" // Port 443 omitted for https
    );

    // Test with non-standard port
    service.set_setting(BODHI_PUBLIC_PORT, "8443");
    assert_eq!(
      service.public_server_url(),
      "https://public.example.com:8443" // Non-standard port included
    );

    // Test HTTP with standard port
    service.set_setting(BODHI_PUBLIC_SCHEME, "http");
    service.set_setting(BODHI_PUBLIC_PORT, "80");
    assert_eq!(
      service.public_server_url(),
      "http://public.example.com" // Port 80 omitted for http
    );

    // Test HTTP with non-standard port
    service.set_setting(BODHI_PUBLIC_PORT, "8080");
    assert_eq!(
      service.public_server_url(),
      "http://public.example.com:8080" // Non-standard port included
    );

    Ok(())
  }

  #[rstest]
  #[case("http", "example.com", "80", "http://example.com")] // Standard port omitted for HTTP
  #[case("https", "example.com", "443", "https://example.com")] // Standard port omitted for HTTPS
  #[case("https", "example.com", "8080", "https://example.com:8080")] // Non-standard port included
  #[case("http", "example.com", "8443", "http://example.com:8443")] // Non-standard port included
  #[case("http", "localhost", "80", "http://localhost")] // Standard port omitted for localhost
  fn test_public_settings_url_construction_edge_cases(
    temp_dir: TempDir,
    #[case] scheme: &str,
    #[case] host: &str,
    #[case] port: &str,
    #[case] expected_url: &str,
  ) -> anyhow::Result<()> {
    let path = temp_dir.path().join("settings.yaml");
    let env_wrapper = EnvWrapperStub::new(HashMap::new());
    let service = DefaultSettingService::new_with_defaults(
      Arc::new(env_wrapper),
      bodhi_home_setting(temp_dir.path(), SettingSource::Environment),
      vec![],
      path.clone(),
    );

    service.set_setting(BODHI_PUBLIC_SCHEME, scheme);
    service.set_setting(BODHI_PUBLIC_HOST, host);
    service.set_setting(BODHI_PUBLIC_PORT, port);

    assert_eq!(service.public_server_url(), expected_url);

    Ok(())
  }

  #[rstest]
  fn test_public_settings_metadata_validation(temp_dir: TempDir) -> anyhow::Result<()> {
    let path = temp_dir.path().join("settings.yaml");
    let env_wrapper = EnvWrapperStub::new(HashMap::new());
    let service = DefaultSettingService::new_with_defaults(
      Arc::new(env_wrapper),
      bodhi_home_setting(temp_dir.path(), SettingSource::Environment),
      vec![],
      path.clone(),
    );

    // Test that BODHI_PUBLIC_PORT has Number metadata
    let metadata = service.get_setting_metadata(BODHI_PUBLIC_PORT);
    assert_eq!(metadata, SettingMetadata::Number { min: 1, max: 65535 });

    // Test that other public settings have String metadata
    let host_metadata = service.get_setting_metadata(BODHI_PUBLIC_HOST);
    assert_eq!(host_metadata, SettingMetadata::String);

    let scheme_metadata = service.get_setting_metadata(BODHI_PUBLIC_SCHEME);
    assert_eq!(scheme_metadata, SettingMetadata::String);

    // Test that public settings appear in the settings list
    let settings = service.list();
    let public_host_setting = settings.iter().find(|s| s.key == BODHI_PUBLIC_HOST);
    let public_port_setting = settings.iter().find(|s| s.key == BODHI_PUBLIC_PORT);
    let public_scheme_setting = settings.iter().find(|s| s.key == BODHI_PUBLIC_SCHEME);

    assert!(
      public_host_setting.is_some(),
      "BODHI_PUBLIC_HOST should be in settings list"
    );
    assert!(
      public_port_setting.is_some(),
      "BODHI_PUBLIC_PORT should be in settings list"
    );
    assert!(
      public_scheme_setting.is_some(),
      "BODHI_PUBLIC_SCHEME should be in settings list"
    );

    // Verify the metadata is correct in the list
    assert_eq!(
      public_port_setting.unwrap().metadata,
      SettingMetadata::Number { min: 1, max: 65535 }
    );
    assert_eq!(
      public_host_setting.unwrap().metadata,
      SettingMetadata::String
    );
    assert_eq!(
      public_scheme_setting.unwrap().metadata,
      SettingMetadata::String
    );

    Ok(())
  }

  #[rstest]
  #[case(
    // Default settings scenario
    None, None, None, // No public settings override
    None, None, None, // No regular settings override
    "http://localhost:1135",
    "http://localhost:1135/ui/chat",
    "http://localhost:1135/ui/auth/callback"
  )]
  #[case(
    // All public settings overridden
    Some("https"), Some("public.example.com"), Some("443"),
    None, None, None, // Regular settings not needed
    "https://public.example.com", // Port 443 omitted
    "https://public.example.com/ui/chat",
    "https://public.example.com/ui/auth/callback"
  )]
  #[case(
    // Public settings with non-standard port
    Some("https"), Some("public.example.com"), Some("8443"),
    None, None, None,
    "https://public.example.com:8443",
    "https://public.example.com:8443/ui/chat",
    "https://public.example.com:8443/ui/auth/callback"
  )]
  #[case(
    // Mixed scenario: only public host set, fallback to regular scheme/port
    None, Some("cdn.example.com"), None, // Only public host
    Some("http"), Some("internal.example.com"), Some("8080"), // Regular settings set
    "http://cdn.example.com:8080", // Uses public host, regular scheme/port
    "http://cdn.example.com:8080/ui/chat",
    "http://cdn.example.com:8080/ui/auth/callback"
  )]
  fn test_integration_method_behaviors(
    temp_dir: TempDir,
    #[case] public_scheme: Option<&str>,
    #[case] public_host: Option<&str>,
    #[case] public_port: Option<&str>,
    #[case] regular_scheme: Option<&str>,
    #[case] regular_host: Option<&str>,
    #[case] regular_port: Option<&str>,
    #[case] expected_public_url: &str,
    #[case] expected_frontend_url: &str,
    #[case] expected_callback_url: &str,
  ) -> anyhow::Result<()> {
    let path = temp_dir.path().join("settings.yaml");
    let env_wrapper = EnvWrapperStub::new(HashMap::new());
    let service = DefaultSettingService::new_with_defaults(
      Arc::new(env_wrapper),
      bodhi_home_setting(temp_dir.path(), SettingSource::Environment),
      vec![],
      path.clone(),
    );

    // Set regular settings if provided
    if let Some(scheme) = regular_scheme {
      service.set_setting(BODHI_SCHEME, scheme);
    }
    if let Some(host) = regular_host {
      service.set_setting(BODHI_HOST, host);
    }
    if let Some(port) = regular_port {
      service.set_setting(BODHI_PORT, port);
    }

    // Set public settings if provided
    if let Some(scheme) = public_scheme {
      service.set_setting(BODHI_PUBLIC_SCHEME, scheme);
    }
    if let Some(host) = public_host {
      service.set_setting(BODHI_PUBLIC_HOST, host);
    }
    if let Some(port) = public_port {
      service.set_setting(BODHI_PUBLIC_PORT, port);
    }

    // Verify all three methods return expected URLs
    assert_eq!(service.public_server_url(), expected_public_url);
    assert_eq!(service.frontend_default_url(), expected_frontend_url);
    assert_eq!(service.login_callback_url(), expected_callback_url);

    Ok(())
  }
}
