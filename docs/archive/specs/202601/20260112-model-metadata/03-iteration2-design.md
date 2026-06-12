# Model Metadata API - Iteration 2 Design Specification

**Date**: 2026-01-12
**Version**: Iteration 2 (Remote API Models)
**Scope**: Iteration 2 extends iteration 1 to support remote API model metadata via sync with api.getbodhi.app

## Overview

Iteration 2 builds on iteration 1's foundation (local GGUF models) by adding support for remote API model metadata. The database schema from iteration 1 is extended to accommodate remote models, and the existing queue infrastructure is reused for sync operations. The approach is **contract-first**: BodhiApp client and api.getbodhi.app contract are designed together, but the server implementation is deferred to a separate project.

## Design Decisions Summary

| Aspect | Decision | Rationale |
|--------|----------|-----------|
| **Sync Source** | Contract-first, server deferred | Design BodhiApp client + contract, build server separately |
| **Queue Reuse** | Same QueueProducer/QueueConsumer | Add SyncAll/SyncSingle task types alongside RefreshAll/RefreshSingle |
| **Sync Trigger** | Manual only (admin) | Consistent with iteration 1 approach, no automatic sync |
| **Alias Storage** | model_metadata table with aliases JSON array | Single source of truth, synced from api.getbodhi.app |
| **ApiAlias Integration** | Enrich existing | Join ApiAlias with model_metadata by matching normalized model names |
| **Seed Strategy** | No seeding | Start empty, wait for first sync from api.getbodhi.app |
| **Sync Protocol** | Cursor-based paginated | Server returns cursor token, client passes for next page |
| **UI Scope** | Minimal | Sync button (admin), pricing badge display |
| **Sync Failure** | Silent log | Log warning, return error to caller, use stale data |
| **Endpoint Design** | Query parameter `scope` | POST /bodhi/v1/models/refresh?scope=local\|remote\|all |
| **Pricing Detail** | Detailed tiers as JSON | Informational only (input/output/cached/batch pricing) |
| **Schema Extension** | aliases + pricing_updated_at + provider + name + description + synced_at | Minimal column additions |
| **Join Strategy** | Normalize + exact match | Remove slashes, use last part, exact match against aliases |
| **API Auth** | No auth | Public endpoint, server handles rate limiting by IP |

---

## API Specification

### Endpoint Changes

#### Modified: POST /bodhi/v1/models/refresh

**New Query Parameters**:
- `scope`: `local` | `remote` | `all` (default: `all`)

**Behavior**:
- `scope=local`: Enqueue RefreshAll task (existing iteration 1 behavior)
- `scope=remote`: Enqueue SyncAll task (new iteration 2)
- `scope=all`: Enqueue both RefreshAll and SyncAll tasks

**Response**: Same 202 Accepted format as iteration 1

```json
{
  "status": "accepted",
  "message": "Metadata sync started in background"
}
```

**Status Codes**:
- `202 Accepted`: Tasks queued successfully
- `401 Unauthorized`: Not authenticated
- `403 Forbidden`: Not admin role

#### Modified: POST /bodhi/v1/models/{id}/refresh

**New Query Parameters**:
- `scope`: `local` | `remote` (required when alias ambiguous between local and remote)

**Behavior**:
- For local alias (UserAlias/ModelAlias): Enqueue RefreshSingle (iteration 1)
- For remote model ID: Enqueue SyncSingle (iteration 2)

**Response**:
```json
{
  "status": "accepted",
  "message": "Metadata refresh started for model 'claude-sonnet-4'"
}
```

#### Modified: GET /bodhi/v1/models

**New Behavior**:
- ApiAliasResponse entries now include optional `metadata` field when matched with synced remote model metadata

**Example Response with Enriched ApiAlias**:
```json
{
  "data": [
    {
      "source": "api",
      "id": "openai-gpt4",
      "api_format": "openai",
      "base_url": "https://api.openai.com/v1",
      "models": ["gpt-4"],
      "metadata": {
        "capabilities": {
          "vision": true,
          "audio": false,
          "thinking": false,
          "tools": {
            "function_calling": true,
            "structured_output": true
          }
        },
        "context": {
          "max_input_tokens": 128000,
          "max_output_tokens": 4096
        },
        "pricing": {
          "input_per_million_tokens": 30.00,
          "output_per_million_tokens": 60.00,
          "cached_input_per_million_tokens": 15.00,
          "batch_input_per_million_tokens": 15.00,
          "batch_output_per_million_tokens": 30.00,
          "image_per_token": null,
          "currency": "USD"
        },
        "architecture": {
          "family": "gpt-4",
          "parameter_count": null,
          "quantization": null,
          "format": null
        }
      }
    }
  ],
  "total": 1,
  "page": 1,
  "page_size": 20
}
```

---

## Database Schema Extension

### Migration: add_remote_model_support

Extend existing `model_metadata` table (from iteration 1):

```sql
-- Add columns for remote model support
ALTER TABLE model_metadata ADD COLUMN aliases TEXT;  -- JSON array: ["anthropic/claude-sonnet-4", "claude-sonnet-4-20241022"]
ALTER TABLE model_metadata ADD COLUMN pricing TEXT;  -- JSON: detailed pricing tiers
ALTER TABLE model_metadata ADD COLUMN pricing_updated_at TEXT;  -- Pricing staleness tracking
ALTER TABLE model_metadata ADD COLUMN provider TEXT;  -- Canonical provider: "anthropic", "openai", etc.
ALTER TABLE model_metadata ADD COLUMN name TEXT;  -- Human-readable name: "Claude Sonnet 4"
ALTER TABLE model_metadata ADD COLUMN description TEXT;  -- Model description
ALTER TABLE model_metadata ADD COLUMN synced_at TEXT;  -- Last sync timestamp from api.getbodhi.app

-- Create index for provider filtering
CREATE INDEX idx_model_metadata_provider ON model_metadata(provider);
```

### Design Rationale

**Aliases Column** (JSON array):
- Stores alternative model identifiers for flexible matching
- Synced from api.getbodhi.app response
- Enables matching "anthropic/claude-sonnet-4" → "claude-sonnet-4"

**Pricing Column** (JSON object):
- Detailed pricing tiers (input/output/cached/batch)
- Informational only - not used for cost calculations
- Staleness tracked via `pricing_updated_at` timestamp

**Provider Column**:
- Canonical provider name: "anthropic", "openai", "google", etc.
- Enables provider-based filtering and grouping
- Indexed for efficient queries

**Name and Description**:
- Human-readable model information
- Displayed in UI alongside technical metadata

**Synced_at Column**:
- Tracks when metadata was last synced from api.getbodhi.app
- Distinct from `updated_at` which tracks metadata changes
- Enables sync freshness monitoring

### Pricing JSON Schema (Informational)

```json
{
  "input_per_million_tokens": 3.00,
  "output_per_million_tokens": 15.00,
  "cached_input_per_million_tokens": 0.30,
  "batch_input_per_million_tokens": 1.50,
  "batch_output_per_million_tokens": 7.50,
  "image_per_token": null,
  "currency": "USD"
}
```

**Fields** (all optional):
- `input_per_million_tokens`: Standard input pricing
- `output_per_million_tokens`: Standard output pricing
- `cached_input_per_million_tokens`: Cached/prompt-cached input pricing
- `batch_input_per_million_tokens`: Batch API input pricing
- `batch_output_per_million_tokens`: Batch API output pricing
- `image_per_token`: Per-token pricing for image inputs
- `currency`: ISO 4217 currency code (default: "USD")

---

## Queue Service Extension

### Extended Task Types

```rust
#[derive(Debug, Clone)]
pub enum RefreshTask {
    // Existing (iteration 1)
    RefreshAll {
        created_at: DateTime<Utc>,
    },
    RefreshSingle {
        alias: String,
        created_at: DateTime<Utc>,
    },

    // New (iteration 2)
    SyncAll {
        created_at: DateTime<Utc>,
    },
    SyncSingle {
        model_id: String,
        created_at: DateTime<Utc>,
    },
}
```

### Worker Processing Extension

Worker's `process_task` method handles new variants:

```rust
async fn process_task(&self, task: RefreshTask) -> Result<()> {
    match task {
        RefreshTask::RefreshAll { .. } => self.refresh_all().await,
        RefreshTask::RefreshSingle { alias, .. } => self.refresh_single(&alias).await,

        // New iteration 2 handlers
        RefreshTask::SyncAll { .. } => self.sync_all().await,
        RefreshTask::SyncSingle { model_id, .. } => self.sync_single(&model_id).await,
    }
}
```

**SyncAll Implementation**:
1. Fetch all pages from api.getbodhi.app using cursor-based pagination
2. For each model in response:
   - Normalize model_id
   - Upsert to model_metadata table
   - Update synced_at timestamp
3. Log sync statistics (added, updated, unchanged, errors)

**SyncSingle Implementation**:
1. Fetch single model by canonical ID from api.getbodhi.app
2. Normalize model_id
3. Upsert to model_metadata table
4. Update synced_at timestamp

### Error Handling

**Network Errors**:
- Log warning with error details
- Return error to caller (endpoint returns 500 with error message)
- Don't fail entire sync - continue with next model

**Parse Errors**:
- Log warning with malformed model data
- Skip model, continue with next
- Don't propagate error to caller

**Database Errors**:
- Log error with details
- Return error to caller
- Don't retry (manual re-trigger required)

---

## api.getbodhi.app API Contract

### Overview

External metadata service providing curated model metadata from multiple providers. The contract is designed for BodhiApp consumption, but the server implementation is deferred to a separate project.

### GET /v1/model-metadata

**Purpose**: Fetch all remote model metadata with cursor-based pagination

**Request Parameters**:
- `cursor`: Optional cursor token from previous response (query parameter)
- `limit`: Optional page size (default: 100, max: 500, query parameter)

**Response Schema**:
```json
{
  "version": "2026-01-12",
  "models": [
    {
      "id": "claude-sonnet-4",
      "name": "Claude Sonnet 4",
      "description": "Anthropic's latest balanced model combining intelligence with speed",
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
        "cached_input_per_million_tokens": 0.30,
        "batch_input_per_million_tokens": 1.50,
        "batch_output_per_million_tokens": 7.50,
        "image_per_token": null,
        "currency": "USD"
      },
      "architecture": {
        "family": "claude",
        "parameter_count": null,
        "quantization": null,
        "format": null
      },
      "updated_at": "2026-01-10T00:00:00Z"
    }
  ],
  "next_cursor": "abc123",
  "has_more": true
}
```

**Response Fields**:
- `version`: ISO date of metadata snapshot (for change detection, optional optimization)
- `models`: Array of model metadata objects
- `next_cursor`: Cursor token for next page (null when no more pages)
- `has_more`: Boolean indicating if more pages exist

**Cursor Logic**:
- First request: omit `cursor` parameter
- Subsequent requests: pass `cursor=<next_cursor>` from previous response
- When `has_more: false`, pagination is complete

**Pagination Example**:
```
Request 1: GET /v1/model-metadata?limit=100
Response 1: { models: [...100], next_cursor: "abc", has_more: true }

Request 2: GET /v1/model-metadata?cursor=abc&limit=100
Response 2: { models: [...100], next_cursor: "def", has_more: true }

Request 3: GET /v1/model-metadata?cursor=def&limit=100
Response 3: { models: [...50], next_cursor: null, has_more: false }
```

**Status Codes**:
- `200 OK`: Successful fetch
- `400 Bad Request`: Invalid cursor or limit
- `500 Internal Server Error`: Server error

### GET /v1/model-metadata/{modelId}

**Purpose**: Fetch single model metadata by canonical ID

**Path Parameter**:
- `modelId`: Canonical model identifier (e.g., "claude-sonnet-4", "gpt-4")

**Response**: Single model metadata object (same schema as array element above)

**Status Codes**:
- `200 OK`: Model found
- `404 Not Found`: Model not in database

### Contract Design Principles

**Simplicity**:
- Public endpoint, no authentication
- Server handles rate limiting by IP
- No versioning in URL path (use `version` field for change detection)

**Extensibility**:
- JSON allows adding new fields without breaking clients
- Optional fields (pricing, architecture) enable partial metadata

**Reliability**:
- Cursor-based pagination handles large datasets
- Idempotent fetches (same cursor returns same results)
- No state in BodhiApp client (cursor returned by server)

---

## ApiAlias Enrichment

### Model Name Normalization

```rust
/// Normalize model name for metadata matching.
///
/// Removes provider prefix (everything before last slash) to match
/// against model_metadata.model_id or aliases array.
///
/// Examples:
/// - "anthropic/claude-sonnet-4" → "claude-sonnet-4"
/// - "gpt-4" → "gpt-4"
/// - "openai/gpt-4-turbo" → "gpt-4-turbo"
fn normalize_model_name(model: &str) -> String {
    model
        .rsplit('/')
        .next()
        .unwrap_or(model)
        .to_string()
}
```

### Join Logic

When listing or detailing ApiAlias entries:

1. **Get models from ApiAlias**:
   ```rust
   let models = api_alias.get_models();
   // Returns Vec<String> based on forward_all_with_prefix flag
   ```

2. **Normalize each model name**:
   ```rust
   let normalized: Vec<String> = models
       .iter()
       .map(|m| normalize_model_name(m))
       .collect();
   ```

3. **Query model_metadata**:
   ```sql
   SELECT * FROM model_metadata
   WHERE model_id IN ($normalized_names)
      OR json_extract(aliases, '$') LIKE '%' || $normalized_name || '%'
   ```

4. **Attach metadata to ApiAliasResponse**:
   ```rust
   let api_alias_response = ApiAliasResponse {
       source: AliasSource::Api,
       id: api_alias.id.clone(),
       api_format: api_alias.api_format.clone(),
       base_url: api_alias.base_url.clone(),
       models: models.clone(),
       metadata: matched_metadata.map(|m| m.into()),
       // ... other fields
   };
   ```

### Matching Strategy

**Exact Match Priority**:
1. Match `model_id` exactly (fastest)
2. Match within `aliases` array (fallback)

**Multiple Matches**:
- If ApiAlias has multiple models, pick first matched metadata
- Metadata is per-model, not per-alias configuration

**No Match**:
- ApiAliasResponse.metadata remains `None`
- No error, graceful degradation

---

## Domain Type Extensions

### Extended ModelMetadata

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct ModelMetadata {
    pub capabilities: ModelCapabilities,
    pub context: ContextLimits,
    pub architecture: ModelArchitecture,

    // New iteration 2 fields (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pricing: Option<ModelPricing>,
}
```

### New ModelPricing Type

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct ModelPricing {
    pub input_per_million_tokens: f64,
    pub output_per_million_tokens: f64,

    // Optional detailed tiers
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_input_per_million_tokens: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_input_per_million_tokens: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_output_per_million_tokens: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_per_token: Option<f64>,

    pub currency: String,  // ISO 4217 code
}
```

---

## Sync Client Implementation

### SyncClient Trait

```rust
use async_trait::async_trait;

#[async_trait]
pub trait SyncClient: Send + Sync {
    /// Fetch all model metadata with cursor-based pagination.
    async fn fetch_all(&self) -> Result<Vec<RemoteModelMetadata>>;

    /// Fetch single model metadata by canonical ID.
    async fn fetch_single(&self, model_id: &str) -> Result<RemoteModelMetadata>;
}
```

### RemoteModelMetadata Type

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteModelMetadata {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub provider: String,
    pub aliases: Vec<String>,
    pub capabilities: ModelCapabilities,
    pub context: ContextLimits,
    pub pricing: Option<ModelPricing>,
    pub architecture: ModelArchitecture,
    pub updated_at: DateTime<Utc>,
}
```

### HttpSyncClient Implementation

```rust
pub struct HttpSyncClient {
    base_url: String,
    client: reqwest::Client,
}

impl HttpSyncClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl SyncClient for HttpSyncClient {
    async fn fetch_all(&self) -> Result<Vec<RemoteModelMetadata>> {
        let mut all_models = Vec::new();
        let mut cursor: Option<String> = None;

        loop {
            let mut url = format!("{}/v1/model-metadata", self.base_url);
            if let Some(c) = cursor {
                url.push_str(&format!("?cursor={}", c));
            }

            let response: SyncResponse = self.client
                .get(&url)
                .send()
                .await?
                .json()
                .await?;

            all_models.extend(response.models);

            if !response.has_more {
                break;
            }

            cursor = response.next_cursor;
        }

        Ok(all_models)
    }

    async fn fetch_single(&self, model_id: &str) -> Result<RemoteModelMetadata> {
        let url = format!("{}/v1/model-metadata/{}", self.base_url, model_id);

        let model = self.client
            .get(&url)
            .send()
            .await?
            .json()
            .await?;

        Ok(model)
    }
}
```

---

## UI Integration

### Admin Sync Button

**Location**: Models list page header (visible to admin users only)

**Component**:
```tsx
{isAdmin && (
  <Button
    onClick={handleSync}
    disabled={isSyncing}
  >
    {isSyncing ? (
      <>
        <Spinner className="mr-2" />
        Syncing...
      </>
    ) : (
      <>
        <RefreshIcon className="mr-2" />
        Sync Metadata
      </>
    )}
  </Button>
)}
```

**Behavior**:
1. Click triggers `POST /bodhi/v1/models/refresh?scope=all`
2. Show spinner during request
3. On 202 Accepted: Show toast "Metadata sync started in background"
4. On error: Show error toast with message

### Pricing Badge

**Location**: ApiAlias entries in models list

**Component**:
```tsx
{metadata?.pricing && (
  <div className="flex items-center gap-1">
    <span className="text-sm text-muted-foreground">
      ${metadata.pricing.input_per_million_tokens}/
      ${metadata.pricing.output_per_million_tokens}
    </span>
    {isPricingStale(metadata.pricing.updated_at) && (
      <WarningIcon className="h-4 w-4 text-yellow-500" />
    )}
  </div>
)}
```

**Staleness Logic**:
```tsx
function isPricingStale(updated_at: string): boolean {
  const now = new Date();
  const updatedAt = new Date(updated_at);
  const daysDiff = (now.getTime() - updatedAt.getTime()) / (1000 * 60 * 60 * 24);
  return daysDiff > 7;
}
```

---

## Error Handling

### Network Errors (api.getbodhi.app unreachable)

**Behavior**:
- Log warning: "Failed to sync from api.getbodhi.app: {error}"
- Worker returns error to caller
- Endpoint returns 500 with error message
- Use stale data from last successful sync

**User Experience**:
- Admin sees error toast: "Metadata sync failed: {error}"
- App continues functioning with existing metadata
- Admin can retry manually

### Parse Errors (malformed response)

**Behavior**:
- Log warning: "Failed to parse model metadata: {error}"
- Skip malformed model, continue with next
- Don't fail entire sync operation

**Example**:
```
Synced 100 models successfully
Failed to parse 3 models: gpt-4-turbo, claude-opus-4, gemini-pro
```

### Database Errors

**Behavior**:
- Log error: "Failed to upsert model metadata: {error}"
- Return error to caller
- Endpoint returns 500 with error message
- No automatic retry (admin must re-trigger)

---

## Testing Strategy

### Unit Tests

**Model Name Normalization**:
```rust
#[test]
fn test_normalize_model_name() {
    assert_eq!(normalize_model_name("gpt-4"), "gpt-4");
    assert_eq!(normalize_model_name("anthropic/claude-sonnet-4"), "claude-sonnet-4");
    assert_eq!(normalize_model_name("openai/gpt-4-turbo"), "gpt-4-turbo");
    assert_eq!(normalize_model_name("prefix/sub/model"), "model");
}
```

**Cursor Pagination Handling**:
- Test first page fetch (no cursor)
- Test subsequent page fetch (with cursor)
- Test final page (has_more: false)
- Test empty response

**Pricing JSON Parsing**:
- Test full pricing object
- Test minimal pricing (input/output only)
- Test null pricing
- Test missing optional fields

### Integration Tests

**Sync Endpoint**:
```rust
#[tokio::test]
async fn test_sync_remote_models() {
    let app = test_app().await;

    // Mock api.getbodhi.app
    let mock_server = MockServer::start().await;
    mock_server.mock(|when, then| {
        when.method(GET).path("/v1/model-metadata");
        then.status(200).json_body(json!({
            "version": "2026-01-12",
            "models": [/* ... */],
            "next_cursor": null,
            "has_more": false
        }));
    });

    // Trigger sync
    let response = app
        .post("/bodhi/v1/models/refresh?scope=remote")
        .with_admin_auth()
        .send()
        .await;

    assert_eq!(response.status(), 202);

    // Wait for worker to process
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Verify metadata in DB
    let metadata = app.db().get_model_metadata("claude-sonnet-4").await;
    assert!(metadata.is_some());
}
```

**ApiAlias Enrichment**:
```rust
#[tokio::test]
async fn test_api_alias_enriched_with_metadata() {
    let app = test_app().await;

    // Seed model_metadata
    app.db().upsert_model_metadata(/* ... */).await;

    // Create ApiAlias with matching model
    app.db().create_api_alias(/* models: ["gpt-4"] */).await;

    // List models
    let response = app.get("/bodhi/v1/models").send().await;
    let body: PaginatedAliasResponse = response.json().await;

    let api_alias = body.data.iter()
        .find(|a| a.source == "api")
        .unwrap();

    assert!(api_alias.metadata.is_some());
    assert_eq!(api_alias.metadata.unwrap().capabilities.vision, true);
}
```

**Sync Failure**:
```rust
#[tokio::test]
async fn test_sync_failure_logged_gracefully() {
    let app = test_app().await;

    // No mock server (unreachable)

    // Trigger sync
    let response = app
        .post("/bodhi/v1/models/refresh?scope=remote")
        .with_admin_auth()
        .send()
        .await;

    assert_eq!(response.status(), 202);

    // Wait for worker
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Verify app continues functioning
    let models = app.get("/bodhi/v1/models").send().await;
    assert_eq!(models.status(), 200);
}
```

### Manual Testing

1. **No api.getbodhi.app**: Start app, trigger sync, verify error logged, app continues
2. **Mock api.getbodhi.app**: Setup mock server, trigger sync, verify pagination works
3. **Pricing badge**: After sync, verify pricing displays on ApiAlias entries
4. **Staleness indicator**: Set pricing_updated_at to 8 days ago, verify ⚠️ shows
5. **Admin-only**: Verify sync button only visible to admin users

---

## Implementation Phases

### Phase schema-extend: Schema Migration

**Files**:
- `crates/services/src/db/migrations/YYYYMMDDHHMMSS_add_remote_model_support.sql`

**Tasks**:
1. Add aliases, pricing, pricing_updated_at, provider, name, description, synced_at columns
2. Create provider index
3. Test migration on existing iteration 1 database

### Phase queue-extend: Queue Task Types

**Files**:
- `crates/services/src/queue_service.rs` - Extend RefreshTask enum

**Tasks**:
1. Add SyncAll and SyncSingle variants to RefreshTask
2. Update worker process_task match arms
3. Unit test new task types

### Phase sync-client: Sync Client Implementation

**Files**:
- `crates/services/src/model_metadata_sync.rs` - HTTP client

**Tasks**:
1. Define SyncClient trait
2. Implement HttpSyncClient with cursor-based pagination
3. Handle network errors gracefully (log, return error)
4. Parse response into RemoteModelMetadata domain types
5. Unit test pagination logic

### Phase sync-worker: Worker Sync Logic

**Files**:
- `crates/services/src/queue_service.rs` - Add sync processing

**Tasks**:
1. Implement sync_all(): fetch all pages, upsert each model
2. Implement sync_single(): fetch by ID, upsert
3. Update synced_at timestamp on successful upsert
4. Log sync statistics
5. Integration test with mock server

### Phase endpoint-extend: Endpoint Changes

**Files**:
- `crates/routes_app/src/routes_models.rs` - Add scope parameter

**Tasks**:
1. Parse `scope` query parameter
2. Dispatch to appropriate task type (local/remote/all)
3. Handle scope=all (enqueue both RefreshAll and SyncAll)
4. Integration test with admin auth

### Phase api-enrich: ApiAlias Enrichment

**Files**:
- `crates/routes_app/src/routes_models.rs` - Join logic
- `crates/services/src/db/model_metadata_repo.rs` - Query by normalized name

**Tasks**:
1. Implement normalize_model_name() utility
2. Add query_by_normalized_name() to repository
3. Enrich ApiAliasResponse with matched metadata
4. Integration test enrichment

### Phase ui-sync: UI Sync Button

**Files**:
- `crates/bodhi/src/app/ui/models/page.tsx` - Add sync button
- `crates/bodhi/src/hooks/useModels.ts` - Add sync mutation

**Tasks**:
1. Add sync button component (admin only)
2. Call refresh endpoint with scope=all
3. Show pricing badge on ApiAlias entries
4. Add staleness indicator for old pricing
5. Manual test UI flow

---

## Files to Modify/Create

### New Files
- `crates/services/src/model_metadata_sync.rs` - Sync client implementation
- `crates/services/src/db/migrations/YYYYMMDDHHMMSS_add_remote_model_support.sql` - Schema extension
- `ai-docs/specs/20260112-model-metadata/03-iteration2-design.md` - This file

### Modified Files
- `crates/objs/src/model_metadata.rs` - Add ModelPricing type
- `crates/services/src/queue_service.rs` - Add SyncAll/SyncSingle tasks
- `crates/services/src/db/model_metadata_repo.rs` - Add query by normalized name
- `crates/routes_app/src/routes_models.rs` - Add scope parameter, enrichment logic
- `crates/routes_app/src/api_dto.rs` - Extend ApiAliasResponse with metadata
- `crates/bodhi/src/app/ui/models/page.tsx` - Add sync button, pricing badge
- `crates/bodhi/src/hooks/useModels.ts` - Add sync mutation

---

## Configuration

### Environment Variables

```bash
# api.getbodhi.app base URL
BODHI_MODEL_METADATA_SYNC_URL=https://api.getbodhi.app

# Timeout for sync HTTP requests in seconds (default: 30)
BODHI_MODEL_METADATA_SYNC_TIMEOUT=30
```

---

## Backwards Compatibility

### Guarantees

1. **Iteration 1 unaffected**: All local GGUF model metadata functionality continues unchanged
2. **Database schema additive**: New columns added, no columns removed or modified
3. **API response additive**: `metadata` field added to ApiAliasResponse, optional
4. **Endpoint behavior unchanged**: Existing endpoints work as before, scope parameter optional
5. **No breaking changes**: Clients ignoring new fields are unaffected

### Migration Path

**Phase 1** (Deployment):
- No remote model metadata (table empty)
- ApiAlias responses have no metadata field

**Phase 2** (First Sync):
- Admin triggers `POST /bodhi/v1/models/refresh?scope=remote`
- Worker fetches from api.getbodhi.app
- ApiAlias responses gain metadata where matched

**Phase 3** (Steady State):
- Remote models enriched with metadata
- Pricing badges display on ApiAlias entries
- Admin triggers periodic sync manually

---

## Future Enhancements (Iteration 3+)

Deferred to future iterations:

- **Automatic Sync**: Background periodic sync (configurable interval)
- **Sync Status Endpoint**: `GET /bodhi/v1/models/sync-status` for progress tracking
- **Seed Data**: Bundled popular model metadata for offline use
- **Provider Filtering**: `GET /bodhi/v1/models?provider=anthropic` query parameter
- **Capability Filtering**: `GET /bodhi/v1/models?capabilities.vision=true`
- **UI Enhancements**: Full remote model catalog browser, pricing comparison
