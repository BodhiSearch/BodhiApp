-- Migration 0008 down: Remove app_client_tool_configs table

DROP INDEX IF EXISTS idx_app_client_tool_configs_client_id;
DROP TABLE IF EXISTS app_client_tool_configs;
