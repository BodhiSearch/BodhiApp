#![allow(unused)]

use std::{env, path::PathBuf, thread, time::Duration};

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
  let src_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("../../../llamacpp-sys/libs")
    .join(target)
    .join("cpu")
    .join("libbodhi-server.so");
  if !src_file.exists() {
    return Err(anyhow::anyhow!(
      "source file does not exist {}",
      src_file.display()
    ));
  }
  let dest_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("gen/android/app/src/main/jniLibs/arm64-v8a/libbodhi-server.so");
  std::fs::copy(src_file, dest_file)?;
  Ok(())
}
