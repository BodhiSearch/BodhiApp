# mcp_client -- PACKAGE.md

## Module Structure

- `src/lib.rs` -- `McpTool` struct, `McpClient` trait (mockall-enabled), `DefaultMcpClient` impl, `RunningMcpClient` type alias
- `src/error.rs` -- `McpClientError` enum (4 variants) implementing `AppError`

## McpClient Trait

- `fetch_tools(url, auth_header)` -> `Result<Vec<McpTool>, McpClientError>` -- Connect, list tools, disconnect
- `call_tool(url, tool_name, args, auth_header)` -> `Result<Value, McpClientError>` -- Connect, call tool, disconnect

## DefaultMcpClient Implementation

Uses `rmcp` crate with `StreamableHttpClientTransport`. Client info identifies as
`"bodhi-mcp-client"` with version from `CARGO_PKG_VERSION`.

Connection flow:
1. Build reqwest client (optionally with auth header)
2. Create `StreamableHttpClientTransport` with URL
3. `ClientInfo.serve(transport)` to establish connection
4. Perform operation (list_tools or call_tool)
5. `client.cancel()` to disconnect

## Features

- `test-utils` -- Enables `MockMcpClient` via mockall
