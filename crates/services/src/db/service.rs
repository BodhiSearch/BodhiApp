use crate::app_access_requests::AccessRequestRepository;
use crate::db::DbCore;
use crate::mcps::{McpAuthRepository, McpInstanceRepository, McpServerRepository};
use crate::models::{
  ApiAliasRepository, DownloadRepository, ModelMetadataRepository, UserAliasRepository,
};
use crate::settings::SettingsRepository;
use crate::tenants::TenantRepository;
use crate::tokens::TokenRepository;
use crate::toolsets::ToolsetRepository;
use crate::users::AccessRepository;

/// Super-trait that combines all repository sub-traits.
/// Any type implementing all sub-traits automatically implements DbService
/// via the blanket impl below.
pub trait DbService:
  DownloadRepository
  + ApiAliasRepository
  + ModelMetadataRepository
  + AccessRepository
  + AccessRequestRepository
  + TenantRepository
  + TokenRepository
  + ToolsetRepository
  + McpServerRepository
  + McpInstanceRepository
  + McpAuthRepository
  + UserAliasRepository
  + SettingsRepository
  + DbCore
  + Send
  + Sync
  + std::fmt::Debug
{
}

impl<T> DbService for T where
  T: DownloadRepository
    + ApiAliasRepository
    + ModelMetadataRepository
    + AccessRepository
    + AccessRequestRepository
    + TenantRepository
    + TokenRepository
    + ToolsetRepository
    + McpServerRepository
    + McpInstanceRepository
    + McpAuthRepository
    + UserAliasRepository
    + SettingsRepository
    + DbCore
    + Send
    + Sync
    + std::fmt::Debug
{
}
