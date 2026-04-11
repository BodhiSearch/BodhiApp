mod anthropic_api_schemas;
mod routes_anthropic;

pub use anthropic_api_schemas::*;
pub use routes_anthropic::*;

pub const ENDPOINT_MESSAGES: &str = "/v1/messages";
pub const ENDPOINT_ANTHROPIC_MESSAGES: &str = "/anthropic/v1/messages";
pub const ENDPOINT_ANTHROPIC_MODELS: &str = "/anthropic/v1/models";
