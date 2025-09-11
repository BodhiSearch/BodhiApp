-- Add up migration script here

-- Create the api_model_aliases table for storing remote API configuration
CREATE TABLE api_model_aliases (
    id TEXT PRIMARY KEY NOT NULL,
    api_format TEXT NOT NULL,
    prefix TEXT,
    base_url TEXT NOT NULL,
    models_json TEXT NOT NULL,
    encrypted_api_key TEXT NOT NULL,
    salt TEXT NOT NULL,
    nonce TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);

-- Create index on api_format for faster lookups when filtering by api format
CREATE INDEX idx_api_model_aliases_api_format ON api_model_aliases(api_format);

-- Add index for prefix-based lookups to optimize routing performance
CREATE INDEX idx_api_model_aliases_prefix ON api_model_aliases(prefix);

-- Create index on updated_at for faster sorting by modification time
CREATE INDEX idx_api_model_aliases_updated_at ON api_model_aliases(updated_at);