use crate::db::{
  AccessRepository, AccessRequestRepository, AppInstanceRepository, DbCore, McpRepository,
  ModelRepository, SettingsRepository, TokenRepository, ToolsetRepository, UserAliasRepository,
};

/// Super-trait that combines all repository sub-traits.
/// Any type implementing all sub-traits automatically implements DbService
/// via the blanket impl below.
pub trait DbService:
  ModelRepository
  + AccessRepository
  + AccessRequestRepository
  + AppInstanceRepository
  + TokenRepository
  + ToolsetRepository
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
  T: ModelRepository
    + AccessRepository
    + AccessRequestRepository
    + AppInstanceRepository
    + TokenRepository
    + ToolsetRepository
    + McpRepository
    + UserAliasRepository
    + SettingsRepository
    + DbCore
    + Send
    + Sync
    + std::fmt::Debug
{
}
