use anyhow::{bail, Context, Result};
use fs2::FileExt;
use once_cell::sync::Lazy;
use std::{
  collections::HashSet,
  env,
  fs::{self, File},
  path::{Path, PathBuf},
  process::{Command, Stdio},
  thread,
  time::{Duration, Instant},
};

const LOCK_FILE: &str = "bodhi-build.lock";
const LOCK_TIMEOUT_SECS: u64 = 180;

static LLAMA_SERVER_BUILDS: Lazy<HashSet<LlamaServerBuild>> = Lazy::new(|| {
  let mut set = HashSet::new();
  set.insert(LlamaServerBuild::new(
    "aarch64-apple-darwin",
    "",
    vec!["metal", "cpu"],
  ));
  set.insert(LlamaServerBuild::new(
    "aarch64-unknown-linux-gnu",
    "",
    vec!["cpu"],
  ));
  set.insert(LlamaServerBuild::new(
    "x86_64-unknown-linux-gnu",
    "",
    vec!["cpu"],
  ));
  set.insert(LlamaServerBuild::new(
    "x86_64-pc-windows-msvc",
    "exe",
    vec!["cpu"],
  ));
  set
});

pub fn main() -> Result<()> {
  println!("cargo:rerun-if-changed=build.rs");
  println!("cargo:rerun-if-changed=Makefile");
  println!("cargo:rerun-if-env-changed=CI");
  println!("cargo:rerun-if-env-changed=CI_RELEASE");
  println!("cargo:rerun-if-env-changed=CI_BUILD_TARGET");
  println!("cargo:rerun-if-env-changed=CI_BUILD_VARIANTS");
  println!("cargo:rerun-if-env-changed=CI_DEFAULT_VARIANT");
  println!("cargo:rerun-if-env-changed=CI_EXEC_NAME");

  let project_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  // Create bin directory and lock file at the start
  let bin_dir = project_dir.join("bin");
  fs::create_dir_all(&bin_dir).context("Failed to create bin directory")?;
  let lock_path = bin_dir.join(LOCK_FILE);
  let lock_file = File::create(&lock_path).context("Failed to create lock file")?;

  // Take exclusive lock for the entire build process with timeout
  try_acquire_exclusive_lock_with_timeout(&lock_file)
    .context("Failed to acquire exclusive lock for llama server bin")?;

  // Rest of the build process
  try_main(&project_dir)?;

  // Release the lock
  let _ = fs2::FileExt::unlock(&lock_file);

  Ok(())
}

fn try_main(project_dir: &Path) -> Result<()> {
  // Check for CI environment with explicit configuration
  if let Ok(ci_target) = env::var("CI_BUILD_TARGET") {
    println!(
      "cargo:warning=Using CI build configuration for target: {}",
      ci_target
    );
    let ci_variants = env::var("CI_BUILD_VARIANTS").unwrap_or_else(|_| "cpu".to_string());
    let ci_default_variant = env::var("CI_DEFAULT_VARIANT").unwrap_or_else(|_| "cpu".to_string());
    let ci_exec_name = env::var("CI_EXEC_NAME").unwrap_or_else(|_| "llama-server".to_string());

    println!("cargo:rustc-env=BUILD_TARGET={}", ci_target);
    println!("cargo:rustc-env=BUILD_VARIANTS={}", ci_variants);
    println!("cargo:rustc-env=DEFAULT_VARIANT={}", ci_default_variant);
    println!("cargo:rustc-env=EXEC_NAME={}", ci_exec_name);
    return Ok(());
  }

  // Get target from Docker TARGETARCH or fallback to Cargo TARGET
  let target = get_target_from_platform()?;
  let build = LLAMA_SERVER_BUILDS.iter().find(|i| i.target == target);

  let Some(build) = build else {
    bail!("Unsupported target platform: {}", target);
  };
  let variant = env::var("CI_DEFAULT_VARIANT").unwrap_or_else(|_| build.default.clone());
  set_build_envs(build, &variant)?;
  if env::var("CI_RELEASE").unwrap_or("false".to_string()) == "true" {
    clean_bin_dir(project_dir)?;
    let script = project_dir.join("../../scripts/download-llama-bins.js");
    let status = Command::new("node")
      .arg(&script)
      .arg("--target")
      .arg(&build.target)
      .arg("--variants")
      .arg(build.variants.join(","))
      .arg("--extension")
      .arg(&build.extension)
      .arg("--bin-dir")
      .arg(project_dir.join("bin"))
      .stdout(Stdio::inherit())
      .stderr(Stdio::inherit())
      .status()
      .context("Failed to run download-llama-bins.js")?;
    if !status.success() {
      bail!("download-llama-bins.js failed");
    }
  } else {
    println!("building default variants");
    clean()?;
    build_llama_server(build, &variant)?;
  }
  Ok(())
}

fn get_makefile_args() -> Vec<&'static str> {
  let target = env::var("TARGET").unwrap();
  let os = target.split('-').nth(2).unwrap();
  if os == "windows" {
    vec!["-f", "Makefile.win.mk"]
  } else {
    vec![]
  }
}

fn exec_make_target(target: &str, envs: Vec<(&str, &str)>) -> Result<()> {
  let mut makefile_args = get_makefile_args();
  makefile_args.push(target);

  let mut command = if cfg!(windows) {
    let mut command = Command::new("pwsh");
    command.args(["-Command", &format!("make {}", makefile_args.join(" "))]);
    command
  } else {
    let mut command = Command::new("make");
    command.args(&makefile_args);
    command
  };
  for (key, value) in envs.iter() {
    command.env(key, value);
  }
  println!("executing make target: {}", makefile_args.join(" "));
  let status = command.status().with_context(|| {
    format!(
      "Failed to execute command: {} with args: {:?} and envs: {:?}",
      "make", makefile_args, envs
    )
  })?;

  if !status.success() {
    bail!(
      "Command failed: {} with args: {:?} and envs: {:?}",
      "make",
      makefile_args,
      envs
    );
  }
  Ok(())
}

fn clean() -> Result<()> {
  exec_make_target("clean", vec![])?;
  Ok(())
}

fn set_build_envs(build: &LlamaServerBuild, default_variant: &str) -> Result<()> {
  println!("cargo:rustc-env=BUILD_TARGET={}", build.target);
  println!(
    "cargo:rustc-env=BUILD_VARIANTS={}",
    build.variants.join(",")
  );
  println!("cargo:rustc-env=DEFAULT_VARIANT={}", default_variant);
  println!("cargo:rustc-env=EXEC_NAME={}", build.execname());
  Ok(())
}

fn build_llama_server(build: &LlamaServerBuild, variant: &str) -> Result<()> {
  let build_target = format!("build-{}-{}", build.target, variant);
  let envs = vec![
    ("TARGET", build.target.as_str()),
    ("VARIANT", variant),
    ("EXTENSION", build.extension.as_str()),
  ];
  exec_make_target(&build_target, envs)?;
  Ok(())
}

// New function to clean the output directory
fn clean_bin_dir(project_dir: &Path) -> Result<()> {
  let bin_dir = project_dir.join("bin");
  if !bin_dir.exists() {
    fs::create_dir_all(&bin_dir)?;
    return Ok(());
  }
  // Remove all contents except the lock file
  for entry in fs::read_dir(&bin_dir)? {
    let entry = entry?;
    let path = entry.path();
    if path.file_name().unwrap() != LOCK_FILE {
      if path.is_dir() {
        fs::remove_dir_all(&path)?;
      } else {
        fs::remove_file(&path)?;
      }
    }
  }
  Ok(())
}

#[derive(Debug, Hash, Eq, PartialEq)]
struct LlamaServerBuild {
  target: String,
  extension: String,
  variants: Vec<String>,
  default: String,
}

impl LlamaServerBuild {
  fn new(target: &str, extension: &str, variants: Vec<&str>) -> Self {
    let default = variants.first().unwrap().to_string();
    Self {
      target: target.to_string(),
      extension: extension.to_string(),
      variants: variants.into_iter().map(|v| v.to_string()).collect(),
      default,
    }
  }

  fn execname(&self) -> String {
    if self.extension.is_empty() {
      "llama-server".to_string()
    } else {
      format!("llama-server.{}", self.extension)
    }
  }
}

fn get_target_from_platform() -> Result<String> {
  // Check for Docker TARGETARCH first (multi-platform builds)
  if let Ok(target_arch) = env::var("TARGETARCH") {
    if !target_arch.is_empty() {
      let target = match target_arch.as_str() {
        "amd64" => "x86_64-unknown-linux-gnu",
        "arm64" => "aarch64-unknown-linux-gnu",
        _ => bail!("Unsupported Docker target architecture: {}", target_arch),
      };
      println!(
        "cargo:warning=Using Docker TARGETARCH: {} -> {}",
        target_arch, target
      );
      return Ok(target.to_string());
    }
  }

  // Fallback to Cargo TARGET for non-Docker builds
  let target = env::var("TARGET").context("TARGET environment variable not set")?;
  println!("cargo:warning=Using Cargo TARGET: {}", target);
  Ok(target)
}

fn try_acquire_exclusive_lock_with_timeout(file: &File) -> Result<()> {
  let start = Instant::now();
  loop {
    match file.try_lock_exclusive() {
      Ok(()) => {
        println!("Acquired exclusive lock for llama server bin");
        return Ok(());
      }
      Err(e) if start.elapsed().as_secs() >= LOCK_TIMEOUT_SECS => {
        bail!(
          "Timeout waiting for exclusive lock after {}s: {}",
          LOCK_TIMEOUT_SECS,
          e
        );
      }
      Err(_) => {
        println!("Waiting for llama server bin exclusive lock...");
        thread::sleep(Duration::from_secs(1));
      }
    }
  }
}
