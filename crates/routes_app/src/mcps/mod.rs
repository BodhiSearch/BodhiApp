mod error;
mod mcp_proxy;
mod mcps_api_schemas;
mod routes_mcps;
mod routes_mcps_auth;
mod routes_mcps_oauth;
mod routes_mcps_servers;

// Endpoint constants for MCP routes
pub const ENDPOINT_MCP_SERVERS: &str = "/bodhi/v1/mcps/servers";
pub const ENDPOINT_MCPS: &str = "/bodhi/v1/mcps";
pub const ENDPOINT_MCPS_FETCH_TOOLS: &str = "/bodhi/v1/mcps/fetch-tools";
pub const ENDPOINT_MCPS_AUTH_CONFIGS: &str = "/bodhi/v1/mcps/auth-configs";
pub const ENDPOINT_MCPS_OAUTH_DISCOVER_AS: &str = "/bodhi/v1/mcps/oauth/discover-as";
pub const ENDPOINT_MCPS_OAUTH_DISCOVER_MCP: &str = "/bodhi/v1/mcps/oauth/discover-mcp";
pub const ENDPOINT_MCPS_OAUTH_DYNAMIC_REGISTER_STANDALONE: &str =
  "/bodhi/v1/mcps/oauth/dynamic-register";

pub use error::*;
pub use mcp_proxy::*;
pub use mcps_api_schemas::*;
pub use routes_mcps::*;
pub use routes_mcps_auth::*;
pub use routes_mcps_oauth::*;
pub use routes_mcps_servers::*;

#[cfg(test)]
#[path = "test_mcps_isolation.rs"]
mod test_mcps_isolation;

#[cfg(test)]
#[path = "test_mcp_servers_isolation.rs"]
mod test_mcp_servers_isolation;
