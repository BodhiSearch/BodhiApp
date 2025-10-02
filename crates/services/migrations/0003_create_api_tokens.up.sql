-- Add up migration script here

-- Create the api_tokens table
CREATE TABLE api_tokens (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    name TEXT DEFAULT '',
    token_prefix TEXT NOT NULL UNIQUE,
    token_hash TEXT NOT NULL,
    scopes TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('active', 'inactive')),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create index on token_prefix for faster lookups
CREATE INDEX idx_api_tokens_token_prefix ON api_tokens(token_prefix);
