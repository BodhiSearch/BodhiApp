use include_dir::{Dir, include_dir};

static STATIC_DIR: Dir = include_dir!("$OUT_DIR/static");

pub mod server;
pub use server::*;
pub mod cli;