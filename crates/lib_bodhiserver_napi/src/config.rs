use lib_bodhiserver::{
  AppOptions, AppOptionsBuilder, DefaultEnvWrapper, ErrorMessage, BODHI_APP_TYPE, BODHI_AUTH_REALM,
  BODHI_AUTH_URL, BODHI_ENV_TYPE, BODHI_VERSION,
};
use napi_derive::napi;

/// FFI-compatible configuration for BodhiApp initialization
#[napi(object)]
#[derive(Debug, Clone)]
pub struct AppConfig {
  /// Environment type (Development, Production, etc.)
  pub env_type: String,
  /// Application type (Container, Native, etc.)
  pub app_type: String,
  /// Application version string
  pub app_version: String,
  /// Authentication server URL
  pub auth_url: String,
  /// Authentication realm
  pub auth_realm: String,

  // Enhanced configuration fields
  /// Environment variables to be set (including sensitive/test keys, exec path, port, etc.)
  pub environment_vars: Option<std::collections::HashMap<String, String>>,
  /// App settings (configurable via settings.yaml)
  pub app_settings: Option<std::collections::HashMap<String, String>>,
  /// OAuth client credentials
  pub oauth_client_id: Option<String>,
  /// OAuth client secret
  pub oauth_client_secret: Option<String>,
  /// App initialization status
  pub app_status: Option<String>,
}

impl AppConfig {
  fn build_options(config: &AppConfig) -> Result<AppOptionsBuilder, ErrorMessage> {
    Ok(
      AppOptionsBuilder::new()
        .set_system_setting(BODHI_ENV_TYPE, &config.env_type)?
        .set_system_setting(BODHI_APP_TYPE, &config.app_type)?
        .set_system_setting(BODHI_VERSION, &config.app_version)?
        .set_system_setting(BODHI_AUTH_URL, &config.auth_url)?
        .set_system_setting(BODHI_AUTH_REALM, &config.auth_realm)?,
    )
  }
}

impl TryFrom<AppConfig> for AppOptions {
  type Error = String;

  fn try_from(config: AppConfig) -> Result<Self, Self::Error> {
    use lib_bodhiserver::{AppStatus, AppType, EnvType, EnvWrapper};
    use std::str::FromStr;
    use std::sync::Arc;

    let mut env_wrapper = DefaultEnvWrapper::default();

    // Set environment variables if provided
    if let Some(ref env_vars) = config.environment_vars {
      for (key, value) in env_vars {
        env_wrapper.set_var(key, value);
      }
    }

    let _env_wrapper: Arc<dyn EnvWrapper> = Arc::new(env_wrapper);
    let _env_type =
      EnvType::from_str(&config.env_type).map_err(|e| format!("Invalid env_type: {}", e))?;
    let _app_type =
      AppType::from_str(&config.app_type).map_err(|e| format!("Invalid app_type: {}", e))?;

    // Use the new builder interface with system settings
    let mut builder = AppConfig::build_options(&config).map_err(|e| e.to_string())?;

    // Set all environment_vars in builder
    if let Some(ref env_vars) = config.environment_vars {
      for (key, value) in env_vars {
        builder = builder.set_env(key, value);
      }
    }

    // Set app settings if provided
    if let Some(app_settings) = config.app_settings {
      for (key, value) in app_settings {
        builder = builder.set_app_setting(&key, &value);
      }
    }

    // Set OAuth credentials if provided
    if let Some(client_id) = config.oauth_client_id {
      if let Some(client_secret) = config.oauth_client_secret {
        builder = builder.set_app_reg_info(&client_id, &client_secret);
      }
    }

    // Set app status if provided
    if let Some(status_str) = config.app_status {
      if let Ok(status) = AppStatus::from_str(&status_str) {
        builder = builder.set_app_status(status);
      }
    }

    let options = builder
      .build()
      .map_err(|e| format!("Failed to build AppOptions: {}", e))?;

    Ok(options)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::collections::HashMap;

  #[test]
  fn test_app_config_development_defaults() {
    let config = AppConfig::development();

    // Verify basic fields
    assert_eq!(config.env_type, "development");
    assert_eq!(config.app_type, "container");
    assert_eq!(config.app_version, "1.0.0-test");
    assert_eq!(config.auth_url, "https://dev-id.getbodhi.app");
    assert_eq!(config.auth_realm, "bodhi");

    // Verify enhanced fields
    assert_eq!(
      config.oauth_client_id,
      Some("test_client_id".to_string())
    );
    assert_eq!(
      config.oauth_client_secret,
      Some("test_client_secret".to_string())
    );
    assert_eq!(config.app_status, Some("ready".to_string()));
  }

  #[test]
  fn test_app_config_with_enhanced_fields() {
    let mut env_vars = HashMap::new();
    env_vars.insert("TEST_VAR".to_string(), "test_value".to_string());

    let mut app_settings = HashMap::new();
    app_settings.insert("BODHI_PORT".to_string(), "9090".to_string());

    let config = AppConfig {
      env_type: "development".to_string(),
      app_type: "container".to_string(),
      app_version: "1.0.0-test".to_string(),
      auth_url: "https://test.example.com".to_string(),
      auth_realm: "test-realm".to_string(),
      environment_vars: Some(env_vars),
      app_settings: Some(app_settings),
      oauth_client_id: Some("test_client_id".to_string()),
      oauth_client_secret: Some("test_client_secret".to_string()),
      app_status: Some("Ready".to_string()),
    };

    // Test conversion to AppOptions
    let result = AppOptions::try_from(config);
    if let Err(e) = &result {
      println!("Conversion error: {}", e);
    }
    assert!(result.is_ok());

    let options = result.unwrap();
    assert_eq!(options.env_type, lib_bodhiserver::EnvType::Development);
    assert_eq!(options.app_type, lib_bodhiserver::AppType::Container);
    assert_eq!(options.app_version, "1.0.0-test");
    assert_eq!(options.auth_url, "https://test.example.com");
    assert_eq!(options.auth_realm, "test-realm");
  }
}
