// -- Test utilities
#[cfg(feature = "test-utils")]
pub mod test_utils;
#[cfg(all(not(feature = "test-utils"), test))]
pub mod test_utils;

// -- Shared infrastructure
pub mod middleware;
mod shared;

// -- Domain route modules (folders)
pub mod anthropic;
mod apps;
mod auth;
mod mcps;
mod models;
pub mod oai;
pub mod ollama;
mod providers;
mod settings;
mod setup;
mod tenants;
mod tokens;
mod users;

// -- Standalone route files
mod routes;
mod routes_dev;
mod routes_ping;
mod routes_proxy;
mod spa_router;

// -- Test modules

// -- Re-exports
pub use anthropic::*;
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
pub use spa_router::*;
pub use tenants::*;
pub use tokens::*;
pub use users::*;
