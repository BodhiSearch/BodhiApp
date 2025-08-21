use anyhow::{bail, Context, Result};
use fs2::FileExt;
use once_cell::sync::Lazy;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::Deserialize;
use std::{
  collections::HashSet,
  env,
  fs::{self, File},
  io::{self},
  path::{Path, PathBuf},
  process::Command,
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
  clean()?;
  if env::var("CI_RELEASE").unwrap_or("false".to_string()) == "true" {
    let Ok(gh_pat) = env::var("GH_PAT") else {
      bail!("GH_PAT is not set");
    };
    println!("building all variants");
    clean_bin_dir(project_dir)?;
    let client = build_gh_client(gh_pat)?;
    let response = client
      .get("https://api.github.com/repos/BodhiSearch/llama.cpp/releases/latest")
      .send()?;

    // Check HTTP status code first
    if !response.status().is_success() {
      let status = response.status();
      let response_text = response
        .text()
        .with_context(|| "Failed to read error response text".to_string())?;
      bail!(
        "GitHub API request failed with status {}: {}",
        status,
        response_text
      );
    }

    let response_text = response
      .text()
      .with_context(|| "Failed to read response text for latest release".to_string())?;
    let release: GithubRelease = serde_json::from_str(&response_text).unwrap_or_else(|err| {
      panic!(
        "Failed to deserialize response: {}\nError: {}",
        response_text, err
      );
    });
    if release.assets.is_empty() {
      bail!("No assets found in latest release: {}", response_text);
    } else {
      println!(
        "assets: {:?}",
        release
          .assets
          .iter()
          .map(|a| a.name.clone())
          .collect::<Vec<String>>()
          .join(",")
      );
    }
    for variant in build.variants.iter() {
      // Check if binary exists for this platform/variant combination first
      if !binary_exists_for_platform(&release, build, variant) {
        bail!(
          "No pre-built binary available for platform: {}-{}. Available assets: {}",
          build.target,
          variant,
          release
            .assets
            .iter()
            .map(|a| a.name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
        );
      }
      fetch_llama_server(&client, build, variant, &release)?;
    }
  } else {
    println!("building default variants");
    build_llama_server(build, &variant)?;
  }
  Ok(())
}

fn build_gh_client(gh_pat: String) -> Result<reqwest::blocking::Client, anyhow::Error> {
  let mut headers = HeaderMap::<HeaderValue>::default();
  headers.append(
    "Authorization",
    format!("Bearer {}", gh_pat).parse().unwrap(),
  );
  headers.append("Accept", "application/vnd.github.v3+json".parse().unwrap());
  headers.append("X-GitHub-Api-Version", "2022-11-28".parse().unwrap());
  headers.append("User-Agent", "Bodhi-Build".parse().unwrap());
  let client = reqwest::blocking::ClientBuilder::default()
    .default_headers(headers)
    .build()?;
  Ok(client)
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

#[derive(Deserialize)]
struct GithubRelease {
  assets: Vec<GithubAsset>,
}

#[derive(Deserialize)]
struct GithubAsset {
  name: String,
  browser_download_url: String,
}

fn fetch_llama_server(
  client: &reqwest::blocking::Client,
  build: &LlamaServerBuild,
  variant: &str,
  release: &GithubRelease,
) -> Result<()> {
  try_fetch_llama_server(client, build, variant, release)
}

fn try_fetch_llama_server(
  client: &reqwest::blocking::Client,
  build: &LlamaServerBuild,
  variant: &str,
  release: &GithubRelease,
) -> Result<()> {
  let target_file_prefix = format!("llama-server--{}--{}", build.target, variant);
  // Filter assets based on the target and variant
  let Some(asset) = release
    .assets
    .iter()
    .find(|asset| asset.name.starts_with(&target_file_prefix))
  else {
    bail!(
      "No matching assets found for {}, found: {}",
      target_file_prefix,
      release
        .assets
        .iter()
        .map(|a| a.name.clone())
        .collect::<Vec<String>>()
        .join(",")
    );
  };

  // Download each matching asset
  let download_url = &asset.browser_download_url;
  println!("cargo:warning=Downloading {}", download_url);
  let response = client.get(download_url).send()?;

  // Ensure the response is successful
  if !response.status().is_success() {
    bail!("Failed to download file: {}", download_url);
  }

  // Create the target directory
  let target_dir = Path::new("bin").join(&build.target).join(variant);
  fs::create_dir_all(&target_dir)?;

  let bytes = response.bytes()?;

  if asset.name.ends_with(".zip") {
    // Handle zip file
    let temp_dir = tempfile::tempdir()?;
    let zip_path = temp_dir.path().join("download.zip");

    // Write zip file to temp location
    let mut temp_file = File::create(&zip_path)?;
    io::copy(&mut bytes.as_ref(), &mut temp_file)?;

    // Check if zip is available
    check_zip_installation()?;

    // Extract using system zip command
    let unzip_status = if cfg!(windows) {
      Command::new("pwsh")
        .args([
          "-Command",
          &format!(
            "Expand-Archive -Path '{}' -DestinationPath '{}'",
            zip_path.display(),
            temp_dir.path().display()
          ),
        ])
        .status()?
    } else {
      Command::new("unzip")
        .args([
          "-o", // overwrite files without prompting
          zip_path.to_str().unwrap(),
          "-d",
          temp_dir.path().to_str().unwrap(),
        ])
        .status()?
    };

    if !unzip_status.success() {
      bail!("Failed to extract zip file");
    }

    // Move extracted contents to target directory
    for entry in fs::read_dir(temp_dir.path())? {
      let entry = entry?;
      let path = entry.path();
      if path.file_name().unwrap() == "download.zip" {
        continue;
      }

      let target_path = target_dir.join(path.file_name().unwrap());
      fs::rename(&path, &target_path)?;

      // Set executable permissions only for llama-server
      #[cfg(unix)]
      {
        if target_path.file_name().unwrap() == "llama-server" {
          use std::os::unix::fs::PermissionsExt;
          let mut perms = fs::metadata(&target_path)?.permissions();
          perms.set_mode(0o755);
          fs::set_permissions(&target_path, perms)?;
        }
      }
    }
  } else {
    // Handle direct file copy
    let target_path = target_dir.join(build.execname());
    let mut file = File::create(&target_path)?;
    io::copy(&mut bytes.as_ref(), &mut file)?;

    // Set executable permissions
    #[cfg(unix)]
    {
      use std::os::unix::fs::PermissionsExt;
      let mut perms = file.metadata()?.permissions();
      perms.set_mode(0o755);
      std::fs::set_permissions(&target_path, perms)?;
    }
  }

  println!(
    "cargo:warning=Successfully downloaded and moved {} for {}-{}",
    asset.name, build.target, variant
  );

  Ok(())
}

fn check_zip_installation() -> Result<()> {
  let check_command = if cfg!(windows) {
    Command::new("pwsh")
      .args([
        "-Command",
        "if (!(Get-Command Expand-Archive -ErrorAction SilentlyContinue)) { exit 1 }",
      ])
      .status()?
  } else {
    Command::new("which").arg("unzip").status()?
  };

  if !check_command.success() {
    let msg = if cfg!(target_os = "macos") {
      "zip utility not found. Please install it using: brew install unzip"
    } else if cfg!(target_os = "linux") {
      "zip utility not found. Please install it using: sudo apt-get install unzip"
    } else if cfg!(windows) {
      "PowerShell 5.0 or later with Expand-Archive cmdlet is required. Please install latest PowerShell: choco install powershell-core"
    } else {
      "zip utility not found. Please install it using your system's package manager"
    };
    bail!(msg);
  }

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

fn binary_exists_for_platform(
  release: &GithubRelease,
  build: &LlamaServerBuild,
  variant: &str,
) -> bool {
  let target_file_prefix = format!("llama-server--{}--{}", build.target, variant);
  release
    .assets
    .iter()
    .any(|asset| asset.name.starts_with(&target_file_prefix))
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
