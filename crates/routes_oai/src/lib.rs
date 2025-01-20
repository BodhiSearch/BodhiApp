#[cfg(feature = "test-utils")]
pub mod test_utils;
#[cfg(all(not(feature = "test-utils"), test))]
pub mod test_utils;

mod routes_chat;
mod routes_oai_models;
mod routes_ollama;

pub use routes_chat::*;
pub use routes_oai_models::*;
pub use routes_ollama::*;

pub const ENDPOINT_OAI_MODELS: &str = "/v1/models";
pub const ENDPOINT_OAI_CHAT_COMPLETIONS: &str = "/v1/chat/completions";
pub const ENDPOINT_OLLAMA_TAGS: &str = "/api/tags";
pub const ENDPOINT_OLLAMA_SHOW: &str = "/api/show";
pub const ENDPOINT_OLLAMA_CHAT: &str = "/api/chat";

pub mod l10n {
  use include_dir::Dir;

  pub const L10N_RESOURCES: &Dir = &include_dir::include_dir!("$CARGO_MANIFEST_DIR/src/resources");
}
