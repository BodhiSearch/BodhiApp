mod auth_configs;
mod error;
mod mcps;
mod oauth_utils;
mod servers;
mod types;

pub use auth_configs::*;
pub use error::*;
pub use mcps::*;
pub use oauth_utils::*;
pub use servers::*;
pub use types::*;

// Endpoint constants for MCP routes
pub const ENDPOINT_MCP_SERVERS: &str = "/bodhi/v1/mcps/servers";
pub const ENDPOINT_MCPS: &str = "/bodhi/v1/mcps";
pub const ENDPOINT_MCPS_FETCH_TOOLS: &str = "/bodhi/v1/mcps/fetch-tools";
pub const ENDPOINT_MCPS_AUTH_CONFIGS: &str = "/bodhi/v1/mcps/auth-configs";
pub const ENDPOINT_MCPS_OAUTH_DISCOVER_AS: &str = "/bodhi/v1/mcps/oauth/discover-as";
pub const ENDPOINT_MCPS_OAUTH_DISCOVER_MCP: &str = "/bodhi/v1/mcps/oauth/discover-mcp";
pub const ENDPOINT_MCPS_OAUTH_DYNAMIC_REGISTER_STANDALONE: &str =
  "/bodhi/v1/mcps/oauth/dynamic-register";
