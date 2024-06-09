// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bodhi::{main_internal, setup_logs};
use bodhicore::service::EnvService;

pub fn main() {
  let env_service = EnvService::new();
  env_service.load_dotenv();
  let bodhi_home = match env_service.bodhi_home() {
    Ok(bodhi_home) => bodhi_home,
    Err(err) => {
      eprintln!("fatal error: {}\nexiting...", err);
      std::process::exit(1);
    }
  };
  let hf_cache = match env_service.hf_cache() {
    Ok(hf_cache) => hf_cache,
    Err(err) => {
      eprintln!("fatal error: {}\nexiting...", err);
      std::process::exit(1);
    }
  };
  let _guard = setup_logs(&bodhi_home);
  if _guard.is_err() {
    eprintln!("failed to configure logging, will be skipped");
  };
  let result = main_internal(bodhi_home, hf_cache);
  if let Err(err) = result {
    tracing::warn!(?err, "application exited with error");
    eprintln!("fatal error: {}\nexiting...", err);
    std::process::exit(1);
  } else {
    tracing::info!("application exited with success");
  }
}
