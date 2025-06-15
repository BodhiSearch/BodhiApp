// production constants
#[cfg(feature = "production")]
mod env_config {
  use lib_bodhiserver::EnvType;

  pub static ENV_TYPE: EnvType = EnvType::Production;
  pub static AUTH_URL: &str = "https://id.getbodhi.app";
  pub static AUTH_REALM: &str = "bodhi";
}

// development constants
#[cfg(not(feature = "production"))]
mod env_config {
  use lib_bodhiserver::EnvType;

  pub static ENV_TYPE: EnvType = EnvType::Development;
  pub static AUTH_URL: &str = "https://dev-id.getbodhi.app";
  pub static AUTH_REALM: &str = "bodhi";
}

pub use env_config::*;
