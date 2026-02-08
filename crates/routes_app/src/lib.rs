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
mod routes_auth;
mod routes_users;
mod routes_models;
mod routes_api_models;
mod routes_toolsets;

// -- Standalone route files
mod routes_app_request_access;
mod routes_api_token;
mod routes_setup;
mod routes_settings;
mod routes_dev;

// -- Test modules
#[cfg(test)]
mod routes_api_token_test;
#[cfg(test)]
mod routes_setup_test;
#[cfg(test)]
mod routes_settings_test;

// -- Re-exports
pub use shared::*;
pub use api_dto::*;
pub use routes_auth::*;
pub use routes_users::*;
pub use routes_models::*;
pub use routes_api_models::*;
pub use routes_toolsets::*;
pub use routes_app_request_access::*;
pub use routes_api_token::*;
pub use routes_setup::*;
pub use routes_settings::*;
pub use routes_dev::*;
