mod api_alias_repository;
pub(crate) mod api_model_alias_entity;
mod api_model_service;
mod auth_scoped_api_models;
mod auth_scoped_data;
mod auth_scoped_downloads;
mod data_service;
mod download_repository;
pub(crate) mod download_request_entity;
mod download_service;
pub mod gguf;
mod hub_service;
pub(crate) mod model_metadata_entity;
mod model_metadata_repository;
pub mod model_objs;
mod multi_tenant_data_service;
mod progress_tracking;
#[cfg(test)]
#[path = "test_api_alias_repository.rs"]
mod test_api_alias_repository;
#[cfg(test)]
#[path = "test_api_model_service.rs"]
mod test_api_model_service;
#[cfg(test)]
#[path = "test_api_alias_repository_isolation.rs"]
mod test_api_alias_repository_isolation;
#[cfg(test)]
#[path = "test_download_repository.rs"]
mod test_download_repository;
#[cfg(test)]
#[path = "test_download_repository_isolation.rs"]
mod test_download_repository_isolation;
#[cfg(test)]
#[path = "test_model_metadata_global.rs"]
mod test_model_metadata_global;
#[cfg(test)]
#[path = "test_model_metadata_repository.rs"]
mod test_model_metadata_repository;
#[cfg(test)]
#[path = "test_user_alias_repository.rs"]
mod test_user_alias_repository;
#[cfg(test)]
#[path = "test_user_alias_repository_isolation.rs"]
mod test_user_alias_repository_isolation;
pub(crate) mod user_alias_entity;
mod user_alias_repository;

pub use api_alias_repository::ApiAliasRepository;
pub use api_model_service::*;
pub use auth_scoped_api_models::*;
pub use auth_scoped_data::*;
pub use auth_scoped_downloads::*;
pub use data_service::*;
pub use download_repository::DownloadRepository;
pub use download_service::*;
pub use gguf::*;
pub use hub_service::*;
pub use model_metadata_entity::ModelMetadataEntity;
pub use model_metadata_repository::ModelMetadataRepository;
pub use model_objs::*;
pub use multi_tenant_data_service::MultiTenantDataService;
pub use progress_tracking::*;
pub use user_alias_repository::UserAliasRepository;

// Entity re-exports for entities that were previously in db/entities
pub use api_model_alias_entity::ApiModelEntity;
pub use download_request_entity::DownloadRequestEntity;
pub use user_alias_entity::UserAliasEntity;
