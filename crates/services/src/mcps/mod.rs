mod auth_scoped;
mod error;
mod mcp_objs;
mod mcp_service;
#[cfg(test)]
pub(crate) mod test_helpers;
#[cfg(test)]
#[path = "test_mcp_auth_repository_isolation.rs"]
mod test_mcp_auth_repository_isolation;
#[cfg(test)]
#[path = "test_mcp_proxy_service.rs"]
mod test_mcp_proxy_service;
#[cfg(test)]
#[path = "test_mcp_repository_isolation.rs"]
mod test_mcp_repository_isolation;

pub(crate) mod mcp_auth_config_entity;
pub(crate) mod mcp_auth_config_param_entity;
pub(crate) mod mcp_auth_param_entity;
pub(crate) mod mcp_entity;
pub(crate) mod mcp_oauth_config_detail_entity;
pub(crate) mod mcp_oauth_token_entity;
pub(crate) mod mcp_server_entity;

mod mcp_repository;
mod mcp_server_repository;

pub use auth_scoped::*;
pub use error::*;
pub use mcp_client::{McpAuthParams, McpTool};
pub use mcp_objs::*;
pub use mcp_service::*;

pub use mcp_auth_config_entity::McpAuthConfigEntity;
pub use mcp_auth_config_param_entity::McpAuthConfigParamEntity;
pub use mcp_auth_param_entity::McpAuthParamEntity;
pub use mcp_entity::{McpEntity, McpWithServerEntity};
pub use mcp_oauth_config_detail_entity::McpOAuthConfigDetailEntity;
pub use mcp_oauth_token_entity::McpOAuthTokenEntity;
pub use mcp_server_entity::McpServerEntity;

pub use mcp_repository::McpRepository;
pub use mcp_server_repository::McpServerRepository;
