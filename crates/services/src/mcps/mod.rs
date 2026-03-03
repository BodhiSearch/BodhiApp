mod error;
mod mcp_objs;
mod mcp_service;
#[cfg(test)]
pub(crate) mod test_helpers;

pub(crate) mod mcp_auth_header_entity;
pub(crate) mod mcp_entity;
pub(crate) mod mcp_oauth_config_entity;
pub(crate) mod mcp_oauth_token_entity;
pub(crate) mod mcp_server_entity;

mod mcp_auth_repository;
mod mcp_instance_repository;
mod mcp_server_repository;

pub use error::*;
pub use mcp_client::McpTool;
pub use mcp_objs::*;
pub use mcp_service::*;

pub use mcp_auth_header_entity::{McpAuthHeaderEntity, McpAuthHeaderRow};
pub use mcp_entity::{McpEntity, McpRow, McpWithServerEntity};
pub use mcp_oauth_config_entity::{McpOAuthConfigEntity, McpOAuthConfigRow};
pub use mcp_oauth_token_entity::{McpOAuthTokenEntity, McpOAuthTokenRow};
pub use mcp_server_entity::{McpServerEntity, McpServerRow};

// Backward-compatible alias
pub type McpWithServerRow = McpWithServerEntity;

pub use mcp_auth_repository::McpAuthRepository;
pub use mcp_instance_repository::McpInstanceRepository;
pub use mcp_server_repository::McpServerRepository;
