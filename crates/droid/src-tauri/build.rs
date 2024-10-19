use std::{env, path::PathBuf};

use anyhow::Context;
use fs_extra::dir::CopyOptions;

type Result<T> = anyhow::Result<T>;

fn main() -> Result<()> {
  tauri_build::build();
  copy_lib()?;
  Ok(())
}

fn copy_lib() -> Result<()> {
  let target = env::var("TARGET").unwrap();
  if target == "aarch64-linux-android" {
    copy_android_libs(&target)?;
  }
  Ok(())
}

fn copy_android_libs(target: &str) -> Result<()> {
  let src_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("../../../llamacpp-sys/libs")
    .join(target);
  let dest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("libs")
    .join(target);
  fs_extra::dir::copy(src_path, dest_dir, &{
    let mut options = CopyOptions::new();
    options.copy_inside = true;
    options.overwrite = true;
    options
  })
  .context("Failed to copy libraries from llamacpp-sys")?;
  Ok(())
}
