pub mod access_requests;
mod anthropic_auth_middleware;
pub mod apis;
pub mod auth;
mod error;
mod openai_auth_middleware;
pub mod redirects;
pub mod token_service;
mod utils;

pub use access_requests::*;
pub use anthropic_auth_middleware::*;
pub use apis::*;
pub use auth::*;
pub use error::MiddlewareError;
pub use openai_auth_middleware::*;
pub use redirects::*;
pub use token_service::*;
pub use utils::*;

/// Placeholder apiKey the chat UI hands to pi-ai SDKs; stripped by
/// `anthropic_auth_middleware` / `openai_auth_middleware` so session auth takes over.
pub const SENTINEL_API_KEY: &str = "bodhiapp_sentinel_api_key_ignored";

/// Pre-computed lowercase `bearer <sentinel>` for hot-path comparison.
pub(crate) const SENTINEL_BEARER_LOWER: &str = "bearer bodhiapp_sentinel_api_key_ignored";
