mod m20250101_000000_extensions;
mod m20250101_000001_download_requests;
mod m20250101_000002_api_model_aliases;
mod m20250101_000003_model_metadata;
mod m20250101_000004_access_requests;
mod m20250101_000005_api_tokens;
mod m20250101_000006_toolsets;
mod m20250101_000007_app_toolset_configs;
mod m20250101_000008_user_aliases;
mod m20250101_000009_app_access_requests;
mod m20250101_000010_mcp_servers;
mod m20250101_000011_mcp_auth_headers;
mod m20250101_000012_mcp_oauth;
mod m20250101_000013_settings;
mod m20250101_000014_tenants;
mod m20250101_000015_tenants_users;
mod m20250101_000016_mcp_auth_redesign;
mod m20250101_000017_drop_toolsets;
mod m20250101_000018_drop_mcp_tools_columns;
mod m20250101_000019_drop_models_cache;
mod m20250101_000020_api_alias_extra_fields;
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
      Box::new(m20250101_000007_app_toolset_configs::Migration),
      Box::new(m20250101_000008_user_aliases::Migration),
      Box::new(m20250101_000009_app_access_requests::Migration),
      Box::new(m20250101_000010_mcp_servers::Migration),
      Box::new(m20250101_000011_mcp_auth_headers::Migration),
      Box::new(m20250101_000012_mcp_oauth::Migration),
      Box::new(m20250101_000013_settings::Migration),
      Box::new(m20250101_000014_tenants::Migration),
      Box::new(m20250101_000015_tenants_users::Migration),
      Box::new(m20250101_000016_mcp_auth_redesign::Migration),
      Box::new(m20250101_000017_drop_toolsets::Migration),
      Box::new(m20250101_000018_drop_mcp_tools_columns::Migration),
      Box::new(m20250101_000019_drop_models_cache::Migration),
      Box::new(m20250101_000020_api_alias_extra_fields::Migration),
    ]
  }
}
