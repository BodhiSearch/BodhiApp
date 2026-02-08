// -- Test utilities
#[cfg(feature = "test-utils")]
pub mod test_utils;
#[cfg(all(not(feature = "test-utils"), test))]
pub mod test_utils;

// -- Shared infrastructure
mod shared;

// -- Shared response DTOs
mod api_dto;

// -- Domain route modules (folders)
mod routes_api_models;
mod routes_auth;
mod routes_models;
mod routes_toolsets;
mod routes_users;

// -- Standalone route files
mod routes_api_token;
mod routes_dev;
mod routes_settings;
mod routes_setup;

// -- Test modules
#[cfg(test)]
mod routes_api_token_test;
#[cfg(test)]
mod routes_settings_test;
#[cfg(test)]
mod routes_setup_test;

// -- Re-exports
pub use api_dto::*;
pub use routes_api_models::*;
pub use routes_api_token::*;
pub use routes_auth::*;
pub use routes_dev::*;
pub use routes_models::*;
pub use routes_settings::*;
pub use routes_setup::*;
pub use routes_toolsets::*;
pub use routes_users::*;
pub use shared::*;
