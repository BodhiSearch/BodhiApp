// -- Test utilities
#[cfg(feature = "test-utils")]
pub mod test_utils;
#[cfg(all(not(feature = "test-utils"), test))]
pub mod test_utils;

// -- Shared infrastructure
pub mod middleware;
mod shared;

// -- Domain route modules (folders)
mod api_models;
mod apps;
mod auth;
mod mcps;
mod models;
pub mod oai;
pub mod ollama;
mod settings;
mod setup;
mod tenants;
mod tokens;
mod toolsets;
mod users;

// -- Standalone route files
mod routes;
mod routes_dev;
mod routes_ping;
mod routes_proxy;

// -- Test modules

// -- Re-exports
pub use api_models::*;
pub use apps::*;
pub use auth::*;
pub use mcps::*;
pub use models::*;
pub use oai::*;
pub use ollama::*;
pub use routes::*;
pub use routes_dev::*;
pub use routes_ping::*;
pub use routes_proxy::*;
pub use settings::*;
pub use setup::*;
pub use shared::*;
pub use tenants::*;
pub use tokens::*;
pub use toolsets::*;
pub use users::*;
