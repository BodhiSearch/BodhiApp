-- Migration 0011 down: Remove mcp_auth_headers table

DROP INDEX IF EXISTS idx_mcp_auth_headers_server;
DROP INDEX IF EXISTS idx_mcp_auth_headers_server_name;
DROP TABLE IF EXISTS mcp_auth_headers;
