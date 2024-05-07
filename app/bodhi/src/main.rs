// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bodhi::{main_internal, setup_logs};

pub fn main() {
  dotenv::dotenv().ok();
  let _guard = setup_logs();
  if _guard.is_err() {
    eprintln!("failed to configure logging, will be skipped");
  };
  let result = main_internal();
  if let Err(err) = result {
    tracing::warn!(err = ?err, "application exited with error");
    eprintln!("application encountered an error: {:?}\nexiting...", err);
    std::process::exit(1);
  } else {
    tracing::info!("application exited with success");
  }
}
