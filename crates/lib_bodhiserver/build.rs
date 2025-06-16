use anyhow::{bail, Context};
use std::{
  env,
  path::{Path, PathBuf},
  process::Command,
};

fn main() {
  if let Err(e) = _main() {
    panic!("Build script failed: {}", e);
  }
}

fn _main() -> anyhow::Result<()> {
  let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let bodhi_dir = manifest_dir.join("../bodhi");

  // Build frontend
  build_frontend(&bodhi_dir)?;

  // Validate assets exist
  validate_frontend_assets(&bodhi_dir)?;

  // Set rerun conditions
  println!("cargo:rerun-if-changed=../bodhi/src");
  println!("cargo:rerun-if-changed=../bodhi/package.json");
  println!("cargo:rerun-if-changed=../bodhi/next.config.js");

  Ok(())
}

fn build_frontend(bodhi_dir: &Path) -> anyhow::Result<()> {
  println!("cargo:warning=Building frontend in {:?}", bodhi_dir);

  // Install dependencies
  let status = Command::new("npm")
    .args(["install"])
    .current_dir(bodhi_dir)
    .status()
    .context("Failed to run npm install")?;

  if !status.success() {
    bail!("npm install failed");
  }

  // Build frontend
  let status = Command::new("npm")
    .args(["run", "build"])
    .current_dir(bodhi_dir)
    .status()
    .context("Failed to run npm build")?;

  if !status.success() {
    bail!("npm build failed");
  }

  Ok(())
}

fn validate_frontend_assets(bodhi_dir: &Path) -> anyhow::Result<()> {
  let out_dir = bodhi_dir.join("out");

  if !out_dir.exists() {
    bail!(
      "Frontend build output directory does not exist: {:?}",
      out_dir
    );
  }

  // Check for essential files
  let index_html = out_dir.join("index.html");
  if !index_html.exists() {
    bail!("index.html not found in build output");
  }

  println!("cargo:warning=Frontend assets validated successfully");
  Ok(())
}
