use anyhow::Context;
use fs_extra::dir::CopyOptions;
use std::{
  ffi::OsStr,
  fs,
  path::{Path, PathBuf},
  process::Command,
};

fn main() -> anyhow::Result<()> {
  // println!("cargo:rerun-if-changed=../app");
  let project_dir =
    std::env::var("CARGO_MANIFEST_DIR").context("failed to get CARGO_MANIFEST_DIR")?;
  let bodhiui_dir = fs::canonicalize(PathBuf::from(project_dir).join(".."))
    .context("error canocilizing bodhiui path")?;
  exec_command(
    &bodhiui_dir,
    "pnpm",
    ["install"],
    "error running `pnpm install` on bodhiui",
  )?;
  exec_command(
    &bodhiui_dir,
    "pnpm",
    ["run", "build"],
    "error running `pnpm run build` on bodhiui",
  )?;

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
