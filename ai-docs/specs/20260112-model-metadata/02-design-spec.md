# Model Metadata API - Design Specification

**Date**: 2026-01-12
**Version**: 1.0

## User Preferences (Interview Results)

### API Design Decisions

**Endpoint Structure**:
- `GET /bodhi/v1/models` - List all models with metadata
- `GET /bodhi/v1/models/{id}/meta` - Get specific model metadata
- `POST /bodhi/v1/models/sync` - Trigger manual sync (admin only)

**Rationale**: Separate from OpenAI-compatible `/v1/models` which has no metadata support. Using `/meta` subresource makes the purpose explicit.

**Response Format**:
- Single flat response (no core/extended distinction)
- All fields returned in one response for simplicity

**Filtering & Computed Fields**:
- No filtering initially (client-side filtering sufficient for expected model counts)
- Raw values only (no computed fields like effective_context, is_loaded)

### Data Source Strategy

**Remote Models**:
- **Hybrid approach**: Proxy provider metadata + static capability mapping
- **Rationale**: Combines freshness (from providers that expose metadata) with reliability (fallback to static)

**Local GGUF Models**:
- Extract metadata directly from GGUF files using existing parser
- **Rationale**: No dependency on running llama.cpp server, works offline

**Unknown Models**:
- Conservative defaults (text-only, no thinking, no special capabilities)
- **Rationale**: Avoid false positives that could break client functionality

### Capability Modeling

**Thinking/Reasoning**:
- Simple capability flag only (`thinking: bool`)
- No activation method abstraction
- **Rationale**: Activation methods too fragmented (suffix, params, native), expose just capability

**Tools**:
- Grouped under `tools: { function_calling, structured_output }`
- **Rationale**: Logical grouping, matches how providers conceptually organize these features

**Model IDs**:
- Canonical model IDs (e.g., "claude-sonnet-4") with separate `provider` field
- **Rationale**: Simplifies matching, provider is orthogonal to model identity

### Storage & Synchronization

**Storage**: SQLite table `model_metadata`
- Consistent with existing BodhiApp data layer
- Enables SQL queries for alias matching

**Seeding**: Database seeded at app install time
- Includes popular models from major providers
- Ensures offline functionality

**Sync Source**: Dedicated API at `api.getbodhi.app`
- Maintained separately from main application
- Enables independent metadata updates

**Sync Trigger**: App polls api.getbodhi.app periodically (default: 24 hours)
- Manual trigger available via admin endpoint
- Configurable via `BODHI_MODEL_METADATA_SYNC_INTERVAL`

**Versioning**: `updated_at` timestamp per record
- Tracks when metadata last changed
- No full audit trail (single version only)

### Pricing Information

**Include with Staleness Warning**:
- Optional `pricing` field with `pricing_updated_at` timestamp
- **Rationale**: Pricing useful but volatile, timestamp lets clients decide freshness tolerance

### Multi-Provider Handling

**Single Canonical Entry** per model:
- One metadata record for "claude-sonnet-4" regardless of provider
- **Rationale**: Capabilities are model-inherent, not provider-specific

### Implementation Scope

**Include GGUF Enhancements**:
- ModelCapabilities struct
- Typed accessors for common fields
- **Rationale**: Essential for local model metadata, foundational work

**Define api.getbodhi.app Contract**:
- Part of this specification
- **Rationale**: Ensures BodhiApp and metadata service stay aligned

---

## Final BodhiApp Schema

### Model Metadata Response Schema

Returned by `GET /bodhi/v1/models/{id}/meta`:

```json
{
  "id": "claude-sonnet-4",
  "name": "Claude Sonnet 4",
  "description": "Anthropic's latest balanced model combining intelligence with speed",
  "provider": "anthropic",

  "capabilities": {
    "vision": true,
    "audio": false,
    "thinking": true,
    "tools": {
      "function_calling": true,
      "structured_output": true
    }
  },

  "context": {
    "max_input_tokens": 200000,
    "max_output_tokens": 8192
  },

  "pricing": {
    "input_per_million_tokens": 3.00,
    "output_per_million_tokens": 15.00,
    "currency": "USD",
    "updated_at": "2026-01-10T00:00:00Z"
  },

  "architecture": {
    "family": "claude",
    "parameter_count": null,
    "quantization": null,
    "format": null
  },

  "updated_at": "2026-01-10T00:00:00Z"
}
```

### Local GGUF Model Response Example

```json
{
  "id": "llama-3.2-1b-instruct-q4_k_m",
  "name": "Llama 3.2 1B Instruct Q4_K_M",
  "description": null,
  "provider": "local",

  "capabilities": {
    "vision": false,
    "audio": false,
    "thinking": false,
    "tools": {
      "function_calling": false,
      "structured_output": false
    }
  },

  "context": {
    "max_input_tokens": 131072,
    "max_output_tokens": null
  },

  "pricing": null,

  "architecture": {
    "family": "llama",
    "parameter_count": 1000000000,
    "quantization": "Q4_K_M",
    "format": "gguf"
  },

  "updated_at": "2026-01-12T10:30:00Z"
}
```

### List Response Schema

Returned by `GET /bodhi/v1/models`:

```json
{
  "models": [
    {
      "id": "claude-sonnet-4",
      "name": "Claude Sonnet 4",
      "provider": "anthropic",
      "capabilities": {
        "vision": true,
        "audio": false,
        "thinking": true,
        "tools": {
          "function_calling": true,
          "structured_output": true
        }
      },
      "context": {
        "max_input_tokens": 200000,
        "max_output_tokens": 8192
      },
      "updated_at": "2026-01-10T00:00:00Z"
    }
  ]
}
```

**Note**: List response may omit some fields (description, pricing, architecture) for performance. Use `/meta` endpoint for complete metadata.

---

## Type Definitions

### Rust Domain Types

```rust
// Core metadata type
pub struct ModelMetadata {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub provider: String,
    pub capabilities: ModelCapabilities,
    pub context: Option<ContextLimits>,
    pub pricing: Option<ModelPricing>,
    pub architecture: Option<ModelArchitecture>,
    pub updated_at: DateTime<Utc>,
}

// Capabilities structure
pub struct ModelCapabilities {
    pub vision: bool,
    pub audio: bool,
    pub thinking: bool,
    pub tools: ToolCapabilities,
}

pub struct ToolCapabilities {
    pub function_calling: bool,
    pub structured_output: bool,
}

// Context limits
pub struct ContextLimits {
    pub max_input_tokens: Option<u64>,
    pub max_output_tokens: Option<u64>,
}

// Pricing information
pub struct ModelPricing {
    pub input_per_million_tokens: f64,
    pub output_per_million_tokens: f64,
    pub currency: String,
    pub updated_at: DateTime<Utc>,
}

// Architecture details
pub struct ModelArchitecture {
    pub family: Option<String>,
    pub parameter_count: Option<u64>,
    pub quantization: Option<String>,
    pub format: Option<String>,
}
```

### Database Schema

```sql
CREATE TABLE model_metadata (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  description TEXT,
  provider TEXT NOT NULL,
  aliases TEXT NOT NULL,  -- JSON array: ["anthropic/claude-sonnet-4", "claude-sonnet-4-20241022"]
  capabilities TEXT NOT NULL,  -- JSON object: ModelCapabilities
  context TEXT,  -- JSON object: ContextLimits
  pricing TEXT,  -- JSON object: ModelPricing
  architecture TEXT,  -- JSON object: ModelArchitecture
  updated_at TEXT NOT NULL,
  synced_at TEXT  -- When last synced from remote
);

CREATE INDEX idx_model_metadata_provider ON model_metadata(provider);
```

**Design Notes**:
- `aliases` stored as JSON array for flexible matching
- Nested objects stored as JSON for schema flexibility
- `synced_at` tracks remote sync, `updated_at` tracks metadata changes

---

## api.getbodhi.app API Contract

### Overview

External metadata service maintained separately from BodhiApp. Provides curated model metadata from multiple providers.

### GET /v1/model-metadata

Returns complete model metadata for synchronization.

**Request**: No parameters

**Response**:
```json
{
  "version": "2026-01-12",
  "models": [
    {
      "id": "claude-sonnet-4",
      "name": "Claude Sonnet 4",
      "description": "Anthropic's latest balanced model",
      "provider": "anthropic",
      "aliases": [
        "anthropic/claude-sonnet-4",
        "claude-sonnet-4-20241022"
      ],
      "capabilities": {
        "vision": true,
        "audio": false,
        "thinking": true,
        "tools": {
          "function_calling": true,
          "structured_output": true
        }
      },
      "context": {
        "max_input_tokens": 200000,
        "max_output_tokens": 8192
      },
      "pricing": {
        "input_per_million_tokens": 3.00,
        "output_per_million_tokens": 15.00,
        "currency": "USD",
        "updated_at": "2026-01-10T00:00:00Z"
      },
      "architecture": {
        "family": "claude"
      },
      "updated_at": "2026-01-10T00:00:00Z"
    }
  ]
}
```

**Response Fields**:
- `version`: ISO date of metadata snapshot, for change detection
- `models`: Array of complete model metadata objects
- `aliases`: Array of alternative model identifiers for matching

### GET /v1/model-metadata/{modelId}

Returns metadata for a specific model.

**Request**: `modelId` path parameter (canonical ID)

**Response**: Single model metadata object (same schema as array element above)

**Status Codes**:
- `200 OK`: Model found
- `404 Not Found`: Model not in database

### Sync Behavior

**Client (BodhiApp) Sync Flow**:

1. BodhiApp calls `GET /v1/model-metadata` periodically (configurable interval)
2. Response includes `version` for change detection (compare with last sync)
3. For each model in response:
   - Upsert by canonical `id`
   - Update `aliases` array for matching
   - Update `synced_at` timestamp
4. Log sync statistics (added, updated, unchanged)

**Version-Based Optimization**:
- Client can store last `version` synced
- Skip processing if `version` unchanged
- Full sync on version change

**Alias Matching**:
- `aliases` array enables matching `anthropic/claude-sonnet-4` → `claude-sonnet-4`
- Client can lookup by any alias, resolves to canonical ID

**Error Handling**:
- Sync failures are logged but don't block application
- Use last successfully synced data
- Retry with exponential backoff

---

## Endpoint Behavior Specification

### GET /bodhi/v1/models

**Purpose**: List all available models with metadata

**Request**:
- No query parameters initially (filtering deferred)

**Response**:
```json
{
  "models": [ModelMetadata]
}
```

**Behavior**:
- Returns merged data from:
  1. Local GGUF models (extracted metadata)
  2. Remote API aliases (from model_metadata table)
- Sorted by provider, then name
- May omit some fields for performance (description, pricing, architecture)

**Status Codes**:
- `200 OK`: Always returns (empty array if no models)

### GET /bodhi/v1/models/{id}/meta

**Purpose**: Get complete metadata for specific model

**Request**:
- `id`: Model identifier (canonical or alias)

**Response**: Single ModelMetadata object

**Behavior**:
- Lookup by canonical ID or alias
- For local GGUF: Extract metadata on-demand
- For remote API: Lookup in model_metadata table
- Returns merged data (DB enrichment + file extraction)

**Status Codes**:
- `200 OK`: Model found
- `404 Not Found`: Model not found (neither local nor remote)

### POST /bodhi/v1/models/sync

**Purpose**: Manually trigger metadata sync from remote

**Authorization**: Admin role required

**Request**: Empty body

**Response**:
```json
{
  "status": "success",
  "statistics": {
    "added": 5,
    "updated": 12,
    "unchanged": 134,
    "errors": 0
  },
  "version": "2026-01-12",
  "synced_at": "2026-01-12T15:30:00Z"
}
```

**Behavior**:
- Triggers immediate sync from api.getbodhi.app
- Returns sync statistics
- Updates last sync timestamp

**Status Codes**:
- `200 OK`: Sync completed (check statistics for errors)
- `401 Unauthorized`: Not authenticated
- `403 Forbidden`: Not admin role
- `503 Service Unavailable`: Remote sync service unreachable

---

## Configuration

### Environment Variables

```bash
# Remote metadata service URL
BODHI_MODEL_METADATA_URL=https://api.getbodhi.app/v1/model-metadata

# Sync interval in seconds (default: 86400 = 24 hours)
BODHI_MODEL_METADATA_SYNC_INTERVAL=86400

# Timeout for sync requests in seconds (default: 30)
BODHI_MODEL_METADATA_SYNC_TIMEOUT=30
```

### Feature Flags

```rust
// Enable automatic background sync (default: true)
pub const AUTO_SYNC_ENABLED: bool = true;

// Enable GGUF capability extraction (default: true)
pub const GGUF_CAPABILITIES_ENABLED: bool = true;
```

---

## Error Handling

### GGUF Parsing Errors

- Log warning, return conservative defaults (text-only)
- Don't fail request if one model fails to parse

### Remote Sync Errors

- Log error, use last successful sync data
- Retry with exponential backoff (1min, 5min, 15min, 1hr)
- Don't block application startup

### Missing Metadata

- Return null for optional fields
- Return conservative defaults for capabilities (all false)
- Include `updated_at: null` to indicate unknown freshness

---

## Migration Strategy

### Phase 1: Database Schema
- Create `model_metadata` table
- Seed with popular models

### Phase 2: GGUF Enhancement
- Add capability extraction to parser
- No breaking changes to existing parser API

### Phase 3: Service Layer
- Add repository for model_metadata
- Add sync service with remote fetch

### Phase 4: API Endpoints
- Add new routes under `/bodhi/v1/models`
- Existing `/v1/models` unchanged (OpenAI compatibility)

### Phase 5: Integration
- Enhance alias resolution to include metadata
- Update clients to consume new endpoints

**No Breaking Changes**: Existing endpoints remain unchanged, new functionality additive.
