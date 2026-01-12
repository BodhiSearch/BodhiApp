# Model Metadata API - Iteration 1 Design Specification

**Date**: 2026-01-12
**Version**: Iteration 1 (Local GGUF Models)
**Scope**: Iteration 1 focuses on local GGUF models with extensible schema for remote API models in iteration 2

## Overview

Iteration 1 extends the existing `/bodhi/v1/models` endpoint with optional model metadata. The database schema is designed to be extensible, supporting both local GGUF models (iteration 1) and remote API models (iteration 2). Core capabilities are stored at column level for efficient querying, while additional metadata is stored as JSON for flexibility. This approach maintains backwards compatibility while providing rich capability information inline, similar to OpenRouter's API design.

## Design Decisions Summary

| Aspect | Decision | Rationale |
|--------|----------|-----------|
| **API Approach** | Extend existing `/bodhi/v1/models` | Backwards compatible, no new endpoints |
| **Metadata Format** | Optional `metadata` field inline | OpenRouter-style, single request, omitted when absent |
| **Scope (Iteration 1)** | Local GGUF models with extensible schema | Focused delivery, reference implementation (llama.cpp) |
| **Extraction Trigger** | Manual admin refresh endpoints | No performance impact, controlled |
| **Execution Model** | In-memory queue with single worker (producer-consumer) | Non-blocking, simple, interface-based for future DB extension |
| **Data Storage** | SQLite `model_metadata` table | Extensible for local and remote models |
| **Primary Key** | Auto-increment ID with composite unique constraint | Supports multiple model sources (user/model/api) |
| **Capability Storage** | Core capabilities at column level | Efficient querying (vision, audio, tools, context) |
| **Additional Metadata** | JSON field for extensibility | Future-proof without schema changes |
| **Schema Style** | Own consistent schema | Not llama.cpp compatible, prepares for iteration 2 |
| **UI Surface** | Per-model refresh button (admin) | Granular control, admin-only |

---

## API Specification

### Endpoint Summary

| Endpoint | Method | Purpose | Auth | Changes |
|----------|--------|---------|------|---------|
| `/bodhi/v1/models` | GET | List with optional metadata | Public | **Extended** with optional `metadata` field |
| `/bodhi/v1/models/{alias}` | GET | Detail with optional metadata | Public | **Extended** with optional `metadata` field |
| `/bodhi/v1/models/refresh` | POST | Refresh all local models | Admin | **New** - triggers background job |
| `/bodhi/v1/models/{id}/refresh` | POST | Refresh single model | Admin | **New** - triggers background job |

### Response Schema Changes

Based on existing domain model `@crates/objs/src/user_alias.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserAlias {
  pub alias: String,
  pub repo: Repo,
  pub filename: String,
  pub snapshot: String,
  #[serde(default, skip_serializing_if = "is_default")]
  pub request_params: OAIRequestParams,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub context_params: Vec<String>,
}
```

#### Extended UserAliasResponse (Iteration 1)

API response DTO extends with optional metadata field:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserAliasResponse {
  pub source: AliasSource,  // AliasSource::User
  pub alias: String,
  pub repo: Repo,
  pub filename: String,
  pub snapshot: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub request_params: Option<OAIRequestParams>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub context_params: Option<Vec<String>>,

  // NEW: Optional metadata (omitted if not extracted)
  #[serde(skip_serializing_if = "Option::is_none")]
  pub metadata: Option<ModelMetadata>,
}
```

#### Extended ModelAliasResponse (Iteration 1)

```rust
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ModelAliasResponse {
  pub source: AliasSource,  // AliasSource::Model
  pub alias: String,
  pub repo: Repo,
  pub filename: String,
  pub snapshot: String,

  // NEW: Optional metadata (omitted if not extracted)
  #[serde(skip_serializing_if = "Option::is_none")]
  pub metadata: Option<ModelMetadata>,
}
```

**Note**: ApiAliasResponse will be extended in iteration 2 for remote model metadata.

---

## Metadata Schema

### ModelMetadata Domain Types

Domain types defined in `crates/objs/src/model_metadata.rs`:

```rust
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Model metadata for API responses
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct ModelMetadata {
    pub capabilities: ModelCapabilities,
    pub context: ContextLimits,
    pub architecture: ModelArchitecture,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct ModelCapabilities {
    pub vision: bool,
    pub audio: bool,
    pub thinking: bool,
    pub tools: ToolCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct ToolCapabilities {
    pub function_calling: bool,
    pub structured_output: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct ContextLimits {
    pub max_input_tokens: Option<u64>,
    pub max_output_tokens: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct ModelArchitecture {
    pub family: Option<String>,
    pub parameter_count: Option<u64>,
    pub quantization: Option<String>,
    pub format: String,  // "gguf" for iteration 1
}
```

### Design Rationale

**Capabilities**:
- `vision`/`audio`: Extracted from GGUF `clip.*` metadata (projector detection)
- `thinking`: Always `false` for GGUF models in iteration 1 (no standard metadata)
- `tools.function_calling`: Always `false` for iteration 1 (GGUF doesn't indicate)
- `tools.structured_output`: Always `false` for iteration 1 (GGUF doesn't indicate)

**Context**:
- `max_input_tokens`: From `{arch}.context_length` GGUF metadata
- `max_output_tokens`: `null` for iteration 1 (GGUF doesn't specify)

**Architecture**:
- `family`: From `general.architecture` GGUF metadata
- `parameter_count`: From `general.parameter_count` GGUF metadata
- `quantization`: From `general.quantization_version` GGUF metadata
- `format`: Always `"gguf"` for local models

---

## Database Schema

### model_metadata Table

Extensible schema supporting local GGUF models (iteration 1) and remote API models (iteration 2):

```sql
CREATE TABLE model_metadata (
  -- Primary key
  id INTEGER PRIMARY KEY AUTOINCREMENT,

  -- Model identification
  source TEXT NOT NULL,        -- AliasSource: "user", "model", "api"
  alias TEXT NOT NULL,         -- Model alias identifier
  repo TEXT,                   -- HuggingFace repo (e.g., "microsoft/phi-3") - NULL for API models
  filename TEXT,               -- GGUF file path - NULL for API models
  snapshot TEXT,               -- Snapshot identifier for change detection - NULL for API models
  model_id TEXT,               -- Remote model ID (e.g., "gpt-4") - NULL for local models

  -- Core capabilities (flattened for queryable filters)
  capabilities_vision BOOLEAN NOT NULL DEFAULT 0,
  capabilities_audio BOOLEAN NOT NULL DEFAULT 0,
  capabilities_thinking BOOLEAN NOT NULL DEFAULT 0,
  capabilities_function_calling BOOLEAN NOT NULL DEFAULT 0,
  capabilities_structured_output BOOLEAN NOT NULL DEFAULT 0,

  -- Context limits (flattened for queryable filters)
  context_max_input_tokens INTEGER,
  context_max_output_tokens INTEGER,

  -- Architecture and additional metadata (JSON for flexibility)
  architecture TEXT,           -- JSON: ModelArchitecture
  additional_metadata TEXT,    -- JSON: Future extensibility (pricing, etc.)

  -- Timestamps
  extracted_at TEXT NOT NULL,  -- When metadata was extracted/synced
  created_at TEXT NOT NULL,    -- Record creation timestamp
  updated_at TEXT NOT NULL,    -- Record update timestamp

  -- Composite unique constraint (one metadata per source+alias combination)
  UNIQUE(source, alias)
);

-- Indexes for common queries
CREATE INDEX idx_model_metadata_source ON model_metadata(source);
CREATE INDEX idx_model_metadata_repo ON model_metadata(repo);
CREATE INDEX idx_model_metadata_filename ON model_metadata(filename);
CREATE INDEX idx_model_metadata_model_id ON model_metadata(model_id);
CREATE INDEX idx_model_metadata_vision ON model_metadata(capabilities_vision) WHERE capabilities_vision = 1;
CREATE INDEX idx_model_metadata_audio ON model_metadata(capabilities_audio) WHERE capabilities_audio = 1;
CREATE INDEX idx_model_metadata_function_calling ON model_metadata(capabilities_function_calling) WHERE capabilities_function_calling = 1;
```

### Design Rationale

**Auto-increment Primary Key**:
- Simple sequential ID for internal references
- Allows flexible identification via composite unique constraint
- Supports both local (filename-based) and remote (model_id-based) models

**Source Field** (AliasSource enum):
- Discriminates between user-configured, auto-discovered, and API models
- Aligns with existing `UserAlias`, `ModelAlias`, `ApiAlias` architecture
- Enables source-specific queries

**Composite Unique Constraint** (source + alias):
- One metadata record per unique alias within its source type
- Multiple UserAlias can reference same filename (different metadata per alias configuration)
- Prevents duplicate metadata for same model

**Flattened Core Capabilities**:
- Enables efficient SQL queries: `WHERE capabilities_vision = 1`
- Supports multi-capability queries: `WHERE capabilities_vision = 1 AND capabilities_audio = 1`
- Partial indexes on boolean columns optimize lookups
- Avoids column explosion by limiting to core queryable fields

**Nullable Fields for Extensibility**:
- `filename`/`snapshot`/`repo`: NULL for API models (iteration 2)
- `model_id`: NULL for local GGUF models (iteration 1)
- `context_max_output_tokens`: NULL when not specified
- Supports hybrid storage without schema changes

**JSON Storage for Non-Queryable Metadata**:
- `architecture`: Model family, parameter count, quantization (low cardinality, infrequent queries)
- `additional_metadata`: Future fields (pricing, rate limits, regional availability)
- Iteration 2 can extend without migration

**Snapshot Change Detection**:
- For local models: compare `snapshot` field with current HuggingFace snapshot
- For API models: compare `updated_at` timestamp with external sync service
- Re-extract/re-sync if changed

---

## Endpoint Behavior Specification

### GET /bodhi/v1/models

**Current Behavior** (Unchanged):
- Returns `PaginatedAliasResponse` with discriminated union
- Pagination with `page`, `page_size`, `total`
- Supports `sort`, `sort_order` query parameters

**New Behavior** (Added):
- If model has metadata in DB, include `metadata` field
- If no metadata, omit `metadata` field entirely (not `null`)
- Backwards compatible - existing clients ignore new field

**Example Response**:

```json
{
  "data": [
    {
      "source": "user",
      "alias": "phi-3-mini",
      "repo": "microsoft/phi-3-mini-4k-instruct-gguf",
      "filename": "Phi-3-mini-4k-instruct-q4.gguf",
      "snapshot": "abc123",
      "metadata": {
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
          "max_input_tokens": 4096,
          "max_output_tokens": null
        },
        "architecture": {
          "family": "phi3",
          "parameter_count": 3800000000,
          "quantization": "Q4_K_M",
          "format": "gguf"
        }
      }
    },
    {
      "source": "model",
      "alias": "llama-3.2-1b-instruct",
      "repo": "meta-llama/llama-3.2-1b-instruct-gguf",
      "filename": "llama-3.2-1b-instruct-q4_k_m.gguf",
      "snapshot": "def456"
      // No metadata field - not yet extracted
    },
    {
      "source": "api",
      "id": "openai-gpt4",
      "api_format": "openai",
      // ... other ApiAlias fields
      // No metadata field - iteration 2
    }
  ],
  "total": 3,
  "page": 1,
  "page_size": 20
}
```

### GET /bodhi/v1/models/{alias}

**Current Behavior** (Unchanged):
- Returns `UserAliasResponse` for user-defined aliases
- Returns 404 for model/api aliases (current limitation)

**New Behavior** (Added):
- Include `metadata` field if available in DB
- Omit `metadata` field if not extracted

**Status Codes**:
- `200 OK`: Alias found (with or without metadata)
- `404 Not Found`: Alias not found

### POST /bodhi/v1/models/refresh

**Purpose**: Trigger metadata refresh for all local GGUF models

**Authorization**: Admin role required

**Request**: Empty body

**Response**:
```json
{
  "status": "accepted",
  "message": "Metadata refresh started in background"
}
```

**Behavior**:
1. Validate admin role via auth middleware
2. Create task object for queue service
3. Submit task to in-memory queue via `QueueProducer` interface
4. Return `202 Accepted` immediately
5. Background worker processes task from queue:
   - Discover all local GGUF models (UserAlias + ModelAlias)
   - For each model, extract metadata from GGUF file
   - Check snapshot in DB vs current snapshot
   - Re-extract if snapshot changed or metadata missing
   - Upsert to `model_metadata` table

**Queue Service Integration**:
- Task enqueued to in-memory queue (no persistence)
- Single worker consumes tasks from queue (producer-consumer pattern)
- Failed tasks logged, no automatic retry in iteration 1
- Tasks lost on server restart (acceptable for manual refresh workflow)

**Status Codes**:
- `202 Accepted`: Task queued successfully
- `401 Unauthorized`: Not authenticated
- `403 Forbidden`: Not admin role

### POST /bodhi/v1/models/{id}/refresh

**Purpose**: Trigger metadata refresh for single model

**Authorization**: Admin role required

**Path Parameter**:
- `id`: Model alias or filename

**Request**: Empty body

**Response**:
```json
{
  "status": "accepted",
  "message": "Metadata refresh started for model 'phi-3-mini'"
}
```

**Behavior**:
1. Validate admin role via auth middleware
2. Resolve alias to model identification (filename/repo/snapshot)
3. Create task object for single model
4. Submit task to in-memory queue via `QueueProducer` interface
5. Return `202 Accepted` immediately
6. Background worker extracts and upserts metadata

**Queue Service Integration**:
- Same in-memory queue infrastructure as bulk refresh
- Tasks processed in FIFO order (no priority in iteration 1)
- Single model tasks complete faster (less work per task)

**Status Codes**:
- `202 Accepted`: Task queued successfully
- `404 Not Found`: Alias not found
- `401 Unauthorized`: Not authenticated
- `403 Forbidden`: Not admin role

---

## Queue Service Architecture

### Overview

Metadata refresh operations are executed asynchronously via an in-memory queue with single worker (producer-consumer pattern). This architecture provides:
- **Simple non-blocking execution**: HTTP endpoints return immediately
- **Sequential processing**: Single worker processes one task at a time
- **Resource management**: Avoids concurrent GGUF parsing
- **Interface-based design**: Easy migration to database-backed queue if needed

### Queue Service Components

**Task Definition**:
```rust
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub enum RefreshTask {
    RefreshAll {
        created_at: DateTime<Utc>,
    },
    RefreshSingle {
        alias: String,
        created_at: DateTime<Utc>,
    },
}

impl RefreshTask {
    pub fn created_at(&self) -> DateTime<Utc> {
        match self {
            RefreshTask::RefreshAll { created_at } => *created_at,
            RefreshTask::RefreshSingle { created_at, .. } => *created_at,
        }
    }
}
```

**Queue Producer Interface** (for HTTP endpoints):
```rust
use async_trait::async_trait;

#[async_trait]
pub trait QueueProducer: Send + Sync {
    /// Enqueue a refresh task (non-blocking)
    async fn enqueue(&self, task: RefreshTask) -> Result<()>;

    /// Get current queue length (for monitoring)
    async fn queue_length(&self) -> usize;
}
```

**Queue Consumer Interface** (for workers):
```rust
#[async_trait]
pub trait QueueConsumer: Send + Sync {
    /// Dequeue next task (blocking until available or shutdown)
    async fn dequeue(&self) -> Option<RefreshTask>;

    /// Signal worker shutdown
    async fn shutdown(&self);
}
```

**In-Memory Queue Implementation**:
```rust
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};

pub struct InMemoryQueue {
    queue: Arc<Mutex<VecDeque<RefreshTask>>>,
    notify: Arc<Notify>,
    shutdown: Arc<AtomicBool>,
}

impl InMemoryQueue {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            notify: Arc::new(Notify::new()),
            shutdown: Arc::new(AtomicBool::new(false)),
        }
    }
}

#[async_trait]
impl QueueProducer for InMemoryQueue {
    async fn enqueue(&self, task: RefreshTask) -> Result<()> {
        self.queue.lock().await.push_back(task);
        self.notify.notify_one();
        Ok(())
    }

    async fn queue_length(&self) -> usize {
        self.queue.lock().await.len()
    }
}

#[async_trait]
impl QueueConsumer for InMemoryQueue {
    async fn dequeue(&self) -> Option<RefreshTask> {
        loop {
            if self.shutdown.load(Ordering::Relaxed) {
                return None;
            }

            let mut queue = self.queue.lock().await;
            if let Some(task) = queue.pop_front() {
                return Some(task);
            }
            drop(queue);

            self.notify.notified().await;
        }
    }

    async fn shutdown(&self) {
        self.shutdown.store(true, Ordering::Relaxed);
        self.notify.notify_one();
    }
}
```

**Worker**:
```rust
pub struct RefreshWorker {
    consumer: Arc<dyn QueueConsumer>,
    hub_service: Arc<dyn HubService>,
    data_service: Arc<dyn DataService>,
    db_service: Arc<dyn DbService>,
}

impl RefreshWorker {
    pub async fn run(&self) {
        log::info!("Refresh worker started");

        while let Some(task) = self.consumer.dequeue().await {
            if let Err(e) = self.process_task(task).await {
                log::error!("Task processing failed: {}", e);
            }
        }

        log::info!("Refresh worker shutting down");
    }

    async fn process_task(&self, task: RefreshTask) -> Result<()> {
        match task {
            RefreshTask::RefreshAll { .. } => self.refresh_all().await,
            RefreshTask::RefreshSingle { alias, .. } => self.refresh_single(&alias).await,
        }
    }
}
```

**Queue Service** (coordinator):
```rust
pub struct QueueService {
    producer: Arc<dyn QueueProducer>,
    worker_handle: Option<JoinHandle<()>>,
}

impl QueueService {
    pub fn new(
        hub_service: Arc<dyn HubService>,
        data_service: Arc<dyn DataService>,
        db_service: Arc<dyn DbService>,
    ) -> Self {
        let queue = Arc::new(InMemoryQueue::new());

        let worker = RefreshWorker {
            consumer: queue.clone(),
            hub_service,
            data_service,
            db_service,
        };

        let worker_handle = tokio::spawn(async move {
            worker.run().await;
        });

        Self {
            producer: queue,
            worker_handle: Some(worker_handle),
        }
    }

    pub async fn enqueue(&self, task: RefreshTask) -> Result<()> {
        self.producer.enqueue(task).await
    }

    pub async fn shutdown(mut self) {
        if let Some(producer) = Arc::get_mut(&mut self.producer) {
            if let Some(queue) = producer.downcast_ref::<InMemoryQueue>() {
                queue.shutdown().await;
            }
        }

        if let Some(handle) = self.worker_handle.take() {
            let _ = handle.await;
        }
    }
}
```

### Task Lifecycle

```
1. Admin triggers refresh endpoint
2. Route handler creates RefreshTask
3. Task enqueued via QueueProducer.enqueue()
4. HTTP 202 Accepted returned immediately
5. Worker awaits next task via QueueConsumer.dequeue()
6. Worker processes task:
   - Discover models via DataService
   - Extract metadata from GGUF files
   - Upsert to model_metadata table
7. Worker logs success/failure
8. Worker awaits next task (back to step 5)
```

### Design Rationale

**In-Memory Queue**:
- Simple implementation (VecDeque with Mutex)
- No database overhead for task storage
- Sufficient for manual admin-triggered refresh workflow
- Tasks lost on restart (acceptable - admin can re-trigger)

**Interface-Based Design**:
- `QueueProducer` trait isolates HTTP endpoints from queue implementation
- `QueueConsumer` trait isolates worker from queue implementation
- Future migration to DB-backed queue requires:
  - New implementation of `QueueProducer` and `QueueConsumer`
  - No changes to HTTP endpoint handlers
  - Minimal changes to worker (same `process_task` logic)

**Single Worker**:
- Sequential GGUF file processing (avoids concurrent I/O contention)
- Simple worker lifecycle (no coordination between workers)
- Sufficient throughput for manual refresh (typical: 10-50 models)
- Can extend to worker pool if needed (future enhancement)

### Future Enhancements (Iteration 2)

**Database-Backed Queue**:
- Implement `DbQueueProducer` and `DbQueueConsumer`
- Persist tasks for restart recovery
- Add retry logic and exponential backoff
- Task status query endpoint: `GET /bodhi/v1/tasks/{task_id}`

**Advanced Features**:
- Worker pool (multiple consumers)
- Task priority (process single-model before bulk)
- WebSocket/SSE for real-time progress
- Scheduled tasks for API model sync

---

## GGUF Metadata Extraction

### Extraction Flow

Worker-based metadata extraction coordinated via in-memory queue:

```
1. Admin triggers refresh (all or single model) via POST endpoint
2. RefreshTask created and enqueued via QueueProducer.enqueue()
3. HTTP 202 Accepted returned immediately
4. Worker dequeues task via QueueConsumer.dequeue() (blocking)
5. Worker discovers model aliases via DataService
6. For each local GGUF model (source: "user" or "model"):
   a. Query metadata from DB by (source, alias)
   b. Compare stored snapshot with current HuggingFace snapshot
   c. If missing or snapshot changed:
      - Locate GGUF file via HubService
      - Parse GGUF file with enhanced GGUFMetadata parser
      - Extract capabilities (vision, audio, thinking, tools)
      - Extract context limits (max_input_tokens, max_output_tokens)
      - Extract architecture (family, parameter_count, quantization)
      - Upsert to model_metadata table with flattened capabilities
   d. If snapshot unchanged: skip (log debug message)
7. Worker logs task completion (success or failure with error)
8. Worker awaits next task (back to step 4)
```

### Capability Detection Logic

**Vision Detection**:
```rust
// Check GGUF metadata for vision support
fn has_vision(metadata: &GGUFMetadata) -> bool {
    metadata.get("clip.has_vision_encoder").as_bool().unwrap_or(false)
    || metadata.contains_key("clip.vision.image_size")
    || metadata.get("general.type").as_str() == Some("mmproj")
}
```

**Audio Detection**:
```rust
fn has_audio(metadata: &GGUFMetadata) -> bool {
    metadata.get("clip.has_audio_encoder").as_bool().unwrap_or(false)
    || metadata.contains_key("clip.audio.embedding_length")
}
```

**Context Length**:
```rust
fn context_length(metadata: &GGUFMetadata) -> Option<u64> {
    let arch = metadata.get("general.architecture")?.as_str()?;
    let key = format!("{}.context_length", arch);
    metadata.get(&key)?.as_u32().map(|v| v as u64)
}
```

---

## UI Integration

### Models List Page Changes

**Location**: `crates/bodhi/src/app/ui/models/page.tsx`

**Changes**:
1. Add refresh button per model row (admin users only)
2. Display metadata badges when available:
   - Vision: 👁️ icon if `metadata.capabilities.vision`
   - Audio: 🔊 icon if `metadata.capabilities.audio`
   - Context: Display `metadata.context.max_input_tokens` value
3. Show "Metadata not available" for models without metadata

**UI Mockup**:
```
┌─────────────────────────────────────────────────────────────┐
│ Model          | Source | Capabilities | Context | Actions  │
├─────────────────────────────────────────────────────────────┤
│ phi-3-mini     | user   | 👁️ 4K ctx   |         | 🔄 Edit  │
│ llama-3.2-1b   | model  | No metadata |         | 🔄       │
│ openai-gpt4    | api    | -           |         | Edit     │
└─────────────────────────────────────────────────────────────┘
```

### Refresh Button Component

**Behavior**:
- Only visible to admin users
- Click triggers `POST /bodhi/v1/models/{id}/refresh`
- Shows spinner while request in flight
- Success: Show toast "Metadata refresh started"
- Error: Show error toast with message

---

## Backwards Compatibility

### Guarantees

1. **Existing fields unchanged**: All current UserAliasResponse/ModelAliasResponse fields remain
2. **Optional metadata**: New `metadata` field is optional, omitted when absent
3. **Existing clients**: Clients ignoring unknown fields are unaffected
4. **Pagination**: Unchanged (page, page_size, total)
5. **Query parameters**: Unchanged (sort, sort_order)
6. **Response format**: PaginatedAliasResponse structure unchanged

### Migration Path

**Phase 1 (Initial Release)**:
- No metadata for any models
- UI shows "Metadata not available"
- Admin can trigger refresh

**Phase 2 (Gradual Enrichment)**:
- Admin refreshes metadata for commonly used models
- Metadata appears in API responses
- UI shows capability badges

**Phase 3 (Full Coverage)**:
- All local models have metadata
- UI leverages metadata for conditional features (future)

---

## Error Handling

### GGUF Parsing Errors

**Scenario**: GGUF file corrupted or unsupported version

**Behavior**:
- Log warning with filename and error details
- Skip model (don't insert metadata)
- Continue with next model
- Don't fail entire refresh job

### Missing GGUF Files

**Scenario**: Alias references non-existent file

**Behavior**:
- Log error with alias and expected path
- Skip model
- Continue with next model

### Database Errors

**Scenario**: DB write failure during upsert

**Behavior**:
- Log error with details
- Retry single model upsert once
- If retry fails, continue with next model
- Don't fail entire refresh job

---

## Testing Strategy

### Unit Test Requirements

**Domain Types** (`crates/objs/src/model_metadata.rs`):
- Serialization/deserialization roundtrip for all types
- Default values for ModelCapabilities and ToolCapabilities
- JSON conversion between flattened DB columns and nested structs

**GGUF Parser** (`crates/objs/src/gguf/capabilities.rs`):
- Vision detection with various metadata patterns (clip.has_vision_encoder, clip.vision.image_size, general.type=mmproj)
- Audio detection (clip.has_audio_encoder, clip.audio.embedding_length)
- Context length extraction with architecture-specific keys
- Architecture extraction (family, parameter_count, quantization)
- Use real GGUF test fixtures (not mocks)

**Repository Layer** (`crates/services/src/db/model_metadata_repo.rs`):
- Upsert creates new record
- Upsert updates existing record on (source, alias) conflict
- Query by (source, alias) returns correct metadata
- List all metadata with flattened capabilities
- Snapshot comparison for change detection

**Queue Service** (`crates/services/src/queue_service.rs`):
- In-memory queue implementation (VecDeque with Mutex)
- QueueProducer/QueueConsumer trait definitions
- Worker spawned on service initialization
- Task enqueue/dequeue with notification (tokio::sync::Notify)
- Graceful shutdown via atomic flag

### Integration Test Requirements

**API Endpoints**:
- GET /bodhi/v1/models returns metadata when available (flattened capabilities)
- GET /bodhi/v1/models returns no metadata field when not extracted
- GET /bodhi/v1/models/{alias} includes metadata if available
- POST /bodhi/v1/models/refresh requires admin auth (401/403 for non-admin)
- POST /bodhi/v1/models/{id}/refresh requires admin auth
- POST refresh endpoints return 202 Accepted (no task_id in iteration 1)

**Metadata Extraction**:
- Worker discovers local GGUF models via DataService
- Worker extracts metadata and upserts to DB with flattened capabilities
- Snapshot change detection triggers re-extraction
- Failed GGUF parsing logs warning, continues with next model
- Missing GGUF files log error, continue with next model

**Backwards Compatibility**:
- Existing clients without metadata field continue to work
- Pagination unchanged (page, page_size, total)
- Response structure (PaginatedAliasResponse) unchanged

### Manual Testing Checklist

1. Fresh database: verify no metadata for any models
2. Call GET /bodhi/v1/models: verify metadata field absent
3. Call POST /bodhi/v1/models/refresh as admin: verify 202 Accepted
4. Monitor worker logs: verify task processing and completion
5. Call GET /bodhi/v1/models: verify metadata present with flattened capabilities
6. Query database: verify vision/audio/thinking columns populated correctly
7. UI testing: verify refresh button visible (admin only)
8. UI testing: verify metadata badges display (👁️ for vision, 🔊 for audio, context)
9. Test with corrupted GGUF: verify warning logged, other models continue
10. Test snapshot change: modify HuggingFace snapshot, verify re-extraction
11. Test queue: trigger multiple refreshes, verify sequential processing
12. Test restart: trigger refresh, restart server, verify task lost (acceptable)

---

## Performance Considerations

### Database Query Optimization

**Flattened Capabilities**:
- Boolean columns enable efficient `WHERE capabilities_vision = 1` queries
- Partial indexes on vision/audio/function_calling optimize common filters
- Composite queries: `WHERE capabilities_vision = 1 AND capabilities_audio = 1` use multiple indexes
- Index-only scans for capability filtering (no table access needed)

**Primary Key Strategy**:
- Auto-increment ID for fast inserts (sequential writes)
- Composite unique constraint (source, alias) for upsert operations
- No foreign key constraints to model_metadata (optional relationship)

**Query Patterns**:
- List all models with metadata: single JOIN on (source, alias)
- Filter vision models: `WHERE capabilities_vision = 1` with partial index
- Count vision models: index-only scan (no table access)
- Lookup by filename: covered by idx_model_metadata_filename

### GGUF Parsing Performance

**File I/O**:
- Memory-mapped access via existing GGUFMetadata implementation
- Read-only operations (no file modifications)
- Metadata typically in first few KB of file (fast access)

**Parse Time**:
- ~10-50ms per file (acceptable for background workers)
- Sequential processing per worker (no parallelism in iteration 1)
- Failed parsing skips model, continues with next (error isolation)

### Background Worker Resource Usage

**CPU**:
- Single worker thread (tokio::spawn)
- GGUF parsing is I/O bound (minimal CPU usage)
- Sequential task processing (no concurrent parsing)

**Memory**:
- In-memory queue bounded by number of pending tasks
- Memory-mapped GGUF access (no full file load)
- Metadata objects are small (~1KB serialized)
- Worker processes one task at a time (bounded memory)
- Typical queue size: 1-2 tasks (manual refresh workflow)

**Disk I/O**:
- Read-only GGUF file access
- SQLite writes are batched by transaction
- No temporary file creation

### API Response Size

**Metadata Payload**:
- ~200-300 bytes per model with metadata (JSON)
- Flattened capabilities reduce nested object overhead
- Conditional serialization (`#[serde(skip_serializing_if = "Option::is_none")]`)
- Pagination limits response size (default page_size: 20)

**Serialization**:
- Serde JSON is highly optimized for Rust structs
- No runtime reflection or dynamic typing
- Architecture JSON field deferred serialization (only when present)

---

## Deployment Notes

### Database Migration

**Migration Execution**:
- Runs automatically on app startup via SQLx migrations
- Creates `model_metadata` table with indexes
- Additive change (no data loss, backwards compatible)

**Rollback Safety**:
- Dropping `model_metadata` table is safe (metadata is optional)
- Core functionality unaffected (models list/detail work without metadata)
- No foreign key constraints to other tables

### Gradual Metadata Enrichment

**Phase 1** (Deployment day):
- No metadata for any models
- UI shows "Metadata not available"
- Admin can trigger refresh manually

**Phase 2** (Days 1-7):
- Admin refreshes commonly used models
- Metadata appears gradually in API responses
- Users see capability badges for refreshed models

**Phase 3** (Steady state):
- Most/all local models have metadata
- New models refreshed as part of onboarding workflow
- Snapshot change detection keeps metadata current

### Queue Service Worker Management

**Worker Startup**:
- Single worker spawned on QueueService initialization
- Worker runs in background tokio task
- Worker awaits tasks via blocking dequeue (no polling)

**Worker Shutdown**:
- SIGTERM triggers graceful shutdown
- Shutdown flag set via atomic bool
- Worker completes current task before exiting
- Pending tasks in queue are lost (acceptable for manual refresh)

**Worker Monitoring**:
- Log worker startup/shutdown events
- Log task processing (start, success, failure)
- Log queue length on enqueue (for monitoring)
- Future: metrics endpoint for queue depth and worker health

---

## Configuration

### Environment Variables

**Iteration 1**:
None added (in-memory queue with single worker, no configuration needed)

### Feature Flags

None added for iteration 1 (core functionality).

---

## Iteration 2 Preview

Deferred to separate iteration:

**Scope**:
- Remote API model metadata (OpenAI, Anthropic, OpenRouter, etc.)
- Pricing information (`pricing` field in metadata)
- Model ID aliasing (e.g., `anthropic/claude-sonnet-4` → `claude-sonnet-4`)
- Sync service with `api.getbodhi.app`
- Background periodic sync (automatic)
- Seed data for popular remote models

**Why Deferred**:
- No reference implementation for remote models (unlike llama.cpp for local)
- Requires external service (api.getbodhi.app) not yet implemented
- Pricing data requires ongoing maintenance
- Iteration 1 provides immediate value for local model users
