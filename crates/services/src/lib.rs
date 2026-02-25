// -- Test utilities
#[cfg(feature = "test-utils")]
pub mod test_utils;
#[cfg(all(not(feature = "test-utils"), test))]
pub mod test_utils;

// -- Core service infrastructure
mod app_service;
mod env_wrapper;
mod macros;

// -- Authentication & security
mod access_request_service;
mod app_instance_service;
mod auth_service;
mod keyring_service;
mod session_service;
#[cfg(test)]
#[path = "test_session_service.rs"]
mod test_session_service;
mod token;

// -- AI & external API services
mod ai_api_service;
mod exa_service;
mod mcp_service;
mod tool_service;

// -- Model & data management
mod cache_service;
mod data_service;
mod hub_service;

// -- Network operations
mod network_service;

// -- Persistence
pub mod db;

// -- Configuration
mod setting_service;

// -- Concurrency & async processing
mod concurrency_service;
mod progress_tracking;
mod queue_service;

// -- Domain object extensions
mod objs;

// -- Re-exports: core service infrastructure
pub use app_service::*;
pub use env_wrapper::*;

// -- Re-exports: authentication & security
pub use access_request_service::*;
pub use app_instance_service::*;
pub use auth_service::*;
pub use keyring_service::*;
pub use session_service::*;
pub use token::*;

// -- Re-exports: AI & external API services
pub use ai_api_service::*;
pub use exa_service::*;
pub use mcp_service::*;
pub use tool_service::*;

// -- Re-exports: model & data management
pub use cache_service::*;
pub use data_service::*;
pub use hub_service::*;

// -- Re-exports: network operations
pub use network_service::*;

// -- Re-exports: configuration
pub use setting_service::*;

// -- Re-exports: concurrency & async processing
pub use concurrency_service::*;
pub use progress_tracking::*;
pub use queue_service::*;

// -- Re-exports: domain object extensions
pub use objs::*;
