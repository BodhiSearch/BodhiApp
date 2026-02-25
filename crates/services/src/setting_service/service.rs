use super::{
  SettingServiceError, BODHI_APP_TYPE, BODHI_AUTH_REALM, BODHI_AUTH_URL, BODHI_CANONICAL_REDIRECT,
  BODHI_COMMIT_SHA, BODHI_ENCRYPTION_KEY, BODHI_ENV_TYPE, BODHI_EXEC_LOOKUP_PATH, BODHI_EXEC_NAME,
  BODHI_EXEC_TARGET, BODHI_EXEC_VARIANT, BODHI_EXEC_VARIANTS, BODHI_HOME, BODHI_HOST,
  BODHI_KEEP_ALIVE_SECS, BODHI_LLAMACPP_ARGS, BODHI_LOGS, BODHI_LOG_LEVEL, BODHI_ON_RUNPOD,
  BODHI_PORT, BODHI_PUBLIC_HOST, BODHI_PUBLIC_PORT, BODHI_PUBLIC_SCHEME, BODHI_SCHEME,
  BODHI_VERSION, DEFAULT_CANONICAL_REDIRECT, DEFAULT_PORT, HF_HOME, LOGIN_CALLBACK_PATH, PROD_DB,
  RUNPOD_POD_ID, SESSION_DB,
};
use objs::{AppType, EnvType, LogLevel, SettingInfo, SettingMetadata, SettingSource};
use serde_yaml::Value;
use std::{path::Path, path::PathBuf, sync::Arc};
use tracing::warn;

type Result<T> = std::result::Result<T, SettingServiceError>;

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

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait SettingService: std::fmt::Debug + Send + Sync {
  async fn load(&self, path: &Path);

  async fn home_dir(&self) -> Option<PathBuf>;

  async fn list(&self) -> Vec<SettingInfo>;

  async fn get_default_value(&self, key: &str) -> Option<Value>;

  async fn get_setting_metadata(&self, key: &str) -> SettingMetadata;

  async fn get_env(&self, key: &str) -> Option<String>;

  async fn get_setting(&self, key: &str) -> Option<String> {
    match self.get_setting_value(key).await {
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

  async fn get_setting_value(&self, key: &str) -> Option<Value> {
    self.get_setting_value_with_source(key).await.0
  }

  async fn get_setting_value_with_source(&self, key: &str) -> (Option<Value>, SettingSource);

  async fn set_setting_with_source(
    &self,
    key: &str,
    value: &Value,
    source: SettingSource,
  ) -> Result<()>;

  async fn set_setting(&self, key: &str, value: &str) -> Result<()> {
    self
      .set_setting_value(key, &Value::String(value.to_owned()))
      .await
  }

  async fn set_setting_value(&self, key: &str, value: &Value) -> Result<()> {
    self
      .set_setting_with_source(key, value, SettingSource::Database)
      .await
  }

  async fn set_default(&self, key: &str, value: &Value) -> Result<()> {
    self
      .set_setting_with_source(key, value, SettingSource::Default)
      .await
  }

  async fn delete_setting(&self, key: &str) -> Result<()>;

  async fn add_listener(&self, listener: Arc<dyn SettingsChangeListener>);

  async fn bodhi_home(&self) -> PathBuf {
    let bodhi_home = self
      .get_setting(BODHI_HOME)
      .await
      .expect("BODHI_HOME should be set");
    PathBuf::from(bodhi_home)
  }

  async fn env_type(&self) -> EnvType {
    self
      .get_setting(BODHI_ENV_TYPE)
      .await
      .expect("BODHI_ENV_TYPE should be set")
      .parse()
      .unwrap()
  }

  async fn app_type(&self) -> AppType {
    self
      .get_setting(BODHI_APP_TYPE)
      .await
      .expect("BODHI_APP_TYPE should be set")
      .parse()
      .unwrap()
  }

  async fn version(&self) -> String {
    self
      .get_setting(BODHI_VERSION)
      .await
      .expect("BODHI_VERSION should be set")
      .to_string()
  }

  async fn commit_sha(&self) -> String {
    self
      .get_setting(BODHI_COMMIT_SHA)
      .await
      .expect("BODHI_COMMIT_SHA should be set")
      .to_string()
  }

  async fn auth_url(&self) -> String {
    self
      .get_setting(BODHI_AUTH_URL)
      .await
      .expect("BODHI_AUTH_URL should be set")
      .to_string()
  }

  async fn auth_realm(&self) -> String {
    self
      .get_setting(BODHI_AUTH_REALM)
      .await
      .expect("BODHI_AUTH_REALM should be set")
      .to_string()
  }

  async fn is_production(&self) -> bool {
    self.env_type().await.is_production()
  }

  async fn is_native(&self) -> bool {
    self.app_type().await.is_native()
  }

  async fn hf_home(&self) -> PathBuf {
    PathBuf::from(
      self
        .get_setting(HF_HOME)
        .await
        .expect("HF_HOME should be set"),
    )
  }

  async fn logs_dir(&self) -> PathBuf {
    PathBuf::from(
      self
        .get_setting(BODHI_LOGS)
        .await
        .expect("BODHI_LOGS should be set"),
    )
  }

  async fn scheme(&self) -> String {
    self
      .get_setting(BODHI_SCHEME)
      .await
      .expect("BODHI_SCHEME should be set")
  }

  async fn host(&self) -> String {
    self
      .get_setting(BODHI_HOST)
      .await
      .expect("BODHI_HOST should be set")
  }

  async fn port(&self) -> u16 {
    match self
      .get_setting_value(BODHI_PORT)
      .await
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

  async fn frontend_default_url(&self) -> String {
    format!("{}/ui/chat", self.public_server_url().await)
  }

  async fn app_db_path(&self) -> PathBuf {
    self.bodhi_home().await.join(PROD_DB)
  }

  async fn session_db_path(&self) -> PathBuf {
    self.bodhi_home().await.join(SESSION_DB)
  }

  async fn log_level(&self) -> LogLevel {
    let log_level = self
      .get_setting(BODHI_LOG_LEVEL)
      .await
      .expect("BODHI_LOG_LEVEL should be set");
    LogLevel::try_from(log_level.as_str()).unwrap_or(LogLevel::Warn)
  }

  async fn exec_lookup_path(&self) -> String {
    self
      .get_setting(BODHI_EXEC_LOOKUP_PATH)
      .await
      .expect("BODHI_EXEC_LOOKUP_PATH should be set")
  }

  async fn exec_variant(&self) -> String {
    self
      .get_setting(BODHI_EXEC_VARIANT)
      .await
      .expect("BODHI_EXEC_VARIANT should be set")
  }

  async fn exec_target(&self) -> String {
    self
      .get_setting(BODHI_EXEC_TARGET)
      .await
      .expect("BODHI_EXEC_TARGET should be set")
  }

  async fn exec_name(&self) -> String {
    self
      .get_setting(BODHI_EXEC_NAME)
      .await
      .expect("BODHI_EXEC_NAME should be set")
  }

  async fn exec_variants(&self) -> Vec<String> {
    self
      .get_setting(BODHI_EXEC_VARIANTS)
      .await
      .expect("BODHI_EXEC_VARIANTS should be set")
      .split(',')
      .map(|s| s.trim().to_string())
      .collect()
  }

  async fn exec_path_from(&self) -> PathBuf {
    let lookup_path = PathBuf::from(self.exec_lookup_path().await);
    let target = self.exec_target().await;
    let variant = self.exec_variant().await;
    let exec_name = self.exec_name().await;
    lookup_path.join(target).join(variant).join(exec_name)
  }

  async fn public_scheme(&self) -> String {
    let (value, source) = self
      .get_setting_value_with_source(BODHI_PUBLIC_SCHEME)
      .await;
    match source {
      SettingSource::Default if self.on_runpod_enabled().await => "https".to_string(),
      _ => match value.and_then(|v| v.as_str().map(|s| s.to_string())) {
        Some(s) => s,
        None => self.scheme().await,
      },
    }
  }

  async fn public_host(&self) -> String {
    let (value, source) = self.get_setting_value_with_source(BODHI_PUBLIC_HOST).await;
    match source {
      SettingSource::Default if self.on_runpod_enabled().await => {
        let pod_id = self
          .get_setting(RUNPOD_POD_ID)
          .await
          .unwrap_or_else(|| "unknown".to_string());
        let port = self.port().await;
        format!("{}-{}.proxy.runpod.net", pod_id, port)
      }
      _ => match value.and_then(|v| v.as_str().map(|s| s.to_string())) {
        Some(s) => s,
        None => self.host().await,
      },
    }
  }

  async fn get_public_host_explicit(&self) -> Option<String> {
    let (value, source) = self.get_setting_value_with_source(BODHI_PUBLIC_HOST).await;
    match source {
      SettingSource::Default => {
        if self.on_runpod_enabled().await {
          Some(self.public_host().await)
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

  async fn public_port(&self) -> u16 {
    let (value, source) = self.get_setting_value_with_source(BODHI_PUBLIC_PORT).await;
    match source {
      SettingSource::Default if self.on_runpod_enabled().await => 443,
      _ => match value {
        Some(Value::Number(n)) => {
          let fallback = self.port().await;
          n.as_u64().unwrap_or(fallback as u64) as u16
        }
        Some(Value::String(s)) => {
          let fallback = self.port().await;
          s.parse().unwrap_or(fallback)
        }
        _ => self.port().await,
      },
    }
  }

  async fn public_server_url(&self) -> String {
    let scheme = self.public_scheme().await;
    let host = self.public_host().await;
    let port = self.public_port().await;
    match (scheme.as_str(), port) {
      ("http", 80) | ("https", 443) => format!("{}://{}", scheme, host),
      _ => format!("{}://{}:{}", scheme, host, port),
    }
  }

  async fn hf_cache(&self) -> PathBuf {
    self.hf_home().await.join("hub")
  }

  async fn login_url(&self) -> String {
    format!(
      "{}/realms/{}/protocol/openid-connect/auth",
      self.auth_url().await,
      self.auth_realm().await
    )
  }

  async fn auth_issuer(&self) -> String {
    format!(
      "{}/realms/{}",
      self.auth_url().await,
      self.auth_realm().await
    )
  }

  async fn token_url(&self) -> String {
    format!(
      "{}/realms/{}/protocol/openid-connect/token",
      self.auth_url().await,
      self.auth_realm().await
    )
  }

  async fn login_callback_url(&self) -> String {
    format!("{}{}", self.public_server_url().await, LOGIN_CALLBACK_PATH)
  }

  async fn encryption_key(&self) -> Option<String> {
    SettingService::get_env(self, BODHI_ENCRYPTION_KEY).await
  }

  #[cfg(not(debug_assertions))]
  async fn get_dev_env(&self, _key: &str) -> Option<String> {
    None
  }

  #[cfg(debug_assertions)]
  async fn get_dev_env(&self, key: &str) -> Option<String> {
    SettingService::get_env(self, key).await
  }

  async fn keep_alive(&self) -> i64 {
    self
      .get_setting_value(BODHI_KEEP_ALIVE_SECS)
      .await
      .expect("BODHI_KEEP_ALIVE_SECS should be set")
      .as_i64()
      .expect("BODHI_KEEP_ALIVE_SECS should be a number")
  }

  async fn canonical_redirect_enabled(&self) -> bool {
    self
      .get_setting_value(BODHI_CANONICAL_REDIRECT)
      .await
      .unwrap_or(Value::Bool(DEFAULT_CANONICAL_REDIRECT))
      .as_bool()
      .expect("BODHI_CANONICAL_REDIRECT should be a boolean")
  }

  async fn get_server_args_common(&self) -> Option<String> {
    self.get_setting(BODHI_LLAMACPP_ARGS).await
  }

  async fn get_server_args_variant(&self, variant: &str) -> Option<String> {
    let key = format!("BODHI_LLAMACPP_ARGS_{}", variant.to_uppercase());
    self.get_setting(&key).await
  }

  async fn on_runpod_enabled(&self) -> bool {
    let runpod_flag = self
      .get_setting(BODHI_ON_RUNPOD)
      .await
      .and_then(|val| val.parse::<bool>().ok())
      .unwrap_or(false);

    let runpod_pod_id_available = self
      .get_setting(RUNPOD_POD_ID)
      .await
      .filter(|pod_id| !pod_id.is_empty())
      .is_some();

    runpod_flag && runpod_pod_id_available
  }
}
