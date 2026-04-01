use crate::{ApiError, AuthScope, API_TAG_MCPS, ENDPOINT_APPS_MCPS};
use axum::{
  body::Body,
  extract::Path,
  response::{IntoResponse, Response},
};
use once_cell::sync::Lazy;
use std::time::Duration;
use tracing::debug;

use super::McpRouteError;

/// Shared reqwest client for upstream MCP requests.
/// Connection pooling is built into reqwest::Client.
static HTTP_CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
  reqwest::Client::builder()
    .connect_timeout(Duration::from_secs(10))
    // No request timeout — SSE streams are long-lived
    .build()
    .expect("Failed to build HTTP client")
});

/// Headers to forward from client request to upstream.
const FORWARD_REQUEST_HEADERS: &[&str] = &[
  "content-type",
  "accept",
  "mcp-session-id",
  "mcp-protocol-version",
  "last-event-id",
];

/// Headers to forward from upstream response to client.
const FORWARD_RESPONSE_HEADERS: &[&str] = &[
  "content-type",
  "mcp-session-id",
  "mcp-protocol-version",
  "cache-control",
];

/// Transparent HTTP reverse proxy for MCP Streamable HTTP endpoints.
///
/// Forwards raw HTTP requests (POST, GET, DELETE) to the upstream MCP server,
/// injecting authentication headers/query params. SSE responses are streamed
/// back without buffering or decoding.
///
/// Mounted at:
///   `/bodhi/v1/apps/mcps/{id}/mcp` (OAuth token-authenticated)
#[utoipa::path(
  post,
  path = ENDPOINT_APPS_MCPS.to_owned() + "/{id}/mcp",
  tag = API_TAG_MCPS,
  operation_id = "mcpProxy",
  summary = "Transparent MCP proxy endpoint",
  description = "Forwards MCP Streamable HTTP requests (POST/GET/DELETE) to upstream MCP server with auth injection",
  params(
    ("id" = String, Path, description = "MCP instance UUID")
  ),
  responses(
    (status = 200, description = "Upstream response forwarded"),
    (status = 403, description = "MCP server or instance disabled"),
    (status = 500, description = "Upstream connection failure"),
  ),
  security(("bearer_oauth_token" = []))
)]
pub async fn mcp_proxy_handler(
  auth_scope: AuthScope,
  Path(id): Path<String>,
  request: axum::extract::Request,
) -> Result<Response, ApiError> {
  // 1. Resolve MCP instance + server
  let mcp = auth_scope
    .mcps()
    .get(&id)
    .await?
    .ok_or_else(|| services::McpError::McpNotFound(id.clone()))?;

  // 2. Check enabled flags
  if !mcp.server_enabled {
    return Err(McpRouteError::McpServerDisabled.into());
  }
  if !mcp.enabled {
    return Err(McpRouteError::McpInstanceDisabled.into());
  }

  // 3. Resolve auth params (headers + query params for upstream)
  let auth_params = auth_scope.mcps().resolve_auth_params(&id).await?;

  // 4. Build upstream URL with auth query params
  let mut upstream_url = url::Url::parse(&mcp.server_url)
    .map_err(|e| McpRouteError::UpstreamConnectionFailed(format!("Invalid URL: {}", e)))?;

  if let Some(ref params) = auth_params {
    for (key, value) in &params.query_params {
      upstream_url.query_pairs_mut().append_pair(key, value);
    }
  }

  // 5. Decompose incoming request
  let (parts, body) = request.into_parts();
  let method = parts.method.clone();

  debug!(
    %method,
    upstream = %upstream_url,
    mcp_id = %id,
    server_name = %mcp.server_name,
    "Proxying MCP request to upstream"
  );

  // 6. Build upstream request
  let upstream_method =
    reqwest::Method::from_bytes(method.as_str().as_bytes()).unwrap_or_else(|_| {
      tracing::warn!(%method, "Unknown HTTP method, falling back to POST");
      reqwest::Method::POST
    });
  let mut upstream_req = HTTP_CLIENT.request(upstream_method, upstream_url.as_str());

  // Forward selected headers from client
  for header_name in FORWARD_REQUEST_HEADERS {
    if let Some(value) = parts.headers.get(*header_name) {
      if let Ok(value_str) = value.to_str() {
        upstream_req = upstream_req.header(*header_name, value_str);
      }
    }
  }

  // Force-set Accept header for MCP protocol compliance.
  // Some clients or browser fetch configurations may not pass Accept correctly.
  // reqwest::header() appends rather than replaces, so we overwrite explicitly.
  if method == axum::http::Method::POST {
    upstream_req = upstream_req.header(
      reqwest::header::ACCEPT,
      "application/json, text/event-stream",
    );
  } else if method == axum::http::Method::GET {
    upstream_req = upstream_req.header(reqwest::header::ACCEPT, "text/event-stream");
  }

  // Inject auth headers
  if let Some(ref params) = auth_params {
    for (key, value) in &params.headers {
      upstream_req = upstream_req.header(key.as_str(), value.as_str());
    }
  }

  // Forward request body (POST requests have JSON-RPC body)
  let body_bytes = axum::body::to_bytes(body, 10 * 1024 * 1024)
    .await
    .map_err(|e| McpRouteError::UpstreamConnectionFailed(format!("Failed to read body: {}", e)))?;
  if !body_bytes.is_empty() {
    upstream_req = upstream_req.body(body_bytes);
  }

  // 7. Send to upstream
  let upstream_resp = upstream_req
    .send()
    .await
    .map_err(|e| McpRouteError::UpstreamConnectionFailed(e.to_string()))?;

  // 8. Build response — stream body from upstream
  let status = upstream_resp.status();
  let mut response_builder = Response::builder().status(status.as_u16());

  // Forward selected headers from upstream
  for header_name in FORWARD_RESPONSE_HEADERS {
    if let Some(value) = upstream_resp.headers().get(*header_name) {
      response_builder = response_builder.header(*header_name, value);
    }
  }

  // Stream response body
  let body = Body::from_stream(upstream_resp.bytes_stream());
  Ok(response_builder.body(body).unwrap().into_response())
}
