// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(feature = "native")]
fn main() {
  app_lib::native::run();
}

#[cfg(not(feature = "native"))]
fn main() {
  println!("BodhiUI is not built with native support");
}
