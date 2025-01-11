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
  let project_dir =
    std::env::var("CARGO_MANIFEST_DIR").context("failed to get CARGO_MANIFEST_DIR")?;
  let bodhiapp_dir = fs::canonicalize(PathBuf::from(project_dir).join(".."))
    .context("error canonicalizing bodhi project path")?;
  if cfg!(debug_assertions) {
    // build only if production build `tauri_build::build()` is already running npm run build, so only run if not production
    println!("cargo:rerun-if-changed=../src");
    run_make_command("build_frontend")?;
  }
  copy_frontend(&bodhiapp_dir)?;
  run_make_command("copy_bins")?;
  #[cfg(feature = "native")]
  tauri_build::build();
  Ok(())
}

fn run_make_command(target: &str) -> anyhow::Result<()> {
  let mut makefile_args: Vec<&str> = get_makefile_args();
  makefile_args.push(target);
  let status = if cfg!(windows) {
    Command::new("pwsh")
      .args(["-Command", &format!("make {}", makefile_args.join(" "))])
      .status()
  } else {
    Command::new("make").args(&makefile_args).status()
  }
  .context(format!("failed to execute make command for {target}"))?;

  if !status.success() {
    let platform = if cfg!(windows) { "Windows" } else { "Linux" };
    bail!("make command `{target}` failed on {platform}");
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
