-- Model metadata table for local GGUF and remote API models
CREATE TABLE model_metadata (
  -- Primary key
  id INTEGER PRIMARY KEY AUTOINCREMENT,

  -- Model identification
  source TEXT NOT NULL,        -- AliasSource: "local", "api"
  repo TEXT,                   -- HuggingFace repo (e.g., "microsoft/phi-3") - NULL for API models
  filename TEXT,               -- GGUF file path - NULL for API models
  snapshot TEXT,               -- Snapshot identifier for change detection - NULL for API models
  api_model_id TEXT,           -- Remote model ID (e.g., "gpt-4") - NULL for local models

  -- Core capabilities (nullable - NULL = unknown/not determined)
  capabilities_vision INTEGER,
  capabilities_audio INTEGER,
  capabilities_thinking INTEGER,
  capabilities_function_calling INTEGER,
  capabilities_structured_output INTEGER,

  -- Context limits (flattened for queryable filters)
  context_max_input_tokens INTEGER,
  context_max_output_tokens INTEGER,

  -- Architecture and additional metadata (JSON for flexibility)
  architecture TEXT,           -- JSON: ModelArchitecture
  additional_metadata TEXT,    -- JSON: Future extensibility (pricing, etc.)
  chat_template TEXT,          -- Chat template from GGUF file (can be large)

  -- Timestamps
  extracted_at TEXT NOT NULL,  -- When metadata was extracted/synced
  created_at TEXT NOT NULL,    -- Record creation timestamp
  updated_at TEXT NOT NULL,    -- Record update timestamp

  -- Composite unique constraint
  -- For local models: unique on (source, repo, filename, snapshot) where source='local'
  -- For API models: api_model_id is unique where source='api'
  UNIQUE(source, repo, filename, snapshot, api_model_id)
);

-- Indexes for common queries
CREATE INDEX idx_model_metadata_source ON model_metadata(source);
CREATE INDEX idx_model_metadata_repo ON model_metadata(repo);
CREATE INDEX idx_model_metadata_filename ON model_metadata(filename);
CREATE INDEX idx_model_metadata_api_model_id ON model_metadata(api_model_id);
CREATE INDEX idx_model_metadata_vision ON model_metadata(capabilities_vision) WHERE capabilities_vision = 1;
CREATE INDEX idx_model_metadata_audio ON model_metadata(capabilities_audio) WHERE capabilities_audio = 1;
CREATE INDEX idx_model_metadata_function_calling ON model_metadata(capabilities_function_calling) WHERE capabilities_function_calling = 1;
