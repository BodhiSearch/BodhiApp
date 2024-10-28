use crate::{
  app::{main_internal, setup_logs},
  error::BodhiError,
};
use objs::{ApiError, AppType, OpenAIApiError};
use services::{DefaultEnvService, DefaultEnvWrapper};
use std::sync::Arc;
use tracing_appender::non_blocking::WorkerGuard;

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
  let mut env_service = DefaultEnvService::new(
    ENV_TYPE.clone(),
    APP_TYPE,
    AUTH_URL.to_string(),
    AUTH_REALM.to_string(),
    Arc::new(DefaultEnvWrapper::default()),
  );
  if let Err(err) = env_service.setup_bodhi_home() {
    eprintln!("fatal error: {}\nexiting...", err);
    std::process::exit(1);
  }
  env_service.load_dotenv();
  if let Err(err) = env_service.setup_hf_cache() {
    eprintln!("fatal error: {}\nexiting...", err);
    std::process::exit(1);
  }
  let _guard = match env_service.setup_logs_dir() {
    Ok(logs_dir) => setup_logs(&logs_dir),
    Err(err) => Err::<WorkerGuard, BodhiError>(err.into()),
  };
  if _guard.is_err() {
    eprintln!("failed to configure logging, will be skipped");
  };
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
}
