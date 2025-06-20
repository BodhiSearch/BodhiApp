use lib_bodhiserver::{AppOptionsBuilder, AppOptionsError, AppStatus};
use napi_derive::napi;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Flexible configuration options for the Bodhi server that can be passed across NAPI boundary
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NapiAppOptions {
  /// Environment variables to set
  pub env_vars: HashMap<String, String>,
  /// App settings (configurable via settings.yaml)
  pub app_settings: HashMap<String, String>,
  /// System settings (immutable)
  pub system_settings: HashMap<String, String>,
  /// OAuth client ID (optional)
  pub client_id: Option<String>,
  /// OAuth client secret (optional)
  pub client_secret: Option<String>,
  /// App status as string (optional)
  pub app_status: Option<String>,
}

/// Create a new NapiAppOptions with empty configuration
#[napi]
pub fn create_napi_app_options() -> NapiAppOptions {
  NapiAppOptions {
    env_vars: HashMap::new(),
    app_settings: HashMap::new(),
    system_settings: HashMap::new(),
    client_id: None,
    client_secret: None,
    app_status: None,
  }
}

/// Set an environment variable
#[napi]
pub fn set_env_var(mut config: NapiAppOptions, key: String, value: String) -> NapiAppOptions {
  config.env_vars.insert(key, value);
  config
}

/// Set an app setting (configurable via settings.yaml)
#[napi]
pub fn set_app_setting(mut config: NapiAppOptions, key: String, value: String) -> NapiAppOptions {
  config.app_settings.insert(key, value);
  config
}

/// Set a system setting (immutable)
#[napi]
pub fn set_system_setting(
  mut config: NapiAppOptions,
  key: String,
  value: String,
) -> NapiAppOptions {
  config.system_settings.insert(key, value);
  config
}

/// Set OAuth client credentials
#[napi]
pub fn set_client_credentials(
  mut config: NapiAppOptions,
  client_id: String,
  client_secret: String,
) -> NapiAppOptions {
  config.client_id = Some(client_id);
  config.client_secret = Some(client_secret);
  config
}

/// Set app status
#[napi]
pub fn set_app_status(mut config: NapiAppOptions, status: String) -> napi::Result<NapiAppOptions> {
  use lib_bodhiserver::services::AppStatus;
  match status.parse::<AppStatus>() {
    Ok(_) => {
      config.app_status = Some(status);
      Ok(config)
    }
    Err(_) => Err(napi::Error::new(
      napi::Status::GenericFailure,
      format!("Invalid app status: {}", status),
    )),
  }
}

/// Internal function to build AppOptions (not exposed to NAPI)
pub fn try_build_app_options_internal(
  config: NapiAppOptions,
) -> Result<AppOptionsBuilder, AppOptionsError> {
  let mut builder = AppOptionsBuilder::default();

  // Set environment variables
  for (key, value) in config.env_vars {
    builder = builder.set_env(&key, &value);
  }

  // Set app settings
  for (key, value) in config.app_settings {
    builder = builder.set_app_setting(&key, &value);
  }

  // Set system settings
  for (key, value) in config.system_settings {
    builder = builder.set_system_setting(&key, &value)?;
  }

  // Set OAuth client credentials if both are provided
  if let (Some(client_id), Some(client_secret)) = (config.client_id, config.client_secret) {
    builder = builder.set_app_reg_info(&client_id, &client_secret);
  }

  // Set app status if provided
  if let Some(status_str) = config.app_status {
    let status = status_str.parse::<AppStatus>().map_err(|_| {
      AppOptionsError::ValidationError(format!("Invalid app status: {}", status_str))
    })?;
    builder = builder.set_app_status(status);
  }
  Ok(builder)
}

// Export constants for safe configuration
#[napi]
pub const BODHI_HOME: &str = "BODHI_HOME";

#[napi]
pub const BODHI_HOST: &str = "BODHI_HOST";

#[napi]
pub const BODHI_PORT: &str = "BODHI_PORT";

#[napi]
pub const BODHI_SCHEME: &str = "BODHI_SCHEME";

#[napi]
pub const BODHI_LOG_LEVEL: &str = "BODHI_LOG_LEVEL";

#[napi]
pub const BODHI_LOG_STDOUT: &str = "BODHI_LOG_STDOUT";

#[napi]
pub const BODHI_LOGS: &str = "BODHI_LOGS";

#[napi]
pub const BODHI_ENV_TYPE: &str = "BODHI_ENV_TYPE";

#[napi]
pub const BODHI_APP_TYPE: &str = "BODHI_APP_TYPE";

#[napi]
pub const BODHI_VERSION: &str = "BODHI_VERSION";

#[napi]
pub const BODHI_AUTH_URL: &str = "BODHI_AUTH_URL";

#[napi]
pub const BODHI_AUTH_REALM: &str = "BODHI_AUTH_REALM";

#[napi]
pub const BODHI_ENCRYPTION_KEY: &str = "BODHI_ENCRYPTION_KEY";

#[napi]
pub const BODHI_EXEC_LOOKUP_PATH: &str = "BODHI_EXEC_LOOKUP_PATH";

#[napi]
pub const BODHI_EXEC_VARIANT: &str = "BODHI_EXEC_VARIANT";

#[napi]
pub const BODHI_KEEP_ALIVE_SECS: &str = "BODHI_KEEP_ALIVE_SECS";

#[napi]
pub const HF_HOME: &str = "HF_HOME";

#[napi]
pub const DEFAULT_HOST: &str = "localhost";

#[napi]
pub const DEFAULT_PORT: u16 = 1135;

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_create_napi_app_options() {
    let config = create_napi_app_options();
    assert!(config.env_vars.is_empty());
    assert!(config.app_settings.is_empty());
    assert!(config.system_settings.is_empty());
    assert!(config.client_id.is_none());
    assert!(config.client_secret.is_none());
    assert!(config.app_status.is_none());
  }

  #[test]
  fn test_set_env_var() {
    let config = create_napi_app_options();
    let config = set_env_var(config, "TEST_KEY".to_string(), "test_value".to_string());

    assert_eq!(
      config.env_vars.get("TEST_KEY"),
      Some(&"test_value".to_string())
    );
  }

  #[test]
  fn test_set_app_setting() {
    let config = create_napi_app_options();
    let config = set_app_setting(
      config,
      "setting_key".to_string(),
      "setting_value".to_string(),
    );

    assert_eq!(
      config.app_settings.get("setting_key"),
      Some(&"setting_value".to_string())
    );
  }

  #[test]
  fn test_set_system_setting() {
    let config = create_napi_app_options();
    let config = set_system_setting(config, "BODHI_VERSION".to_string(), "1.0.0".to_string());

    assert_eq!(
      config.system_settings.get("BODHI_VERSION"),
      Some(&"1.0.0".to_string())
    );
  }

  #[test]
  fn test_set_client_credentials() {
    let config = create_napi_app_options();
    let config = set_client_credentials(config, "client123".to_string(), "secret456".to_string());

    assert_eq!(config.client_id, Some("client123".to_string()));
    assert_eq!(config.client_secret, Some("secret456".to_string()));
  }

  #[test]
  fn test_set_app_status() {
    let config = create_napi_app_options();
    let config = set_app_status(config, "ready".to_string()).unwrap();

    assert_eq!(config.app_status, Some("ready".to_string()));
  }

  #[test]
  fn test_set_app_status_invalid() {
    let config = create_napi_app_options();
    let result = set_app_status(config, "invalid".to_string());

    assert!(result.is_err());
  }

  #[test]
  fn test_build_app_options_success() -> anyhow::Result<()> {
    let mut config = create_napi_app_options();
    config = set_system_setting(
      config,
      BODHI_ENV_TYPE.to_string(),
      "development".to_string(),
    );
    config = set_system_setting(config, BODHI_APP_TYPE.to_string(), "native".to_string());
    config = set_system_setting(config, BODHI_VERSION.to_string(), "1.0.0".to_string());
    config = set_system_setting(
      config,
      BODHI_AUTH_URL.to_string(),
      "http://localhost:8080".to_string(),
    );
    config = set_system_setting(config, BODHI_AUTH_REALM.to_string(), "bodhi".to_string());

    let _ = try_build_app_options_internal(config)?;
    Ok(())
  }

  #[test]
  fn test_constants() {
    assert_eq!(BODHI_HOME, "BODHI_HOME");
    assert_eq!(BODHI_HOST, "BODHI_HOST");
    assert_eq!(BODHI_PORT, "BODHI_PORT");
    assert_eq!(DEFAULT_HOST, "localhost");
    assert_eq!(DEFAULT_PORT, 1135);
  }

  #[test]
  fn test_try_build_app_options_internal() -> anyhow::Result<()> {
    let mut config = create_napi_app_options();
    config = set_env_var(config, "TEST_ENV".to_string(), "test_value".to_string());
    config = set_app_setting(
      config,
      "test_setting".to_string(),
      "setting_value".to_string(),
    );
    config = set_system_setting(
      config,
      BODHI_ENV_TYPE.to_string(),
      "development".to_string(),
    );
    config = set_system_setting(config, BODHI_APP_TYPE.to_string(), "native".to_string());
    config = set_system_setting(config, BODHI_VERSION.to_string(), "1.0.0".to_string());
    config = set_system_setting(
      config,
      BODHI_AUTH_URL.to_string(),
      "http://localhost:8080".to_string(),
    );
    config = set_system_setting(config, BODHI_AUTH_REALM.to_string(), "bodhi".to_string());
    config = set_client_credentials(config, "client123".to_string(), "secret456".to_string());
    config = set_app_status(config, "ready".to_string()).unwrap();

    let result = try_build_app_options_internal(config)?.build();
    assert!(result.is_ok());

    let app_options = result.unwrap();
    assert_eq!(app_options.app_version, "1.0.0");
    assert_eq!(app_options.auth_url, "http://localhost:8080");
    assert_eq!(app_options.auth_realm, "bodhi");
    assert!(app_options.app_reg_info.is_some());
    assert!(app_options.app_status.is_some());
    Ok(())
  }
}
