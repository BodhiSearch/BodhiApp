mod chat;
mod error;
mod models;

#[cfg(test)]
mod tests;

pub use chat::*;
pub use error::*;
pub use models::*;

pub const ENDPOINT_OAI_MODELS: &str = "/v1/models";
pub const ENDPOINT_OAI_CHAT_COMPLETIONS: &str = "/v1/chat/completions";
pub const ENDPOINT_OAI_EMBEDDINGS: &str = "/v1/embeddings";
