use clap::Parser;
use objs::ApiError;
use services::{AuthService, KeycloakAuthService};

#[derive(Parser)]
#[command(name = "setup_client")]
#[command(about = "A CLI tool to register a client and make a user resource admin")]
struct Cli {
  /// Auth server URL (e.g., https://auth.example.com)
  #[arg(long)]
  auth_server_url: String,

  /// Realm name
  #[arg(long)]
  realm: String,

  /// Username as email
  #[arg(long)]
  username: String,

  /// Application version
  #[arg(long)]
  version: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  if let Err(err) = internal_main().await {
    eprintln!("Error: {}", serde_json::to_string(&err)?);
    std::process::exit(1);
  }
  Ok(())
}

async fn internal_main() -> Result<(), ApiError> {
  let cli = Cli::parse();

  // Create the auth service
  let auth_service =
    KeycloakAuthService::new(&cli.version, cli.auth_server_url.clone(), cli.realm.clone());

  // Register a new client with default redirect URIs
  let redirect_uris = vec![
    "http://localhost:1135/ui/auth/callback".to_string(),
    "http://0.0.0.0:1135/ui/auth/callback".to_string(),
    "http://127.0.0.1:1135/ui/auth/callback".to_string(),
  ];

  println!("Registering client with auth server...");
  let app_reg_info = auth_service.register_client(redirect_uris).await?;

  println!("Client registered successfully!");
  println!("Client ID: {}", app_reg_info.client_id);
  println!("Client Secret: {}", app_reg_info.client_secret);

  // Make the user a resource admin
  println!("Making user '{}' a resource admin...", cli.username);
  auth_service
    .make_resource_admin(
      &app_reg_info.client_id,
      &app_reg_info.client_secret,
      &cli.username,
    )
    .await?;

  println!("User '{}' is now a resource admin!", cli.username);
  println!();
  println!("=== CLIENT CREDENTIALS ===");
  println!("Client ID: {}", app_reg_info.client_id);
  println!("Client Secret: {}", app_reg_info.client_secret);
  println!("===========================");

  Ok(())
}
