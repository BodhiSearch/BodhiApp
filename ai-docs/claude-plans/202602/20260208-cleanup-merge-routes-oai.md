# Plan: Merge routes_oai into routes_app

## Context

The three-crate routing split (`routes_oai`, `routes_app`, `routes_all`) was an early architectural choice. Now that `routes_app` is organized into domain folders and `routes_oai` is small (3 source files, 7 handlers), the split creates overhead without benefit. The merge also integrates the code to follow routes_app's established conventions (separate error.rs, tests/ subdirectory, etc.).

**Strategy**: Split `routes_oai` crate into two domain folders in `routes_app`:
- `routes_oai/` -- OpenAI endpoints, restructured to match routes_app conventions (error.rs, tests/ subdirectory)
- `routes_ollama/` -- Ollama endpoints, moved as-is (single mod.rs, no internal restructuring)

Keep `routes_all` as the composition/middleware layer.

## Decisions

| Decision | Choice |
|----------|--------|
| Merge scope | routes_oai → routes_app only; keep routes_all |
| Split | Two domain folders: `routes_oai/` (OpenAI) and `routes_ollama/` (Ollama) |
| routes_oai structure | Full restructure: error.rs, handler files, tests/ subdirectory |
| routes_ollama structure | Moved as-is into mod.rs (skip restructuring) |
| Test patterns | Migrate routes_oai tests to `AppServiceStubBuilder`; leave routes_ollama tests as-is |

## Target File Structure

```
crates/routes_app/src/
├── routes_oai/              # NEW - OpenAI compatible endpoints
│   ├── mod.rs               # Module declarations, endpoint constants, re-exports
│   ├── error.rs             # OAIRouteError enum (extracted from routes_chat.rs)
│   ├── chat.rs              # chat_completions_handler, embeddings_handler
│   ├── models.rs            # oai_models_handler, oai_model_handler + helpers
│   └── tests/
│       ├── mod.rs
│       ├── chat_test.rs     # Tests from routes_chat.rs #[cfg(test)]
│       └── models_test.rs   # Tests from routes_oai_models.rs #[cfg(test)]
├── routes_ollama/           # NEW - Ollama compatible endpoints
│   └── mod.rs               # Everything from routes_ollama.rs (types, From impls, handlers, tests)
├── ... (existing modules unchanged)
```

---

## Phase 1: Create routes_oai/ Domain Module (Restructured)

### 1.1 Create `routes_oai/error.rs`

Extract `OAIRouteError` from `crates/routes_oai/src/routes_chat.rs` (lines 14-28):

```rust
use objs::{AppError, ErrorType};

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum OAIRouteError {
  #[error("Error constructing HTTP response: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer, args_delegate = false)]
  Http(#[from] http::Error),

  #[error("Response serialization failed: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer, args_delegate = false)]
  Serialization(#[from] serde_json::Error),

  #[error("{0}")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  InvalidRequest(String),
}
```

### 1.2 Create `routes_oai/chat.rs`

Copy `crates/routes_oai/src/routes_chat.rs` **without** the `OAIRouteError` enum and **without** the `#[cfg(test)] mod test` block.

Update imports:
- Remove `OAIRouteError` definition (now in error.rs)
- `use crate::{ENDPOINT_OAI_CHAT_COMPLETIONS, ENDPOINT_OAI_EMBEDDINGS};` → `use super::{ENDPOINT_OAI_CHAT_COMPLETIONS, ENDPOINT_OAI_EMBEDDINGS};`
- Add `use super::error::OAIRouteError;`

Contents: `validate_chat_completion_request()`, `chat_completions_handler()`, `embeddings_handler()`.

### 1.3 Create `routes_oai/models.rs`

Copy `crates/routes_oai/src/routes_oai_models.rs` **without** the `#[cfg(test)] mod tests` block.

Update imports:
- `use crate::ENDPOINT_OAI_MODELS;` → `use super::ENDPOINT_OAI_MODELS;`

Contents: `oai_models_handler()`, `oai_model_handler()`, `user_alias_to_oai_model()`, `model_alias_to_oai_model()`, `api_model_to_oai_model()`.

### 1.4 Create `routes_oai/tests/`

**tests/mod.rs:**
```rust
mod chat_test;
mod models_test;
```

**tests/chat_test.rs:**
Extract the `#[cfg(test)] mod test` block from `routes_chat.rs`. Update imports:
- `use crate::routes_chat::{chat_completions_handler, embeddings_handler};` → `use crate::routes_oai::{chat_completions_handler, embeddings_handler};`
- Migrate from `app_service_stub` fixture to `AppServiceStubBuilder` (see Phase 3)

**tests/models_test.rs:**
Extract the `#[cfg(test)] mod tests` block from `routes_oai_models.rs`. Update imports:
- `use super::{oai_model_handler, oai_models_handler};` → `use crate::routes_oai::{oai_model_handler, oai_models_handler};`
- Already uses `AppServiceStubBuilder` -- no migration needed.

### 1.5 Create `routes_oai/mod.rs`

```rust
mod error;
mod chat;
mod models;

#[cfg(test)]
mod tests;

pub use error::*;
pub use chat::*;
pub use models::*;

pub const ENDPOINT_OAI_MODELS: &str = "/v1/models";
pub const ENDPOINT_OAI_CHAT_COMPLETIONS: &str = "/v1/chat/completions";
pub const ENDPOINT_OAI_EMBEDDINGS: &str = "/v1/embeddings";
```

---

## Phase 2: Create routes_ollama/ Domain Module (As-Is)

### 2.1 Create `routes_ollama/mod.rs`

Copy entire content of `crates/routes_oai/src/routes_ollama.rs` into `crates/routes_app/src/routes_ollama/mod.rs`.

Add endpoint constants at top (moved from routes_oai/lib.rs):
```rust
pub const ENDPOINT_OLLAMA_TAGS: &str = "/api/tags";
pub const ENDPOINT_OLLAMA_SHOW: &str = "/api/show";
pub const ENDPOINT_OLLAMA_CHAT: &str = "/api/chat";
```

Update imports:
- Remove `use crate::{ENDPOINT_OLLAMA_CHAT, ENDPOINT_OLLAMA_SHOW, ENDPOINT_OLLAMA_TAGS};` (constants are now local)
- In `#[cfg(test)] mod test`: `use crate::{ollama_model_show_handler, ollama_models_handler};` → `use super::{ollama_model_show_handler, ollama_models_handler};`

No other changes -- types, From impls, handlers, and inline tests stay as-is.

---

## Phase 3: Update routes_app Integration

### 3.1 Update `crates/routes_app/Cargo.toml`

Remove dependency:
```toml
# DELETE:
routes_oai = { workspace = true }
```

Add new dependencies (from routes_oai that routes_app doesn't have):
```toml
# Add to [dependencies]:
futures-util = { workspace = true }
http = { workspace = true }

# Add/update in [dev-dependencies]:
objs = { workspace = true, features = ["test-utils"] }
llama_server_proc = { workspace = true, features = ["test-utils"] }
reqwest = { workspace = true }
```

### 3.2 Update `crates/routes_app/src/lib.rs`

Add module declarations (in the domain folders section):
```rust
// -- Domain route modules (folders)
mod routes_api_models;
mod routes_auth;
mod routes_models;
pub mod routes_oai;       // ADD - pub for type access
pub mod routes_ollama;    // ADD - pub for type access
mod routes_toolsets;
mod routes_users;
```

Add re-exports (after existing re-export block):
```rust
pub use routes_oai::{
  chat_completions_handler, embeddings_handler,
  oai_model_handler, oai_models_handler,
  OAIRouteError,
  ENDPOINT_OAI_CHAT_COMPLETIONS, ENDPOINT_OAI_EMBEDDINGS, ENDPOINT_OAI_MODELS,
};
pub use routes_ollama::{
  ollama_model_chat_handler, ollama_model_show_handler, ollama_models_handler,
  ENDPOINT_OLLAMA_CHAT, ENDPOINT_OLLAMA_SHOW, ENDPOINT_OLLAMA_TAGS,
};
```

### 3.3 Update `crates/routes_app/src/shared/openapi.rs`

Change external import to crate-local (lines 57-61):
```rust
// BEFORE:
use routes_oai::{
  __path_chat_completions_handler, __path_embeddings_handler, __path_oai_model_handler,
  __path_oai_models_handler, __path_ollama_model_chat_handler, __path_ollama_model_show_handler,
  __path_ollama_models_handler,
};

// AFTER:
use crate::routes_oai::{
  __path_chat_completions_handler, __path_embeddings_handler, __path_oai_model_handler,
  __path_oai_models_handler,
};
use crate::routes_ollama::{
  __path_ollama_model_chat_handler, __path_ollama_model_show_handler,
  __path_ollama_models_handler,
};
```

### 3.4 Verify Phase 1-3

```bash
cargo check -p routes_app
```

---

## Phase 4: Migrate routes_oai Tests to AppServiceStubBuilder

### 4.1 Migrate `chat_test.rs`

Update imports:
```rust
// BEFORE:
use services::test_utils::{app_service_stub, AppServiceStub};

// AFTER:
use services::test_utils::AppServiceStubBuilder;
```

Replace fixture parameter with inline builder in all 3 test functions (`test_routes_chat_completions_non_stream`, `test_routes_chat_completions_stream`, `test_routes_embeddings_non_stream`):

```rust
// BEFORE:
async fn test_routes_chat_completions_non_stream(
    #[future] app_service_stub: AppServiceStub,
) -> anyhow::Result<()> {
    // ...
    let router_state = DefaultRouterState::new(Arc::new(ctx), Arc::new(app_service_stub));

// AFTER:
async fn test_routes_chat_completions_non_stream() -> anyhow::Result<()> {
    let app_service = AppServiceStubBuilder::default()
        .with_data_service().await
        .build()?;
    // ...
    let router_state = DefaultRouterState::new(Arc::new(ctx), Arc::new(app_service));
```

Keep `MockSharedContext` -- required for `forward_request()` mocking.

### 4.2 `models_test.rs` -- No Changes

Already uses `AppServiceStubBuilder`. No migration needed.

### 4.3 routes_ollama tests -- No Changes

Leave as-is per decision to skip ollama restructuring.

### 4.4 Verify Phase 4

```bash
cargo test -p routes_app
```

---

## Phase 5: Update Consumers

### 5.1 Update `crates/routes_all/Cargo.toml`

Remove `routes_oai` dependency (line 12):
```toml
# DELETE:
routes_oai = { workspace = true }
```

### 5.2 Update `crates/routes_all/src/routes.rs`

Merge routes_oai imports into the routes_app import block (lines 44-49 → merge into lines 19-43):

```rust
// DELETE these lines (44-49):
use routes_oai::{
  chat_completions_handler, embeddings_handler, oai_model_handler, oai_models_handler,
  ollama_model_chat_handler, ollama_model_show_handler, ollama_models_handler,
  ENDPOINT_OAI_CHAT_COMPLETIONS, ENDPOINT_OAI_EMBEDDINGS, ENDPOINT_OAI_MODELS,
  ENDPOINT_OLLAMA_CHAT, ENDPOINT_OLLAMA_SHOW, ENDPOINT_OLLAMA_TAGS,
};

// ADD to existing routes_app import block:
use routes_app::{
  // ... existing imports ...
  // ADD these:
  chat_completions_handler, embeddings_handler, oai_model_handler, oai_models_handler,
  ollama_model_chat_handler, ollama_model_show_handler, ollama_models_handler,
  ENDPOINT_OAI_CHAT_COMPLETIONS, ENDPOINT_OAI_EMBEDDINGS, ENDPOINT_OAI_MODELS,
  ENDPOINT_OLLAMA_CHAT, ENDPOINT_OLLAMA_SHOW, ENDPOINT_OLLAMA_TAGS,
};
```

### 5.3 Update `crates/lib_bodhiserver/Cargo.toml`

Remove unused `routes_oai` dependency (line 14):
```toml
# DELETE:
routes_oai = { workspace = true }
```

### 5.4 Verify Phase 5

```bash
cargo check -p routes_all && cargo check -p lib_bodhiserver
```

---

## Phase 6: Delete routes_oai Crate and Clean Workspace

### 6.1 Delete crate directory

```bash
rm -rf crates/routes_oai/
```

### 6.2 Update workspace `Cargo.toml`

- Remove `"crates/routes_oai"` from `[workspace] members` array (line 12)
- Remove `routes_oai = { path = "crates/routes_oai" }` from `[workspace.dependencies]` (line 40)

### 6.3 Final Verification

```bash
cargo check --workspace
make test.backend
```

---

## Phase 7: Update Documentation

### 7.1 Regenerate crate docs

Use `docs-updater` agent for:
- `crates/routes_app/CLAUDE.md` and `PACKAGE.md` (incorporate routes_oai + routes_ollama info)
- `crates/routes_all/CLAUDE.md` and `PACKAGE.md` (reflect single dependency on routes_app)

### 7.2 Update root CLAUDE.md

In "Key Crates Structure" section, remove `routes_oai` as separate entry. Note that OpenAI/Ollama endpoints are now domain modules within `routes_app`.

### 7.3 Run symlink update

```bash
make docs.context-update
```

---

## Cross-Crate Boundary Inventory (What Changes)

| Consumer | Current Import Source | After Merge |
|----------|---------------------|-------------|
| `routes_app/shared/openapi.rs` | `routes_oai::__path_*` | `crate::routes_oai::__path_*` + `crate::routes_ollama::__path_*` |
| `routes_all/routes.rs` | `routes_oai::{handlers, ENDPOINT_*}` | `routes_app::{handlers, ENDPOINT_*}` |
| `lib_bodhiserver/Cargo.toml` | `routes_oai` dep (unused) | Removed |

## Dependency Changes for routes_app

| Dep | Section | Status |
|-----|---------|--------|
| `futures-util` | `[dependencies]` | ADD (used in chat.rs streaming) |
| `http` | `[dependencies]` | ADD (used in chat.rs for http::Error) |
| `objs` features=["test-utils"] | `[dev-dependencies]` | ADD (test fixtures) |
| `llama_server_proc` features=["test-utils"] | `[dev-dependencies]` | ADD (mock_response helper) |
| `reqwest` | `[dev-dependencies]` | ADD (reqwest::Response, StatusCode in tests) |
| `routes_oai` | `[dependencies]` | DELETE |

## Error Code Preservation

No error codes change:
- `oai_route_error-http` (500)
- `oai_route_error-serialization` (500)
- `oai_route_error-invalid_request` (400)
- `OllamaError` has no error code (it's a simple `{"error": "string"}` response)

## OpenAPI Preservation

The `BodhiOpenAPIDoc` struct in `shared/openapi.rs` continues to list the same handler functions in `paths(...)`. The only change is import source (`crate::routes_oai::__path_*` and `crate::routes_ollama::__path_*` instead of `routes_oai::__path_*`). Generated OpenAPI spec is identical.
