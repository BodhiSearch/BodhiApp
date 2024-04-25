// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(feature = "native_app")]
mod internal;

#[cfg(not(feature = "native_app"))]
mod internal {
  pub(crate) fn main_native() {
    panic!("native_app feature is not enabled")
  }
}

#[tokio::main]
async fn main() {
  internal::main_native();
}
