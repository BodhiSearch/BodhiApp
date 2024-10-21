// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;

use objs::EnvType;
use services::{DefaultEnvService, DefaultEnvWrapper};

fn main() {
  let mut env_service = DefaultEnvService::new(
    EnvType::Development,
    "".to_string(),
    "".to_string(),
    Arc::new(DefaultEnvWrapper::default()),
  );
  if let Err(err) = env_service.setup_bodhi_home() {
    eprintln!("fatal error: {}\nexiting...", err);
    std::process::exit(1);
  }
  app_lib::run();
}
