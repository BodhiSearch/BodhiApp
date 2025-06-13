use anyhow::{bail, Context};
use fs_extra::dir::CopyOptions;
use std::{
  env,
  fs::{self, File},
  path::{Path, PathBuf},
  process::Command,
  thread,
  time::Duration,
};

const LOCK_FILE: &str = "bodhi-build.lock";

fn main() {
  _main().unwrap();
}

fn _main() -> anyhow::Result<()> {
  let project_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let bodhiapp_dir =
    fs::canonicalize(project_dir.join("..")).context("error canonicalizing bodhi project path")?;
  if cfg!(debug_assertions) {
    // build only if production build `tauri_build::build()` is already running npm run build, so only run if not production
    println!("cargo:rerun-if-changed=../src");
    run_make_command("build_frontend")?;
  }
  copy_frontend(&bodhiapp_dir)?;
  copy_llama_bins(&project_dir)?;
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

fn copy_llama_bins(project_dir: &Path) -> Result<(), anyhow::Error> {
  let llama_server_dir = project_dir.join("../../llama_server_proc");

  if !llama_server_dir.exists() {
    bail!("Source directory '../../llama_server_proc' not found");
  }

  // Try to acquire lock from llama_server_proc/bin/.lock
  let lock_path = llama_server_dir.join("bin").join(LOCK_FILE);
  let lock_file = File::open(&lock_path)
    .context("Failed to open lock file - ensure llama_server_proc has been built")?;

  // Try to acquire the lock, retry if locked
  let max_attempts = 60; // Maximum 60 seconds wait
  let mut attempts = 0;
  while let Err(e) = fs2::FileExt::try_lock_shared(&lock_file) {
    if attempts >= max_attempts {
      bail!("Timeout waiting for llama server bin lock: {}", e);
    }
    println!("Waiting for llama server bin lock...");
    thread::sleep(Duration::from_secs(1));
    attempts += 1;
  }
  println!("Acquired llama server bin lock");

  // Perform the copy operation
  try_copy_bins(project_dir, &llama_server_dir)?;

  // Sign binaries if in CI and on macOS
  if cfg!(target_os = "macos") && is_ci() {
    sign_binaries()?;
  }

  // Lock will be automatically released when lock_file is dropped
  Ok(())
}

fn is_ci() -> bool {
  env::var("CI").map(|v| v == "true").unwrap_or(false)
    && env::var("CI_RELEASE").map(|v| v == "true").unwrap_or(false)
}

fn sign_binaries() -> Result<(), anyhow::Error> {
  println!("Signing llama-server binaries for macOS...");

  // Check if we're in CI and have required environment variables
  for var in &[
    "APPLE_CERTIFICATE",
    "APPLE_CERTIFICATE_PASSWORD",
    "APPLE_SIGNING_IDENTITY",
  ] {
    if env::var(var).is_err() {
      bail!("Required environment variable {} not set for signing", var);
    }
  }

  // Run the make command for signing
  let status = Command::new("make")
    .arg("ci.sign-binaries")
    .current_dir(env!("CARGO_MANIFEST_DIR"))
    .status()
    .context("Failed to execute make ci.sign-binaries command")?;

  if !status.success() {
    bail!("Failed to sign binaries using make ci.sign-binaries");
  }

  println!("Successfully signed llama-server binaries");
  Ok(())
}

fn try_copy_bins(project_dir: &Path, llama_server_dir: &Path) -> Result<(), anyhow::Error> {
  let bin_dir = project_dir.join("bin");
  // Delete the bin directory if it exists
  if bin_dir.exists() {
    fs::remove_dir_all(&bin_dir).context("Failed to delete existing bin directory")?;
  }
  let source_bin_dir = llama_server_dir.join("bin");

  // Create destination directory if it doesn't exist
  fs::create_dir_all(&bin_dir).context("Failed to create bin directory")?;

  // Copy each file/directory except the lock file
  for entry in fs::read_dir(&source_bin_dir).context("Failed to read source bin directory")? {
    let entry = entry?;
    let path = entry.path();
    let file_name = path.file_name().unwrap();

    if file_name != LOCK_FILE {
      let dest_path = bin_dir.join(file_name);
      if path.is_dir() {
        fs_extra::dir::copy(&path, &bin_dir, &{
          let mut options = CopyOptions::new();
          options.overwrite = true;
          options
        })
        .context("Failed to copy directory")?;
      } else {
        fs::copy(&path, dest_path).context("Failed to copy file")?;
      }
    }
  }

  Ok(())
}
