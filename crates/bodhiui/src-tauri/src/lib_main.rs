use crate::app::{main_internal, setup_logs};
use objs::{ApiError, AppType, OpenAIApiError};
use services::{DefaultEnvService, DefaultEnvWrapper, EnvService, InitService};
use std::sync::Arc;

#[cfg(feature = "production")]
mod env_config {
  use objs::EnvType;

  pub static ENV_TYPE: EnvType = EnvType::Production;
  pub static AUTH_URL: &str = "https://id.getbodhi.app";
  pub static AUTH_REALM: &str = "bodhi";
}

#[cfg(not(feature = "production"))]
mod env_config {
  use objs::EnvType;

  pub static ENV_TYPE: EnvType = EnvType::Development;
  pub static AUTH_URL: &str = "https://dev-id.getbodhi.app";
  pub static AUTH_REALM: &str = "bodhi";
}

pub use env_config::*;

#[cfg(feature = "native")]
pub const APP_TYPE: AppType = AppType::Native;

#[cfg(not(feature = "native"))]
pub const APP_TYPE: AppType = AppType::Container;

pub fn _main() {
  let mut env_wrapper = DefaultEnvWrapper::default();
  let (bodhi_home, hf_home, logs_dir) = match InitService::new(&mut env_wrapper, &ENV_TYPE).setup()
  {
    Ok(paths) => paths,
    Err(err) => {
      let api_error: ApiError = err.into();
      eprintln!(
        "fatal error, setting up app dirs, error: {}\nexiting...",
        api_error
      );
      std::process::exit(1);
    }
  };
  let env_service = match DefaultEnvService::new(
    bodhi_home,
    hf_home,
    logs_dir,
    ENV_TYPE.clone(),
    APP_TYPE.clone(),
    AUTH_URL.to_string(),
    AUTH_REALM.to_string(),
    Arc::new(env_wrapper),
  ) {
    Ok(env_service) => env_service,
    Err(err) => {
      let api_error: ApiError = err.into();
      eprintln!(
        "fatal error, setting up environment service, error: {}\nexiting...",
        api_error
      );
      std::process::exit(1);
    }
  };
  // let _guard = setup_logs(&env_service.logs_dir());
  // if _guard.is_err() {
  //   eprintln!("failed to configure logging, will be skipped");
  // };
  let result = main_internal(Arc::new(env_service));
  if let Err(err) = result {
    tracing::warn!(?err, "application exited with error");
    let err: ApiError = err.into();
    let err: OpenAIApiError = err.into();
    eprintln!("fatal error: {}\nexiting...", err);
    std::process::exit(1);
  } else {
    tracing::info!("application exited with success");
  }
  // drop(_guard);
}
