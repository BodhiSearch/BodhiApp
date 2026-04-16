mod api_error;
mod error;
mod openapi;
mod routes_oai_chat;
mod routes_oai_models;
mod routes_oai_responses;

#[cfg(test)]
#[path = "test_live_utils.rs"]
pub(crate) mod test_live_utils;

pub use api_error::*;
pub use error::*;
pub use openapi::*;
pub use routes_oai_chat::*;
pub use routes_oai_models::*;
pub use routes_oai_responses::*;

pub const ENDPOINT_OAI_MODELS: &str = "/v1/models";
pub const ENDPOINT_OAI_CHAT_COMPLETIONS: &str = "/v1/chat/completions";
pub const ENDPOINT_OAI_EMBEDDINGS: &str = "/v1/embeddings";
pub const ENDPOINT_OAI_RESPONSES: &str = "/v1/responses";
