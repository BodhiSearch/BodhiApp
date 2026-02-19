-- Migration 0011: Add header-based authentication columns to MCP instances
-- Supports per-instance auth with encrypted header values (AES-256-GCM)

ALTER TABLE mcps ADD COLUMN auth_type TEXT NOT NULL DEFAULT 'public';
ALTER TABLE mcps ADD COLUMN auth_header_key TEXT;
ALTER TABLE mcps ADD COLUMN encrypted_auth_header_value TEXT;
ALTER TABLE mcps ADD COLUMN auth_header_salt TEXT;
ALTER TABLE mcps ADD COLUMN auth_header_nonce TEXT;
