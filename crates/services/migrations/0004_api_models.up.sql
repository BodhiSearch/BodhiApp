-- Add up migration script here

-- Create the api_model_aliases table for storing remote API configuration
CREATE TABLE api_model_aliases (
    id TEXT PRIMARY KEY NOT NULL,
    provider TEXT NOT NULL,
    base_url TEXT NOT NULL,
    models_json TEXT NOT NULL,
    encrypted_api_key TEXT NOT NULL,
    salt TEXT NOT NULL,
    nonce TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);

-- Create index on provider for faster lookups when filtering by provider
CREATE INDEX idx_api_model_aliases_provider ON api_model_aliases(provider);

-- Create index on updated_at for faster sorting by modification time
CREATE INDEX idx_api_model_aliases_updated_at ON api_model_aliases(updated_at);