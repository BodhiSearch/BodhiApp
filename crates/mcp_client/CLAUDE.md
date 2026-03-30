# mcp_client -- CLAUDE.md
**Companion docs** (load as needed):
- `PACKAGE.md` -- Implementation details and file index

## Purpose

MCP (Model Context Protocol) client for connecting to MCP servers over Streamable HTTP
transport. Provides `McpClient` trait for `fetch_tools` and `call_tool` operations.

## Architecture Position

- **Depends on**: `errmeta` (AppError, ErrorType), `errmeta_derive` (ErrorMeta), `rmcp` (protocol impl)
- **Consumed by**: `services` (MCP service layer uses `McpClient` trait)
- Lightweight: no `axum` or `sea-orm` dependency

## Non-Obvious Rules

### Dual connection patterns
1. **Per-request** (existing): `fetch_tools`, `call_tool`, `get_server_info` each create a fresh connection, operate, then `client.cancel()`. No pooling.
2. **Persistent** (new): `connect_persistent` returns a `PersistentMcpConnection` that stays open for multiple operations. Caller manages lifecycle via `close()`. Used by the MCP proxy endpoint (`routes_app::mcps::mcp_proxy`).

### PersistentMcpConnection
Wraps `RunningMcpClient` with methods for the full MCP protocol surface: `list_tools`, `call_tool`, `list_resources`, `read_resource`, `list_resource_templates`, `list_prompts`, `get_prompt`, `complete`, `subscribe`, `unsubscribe`, `close`. Returns `rmcp::service::ServiceError` (not `McpClientError`) from protocol methods. See `src/mcp_client.rs:30-145`.

### get_server_info
Connects, captures `peer_info()` (the `InitializeResult` from the handshake), serializes to `serde_json::Value`, disconnects. Used to inspect upstream MCP server capabilities without maintaining a connection.

### McpAuthParams -- headers + query params
All `McpClient` methods accept `auth_params: Option<McpAuthParams>`.
`McpAuthParams` (defined in `src/mcp_objs.rs`) carries:
- `headers: Vec<(String, String)>` -- injected as default headers on the reqwest client
- `query_params: Vec<(String, String)>` -- appended to the MCP server URL

The `prepare_auth` method handles URL construction and header validation. Query params are URL-encoded. Headers are lowercased. Invalid header names or unparseable URLs produce `ConnectionFailed` errors.

### McpTool schema caching
`McpTool` struct (name, description, input_schema as `serde_json::Value`) is the cached
representation of tools from an MCP server's `tools/list` response. It implements `ToSchema`
for OpenAPI integration.

### McpClientError variants
- `ConnectionFailed { url, reason }` -- ServiceUnavailable (503)
- `ProtocolError { operation, reason }` -- InternalServer (500)
- `ExecutionFailed { tool, reason }` -- InternalServer (500)
- `SerializationError { reason }` -- InternalServer (500)

All implement `AppError` via `#[derive(ErrorMeta)]`. See `src/error.rs`.

### MockMcpClient via mockall
`McpClient` trait has `#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]`.
Enable `test-utils` feature to get `MockMcpClient` in downstream crates. Note: `connect_persistent` returns `PersistentMcpConnection` which wraps a real rmcp client, so mocking returns errors (not real connections).
