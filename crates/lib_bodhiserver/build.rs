use anyhow::{bail, Context};
use std::{
  env,
  path::{Path, PathBuf},
  process::{Command, Stdio},
};

fn main() {
  if let Err(e) = _main() {
    panic!("Build script failed: {}", e);
  }
}

fn _main() -> anyhow::Result<()> {
  let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let bodhi_dir = manifest_dir.join("../bodhi");
  let project_root = manifest_dir.join("../..");
  let out_dir = bodhi_dir.join("out");
  // Build frontend
  if is_ci() || !out_dir.exists() {
    ensure_ts_client_built(&project_root)?;
    build_frontend(&bodhi_dir)?;
  }
  // Validate assets exist
  validate_frontend_assets(&bodhi_dir)?;

  // Set rerun conditions
  // println!("cargo:rerun-if-changed=../bodhi/src");
  // println!("cargo:rerun-if-changed=../bodhi/package.json");
  // println!("cargo:rerun-if-changed=../bodhi/next.config.js");

  Ok(())
}

fn is_ci() -> bool {
  env::var("CI").map(|v| v == "true").unwrap_or(false)
}

#[allow(unused)]
fn build_frontend(bodhi_dir: &Path) -> anyhow::Result<()> {
  println!("cargo:warning=Building frontend in {:?}", bodhi_dir);

  // Install dependencies
  let status = create_npm_command()
    .args(["install"])
    .current_dir(bodhi_dir)
    .stdout(Stdio::inherit())
    .stderr(Stdio::inherit())
    .status()
    .context("Failed to run npm install")?;

  if !status.success() {
    bail!("npm install failed");
  }

  // Build frontend
  let status = create_npm_command()
    .args(["run", "build"])
    .current_dir(bodhi_dir)
    .status()
    .context("Failed to run npm build")?;

  if !status.success() {
    bail!("npm build failed");
  }

  Ok(())
}

fn create_npm_command() -> Command {
  if cfg!(target_os = "windows") {
    let mut cmd = Command::new("cmd");
    cmd.args(["/C", "npm"]);
    cmd
  } else {
    Command::new("npm")
  }
}

fn validate_frontend_assets(bodhi_dir: &Path) -> anyhow::Result<()> {
  let out_dir = bodhi_dir.join("out");
  if !out_dir.exists() {
    bail!(
      "Frontend build output directory does not exist: {:?}",
      out_dir
    );
  }
  Ok(())
}

fn ensure_ts_client_built(project_root: &Path) -> anyhow::Result<()> {
  let ts_client_dir = project_root.join("ts-client");
  let ts_client_dist = ts_client_dir.join("dist");

  if !ts_client_dist.exists() {
    println!("cargo:warning=ts-client dist directory not found, building ts-client");
    build_ts_client(&ts_client_dir)?;
  }

  Ok(())
}

fn build_ts_client(ts_client_dir: &Path) -> anyhow::Result<()> {
  println!("cargo:warning=Building ts-client in {:?}", ts_client_dir);

  // Install dependencies
  let status = create_npm_command()
    .args(["install"])
    .current_dir(ts_client_dir)
    .stdout(Stdio::inherit())
    .stderr(Stdio::inherit())
    .status()
    .context("Failed to run npm install in ts-client")?;

  if !status.success() {
    bail!("npm install failed in ts-client");
  }

  // Build ts-client with OpenAPI generation
  let status = create_npm_command()
    .args(["run", "build:openapi"])
    .current_dir(ts_client_dir)
    .stdout(Stdio::inherit())
    .stderr(Stdio::inherit())
    .status()
    .context("Failed to run npm run build:openapi in ts-client")?;

  if !status.success() {
    bail!("npm run build:openapi failed in ts-client");
  }

  Ok(())
}
