use lib_bodhiserver::DefaultEnvWrapper;
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
  /// Optional custom Bodhi home directory
  pub bodhi_home: Option<String>,
  /// Optional encryption key for testing (bypasses keyring)
  pub encryption_key: Option<String>,
  /// Optional exec lookup path for testing
  pub exec_lookup_path: Option<String>,
  /// Optional port configuration for testing
  pub port: Option<u16>,
}

impl AppConfig {
  /// Creates a development configuration with sensible defaults
  pub fn development() -> Self {
    Self {
      env_type: "Development".to_string(),
      app_type: "Container".to_string(),
      app_version: "1.0.0-test".to_string(),
      auth_url: "https://dev-id.getbodhi.app".to_string(),
      auth_realm: "bodhi".to_string(),
      bodhi_home: None,
      encryption_key: Some("test-encryption-key".to_string()),
      exec_lookup_path: Some("/tmp".to_string()), // Default to /tmp for testing
      port: Some(54321),                          // Default test port
    }
  }
}

impl TryFrom<AppConfig> for lib_bodhiserver::AppOptions {
  type Error = String;

  fn try_from(config: AppConfig) -> Result<Self, Self::Error> {
    use lib_bodhiserver::{
      AppOptionsBuilder, AppType, EnvType, EnvWrapper, BODHI_ENCRYPTION_KEY,
      BODHI_EXEC_LOOKUP_PATH, BODHI_HOME, BODHI_PORT,
    };
    use std::str::FromStr;
    use std::sync::Arc;

    let mut env_wrapper = DefaultEnvWrapper::default();

    // Set bodhi_home if provided
    if let Some(bodhi_home) = config.bodhi_home {
      env_wrapper.set_var(BODHI_HOME, &bodhi_home);
    }

    // Set encryption key if provided (for testing)
    if let Some(encryption_key) = config.encryption_key {
      env_wrapper.set_var(BODHI_ENCRYPTION_KEY, &encryption_key);
    }

    // Set exec lookup path if provided
    if let Some(exec_lookup_path) = config.exec_lookup_path {
      env_wrapper.set_var(BODHI_EXEC_LOOKUP_PATH, &exec_lookup_path);
    }

    // Set port if provided
    if let Some(port) = config.port {
      env_wrapper.set_var(BODHI_PORT, &port.to_string());
    }

    let env_wrapper: Arc<dyn EnvWrapper> = Arc::new(env_wrapper);
    let env_type =
      EnvType::from_str(&config.env_type).map_err(|e| format!("Invalid env_type: {}", e))?;
    let app_type =
      AppType::from_str(&config.app_type).map_err(|e| format!("Invalid app_type: {}", e))?;

    let builder = AppOptionsBuilder::default()
      .env_wrapper(env_wrapper)
      .env_type(env_type)
      .app_type(app_type)
      .app_version(config.app_version)
      .auth_url(config.auth_url)
      .auth_realm(config.auth_realm)
      .build()
      .map_err(|e| format!("Failed to build AppOptions: {}", e))?;

    Ok(builder)
  }
}
