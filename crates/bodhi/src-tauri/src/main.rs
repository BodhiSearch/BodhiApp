// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use app_lib::app;

pub fn main() {
  app::main(std::env::args().collect::<Vec<_>>().as_slice());
}
