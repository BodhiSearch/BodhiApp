use crate::AppOptionsBuilder;
use objs::{AppType, EnvType};
use services::test_utils::EnvWrapperStub;
use std::{collections::HashMap, sync::Arc};

impl AppOptionsBuilder {
  /// Creates a basic test AppOptionsBuilder with development defaults.
  /// Uses an empty environment wrapper and standard test values.
  pub fn development() -> AppOptionsBuilder {
    let env_wrapper: Arc<dyn services::EnvWrapper> = Arc::new(EnvWrapperStub::new(HashMap::new()));
    AppOptionsBuilder::default()
      .env_wrapper(env_wrapper)
      .env_type(EnvType::Development)
      .app_type(AppType::Container)
      .app_version("1.0.0")
      .auth_url("https://dev-id.getbodhi.app")
      .auth_realm("bodhi")
      .to_owned()
  }

  /// Creates AppOptionsBuilder with custom environment variables.
  /// Useful for testing specific environment configurations.
  pub fn with_env(env_vars: HashMap<String, String>) -> AppOptionsBuilder {
    let env_wrapper: Arc<dyn services::EnvWrapper> = Arc::new(EnvWrapperStub::new(env_vars));
    Self::development().env_wrapper(env_wrapper).to_owned()
  }

  /// Creates AppOptionsBuilder with a specific BODHI_HOME environment variable.
  /// Commonly used in tests that need a specific home directory.
  pub fn with_bodhi_home(bodhi_home: &str) -> AppOptionsBuilder {
    let env_vars = HashMap::from([("BODHI_HOME".to_string(), bodhi_home.to_string())]);
    Self::with_env(env_vars)
  }
}
