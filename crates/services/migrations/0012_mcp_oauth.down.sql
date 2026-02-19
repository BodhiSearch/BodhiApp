-- Migration 0012 down: Remove OAuth tables

DROP TABLE IF EXISTS mcp_oauth_tokens;
DROP TABLE IF EXISTS mcp_oauth_configs;
