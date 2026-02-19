-- Migration 0010 down: Remove MCP server tables

-- Drop mcps indexes and table first
DROP INDEX IF EXISTS idx_mcps_mcp_server_id;
DROP INDEX IF EXISTS idx_mcps_created_by;
DROP TABLE IF EXISTS mcps;

-- Drop mcp_servers index and table
DROP INDEX IF EXISTS idx_mcp_servers_url;
DROP TABLE IF EXISTS mcp_servers;
