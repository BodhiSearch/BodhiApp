mod handlers;
mod types;

#[cfg(test)]
mod tests;

pub use handlers::*;
pub use types::*;

pub const ENDPOINT_OLLAMA_TAGS: &str = "/api/tags";
pub const ENDPOINT_OLLAMA_SHOW: &str = "/api/show";
pub const ENDPOINT_OLLAMA_CHAT: &str = "/api/chat";
