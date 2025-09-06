-- Add prefix column to api_model_aliases table for model namespace routing
-- This enables differentiating between same models from different providers (e.g., azure/gpt-4 vs openai/gpt-4)

ALTER TABLE api_model_aliases ADD COLUMN prefix TEXT;

-- Add index for prefix-based lookups to optimize routing performance
CREATE INDEX idx_api_model_aliases_prefix ON api_model_aliases(prefix);