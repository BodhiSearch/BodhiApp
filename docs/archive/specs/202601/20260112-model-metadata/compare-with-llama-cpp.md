# Model Metadata API - Iteration 1 Implementation Plan

**Spec Reference**: `ai-docs/specs/20260112-model-metadata/02-iteration1-design.md`
**Scope**: Local GGUF models with extensible schema for iteration 2

## Summary

Extend `/bodhi/v1/models` endpoints with optional `metadata` field containing capabilities (vision, audio, thinking, tools), context limits, and architecture info extracted from GGUF files via admin-triggered background refresh.

## Implementation Decisions (from interview)

| Decision | Choice |
|----------|--------|
| QueueService integration | Add as trait to AppService (DI pattern) |
| Task failure visibility | Logs only, no UI feedback |
| UI layout | Actions column dropdown + icon badges |
| GGUF test fixtures | Python scripts (real GGUF files) |
| Snapshot detection | From alias config (not HF cache) |
| DB operations | Extend existing DbService |
| Worker shutdown | Natural termination |
| API path pattern | Keep `{id}/refresh` as designed |
| Scope parameter | Add `scope=local` now (iteration 2 prep) |
| GGUF API | Extend GGUFMetadata with get/contains_key |

---

## Phase schema: Database Schema

### Files to Create
- `crates/services/migrations/0006_model_metadata.up.sql`
- `crates/services/migrations/0006_model_metadata.down.sql`

### Schema
```sql
CREATE TABLE model_metadata (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  source TEXT NOT NULL,
  alias TEXT NOT NULL,
  repo TEXT,
  filename TEXT,
  snapshot TEXT,
  model_id TEXT,
  capabilities_vision BOOLEAN NOT NULL DEFAULT 0,
  capabilities_audio BOOLEAN NOT NULL DEFAULT 0,
  capabilities_thinking BOOLEAN NOT NULL DEFAULT 0,
  capabilities_function_calling BOOLEAN NOT NULL DEFAULT 0,
  capabilities_structured_output BOOLEAN NOT NULL DEFAULT 0,
  context_max_input_tokens INTEGER,
  context_max_output_tokens INTEGER,
  architecture TEXT,
  additional_metadata TEXT,
  extracted_at TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  UNIQUE(source, alias)
);
-- Indexes for common queries
CREATE INDEX idx_model_metadata_source ON model_metadata(source);
CREATE INDEX idx_model_metadata_repo ON model_metadata(repo);
CREATE INDEX idx_model_metadata_filename ON model_metadata(filename);
CREATE INDEX idx_model_metadata_vision ON model_metadata(capabilities_vision) WHERE capabilities_vision = 1;
```

### Tasks
1. Create migration files following `0005_*.sql` pattern
2. Add `ModelMetadataRow` struct to `crates/services/src/db/objs.rs`
3. Add DbService trait methods: `upsert_model_metadata`, `get_model_metadata`, `list_model_metadata`, `get_model_metadata_by_source_alias`
4. Implement in SqliteDbService with TimeService for timestamps
5. Unit tests for CRUD operations

---

## Phase domain: Domain Types

### Files to Create
- `crates/objs/src/model_metadata.rs`

### Files to Modify
- `crates/objs/src/lib.rs` (add module export)
- `crates/objs/src/gguf/metadata.rs` (add get/contains_key methods)

### Types
```rust
// crates/objs/src/model_metadata.rs
pub struct ModelMetadata {
    pub capabilities: ModelCapabilities,
    pub context: ContextLimits,
    pub architecture: ModelArchitecture,
}

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

pub struct ContextLimits {
    pub max_input_tokens: Option<u64>,
    pub max_output_tokens: Option<u64>,
}

pub struct ModelArchitecture {
    pub family: Option<String>,
    pub parameter_count: Option<u64>,
    pub quantization: Option<String>,
    pub format: String,
}
```

### Tasks
1. Add domain types with Serialize/Deserialize/ToSchema derives
2. Implement Default for types where appropriate
3. Add `get(&self, key: &str) -> Option<&GGUFValue>` to GGUFMetadata
4. Add `contains_key(&self, key: &str) -> bool` to GGUFMetadata
5. Unit tests for serialization roundtrip

---

## Phase gguf-caps: GGUF Capability Detection

### Files to Create
- `crates/objs/src/gguf/capabilities.rs`
- `crates/objs/tests/scripts/test_data_gguf_multimodal.py` (test fixture generator)

### Files to Modify
- `crates/objs/src/gguf/mod.rs` (add module)

### Capability Detection Logic
```rust
pub fn extract_capabilities(metadata: &GGUFMetadata) -> ModelCapabilities {
    ModelCapabilities {
        vision: has_vision(metadata),
        audio: has_audio(metadata),
        thinking: false,  // Always false for GGUF
        tools: ToolCapabilities {
            function_calling: false,
            structured_output: false,
        },
    }
}

fn has_vision(metadata: &GGUFMetadata) -> bool {
    metadata.get("clip.has_vision_encoder").map(|v| v.as_bool()).flatten().unwrap_or(false)
        || metadata.contains_key("clip.vision.image_size")
        || metadata.get("general.type").map(|v| v.as_str()) == Some("mmproj")
}

fn has_audio(metadata: &GGUFMetadata) -> bool {
    metadata.get("clip.has_audio_encoder").map(|v| v.as_bool()).flatten().unwrap_or(false)
        || metadata.contains_key("clip.audio.embedding_length")
}

pub fn extract_context(metadata: &GGUFMetadata) -> ContextLimits {
    let arch = metadata.get("general.architecture").and_then(|v| v.as_str());
    let max_input = arch.and_then(|a| {
        metadata.get(&format!("{}.context_length", a)).and_then(|v| v.as_u64())
    });
    ContextLimits { max_input_tokens: max_input, max_output_tokens: None }
}

pub fn extract_architecture(metadata: &GGUFMetadata) -> ModelArchitecture {
    ModelArchitecture {
        family: metadata.get("general.architecture").and_then(|v| v.as_str().map(String::from)),
        parameter_count: metadata.get("general.parameter_count").and_then(|v| v.as_u64()),
        quantization: metadata.get("general.quantization_version").and_then(|v| v.as_str().map(String::from)),
        format: "gguf".to_string(),
    }
}
```

### Tasks
1. Create Python script to generate test GGUF files with `clip.*` metadata
2. Add `as_bool()`, `as_str()`, `as_u64()` methods to GGUFValue
3. Implement capability extraction functions
4. Unit tests with real GGUF fixtures

---

## Phase queue: Queue Service

### Files to Create
- `crates/services/src/queue_service.rs`

### Files to Modify
- `crates/services/src/lib.rs` (add module export)
- `crates/services/src/app_service.rs` (add QueueService trait)

### Types
```rust
#[derive(Debug, Clone)]
pub enum RefreshTask {
    RefreshAll { created_at: DateTime<Utc> },
    RefreshSingle { alias: String, created_at: DateTime<Utc> },
}

#[async_trait]
pub trait QueueProducer: Send + Sync + std::fmt::Debug {
    async fn enqueue(&self, task: RefreshTask) -> Result<()>;
    async fn queue_length(&self) -> usize;
}

#[async_trait]
pub trait QueueConsumer: Send + Sync {
    async fn dequeue(&self) -> Option<RefreshTask>;
    fn shutdown(&self);
}

pub struct InMemoryQueue { /* VecDeque + Mutex + Notify + AtomicBool */ }

pub struct QueueService {
    producer: Arc<dyn QueueProducer>,
    // Worker runs in background tokio task
}
```

### Tasks
1. Define RefreshTask enum
2. Define QueueProducer/QueueConsumer traits
3. Implement InMemoryQueue with tokio sync primitives
4. Create QueueService coordinator that spawns worker on new()
5. Add QueueService to AppService trait
6. Unit tests for queue enqueue/dequeue

---

## Phase worker: Refresh Worker

### Files to Modify
- `crates/services/src/queue_service.rs` (add worker implementation)

### Worker Logic
```rust
pub struct RefreshWorker {
    consumer: Arc<dyn QueueConsumer>,
    hub_service: Arc<dyn HubService>,
    data_service: Arc<dyn DataService>,
    db_service: Arc<dyn DbService>,
}

impl RefreshWorker {
    pub async fn run(&self) {
        while let Some(task) = self.consumer.dequeue().await {
            if let Err(e) = self.process_task(task).await {
                log::error!("Task failed: {}", e);
            }
        }
    }

    async fn process_task(&self, task: RefreshTask) -> Result<()> {
        match task {
            RefreshTask::RefreshAll { .. } => self.refresh_all().await,
            RefreshTask::RefreshSingle { alias, .. } => self.refresh_single(&alias).await,
        }
    }

    async fn refresh_all(&self) -> Result<()> {
        let aliases = self.data_service.list_aliases().await?;
        for alias in aliases.iter().filter(|a| matches!(a, Alias::User(_) | Alias::Model(_))) {
            self.extract_and_store(alias).await?;
        }
        Ok(())
    }
}
```

### Tasks
1. Implement RefreshWorker with service dependencies
2. Implement refresh_all: iterate UserAlias + ModelAlias
3. Implement refresh_single: find alias, extract, store
4. Implement extract_and_store: locate GGUF, parse, upsert
5. Snapshot comparison: skip if alias.snapshot matches DB
6. Integration tests with mock services

---

## Phase endpoints: API Endpoints

### Files to Modify
- `crates/routes_app/src/routes_models.rs` (add refresh handlers)
- `crates/routes_app/src/api_dto.rs` (extend response types)
- `crates/routes_all/src/routes.rs` (add routes)

### Endpoints
```rust
// POST /bodhi/v1/models/refresh?scope=local
#[utoipa::path(post, path = "/bodhi/v1/models/refresh", tag = API_TAG_MODELS)]
pub async fn refresh_all_handler(
    State(state): State<Arc<dyn RouterState>>,
    Query(params): Query<RefreshParams>,  // scope: "local"
) -> Result<Json<RefreshResponse>, ApiError>

// POST /bodhi/v1/models/{id}/refresh
#[utoipa::path(post, path = "/bodhi/v1/models/{id}/refresh", tag = API_TAG_MODELS)]
pub async fn refresh_single_handler(
    State(state): State<Arc<dyn RouterState>>,
    Path(id): Path<String>,
) -> Result<Json<RefreshResponse>, ApiError>
```

### Response Types
```rust
#[derive(Serialize, ToSchema)]
pub struct RefreshResponse {
    pub status: String,  // "accepted"
    pub message: String,
}

// Extend UserAliasResponse/ModelAliasResponse
pub struct UserAliasResponse {
    // ... existing fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<ModelMetadata>,
}
```

### Tasks
1. Add RefreshParams query struct with scope field
2. Implement refresh_all_handler: enqueue RefreshAll task, return 202
3. Implement refresh_single_handler: resolve alias, enqueue RefreshSingle, return 202
4. Extend UserAliasResponse/ModelAliasResponse with optional metadata
5. Modify list_aliases_handler: join with model_metadata table
6. Modify get_user_alias_handler: include metadata if exists
7. Add routes with Admin auth middleware
8. Integration tests

---

## Phase ui: UI Integration

### Files to Modify
- `crates/bodhi/src/app/ui/models/page.tsx`
- `crates/bodhi/src/hooks/useModels.ts` (add refresh mutation)

### UI Changes
1. Add refresh option to actions dropdown (admin only)
2. Add "Refresh All" button in page header (admin only)
3. Add capability badges as icons:
   - 👁️ (Eye icon) for vision
   - 🔊 (Speaker icon) for audio
   - Context length display
4. Show "No metadata" text when metadata absent

### Tasks
1. Add `useRefreshMetadata` mutation hook
2. Add `useRefreshAllMetadata` mutation hook
3. Add refresh option to model row actions dropdown
4. Add "Refresh All" button in header
5. Add capability badges/icons column
6. Add "scope=local" to refresh calls
7. Build UI: `make build.ui-rebuild`
8. Manual testing

---

## Phase finalize: Finalization

### Tasks
1. Run `cargo fmt --all`
2. Run `make test.backend` - ensure all tests pass
3. Run `make test.ui` - ensure UI tests pass
4. Regenerate OpenAPI: `cargo run --package xtask openapi`
5. Regenerate TS client: `cd ts-client && npm run generate`
6. Manual testing checklist from spec
7. Update CLAUDE.md/PACKAGE.md if needed

---

## Files Summary

### New Files
- `crates/services/migrations/0006_model_metadata.up.sql`
- `crates/services/migrations/0006_model_metadata.down.sql`
- `crates/objs/src/model_metadata.rs`
- `crates/objs/src/gguf/capabilities.rs`
- `crates/objs/tests/scripts/test_data_gguf_multimodal.py`
- `crates/services/src/queue_service.rs`

### Modified Files
- `crates/objs/src/lib.rs`
- `crates/objs/src/gguf/mod.rs`
- `crates/objs/src/gguf/metadata.rs`
- `crates/objs/src/gguf/value.rs`
- `crates/services/src/lib.rs`
- `crates/services/src/db/service.rs`
- `crates/services/src/db/objs.rs`
- `crates/services/src/app_service.rs`
- `crates/routes_app/src/routes_models.rs`
- `crates/routes_app/src/api_dto.rs`
- `crates/routes_all/src/routes.rs`
- `crates/bodhi/src/app/ui/models/page.tsx`
- `crates/bodhi/src/hooks/useModels.ts`

---

## Verification

### Automated Tests
- Unit tests for domain types (serialization roundtrip)
- Unit tests for GGUF capability detection with real fixtures
- Unit tests for queue enqueue/dequeue
- Integration tests for refresh endpoints (202, auth)
- Integration tests for metadata in model list response

### Manual Testing
1. Fresh database: verify no metadata for any models
2. GET /bodhi/v1/models: verify metadata field absent
3. POST /bodhi/v1/models/refresh?scope=local as admin: verify 202
4. Check server logs: verify worker processing
5. GET /bodhi/v1/models: verify metadata present
6. UI: verify refresh button visible (admin only)
7. UI: verify metadata badges display
8. Test with model lacking GGUF metadata keys: verify graceful handling
