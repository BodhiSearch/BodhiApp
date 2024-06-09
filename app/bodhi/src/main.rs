// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bodhi::{main_internal, setup_logs};
use bodhicore::service::InitService;

pub fn main() {
  let init_service = InitService::new();
  let bodhi_home = match init_service.bodhi_home() {
    Ok(bodhi_home) => bodhi_home,
    Err(err) => {
      eprintln!("fatal error: {}\nexiting...", err);
      std::process::exit(1);
    }
  };
  let hf_cache = match init_service.hf_cache() {
    Ok(hf_cache) => hf_cache,
    Err(err) => {
      eprintln!("fatal error: {}\nexiting...", err);
      std::process::exit(1);
    }
  };
  let envfile = bodhi_home.join(".env");
  if envfile.exists() {
    match dotenv::from_path(&envfile) {
      Ok(_) => {}
      Err(err) => eprintln!(
        "error loading .env file. err: {}, path: {}",
        err,
        envfile.display()
      ),
    }
  }
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
