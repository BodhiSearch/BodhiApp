# mcp_client -- PACKAGE.md

## Module Structure

- `src/lib.rs` -- `McpTool` struct, `McpClient` trait (mockall-enabled), `DefaultMcpClient` impl, `PersistentMcpConnection`, `RunningMcpClient` type alias
- `src/error.rs` -- `McpClientError` enum (4 variants) implementing `AppError`

## McpClient Trait

- `fetch_tools(url, auth_params)` -> `Result<Vec<McpTool>, McpClientError>` -- Connect, list tools, disconnect
- `call_tool(url, tool_name, args, auth_params)` -> `Result<Value, McpClientError>` -- Connect, call tool, disconnect
- `get_server_info(url, auth_params)` -> `Result<Value, McpClientError>` -- Connect, get InitializeResult as JSON, disconnect
- `connect_persistent(url, auth_params)` -> `Result<PersistentMcpConnection, McpClientError>` -- Connect and return long-lived connection

## PersistentMcpConnection

Wraps `RunningMcpClient`. Methods return `Result<T, rmcp::service::ServiceError>`:
- `list_tools`, `call_tool`, `list_resources`, `read_resource`, `list_resource_templates`
- `list_prompts`, `get_prompt`, `complete`, `subscribe`, `unsubscribe`
- `server_info() -> Option<Value>` -- peer info from handshake
- `is_closed() -> bool`
- `close(self) -> Result<(), McpClientError>` -- consumes self

## DefaultMcpClient Implementation

Uses `rmcp` crate with `StreamableHttpClientTransport`. Client info identifies as
`"bodhi-mcp-client"` with version from `CARGO_PKG_VERSION`.

Connection flow:
1. Build reqwest client (optionally with auth headers)
2. Create `StreamableHttpClientTransport` with URL
3. `ClientInfo.serve(transport)` to establish connection
4. Perform operation (per-request methods cancel after; persistent returns connection)

## Features

- `test-utils` -- Enables `MockMcpClient` via mockall
