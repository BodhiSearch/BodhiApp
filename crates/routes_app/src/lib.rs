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
mod routes_apps;
mod routes_auth;
mod routes_models;
pub mod routes_oai;
pub mod routes_ollama;
mod routes_toolsets;
mod routes_users;

// -- Standalone route files
mod routes;
mod routes_api_token;
mod routes_dev;
mod routes_ping;
mod routes_proxy;
mod routes_settings;
mod routes_setup;

// -- Test modules

// -- Re-exports
pub use api_dto::*;
pub use routes::*;
pub use routes_api_models::*;
pub use routes_api_token::*;
pub use routes_apps::*;
pub use routes_auth::*;
pub use routes_dev::*;
pub use routes_models::*;
pub use routes_oai::{
  chat_completions_handler, embeddings_handler, oai_model_handler, oai_models_handler,
  OAIRouteError, ENDPOINT_OAI_CHAT_COMPLETIONS, ENDPOINT_OAI_EMBEDDINGS, ENDPOINT_OAI_MODELS,
};
pub use routes_ollama::{
  ollama_model_chat_handler, ollama_model_show_handler, ollama_models_handler,
  ENDPOINT_OLLAMA_CHAT, ENDPOINT_OLLAMA_SHOW, ENDPOINT_OLLAMA_TAGS,
};
pub use routes_ping::*;
pub use routes_proxy::*;
pub use routes_settings::*;
pub use routes_setup::*;
pub use routes_toolsets::*;
pub use routes_users::*;
pub use shared::*;
