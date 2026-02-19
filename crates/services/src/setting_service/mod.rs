use objs::SettingSource;
use serde_yaml::Value;
use std::sync::Arc;

mod default_service;
mod error;
mod service;
#[cfg(test)]
mod tests;

pub use default_service::*;
pub use error::*;
pub use service::*;

// System settings
pub const BODHI_HOME: &str = "BODHI_HOME";
pub const BODHI_ENV_TYPE: &str = "BODHI_ENV_TYPE";
pub const BODHI_APP_TYPE: &str = "BODHI_APP_TYPE";
pub const BODHI_VERSION: &str = "BODHI_VERSION";
pub const BODHI_COMMIT_SHA: &str = "BODHI_COMMIT_SHA";
pub const BODHI_AUTH_URL: &str = "BODHI_AUTH_URL";
pub const BODHI_AUTH_REALM: &str = "BODHI_AUTH_REALM";

// App Settings
pub const HF_HOME: &str = "HF_HOME";
pub const HF_TOKEN: &str = "HF_TOKEN";
pub const BODHI_LOGS: &str = "BODHI_LOGS";
pub const BODHI_LOG_LEVEL: &str = "BODHI_LOG_LEVEL";
pub const BODHI_LOG_STDOUT: &str = "BODHI_LOG_STDOUT";
pub const BODHI_SCHEME: &str = "BODHI_SCHEME";
pub const BODHI_HOST: &str = "BODHI_HOST";
pub const BODHI_PORT: &str = "BODHI_PORT";
// Public-facing host settings for Docker compatibility
pub const BODHI_PUBLIC_SCHEME: &str = "BODHI_PUBLIC_SCHEME";
pub const BODHI_PUBLIC_HOST: &str = "BODHI_PUBLIC_HOST";
pub const BODHI_PUBLIC_PORT: &str = "BODHI_PUBLIC_PORT";
pub const BODHI_CANONICAL_REDIRECT: &str = "BODHI_CANONICAL_REDIRECT";
// Exec settings
pub const BODHI_EXEC_LOOKUP_PATH: &str = "BODHI_EXEC_LOOKUP_PATH";
pub const BODHI_EXEC_VARIANT: &str = "BODHI_EXEC_VARIANT";
pub const BODHI_EXEC_TARGET: &str = "BODHI_EXEC_TARGET";
pub const BODHI_EXEC_NAME: &str = "BODHI_EXEC_NAME";
pub const BODHI_EXEC_VARIANTS: &str = "BODHI_EXEC_VARIANTS";
// Server arguments settings
pub const BODHI_LLAMACPP_ARGS: &str = "BODHI_LLAMACPP_ARGS";
// Server operations settings
pub const BODHI_KEEP_ALIVE_SECS: &str = "BODHI_KEEP_ALIVE_SECS";

pub const DEFAULT_SCHEME: &str = "http";
pub const DEFAULT_HOST: &str = "0.0.0.0";
pub const DEFAULT_PORT: u16 = 1135;
pub const DEFAULT_PORT_STR: &str = "1135";
pub const DEFAULT_LOG_LEVEL: &str = "warn";
pub const DEFAULT_LOG_STDOUT: bool = false;
pub const DEFAULT_KEEP_ALIVE_SECS: i64 = 300;
pub const DEFAULT_CANONICAL_REDIRECT: bool = true;

pub const SETTINGS_YAML: &str = "settings.yaml";

pub const LOGIN_CALLBACK_PATH: &str = "/ui/auth/callback";
pub const CHAT_PATH: &str = "/ui/chat";

const PROD_DB: &str = "bodhi.sqlite";
const SESSION_DB: &str = "session.sqlite";

const LOGS_DIR: &str = "logs";

pub const BODHI_ENCRYPTION_KEY: &str = "BODHI_ENCRYPTION_KEY";
pub const BODHI_DEV_PROXY_UI: &str = "BODHI_DEV_PROXY_UI";

// RunPod settings
pub const BODHI_ON_RUNPOD: &str = "BODHI_ON_RUNPOD";
pub const RUNPOD_POD_ID: &str = "RUNPOD_POD_ID";

pub const SETTING_VARS: &[&str] = &[
  HF_HOME,
  BODHI_LOGS,
  BODHI_LOG_LEVEL,
  BODHI_LOG_STDOUT,
  BODHI_SCHEME,
  BODHI_HOST,
  BODHI_PORT,
  BODHI_PUBLIC_SCHEME,
  BODHI_PUBLIC_HOST,
  BODHI_PUBLIC_PORT,
  BODHI_CANONICAL_REDIRECT,
  BODHI_EXEC_LOOKUP_PATH,
  BODHI_EXEC_VARIANT,
  BODHI_EXEC_TARGET,
  BODHI_EXEC_NAME,
  BODHI_EXEC_VARIANTS,
  BODHI_KEEP_ALIVE_SECS,
  BODHI_LLAMACPP_ARGS,
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
