use anyhow::{bail, Context};
use fs_extra::dir::CopyOptions;
use std::{
  ffi::OsStr,
  fs,
  path::{Path, PathBuf},
  process::Command,
};

fn main() -> anyhow::Result<()> {
  #[cfg(feature = "native")]
  tauri_build::build();
  let project_dir =
    std::env::var("CARGO_MANIFEST_DIR").context("failed to get CARGO_MANIFEST_DIR")?;
  let bodhiui_dir = fs::canonicalize(PathBuf::from(project_dir).join(".."))
    .context("error canocilizing bodhiui path")?;
  if cfg!(debug_assertions) {
    // build only if non-production build, as `tauri_build::build()` is already doing the job
    println!("cargo:rerun-if-changed=../src");
    build_frontend(&bodhiui_dir)?;
  }
  copy_frontend(&bodhiui_dir)?;
  let llamacpp_sys_libs = copy_libs()?;
  println!("cargo:rerun-if-changed={}", llamacpp_sys_libs.display());
  Ok(())
}

fn copy_libs() -> anyhow::Result<PathBuf> {
  let project_dir =
    std::env::var("CARGO_MANIFEST_DIR").context("failed to get CARGO_MANIFEST_DIR")?;
  let llamacpp_sys = PathBuf::from(&project_dir).join("../../../llamacpp-sys/libs");
  if !llamacpp_sys.exists() {
    bail!(
      "{} directory does not exist, did you forget to checkout the submodule?",
      llamacpp_sys.display()
    );
  }
  let llamacpp_sys_libs =
    fs::canonicalize(llamacpp_sys).context("error canocilizing llamacpp-sys path")?;
  let dest_dir = PathBuf::from(&project_dir).join("libs");
  fs_extra::dir::copy(&llamacpp_sys_libs, &dest_dir, &{
    let mut options = CopyOptions::new();
    options.copy_inside = true;
    options.skip_exist = false;
    options.overwrite = true;
    options.content_only = true;
    options
  })
  .context("failed to copy libs")?;
  Ok(llamacpp_sys_libs)
}

fn build_frontend(bodhiui_dir: &Path) -> anyhow::Result<()> {
  exec_command(
    bodhiui_dir,
    "pnpm",
    ["install"],
    "error running `npm install` on bodhiui",
  )?;
  exec_command(
    bodhiui_dir,
    "pnpm",
    ["run", "build"],
    "error running `npm run build` on bodhiui",
  )?;
  Ok(())
}

fn copy_frontend(bodhiui_dir: &Path) -> Result<(), anyhow::Error> {
  let out_dir = std::env::var("OUT_DIR").context("Failed to get OUT_DIR environment variable")?;
  let out_dir = Path::new(&out_dir);
  let dest_dir = out_dir.join("static");
  let source_dir = bodhiui_dir.join("out");
  fs_extra::dir::copy(source_dir, dest_dir, &{
    let mut options = CopyOptions::new();
    options.copy_inside = true;
    options.overwrite = true;
    options
  })
  .context("Failed to copy directory to OUT_DIR")?;
  Ok(())
}

fn exec_command<I, S>(cwd: &Path, cmd: &str, args: I, err_msg: &str) -> anyhow::Result<()>
where
  I: IntoIterator<Item = S>,
  S: AsRef<OsStr>,
{
  Command::new(cmd)
    .current_dir(cwd)
    .args(args)
    .status()
    .context(err_msg.to_string())?
    .success()
    .then_some(())
    .context(err_msg.to_string())?;
  Ok(())
}
