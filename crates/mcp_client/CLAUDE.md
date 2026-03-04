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

### Per-request connection pattern
`DefaultMcpClient` creates a fresh connection for every `fetch_tools` or `call_tool` call.
No connection pooling. Each call: connect -> operate -> `client.cancel()` -> return.
See `src/lib.rs:90-131` for the `connect` method.

### Auth header injection
Both `fetch_tools` and `call_tool` accept `auth_header: Option<(String, String)>` as
`(header_name, header_value)`. This is injected as a default header on the reqwest client
before establishing the MCP connection. Used for API key forwarding to MCP servers.

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
Enable `test-utils` feature to get `MockMcpClient` in downstream crates.
