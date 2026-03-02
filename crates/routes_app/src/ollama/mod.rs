mod error;
mod ollama_api_schemas;
mod routes_ollama;

pub use error::*;
pub use ollama_api_schemas::*;
pub use routes_ollama::*;

pub const ENDPOINT_OLLAMA_TAGS: &str = "/api/tags";
pub const ENDPOINT_OLLAMA_SHOW: &str = "/api/show";
pub const ENDPOINT_OLLAMA_CHAT: &str = "/api/chat";
