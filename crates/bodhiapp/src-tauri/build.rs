use anyhow::{bail, Context};
use fs_extra::dir::CopyOptions;
use std::{
  fs,
  path::{Path, PathBuf},
  process::Command,
};

fn main() {
  _main().unwrap();
}

fn _main() -> anyhow::Result<()> {
  #[cfg(feature = "native")]
  tauri_build::build();
  let project_dir =
    std::env::var("CARGO_MANIFEST_DIR").context("failed to get CARGO_MANIFEST_DIR")?;
  let bodhiapp_dir = fs::canonicalize(PathBuf::from(project_dir).join(".."))
    .context("error canonicalizing bodhiapp path")?;
  if cfg!(debug_assertions) {
    // build only if non-production build, as `tauri_build::build()` is already doing the job
    println!("cargo:rerun-if-changed=../src");
    build_frontend()?;
  }
  copy_frontend(&bodhiapp_dir)?;
  let llamacpp_sys_libs = copy_libs()?;
  println!("cargo:rerun-if-changed={}", llamacpp_sys_libs.display());
  Ok(())
}

fn copy_libs() -> anyhow::Result<PathBuf> {
  let project_dir =
    std::env::var("CARGO_MANIFEST_DIR").context("failed to get CARGO_MANIFEST_DIR")?;
  let llamacpp_sys = PathBuf::from(&project_dir)
    .join("..")
    .join("..")
    .join("..")
    .join("llamacpp-sys")
    .join("libs");
  // if !llamacpp_sys.exists() {
  //   bail!(
  //     "{} directory does not exist, did you forget to checkout the submodule?",
  //     llamacpp_sys.display()
  //   );
  // }
  let dest_dir = PathBuf::from(&project_dir).join("libs");
  fs_extra::dir::copy(&llamacpp_sys, &dest_dir, &{
    let mut options = CopyOptions::new();
    options.copy_inside = true;
    options.skip_exist = false;
    options.overwrite = true;
    options.content_only = true;
    options
  })
  .context("failed to copy libs")?;
  Ok(llamacpp_sys.to_path_buf())
}

fn build_frontend() -> anyhow::Result<()> {
  let mut makefile_args: Vec<&str> = get_makefile_args();
  makefile_args.push("build_frontend");
  let status = if cfg!(windows) {
    Command::new("pwsh")
      .args(["-Command", &format!("make {}", makefile_args.join(" "))])
      .status()
  } else {
    Command::new("make").args(&makefile_args).status()
  }
  .expect("failed to execute make command for clean");
  if !status.success() {
    let platform = if cfg!(windows) { "Windows" } else { "Linux" };
    bail!("make command `build_frontend` failed on {platform}");
  }
  Ok(())
}

fn get_makefile_args() -> Vec<&'static str> {
  if cfg!(windows) {
    vec!["-f", "Makefile.win.mk"]
  } else {
    vec![]
  }
}

fn copy_frontend(bodhiapp_dir: &Path) -> Result<(), anyhow::Error> {
  let out_dir = std::env::var("OUT_DIR").context("Failed to get OUT_DIR environment variable")?;
  let out_dir = Path::new(&out_dir);
  let dest_dir = out_dir.join("static");
  let source_dir = bodhiapp_dir.join("out");
  fs_extra::dir::copy(source_dir, dest_dir, &{
    let mut options = CopyOptions::new();
    options.copy_inside = true;
    options.overwrite = true;
    options
  })
  .context("Failed to copy directory to OUT_DIR")?;
  Ok(())
}
