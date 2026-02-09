use super::{Result, *};
use crate::{asref_impl, EnvWrapper};
use objs::{AppType, EnvType, LogLevel, Setting, SettingInfo, SettingMetadata, SettingSource};
use serde_yaml::Value;
use std::{
  collections::HashMap,
  fs,
  path::{Path, PathBuf},
  sync::{Arc, RwLock},
};
use tracing::warn;

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

  fn commit_sha(&self) -> String {
    self
      .get_setting(BODHI_COMMIT_SHA)
      .expect("BODHI_COMMIT_SHA should be set")
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
      Value::Number(n) => n.as_u64().expect("BODHI_PORT should be a number") as u16,
      Value::String(s) => match s.parse() {
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
    let (value, source) = self.get_setting_value_with_source(BODHI_PUBLIC_SCHEME);
    match source {
      SettingSource::Default if self.on_runpod_enabled() => "https".to_string(),
      _ => value
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| self.scheme()),
    }
  }

  fn public_host(&self) -> String {
    let (value, source) = self.get_setting_value_with_source(BODHI_PUBLIC_HOST);
    match source {
      SettingSource::Default if self.on_runpod_enabled() => {
        let pod_id = self
          .get_setting(RUNPOD_POD_ID)
          .unwrap_or_else(|| "unknown".to_string());
        let port = self.port();
        format!("{}-{}.proxy.runpod.net", pod_id, port)
      }
      _ => value
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| self.host()),
    }
  }

  fn get_public_host_explicit(&self) -> Option<String> {
    let (value, source) = self.get_setting_value_with_source(BODHI_PUBLIC_HOST);
    match source {
      SettingSource::Default => {
        // Check if RunPod is enabled - if so, treat as explicitly configured
        if self.on_runpod_enabled() {
          Some(self.public_host())
        } else {
          None
        }
      }
      _ => value.and_then(|v| match v {
        Value::String(s) => Some(s),
        _ => None,
      }),
    }
  }

  fn public_port(&self) -> u16 {
    let (value, source) = self.get_setting_value_with_source(BODHI_PUBLIC_PORT);
    match source {
      SettingSource::Default if self.on_runpod_enabled() => 443,
      _ => match value {
        Some(Value::Number(n)) => n.as_u64().unwrap_or(self.port() as u64) as u16,
        Some(Value::String(s)) => s.parse().unwrap_or(self.port()),
        _ => self.port(),
      },
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
    format!("{}{}", self.public_server_url(), LOGIN_CALLBACK_PATH)
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

  fn canonical_redirect_enabled(&self) -> bool {
    self
      .get_setting_value(BODHI_CANONICAL_REDIRECT)
      .unwrap_or(Value::Bool(DEFAULT_CANONICAL_REDIRECT))
      .as_bool()
      .expect("BODHI_CANONICAL_REDIRECT should be a boolean")
  }

  fn get_server_args_common(&self) -> Option<String> {
    self.get_setting(BODHI_LLAMACPP_ARGS)
  }

  fn get_server_args_variant(&self, variant: &str) -> Option<String> {
    let key = format!("BODHI_LLAMACPP_ARGS_{}", variant.to_uppercase());
    self.get_setting(&key)
  }

  fn on_runpod_enabled(&self) -> bool {
    let runpod_flag = self
      .get_setting(BODHI_ON_RUNPOD)
      .and_then(|val| val.parse::<bool>().ok())
      .unwrap_or(false);

    let runpod_pod_id_available = self
      .get_setting(RUNPOD_POD_ID)
      .filter(|pod_id| !pod_id.is_empty())
      .is_some();

    runpod_flag && runpod_pod_id_available
  }
}

#[derive(Debug)]
pub struct DefaultSettingService {
  env_wrapper: Arc<dyn EnvWrapper>,
  settings_file: PathBuf,
  system_settings: Vec<Setting>,
  settings_lock: RwLock<()>,
  defaults: RwLock<HashMap<String, Value>>,
  listeners: RwLock<Vec<Arc<dyn SettingsChangeListener>>>,
  cmd_lines: RwLock<HashMap<String, Value>>,
}

impl DefaultSettingService {
  pub(crate) fn new(
    env_wrapper: Arc<dyn EnvWrapper>,
    settings_file: PathBuf,
    system_settings: Vec<Setting>,
  ) -> DefaultSettingService {
    Self {
      env_wrapper,
      settings_file,
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
    file_defaults: HashMap<String, Value>,
    settings_file: PathBuf,
  ) -> Self {
    system_settings.push(bodhi_home.clone());
    let service = Self::new(env_wrapper, settings_file, system_settings);
    service.init_defaults(bodhi_home, file_defaults);
    service
  }

  fn init_defaults(&self, bodhi_home: Setting, file_defaults: HashMap<String, Value>) {
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
      macro_rules! set_default {
        ($key:expr, $hardcoded_value:expr) => {
          defaults.insert(
            $key.to_string(),
            file_defaults.get($key).cloned().unwrap_or($hardcoded_value),
          );
        };
      }
      set_default!(BODHI_SCHEME, Value::String(DEFAULT_SCHEME.to_string()));
      set_default!(BODHI_HOST, Value::String(DEFAULT_HOST.to_string()));
      set_default!(BODHI_PORT, Value::Number(DEFAULT_PORT.into()));
      set_default!(
        BODHI_LOG_LEVEL,
        Value::String(DEFAULT_LOG_LEVEL.to_string())
      );
      set_default!(BODHI_LOG_STDOUT, Value::Bool(DEFAULT_LOG_STDOUT));
      set_default!(
        BODHI_EXEC_TARGET,
        Value::String(llama_server_proc::BUILD_TARGET.to_string())
      );
      set_default!(
        BODHI_EXEC_VARIANTS,
        Value::String(
          llama_server_proc::BUILD_VARIANTS
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(","),
        )
      );
      set_default!(
        BODHI_EXEC_VARIANT,
        Value::String(llama_server_proc::DEFAULT_VARIANT.to_string())
      );
      set_default!(
        BODHI_EXEC_NAME,
        Value::String(llama_server_proc::EXEC_NAME.to_string())
      );
      set_default!(
        BODHI_LLAMACPP_ARGS,
        Value::String("--jinja --no-webui".to_string())
      );
      set_default!(
        BODHI_KEEP_ALIVE_SECS,
        Value::Number(DEFAULT_KEEP_ALIVE_SECS.into())
      );
      set_default!(
        BODHI_CANONICAL_REDIRECT,
        Value::Bool(DEFAULT_CANONICAL_REDIRECT)
      );
      for (key, value) in file_defaults {
        defaults.insert(key, value);
      }
    });
  }

  pub fn with_settings_read_lock<F, R>(&self, f: F) -> R
  where
    F: FnOnce(&serde_yaml::Mapping) -> R,
  {
    let _guard = self.settings_lock.read().unwrap();
    if !self.settings_file.exists() {
      return f(&serde_yaml::Mapping::new());
    }
    let contents = fs::read_to_string(&self.settings_file).unwrap_or_else(|_| String::new());
    let settings: serde_yaml::Mapping =
      serde_yaml::from_str(&contents).unwrap_or_else(|_| serde_yaml::Mapping::new());
    f(&settings)
  }

  pub fn with_settings_write_lock<F>(&self, f: F)
  where
    F: FnOnce(&mut serde_yaml::Mapping),
  {
    let _guard = self.settings_lock.write().unwrap();
    let mut settings = if !self.settings_file.exists() {
      serde_yaml::Mapping::new()
    } else {
      let contents = fs::read_to_string(&self.settings_file).unwrap_or_else(|_| String::new());
      serde_yaml::from_str(&contents).unwrap_or_else(|_| serde_yaml::Mapping::new())
    };
    f(&mut settings);
    let contents = serde_yaml::to_string(&settings).unwrap();
    fs::write(&self.settings_file, contents).unwrap();
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
    let mut app_settings = SETTING_VARS
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

    // Add variant-specific server args settings
    let variants = self.exec_variants();
    for variant in variants {
      let variant_key = format!("BODHI_LLAMACPP_ARGS_{}", variant.to_uppercase());
      let (current_value, source) = self.get_setting_value_with_source(&variant_key);
      let metadata = self.get_setting_metadata(&variant_key);
      let current_value = current_value.map(|value| metadata.parse(value));

      app_settings.push(SettingInfo {
        key: variant_key.clone(),
        current_value: current_value.unwrap_or(Value::Null),
        default_value: self.get_default_value(&variant_key).unwrap_or(Value::Null),
        source,
        metadata,
      });
    }

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
      BODHI_CANONICAL_REDIRECT => SettingMetadata::Boolean,
      BODHI_LLAMACPP_ARGS => SettingMetadata::String,
      key if key.starts_with("BODHI_LLAMACPP_ARGS_") => SettingMetadata::String,
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
            .map(|path| Value::String(path.display().to_string()));
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
