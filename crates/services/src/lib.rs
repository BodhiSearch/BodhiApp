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
mod app_access_requests;
mod apps;
mod auth;
mod token;
mod tokens;

// -- AI & external API services
mod ai_apis;
mod mcps;
mod toolsets;

// -- Model & data management
mod models;

// -- Persistence
pub mod db;

// -- Configuration
mod settings;

// -- Utility services
mod utils;

// -- Domain object extensions
mod objs;

// -- Re-exports: core service infrastructure
pub use app_service::*;
pub use env_wrapper::*;

// -- Re-exports: authentication & security
pub use app_access_requests::*;
pub use apps::*;
pub use auth::*;
pub use token::*;
pub use tokens::*;

// -- Re-exports: AI & external API services
pub use ai_apis::*;
pub use mcps::*;
pub use toolsets::*;

// -- Re-exports: model & data management
pub use models::*;

// -- Re-exports: configuration
pub use settings::*;

// -- Re-exports: utility services
pub use utils::*;

// -- Re-exports: domain object extensions
pub use objs::*;
