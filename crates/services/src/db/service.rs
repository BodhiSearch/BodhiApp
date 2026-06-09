use crate::app_access_requests::AccessRequestRepository;
use crate::db::DbCore;
use crate::mcps::{McpRepository, McpServerRepository};
use crate::models::{
  ApiAliasRepository, DownloadRepository, LlmLibertyCredentialsRepository, ModelMetadataRepository,
  ModelRouterRepository, UserAliasRepository,
};
use crate::settings::SettingsRepository;
use crate::tenants::TenantRepository;
use crate::tokens::TokenRepository;
use crate::users::AccessRepository;

pub trait DbService:
  DownloadRepository
  + ApiAliasRepository
  + ModelRouterRepository
  + LlmLibertyCredentialsRepository
  + ModelMetadataRepository
  + AccessRepository
  + AccessRequestRepository
  + TenantRepository
  + TokenRepository
  + McpServerRepository
  + McpRepository
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
    + ModelRouterRepository
    + LlmLibertyCredentialsRepository
    + ModelMetadataRepository
    + AccessRepository
    + AccessRequestRepository
    + TenantRepository
    + TokenRepository
    + McpServerRepository
    + McpRepository
    + UserAliasRepository
    + SettingsRepository
    + DbCore
    + Send
    + Sync
    + std::fmt::Debug
{
}
