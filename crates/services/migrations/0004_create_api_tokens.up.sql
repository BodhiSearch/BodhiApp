-- Add up migration script here

-- Create the api_tokens table
CREATE TABLE api_tokens (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    name TEXT DEFAULT '',
    token_id TEXT NOT NULL UNIQUE,
    status TEXT NOT NULL CHECK (status IN ('active', 'inactive')),
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
