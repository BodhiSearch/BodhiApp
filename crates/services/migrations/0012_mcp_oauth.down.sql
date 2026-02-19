-- Migration 0012 down: Remove OAuth tables

DROP INDEX IF EXISTS idx_mcp_oauth_configs_server_name;
DROP TABLE IF EXISTS mcp_oauth_tokens;
DROP TABLE IF EXISTS mcp_oauth_configs;
