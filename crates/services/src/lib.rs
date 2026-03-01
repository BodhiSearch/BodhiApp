// -- Test utilities
#[cfg(feature = "test-utils")]
pub mod test_utils;
#[cfg(all(not(feature = "test-utils"), test))]
pub mod test_utils;

// -- Cross-cutting types
pub mod shared_objs;

// -- Core service infrastructure
mod app_service;
mod env_wrapper;
mod macros;

// -- Authentication & security
mod app_access_requests;
mod apps;
mod auth;
mod tokens;

// -- AI & external API services
mod ai_apis;
mod mcps;
mod toolsets;

// -- Model & data management
mod models;

// -- Persistence
pub mod db;

// -- User management
mod users;

// -- Configuration
mod settings;

// -- Utility services
mod utils;

// -- Re-exports: cross-cutting types
pub use shared_objs::*;

// -- Re-exports: core service infrastructure
pub use app_service::*;
pub use env_wrapper::*;

// -- Re-exports: authentication & security
pub use app_access_requests::*;
pub use apps::*;
pub use auth::*;
pub use tokens::*;

// -- Re-exports: AI & external API services
pub use ai_apis::*;
pub use mcps::*;
pub use toolsets::*;

// -- Re-exports: model & data management
pub use models::*;

// -- Re-exports: user management
pub use users::*;

// -- Re-exports: configuration
pub use settings::*;

// -- Re-exports: utility services
pub use utils::*;

// -- Re-exports: error types for downstream crates
// These allow downstream crates to use services:: instead of errmeta:: directly.
pub use errmeta::{impl_error_from, AppError, EntityError, ErrorType, IoError, RwLockReadError};
// These are defined in shared_objs (axum/serde-dependent error types)
pub use shared_objs::{
  ApiError, ErrorBody, JsonRejectionError, ObjValidationError, OpenAIApiError, ReqwestError,
  SerdeJsonError, SerdeYamlError,
};
