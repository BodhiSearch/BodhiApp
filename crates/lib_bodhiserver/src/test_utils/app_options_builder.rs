use crate::AppOptionsBuilder;
use objs::{AppType, EnvType};
use services::BODHI_HOME;

impl AppOptionsBuilder {
  /// Creates a development configuration builder for testing
  pub fn development() -> Self {
    Self::default()
      .env_type(EnvType::Development)
      .app_type(AppType::Container)
      .app_version(env!("CARGO_PKG_VERSION"))
      .auth_url("https://test-id.getbodhi.app")
      .auth_realm("bodhi")
  }

  /// Creates a builder with a specific bodhi home directory for testing
  pub fn with_bodhi_home(bodhi_home: &str) -> Self {
    Self::development().set_env(BODHI_HOME, bodhi_home)
  }
}
