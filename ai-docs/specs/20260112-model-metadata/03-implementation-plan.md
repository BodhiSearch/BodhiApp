# Model Metadata API - Implementation Plan

**Date**: 2026-01-12
**Delivery**: 7 incremental PRs

## Delivery Strategy

**Approach**: Incremental phases with separate PRs for easier review and testing.

| PR | Phase | Description | Dependencies | Est. Complexity |
|----|-------|-------------|--------------|-----------------|
| 1 | db-schema | Database schema + domain types | None | Low |
| 2 | gguf-enhance | GGUF parser capability extraction | None | Medium |
| 3 | db-service | Model metadata repository | PR 1 | Low |
| 4 | sync-service | Remote sync service | PR 3 | Medium |
| 5 | routes | API endpoints | PR 2, PR 3 | Medium |
| 6 | seed-data | Initial seed data | PR 3 | Low |
| 7 | integration | Integration with alias system | PR 5 | Low |

**Parallel Work**: PRs 1 and 2 can be developed in parallel (no dependencies).

---

## Phase 1: Database Schema

**Goal**: Define database schema and domain types

### Files to Create
- `crates/objs/src/model_metadata.rs` - Domain types
- `crates/services/src/db/migrations/YYYYMMDDHHMMSS_create_model_metadata.sql` - Migration

### Files to Modify
- `crates/objs/src/lib.rs` - Export new types

### Tasks

#### 1.1 Define Rust Domain Types

```rust
// crates/objs/src/model_metadata.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCapabilities {
    pub vision: bool,
    pub audio: bool,
    pub thinking: bool,
    pub tools: ToolCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCapabilities {
    pub function_calling: bool,
    pub structured_output: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextLimits {
    pub max_input_tokens: Option<u64>,
    pub max_output_tokens: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricing {
    pub input_per_million_tokens: f64,
    pub output_per_million_tokens: f64,
    pub currency: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelArchitecture {
    pub family: Option<String>,
    pub parameter_count: Option<u64>,
    pub quantization: Option<String>,
    pub format: Option<String>,
}
```

#### 1.2 Create Database Migration

```sql
-- crates/services/src/db/migrations/YYYYMMDDHHMMSS_create_model_metadata.sql

CREATE TABLE model_metadata (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  description TEXT,
  provider TEXT NOT NULL,
  aliases TEXT NOT NULL,  -- JSON array
  capabilities TEXT NOT NULL,  -- JSON object
  context TEXT,  -- JSON object
  pricing TEXT,  -- JSON object
  architecture TEXT,  -- JSON object
  updated_at TEXT NOT NULL,
  synced_at TEXT  -- When last synced from remote
);

CREATE INDEX idx_model_metadata_provider ON model_metadata(provider);
CREATE INDEX idx_model_metadata_updated_at ON model_metadata(updated_at);
```

#### 1.3 Add Tests

**Unit Tests**:
- Serialization/deserialization roundtrip
- Validation of required fields
- JSON schema compliance

### Verification
- [ ] Migration runs successfully
- [ ] Domain types serialize/deserialize correctly
- [ ] Table created with correct schema and indexes

---

## Phase 2: GGUF Parser Enhancements

**Goal**: Add capability extraction to existing GGUF parser

### Files to Create
- `crates/objs/src/gguf/capabilities.rs` - Capability extraction module

### Files to Modify
- `crates/objs/src/gguf/mod.rs` - Add typed accessors
- `crates/objs/tests/gguf_tests.rs` - Add capability tests

### Tasks

#### 2.1 Add ModelCapabilities Extraction

```rust
// crates/objs/src/gguf/capabilities.rs

use super::{GGUFMetadata, GGUFValue};
use crate::model_metadata::ModelCapabilities;

impl GGUFMetadata {
    /// Extract model capabilities from GGUF metadata
    pub fn capabilities(&self) -> ModelCapabilities {
        ModelCapabilities {
            vision: self.has_vision(),
            audio: self.has_audio(),
            thinking: false, // GGUF doesn't indicate thinking capability
            tools: ToolCapabilities {
                function_calling: false,
                structured_output: false,
            },
        }
    }

    /// Check if model has vision support
    pub fn has_vision(&self) -> bool {
        // Check for vision encoder metadata
        self.metadata.get("clip.has_vision_encoder")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        || self.metadata.contains_key("clip.vision.image_size")
        || self.metadata.get("general.type")
            .and_then(|v| v.as_str())
            .map(|s| s.contains("mmproj"))
            .unwrap_or(false)
    }

    /// Check if model has audio support
    pub fn has_audio(&self) -> bool {
        self.metadata.get("clip.has_audio_encoder")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        || self.metadata.contains_key("clip.audio.embedding_length")
    }
}
```

#### 2.2 Add Typed Accessors

```rust
// crates/objs/src/gguf/mod.rs

impl GGUFMetadata {
    /// Get model architecture
    pub fn architecture(&self) -> Option<&str> {
        self.metadata.get("general.architecture")
            .and_then(|v| v.as_str())
    }

    /// Get context length
    pub fn context_length(&self) -> Option<u64> {
        // Try architecture-specific key first
        if let Some(arch) = self.architecture() {
            let key = format!("{}.context_length", arch);
            if let Some(val) = self.metadata.get(&key).and_then(|v| v.as_u32()) {
                return Some(val as u64);
            }
        }
        // Fallback to generic key
        self.metadata.get("context_length").and_then(|v| v.as_u32()).map(|v| v as u64)
    }

    /// Get parameter count
    pub fn parameter_count(&self) -> Option<u64> {
        self.metadata.get("general.parameter_count")
            .and_then(|v| v.as_u64())
    }

    /// Get quantization level
    pub fn quantization(&self) -> Option<&str> {
        self.metadata.get("general.quantization_version")
            .and_then(|v| v.as_str())
    }
}
```

#### 2.3 Add Test Fixtures

**Tasks**:
- Create test GGUF files with vision/audio metadata
- Update Python generation scripts if needed
- Add unit tests for capability detection

### Verification
- [ ] Vision capability detected correctly
- [ ] Audio capability detected correctly
- [ ] Context length extracted correctly
- [ ] Architecture and parameter count extracted
- [ ] Tests pass for all capability combinations

---

## Phase 3: Database Service Layer

**Goal**: Implement repository for model_metadata table

### Files to Create
- `crates/services/src/db/model_metadata_repo.rs` - Repository implementation

### Files to Modify
- `crates/services/src/db/service.rs` - Add repository methods to DbService trait
- `crates/services/src/lib.rs` - Export repository

### Tasks

#### 3.1 Implement Repository

```rust
// crates/services/src/db/model_metadata_repo.rs

use crate::db::DbService;
use objs::model_metadata::ModelMetadata;
use sqlx::SqlitePool;

pub struct ModelMetadataRepository {
    pool: SqlitePool,
}

impl ModelMetadataRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn list_all(&self) -> Result<Vec<ModelMetadata>> {
        // SELECT * FROM model_metadata ORDER BY provider, name
    }

    pub async fn get_by_id(&self, id: &str) -> Result<Option<ModelMetadata>> {
        // SELECT * FROM model_metadata WHERE id = ?
    }

    pub async fn get_by_alias(&self, alias: &str) -> Result<Option<ModelMetadata>> {
        // SELECT * FROM model_metadata WHERE json_extract(aliases, '$') LIKE ?
    }

    pub async fn upsert(&self, metadata: &ModelMetadata) -> Result<()> {
        // INSERT OR REPLACE INTO model_metadata ...
    }

    pub async fn upsert_batch(&self, models: Vec<ModelMetadata>) -> Result<()> {
        // Batch INSERT OR REPLACE with transaction
    }
}
```

#### 3.2 Add to DbService Trait

```rust
// crates/services/src/db/service.rs

#[async_trait]
pub trait DbService: Send + Sync {
    // ... existing methods ...

    async fn list_all_model_metadata(&self) -> Result<Vec<ModelMetadata>>;
    async fn get_model_metadata(&self, id: &str) -> Result<Option<ModelMetadata>>;
    async fn get_model_metadata_by_alias(&self, alias: &str) -> Result<Option<ModelMetadata>>;
    async fn upsert_model_metadata(&self, metadata: &ModelMetadata) -> Result<()>;
    async fn upsert_model_metadata_batch(&self, models: Vec<ModelMetadata>) -> Result<()>;
}
```

#### 3.3 Add Tests

**Unit Tests**:
- CRUD operations (create, read, update)
- Alias lookup (exact match, prefix match)
- Batch upsert (transaction rollback on error)
- JSON field serialization

### Verification
- [ ] CRUD operations work correctly
- [ ] Alias lookup finds models by alternative IDs
- [ ] Batch upsert is atomic (all or nothing)
- [ ] JSON fields serialize/deserialize correctly

---

## Phase 4: Remote Sync Service

**Goal**: Implement synchronization with api.getbodhi.app

### Files to Create
- `crates/services/src/model_metadata_sync.rs` - Sync service implementation

### Files to Modify
- `crates/services/src/lib.rs` - Export sync service

### Tasks

#### 4.1 Implement Sync Service

```rust
// crates/services/src/model_metadata_sync.rs

use crate::db::DbService;
use objs::model_metadata::ModelMetadata;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncResponse {
    pub version: String,
    pub models: Vec<RemoteModelMetadata>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemoteModelMetadata {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub provider: String,
    pub aliases: Vec<String>,
    pub capabilities: ModelCapabilities,
    pub context: Option<ContextLimits>,
    pub pricing: Option<ModelPricing>,
    pub architecture: Option<ModelArchitecture>,
    pub updated_at: DateTime<Utc>,
}

pub struct ModelMetadataSyncService {
    db_service: Arc<dyn DbService>,
    http_client: Client,
    base_url: String,
    timeout: Duration,
}

impl ModelMetadataSyncService {
    pub async fn sync_from_remote(&self) -> Result<SyncResult> {
        // 1. Fetch from api.getbodhi.app/v1/model-metadata
        // 2. Parse response
        // 3. Upsert batch to database
        // 4. Return statistics (added, updated, unchanged)
    }
}

#[derive(Debug)]
pub struct SyncResult {
    pub version: String,
    pub added: usize,
    pub updated: usize,
    pub unchanged: usize,
    pub errors: usize,
    pub synced_at: DateTime<Utc>,
}
```

#### 4.2 Add Configuration

```rust
// In config module

pub struct ModelMetadataSyncConfig {
    pub url: String,
    pub interval: Duration,
    pub timeout: Duration,
}

impl Default for ModelMetadataSyncConfig {
    fn default() -> Self {
        Self {
            url: env::var("BODHI_MODEL_METADATA_URL")
                .unwrap_or_else(|_| "https://api.getbodhi.app/v1/model-metadata".to_string()),
            interval: Duration::from_secs(
                env::var("BODHI_MODEL_METADATA_SYNC_INTERVAL")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(86400)
            ),
            timeout: Duration::from_secs(30),
        }
    }
}
```

#### 4.3 Add Tests

**Unit Tests**:
- Mock HTTP responses with reqwest-mock
- Test successful sync
- Test partial failure (some models fail, others succeed)
- Test network errors
- Test timeout handling

### Verification
- [ ] Sync fetches from remote successfully
- [ ] Response parsed correctly
- [ ] Database updated with new/changed models
- [ ] Statistics accurate (added/updated/unchanged counts)
- [ ] Errors handled gracefully (logged, don't crash)

---

## Phase 5: API Endpoints

**Goal**: Implement REST API endpoints

### Files to Create
- `crates/routes_app/src/routes_model_metadata.rs` - Route handlers

### Files to Modify
- `crates/routes_app/src/lib.rs` - Register routes

### Tasks

#### 5.1 Implement Route Handlers

```rust
// crates/routes_app/src/routes_model_metadata.rs

use axum::{
    extract::{Path, State},
    Json,
};
use objs::model_metadata::ModelMetadata;
use services::db::DbService;

/// GET /bodhi/v1/models
pub async fn list_models_handler(
    State(db): State<Arc<dyn DbService>>,
) -> Result<Json<ListModelsResponse>> {
    let models = db.list_all_model_metadata().await?;
    Ok(Json(ListModelsResponse { models }))
}

/// GET /bodhi/v1/models/{id}/meta
pub async fn get_model_metadata_handler(
    Path(id): Path<String>,
    State(db): State<Arc<dyn DbService>>,
) -> Result<Json<ModelMetadata>> {
    let metadata = db.get_model_metadata(&id).await?
        .or_else(|| db.get_model_metadata_by_alias(&id).await.ok().flatten())
        .ok_or_else(|| AppError::not_found("Model not found"))?;

    Ok(Json(metadata))
}

/// POST /bodhi/v1/models/sync (admin only)
pub async fn sync_models_handler(
    State(sync_service): State<Arc<ModelMetadataSyncService>>,
    // Auth middleware ensures admin role
) -> Result<Json<SyncResult>> {
    let result = sync_service.sync_from_remote().await?;
    Ok(Json(result))
}
```

#### 5.2 Register Routes

```rust
// crates/routes_app/src/lib.rs

pub fn model_metadata_routes() -> Router<AppState> {
    Router::new()
        .route("/bodhi/v1/models", get(list_models_handler))
        .route("/bodhi/v1/models/:id/meta", get(get_model_metadata_handler))
        .route("/bodhi/v1/models/sync", post(sync_models_handler))
}
```

#### 5.3 Add Integration Tests

**Integration Tests**:
- GET /bodhi/v1/models returns list
- GET /bodhi/v1/models/{id}/meta returns model
- GET /bodhi/v1/models/unknown/meta returns 404
- POST /bodhi/v1/models/sync requires admin role
- POST /bodhi/v1/models/sync returns statistics

### Verification
- [ ] List endpoint returns all models
- [ ] Detail endpoint returns correct model by ID
- [ ] Detail endpoint resolves aliases
- [ ] Sync endpoint requires admin authorization
- [ ] Error responses follow OpenAI format

---

## Phase 6: Initial Seed Data

**Goal**: Populate database with popular models

### Files to Create
- `crates/services/src/db/migrations/YYYYMMDDHHMMSS_seed_model_metadata.sql` - Seed migration

### Tasks

#### 6.1 Define Seed Data

**Popular Models to Seed**:

**OpenAI**:
- gpt-4o
- gpt-4o-mini
- gpt-4-turbo
- gpt-3.5-turbo

**Anthropic**:
- claude-sonnet-4
- claude-opus-4
- claude-haiku-3.5

**Google**:
- gemini-2.0-flash
- gemini-1.5-pro

**Meta**:
- llama-3.3-70b
- llama-3.2-3b

**DeepSeek**:
- deepseek-r1
- deepseek-v3

**Mistral**:
- mistral-large
- mistral-small

#### 6.2 Create Seed Migration

```sql
-- crates/services/src/db/migrations/YYYYMMDDHHMMSS_seed_model_metadata.sql

INSERT INTO model_metadata (id, name, description, provider, aliases, capabilities, context, pricing, architecture, updated_at)
VALUES
(
  'claude-sonnet-4',
  'Claude Sonnet 4',
  'Anthropic''s latest balanced model',
  'anthropic',
  '["anthropic/claude-sonnet-4", "claude-sonnet-4-20241022"]',
  '{"vision": true, "audio": false, "thinking": true, "tools": {"function_calling": true, "structured_output": true}}',
  '{"max_input_tokens": 200000, "max_output_tokens": 8192}',
  '{"input_per_million_tokens": 3.0, "output_per_million_tokens": 15.0, "currency": "USD", "updated_at": "2026-01-10T00:00:00Z"}',
  '{"family": "claude", "parameter_count": null, "quantization": null, "format": null}',
  '2026-01-10T00:00:00Z'
),
-- ... repeat for other models
;
```

### Verification
- [ ] Seed migration runs successfully
- [ ] All popular models present in database
- [ ] Aliases work for lookups
- [ ] Metadata accurate (capabilities, context, pricing)

---

## Phase 7: Integration with Alias System

**Goal**: Enhance alias resolution to include metadata

### Files to Modify
- `crates/services/src/data_service.rs` - Enhance alias resolution

### Tasks

#### 7.1 Enhance Alias Resolution

```rust
// crates/services/src/data_service.rs

impl DataService {
    pub async fn find_alias_with_metadata(&self, alias: &str) -> Result<(Alias, Option<ModelMetadata>)> {
        // 1. Resolve alias (existing logic)
        let alias = self.find_alias(alias).await?;

        // 2. Lookup metadata
        let metadata = match &alias {
            Alias::User(ua) => self.db.get_model_metadata(&ua.model_ref).await.ok().flatten(),
            Alias::Model(ma) => self.extract_gguf_metadata(&ma.filename).await.ok(),
            Alias::Api(aa) => self.db.get_model_metadata(&aa.id).await.ok().flatten(),
        };

        Ok((alias, metadata))
    }

    async fn extract_gguf_metadata(&self, filename: &str) -> Result<ModelMetadata> {
        let path = self.hub_service.find_local_file(filename).await?.path;
        let gguf = GGUFMetadata::new(&path)?;

        Ok(ModelMetadata {
            id: filename.to_string(),
            name: filename.to_string(),
            description: None,
            provider: "local".to_string(),
            capabilities: gguf.capabilities(),
            context: gguf.context_length().map(|max_input_tokens| ContextLimits {
                max_input_tokens: Some(max_input_tokens),
                max_output_tokens: None,
            }),
            pricing: None,
            architecture: Some(ModelArchitecture {
                family: gguf.architecture().map(str::to_string),
                parameter_count: gguf.parameter_count(),
                quantization: gguf.quantization().map(str::to_string),
                format: Some("gguf".to_string()),
            }),
            updated_at: Utc::now(),
        })
    }
}
```

#### 7.2 Update Route Handlers

```rust
// Use new method in route handlers
let (alias, metadata) = data_service.find_alias_with_metadata(&id).await?;
```

### Verification
- [ ] Alias resolution includes metadata
- [ ] Local GGUF models have extracted metadata
- [ ] Remote API models have DB metadata
- [ ] Metadata enriches existing endpoints

---

## Verification Plan

### Unit Tests (Per Phase)
- [x] Phase 1: Domain type serialization, validation
- [x] Phase 2: GGUF capability extraction with fixtures
- [x] Phase 3: Repository CRUD operations
- [x] Phase 4: Sync service with mocked HTTP
- [x] Phase 5: Route handler logic
- [x] Phase 6: Seed data integrity
- [x] Phase 7: Alias resolution with metadata

### Integration Tests
- [ ] GET /bodhi/v1/models returns expected schema
- [ ] GET /bodhi/v1/models/{id}/meta returns correct model
- [ ] Local GGUF models have extracted metadata
- [ ] Remote API models have DB metadata
- [ ] Sync updates database correctly
- [ ] Alias lookup by canonical ID and aliases works

### Manual Testing Checklist

**Setup**:
1. Start BodhiApp with fresh database
2. Verify seed data present: `sqlite3 bodhi.db "SELECT COUNT(*) FROM model_metadata"`

**List Endpoint**:
3. Call `GET /bodhi/v1/models`
4. Verify response contains seeded models
5. Verify schema matches specification

**Detail Endpoint**:
6. Call `GET /bodhi/v1/models/claude-sonnet-4/meta`
7. Verify full metadata returned
8. Call `GET /bodhi/v1/models/anthropic/claude-sonnet-4/meta` (alias)
9. Verify same metadata returned

**Local GGUF**:
10. Add a local GGUF model to HuggingFace cache
11. Call `/bodhi/v1/models/{filename}/meta`
12. Verify metadata extracted from GGUF (capabilities, context, architecture)

**Sync Endpoint** (Admin):
13. Call `POST /bodhi/v1/models/sync`
14. Verify sync statistics returned
15. Verify database updated with new models

**UI Integration**:
16. Update UI to consume `/bodhi/v1/models`
17. Verify capability-based features work (vision upload, etc.)

---

## Rollback Plan

### Phase Rollback

If a phase fails, rollback is clean due to incremental approach:

**Phase 1-3**: Database changes only
- Rollback: Revert migration, remove table

**Phase 4**: Service layer only
- Rollback: Remove sync service, no data impact

**Phase 5**: New endpoints only
- Rollback: Remove route registration

**Phase 6**: Data only
- Rollback: Delete from model_metadata table

**Phase 7**: Service enhancement only
- Rollback: Revert to old find_alias method

### Feature Flag

Add feature flag for complete feature disable:

```rust
pub const MODEL_METADATA_ENABLED: bool = env::var("BODHI_MODEL_METADATA_ENABLED")
    .ok()
    .and_then(|s| s.parse().ok())
    .unwrap_or(true);
```

---

## Performance Considerations

### Database

- **Indexes**: `provider`, `updated_at` for fast lookups
- **JSON fields**: Consider extracting frequently queried fields to columns if performance issue
- **Batch operations**: Use transactions for batch upserts

### GGUF Parsing

- **Caching**: Cache parsed metadata in memory (LRU cache)
- **Lazy loading**: Only parse when metadata requested
- **Background parsing**: Parse on model discovery in background thread

### API Response

- **Pagination**: Add pagination if model count exceeds 100
- **Field selection**: Add `?fields=` parameter to reduce response size
- **Caching**: Add ETag support for client-side caching

### Remote Sync

- **Rate limiting**: Respect api.getbodhi.app rate limits
- **Incremental sync**: Only fetch changed models (version-based)
- **Background job**: Run sync in background, don't block startup

---

## Files Summary

### New Files
- `crates/objs/src/model_metadata.rs`
- `crates/objs/src/gguf/capabilities.rs`
- `crates/services/src/db/model_metadata_repo.rs`
- `crates/services/src/model_metadata_sync.rs`
- `crates/routes_app/src/routes_model_metadata.rs`
- `crates/services/src/db/migrations/YYYYMMDDHHMMSS_create_model_metadata.sql`
- `crates/services/src/db/migrations/YYYYMMDDHHMMSS_seed_model_metadata.sql`

### Modified Files
- `crates/objs/src/lib.rs` - Export model_metadata module
- `crates/objs/src/gguf/mod.rs` - Add capability extraction methods
- `crates/objs/tests/gguf_tests.rs` - Add capability tests
- `crates/services/src/db/service.rs` - Add repository methods to trait
- `crates/services/src/lib.rs` - Export new services
- `crates/services/src/data_service.rs` - Enhance alias resolution
- `crates/routes_app/src/lib.rs` - Register model_metadata routes
- `crates/routes_all/src/lib.rs` - Include model_metadata routes

### Test Files
- `crates/objs/tests/model_metadata_tests.rs`
- `crates/services/tests/model_metadata_repo_tests.rs`
- `crates/services/tests/model_metadata_sync_tests.rs`
- `crates/routes_app/tests/model_metadata_routes_tests.rs`
