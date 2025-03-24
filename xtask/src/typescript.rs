use anyhow::{Context, Result};
use std::process::Command;

pub fn generate_types() -> Result<()> {
  // First ensure openapi.json exists by running our openapi generation
  crate::openapi::generate()?;

  // Check if openapi-typescript is installed
  let npm_ls = Command::new("npm")
    .args(["ls", "-g", "openapi-typescript"])
    .output()
    .context("Failed to check if openapi-typescript is installed")?;

  // Install openapi-typescript if not found
  if !String::from_utf8_lossy(&npm_ls.stdout).contains("openapi-typescript") {
    println!("Installing openapi-typescript...");
    Command::new("npm")
      .args(["install", "-g", "openapi-typescript"])
      .status()
      .context("Failed to install openapi-typescript")?;
  }

  // Generate TypeScript types
  println!("Generating TypeScript types...");
  let output_path = "app/types/api.d.ts";
  Command::new("openapi-typescript")
    .args([
      "openapi.json",
      "--output",
      output_path,
      "--export-type",
      "components",
    ])
    .status()
    .context("Failed to generate TypeScript types")?;

  println!("TypeScript types generated at: {}", output_path);

  // Also generate types for the ts-client if it exists
  let ts_client_dir = std::path::Path::new("ts-client");
  if ts_client_dir.exists() && ts_client_dir.is_dir() {
    println!("Detected ts-client directory, generating types there as well...");

    // Ensure the types directory exists
    let types_dir = ts_client_dir.join("src").join("types");
    std::fs::create_dir_all(&types_dir).context("Failed to create types directory in ts-client")?;

    // Generate TypeScript types for ts-client
    let ts_client_output_path = "ts-client/src/types/api.d.ts";
    Command::new("openapi-typescript")
      .args([
        "openapi.json",
        "--output",
        ts_client_output_path,
        "--export-type",
        "components",
      ])
      .status()
      .context("Failed to generate TypeScript types for ts-client")?;

    println!("TypeScript types generated at: {}", ts_client_output_path);
  }

  Ok(())
}
