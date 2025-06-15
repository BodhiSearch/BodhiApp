use crate::env::{AUTH_REALM, AUTH_URL, ENV_TYPE};
use lib_bodhiserver::{AppOptions, AppOptionsBuilder};
use objs::{AppType, ErrorMessage, ErrorType};
use services::EnvWrapper;
use std::sync::Arc;

pub fn build_app_options(
  env_wrapper: Arc<dyn EnvWrapper>,
  app_type: AppType,
) -> Result<AppOptions, ErrorMessage> {
  AppOptionsBuilder::default()
    .env_wrapper(env_wrapper.clone())
    .env_type(ENV_TYPE.clone())
    .app_type(app_type)
    .app_version(env!("CARGO_PKG_VERSION"))
    .auth_url(AUTH_URL)
    .auth_realm(AUTH_REALM)
    .build()
    .map_err(|err| {
      ErrorMessage::new(
        "app_options_builder_error".to_string(),
        ErrorType::InternalServer.to_string(),
        err.to_string(),
      )
    })
}
