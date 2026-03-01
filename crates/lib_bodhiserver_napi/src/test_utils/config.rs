use crate::{
  create_napi_app_options, set_env_var, set_system_setting, NapiAppOptions, BODHI_APP_TYPE,
  BODHI_AUTH_REALM, BODHI_AUTH_URL, BODHI_ENCRYPTION_KEY, BODHI_ENV_TYPE, BODHI_EXEC_LOOKUP_PATH,
  BODHI_HOME, BODHI_HOST, BODHI_LOG_LEVEL, BODHI_LOG_STDOUT, BODHI_PORT, BODHI_VERSION,
};
use rstest::fixture;
use services::test_utils::temp_dir;
use std::{collections::HashMap, path::PathBuf};
use tempfile::TempDir;

#[fixture]
pub fn test_config(temp_dir: TempDir) -> (NapiAppOptions, TempDir) {
  let bodhi_home = temp_dir.path().to_string_lossy().to_string();
  // Generate a random port for testing
  let port = {
    use rand::Rng;
    rand::rng().random_range(20000..30000)
  };

  // Get default exec lookup path for testing
  let exec_lookup_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("..")
    .join("llama_server_proc")
    .join("bin")
    .canonicalize()
    .ok()
    .unwrap()
    .display()
    .to_string();
  let mut config = create_napi_app_options();

  // Set basic configuration
  let app_db_url = format!("sqlite:{}", temp_dir.path().join("app.db").display());
  config = set_env_var(config, BODHI_HOME.to_string(), bodhi_home);
  config = set_env_var(config, BODHI_HOST.to_string(), "127.0.0.1".to_string());
  config = set_env_var(config, BODHI_PORT.to_string(), port.to_string());
  config = set_env_var(config, "BODHI_APP_DB_URL".to_string(), app_db_url);

  // Set system settings for basic app setup
  config = set_system_setting(
    config,
    BODHI_ENV_TYPE.to_string(),
    "development".to_string(),
  );
  config = set_system_setting(config, BODHI_APP_TYPE.to_string(), "container".to_string());
  config = set_system_setting(config, BODHI_VERSION.to_string(), "1.0.0-test".to_string());
  config = set_system_setting(
    config,
    BODHI_AUTH_URL.to_string(),
    "https://test-id.getbodhi.app".to_string(),
  );
  config = set_system_setting(config, BODHI_AUTH_REALM.to_string(), "bodhiapp".to_string());

  // Set optional configurations
  config = set_env_var(config, BODHI_EXEC_LOOKUP_PATH.to_string(), exec_lookup_path);

  config = set_env_var(config, BODHI_LOG_LEVEL.to_string(), "info".to_string());
  config = set_env_var(config, BODHI_LOG_STDOUT.to_string(), "true".to_string());
  config = set_env_var(
    config,
    BODHI_ENCRYPTION_KEY.to_string(),
    "test-encryption-key".to_string(),
  );
  (config, temp_dir)
}

/// Create a test configuration with custom settings
pub fn test_config_with_settings(
  mut config: NapiAppOptions,
  settings: HashMap<String, String>,
) -> NapiAppOptions {
  for (key, value) in settings {
    match key.as_str() {
      "host" => {
        config = set_env_var(config, BODHI_HOST.to_string(), value);
      }
      "port" => {
        config = set_env_var(config, BODHI_PORT.to_string(), value);
      }
      "log_level" => {
        config = set_env_var(config, BODHI_LOG_LEVEL.to_string(), value);
      }
      "log_stdout" => {
        config = set_env_var(config, BODHI_LOG_STDOUT.to_string(), value);
      }
      _ => {
        config = set_env_var(config, key, value);
      }
    }
  }

  config
}
