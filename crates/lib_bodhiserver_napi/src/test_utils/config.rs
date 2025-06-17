use objs::{AppType, EnvType};
use services::{BODHI_ENCRYPTION_KEY, BODHI_EXEC_LOOKUP_PATH, BODHI_PORT};

use crate::AppConfig;

impl AppConfig {
  /// Creates a development configuration with sensible defaults
  pub fn development() -> Self {
    let mut environment_vars = std::collections::HashMap::new();
    environment_vars.insert(
      BODHI_ENCRYPTION_KEY.to_string(),
      "test-encryption-key".to_string(),
    );
    environment_vars.insert(BODHI_EXEC_LOOKUP_PATH.to_string(), "/tmp".to_string());
    environment_vars.insert(BODHI_PORT.to_string(), "54321".to_string());
    Self {
      env_type: EnvType::Development.to_string(),
      app_type: AppType::Container.to_string(),
      app_version: "1.0.0-test".to_string(),
      auth_url: "https://dev-id.getbodhi.app".to_string(),
      auth_realm: "bodhi".to_string(),
      environment_vars: Some(environment_vars),
      app_settings: None,
      oauth_client_id: Some("test_client_id".to_string()),
      oauth_client_secret: Some("test_client_secret".to_string()),
      app_status: Some("ready".to_string()),
    }
  }
}
