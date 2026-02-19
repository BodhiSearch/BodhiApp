use super::*;
use objs::{AppType, EnvType, LogLevel, SettingInfo, SettingMetadata, SettingSource};
use serde_yaml::Value;
use std::{path::Path, path::PathBuf, sync::Arc};
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
