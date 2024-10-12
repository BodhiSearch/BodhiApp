// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bodhi::{main_internal, setup_logs, BodhiError};
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

pub fn main() {
  let mut env_service = DefaultEnvService::new(
    ENV_TYPE.clone(),
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
    eprintln!("fatal error: {}\nexiting...", err);
    std::process::exit(1);
  } else {
    tracing::info!("application exited with success");
  }
}
