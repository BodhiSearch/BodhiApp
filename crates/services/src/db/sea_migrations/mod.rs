mod m20250101_000000_extensions;
mod m20250101_000001_download_requests;
mod m20250101_000002_api_model_aliases;
mod m20250101_000003_model_metadata;
mod m20250101_000004_access_requests;
mod m20250101_000005_api_tokens;
mod m20250101_000006_toolsets;
mod m20250101_000007_user_aliases;
mod m20250101_000008_app_access_requests;
mod m20250101_000009_mcp_servers;
mod m20250101_000010_mcp_auth_headers;
mod m20250101_000011_mcp_oauth;
mod m20250101_000012_settings;
mod m20250101_000013_apps;
use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
  fn migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![
      Box::new(m20250101_000000_extensions::Migration),
      Box::new(m20250101_000001_download_requests::Migration),
      Box::new(m20250101_000002_api_model_aliases::Migration),
      Box::new(m20250101_000003_model_metadata::Migration),
      Box::new(m20250101_000004_access_requests::Migration),
      Box::new(m20250101_000005_api_tokens::Migration),
      Box::new(m20250101_000006_toolsets::Migration),
      Box::new(m20250101_000007_user_aliases::Migration),
      Box::new(m20250101_000008_app_access_requests::Migration),
      Box::new(m20250101_000009_mcp_servers::Migration),
      Box::new(m20250101_000010_mcp_auth_headers::Migration),
      Box::new(m20250101_000011_mcp_oauth::Migration),
      Box::new(m20250101_000012_settings::Migration),
      Box::new(m20250101_000013_apps::Migration),
    ]
  }
}
