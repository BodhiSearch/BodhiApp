// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod internal;

#[tokio::main]
async fn main() {
  internal::main_native();
}
