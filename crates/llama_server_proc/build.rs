use anyhow::{bail, Context, Result};
use once_cell::sync::Lazy;
use std::collections::HashSet;
use std::env;
use std::process::Command;

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
    vec!["cpu", "cuda-12_6"],
  ));
  set.insert(LlamaServerBuild::new(
    "x86_64-pc-windows-msvc",
    "exe",
    vec!["cpu", "cuda-11.7", "cuda-12.4"],
  ));
  set
});

pub fn main() -> Result<()> {
  println!("cargo:rerun-if-changed=build.rs");
  println!("cargo:rerun-if-changed=Makefile");
  println!("cargo:rerun-if-env-changed=CI");
  println!("cargo:rerun-if-env-changed=CI_FULL_BUILD");
  println!("cargo:rerun-if-env-changed=CI_DEFAULT_VARIANT");
  let target = env::var("TARGET").unwrap();
  let build = LLAMA_SERVER_BUILDS.iter().find(|i| i.target == target);

  #[allow(clippy::unnecessary_unwrap)]
  let build = if build.is_none() {
    bail!("Unsupported target platform: {}", target);
  } else {
    build.unwrap()
  };
  let variant = env::var("CI_DEFAULT_VARIANT").unwrap_or_else(|_| build.default.clone());
  set_build_envs(build, &variant)?;
  clean()?;
  if env::var("CI_FULL_BUILD").unwrap_or("false".to_string()) == "true" {
    println!("building all variants");
    for variant in build.variants.iter() {
      build_llama_server(build, variant)?;
    }
  } else {
    println!("building default variants");
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
