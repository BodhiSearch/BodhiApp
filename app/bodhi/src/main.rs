// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;

use bodhi::{main_internal, setup_logs, AppError};
use bodhicore::service::{env_wrapper::EnvWrapper, EnvService};
use tracing_appender::non_blocking::WorkerGuard;

pub fn main() {
  let mut env_service = EnvService::new(EnvWrapper::default());
  match env_service.setup_bodhi_home() {
    Ok(bodhi_home) => bodhi_home,
    Err(err) => {
      eprintln!("fatal error: {}\nexiting...", err);
      std::process::exit(1);
    }
  };
  env_service.load_dotenv();
  match env_service.setup_hf_cache() {
    Ok(hf_cache) => hf_cache,
    Err(err) => {
      eprintln!("fatal error: {}\nexiting...", err);
      std::process::exit(1);
    }
  };
  let _guard = match env_service.setup_logs_dir() {
    Ok(logs_dir) => setup_logs(&logs_dir),
    Err(err) => Err::<WorkerGuard, AppError>(err.into()),
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
