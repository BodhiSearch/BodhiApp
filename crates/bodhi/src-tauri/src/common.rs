use crate::env::{AUTH_REALM, AUTH_URL, ENV_TYPE};
use lib_bodhiserver::{AppOptions, AppOptionsBuilder, AppType, ErrorMessage};

pub fn build_app_options(app_type: AppType) -> Result<AppOptions, ErrorMessage> {
  Ok(
    AppOptionsBuilder::default()
      .env_type(ENV_TYPE.clone())
      .app_type(app_type)
      .app_version(env!("CARGO_PKG_VERSION"))
      .auth_url(AUTH_URL)
      .auth_realm(AUTH_REALM)
      .build()?,
  )
}
