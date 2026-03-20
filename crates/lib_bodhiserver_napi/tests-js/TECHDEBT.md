# E2E Tests — Tech Debt

## Page Object Split

`McpsPage.mjs` currently handles both MCP instances (`/ui/mcps/`) and MCP servers (`/ui/mcps/servers/`). Split into separate `McpServersPage` and `McpInstancesPage` page objects for clearer ownership and reduced file size.
