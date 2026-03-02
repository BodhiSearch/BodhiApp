mod error;
mod routes_oai_chat;
mod routes_oai_models;

#[cfg(test)]
#[path = "test_live_utils.rs"]
pub(crate) mod test_live_utils;

pub use error::*;
pub use routes_oai_chat::*;
pub use routes_oai_models::*;

pub const ENDPOINT_OAI_MODELS: &str = "/v1/models";
pub const ENDPOINT_OAI_CHAT_COMPLETIONS: &str = "/v1/chat/completions";
pub const ENDPOINT_OAI_EMBEDDINGS: &str = "/v1/embeddings";
