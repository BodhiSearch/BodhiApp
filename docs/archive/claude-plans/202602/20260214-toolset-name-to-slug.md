# Toolset Cleanup: Rename name->slug, Remove Dead Code, Unify Validation, Deduplicate Types

## Phases 0-2: COMPLETED (name->slug rename, dead code removal)

## Phase 2.5: Fix remaining compilation errors from name->slug rename

### 2.5A: Remove extra `false` arg from `DefaultToolService::new()` in access_request_test.rs
- `crates/routes_app/src/routes_apps/tests/access_request_test.rs:47` - remove `, false` arg

### 2.5B: Fix `ToolsetDefinition.name` test assertion
- Already fixed: `crates/services/src/tool_service/tests.rs:192` - `.slug` back to `.name` (it's a ToolsetDefinition, not Toolset)

### 2.5C: Fix `ToolInstanceInfo` -> use `Toolset` directly
- Already fixed: `crates/routes_app/src/routes_apps/types.rs` and `handlers.rs`

### 2.5D: Fix `DefaultAccessRequestService::new()` extra `tool_service` arg
- Already fixed: `crates/lib_bodhiserver/src/app_service_builder.rs`

---

## Phase 2.6: Rename + Deduplicate Access Request Types

### Goal
- Rename `ToolTypeRequest` → `ToolsetTypeRequest`, `ToolApproval` → `ToolsetApproval` (naming consistency)
- Remove duplicate `ToolTypeRequest`, `ToolApproval` from `routes_app` - use objs versions directly
- Remove `ToolInstanceInfo` - use `objs::Toolset` directly
- Remove `ToolTypeReviewInfo` - compose from `objs::ToolsetDefinition` + `Vec<Toolset>`
- Remove `ToolsetTypeResponse` - use `objs::ToolsetDefinition` directly

### Where types belong (analysis)
- **`ToolsetTypeRequest` stays in objs**: Used in service trait signatures (`AccessRequestService::create_draft`), service domain objects (`AppAccessRequest.tools`, `AppAccessRequestDetail.tools_requested`), and routes. Cross-boundary domain object.
- **`ToolsetApproval` stays in objs**: Used in service trait signatures (`AccessRequestService::approve_request`), service domain objects (`AppAccessRequestDetail.tools_approved`), and routes. Cross-boundary domain object.
- **`RequestedResources`, `ApprovedResources` stay in routes_app**: Pure HTTP request/response wrappers only used in route handlers.

### Step 1: objs crate - rename types

**`crates/objs/src/access_request.rs`**:
- `ToolTypeRequest` → `ToolsetTypeRequest`
- `ToolApproval` → `ToolsetApproval`

**Verify**: `cargo check -p objs`

### Step 2: services crate - update references

**`crates/services/src/objs.rs`**:
- `objs::ToolTypeRequest` → `objs::ToolsetTypeRequest` (2 occurrences)
- `objs::ToolApproval` → `objs::ToolsetApproval` (1 occurrence)

**`crates/services/src/access_request_service/service.rs`**:
- `objs::ToolTypeRequest` → `objs::ToolsetTypeRequest` (2 occurrences)
- `objs::ToolApproval` → `objs::ToolsetApproval` (3 occurrences)

**Verify**: `cargo check -p services && cargo test -p services`

### Step 3: routes_app crate - deduplicate types

**`crates/routes_app/src/routes_apps/types.rs`**:
- DELETE `ToolTypeRequest` struct (duplicate of objs)
- DELETE `ToolApproval` struct (duplicate of objs)
- DELETE `ToolInstanceInfo` struct (subset of `objs::Toolset`)
- KEEP `ToolTypeReviewInfo` but change `instances` field type: `Vec<ToolInstanceInfo>` → `Vec<objs::Toolset>`
- `RequestedResources.toolset_types`: change from `Vec<ToolTypeRequest>` → `Vec<objs::ToolsetTypeRequest>`
- `ApprovedResources.toolset_types`: change from `Vec<ToolApproval>` → `Vec<objs::ToolsetApproval>`
- Update schema examples to match new field shapes

**`crates/routes_app/src/routes_apps/handlers.rs`**:
- Remove manual `ToolTypeRequest` conversion (lines 78-103) - `RequestedResources` now directly contains `objs::ToolsetTypeRequest`
- Remove `ToolInstanceInfo` mapping (lines 240-249) - use `Toolset` directly via `.clone()` + `.filter()`
- Remove `objs::ToolApproval` conversion (lines 348-358) - `ApprovedResources` now directly contains `objs::ToolsetApproval`
- Update imports

**`crates/routes_app/src/routes_toolsets/types.rs`**:
- DELETE `ToolsetTypeResponse` struct - replace with `objs::ToolsetDefinition`

**`crates/routes_app/src/routes_toolsets/toolsets.rs`**:
- `list_toolset_types_handler`: return `Vec<ToolsetDefinition>` instead of mapping to `ToolsetTypeResponse`
- `ListToolsetTypesResponse.types`: change type from `Vec<ToolsetTypeResponse>` to `Vec<ToolsetDefinition>`

**Verify**: `cargo check -p routes_app && cargo test -p routes_app`

### Step 4: auth_middleware - update if needed

Check if `ToolTypeRequest`/`ToolApproval` are referenced in auth_middleware.

### Step 5: Remaining crates

Update any other references found by cargo check.

---

## Phase 3: OpenAPI + TypeScript Client Regeneration (pending)

```bash
cargo run --package xtask openapi
cd ts-client && npm run generate && npm run build
```

## Phase 4: Frontend rename (pending)

Update TypeScript interfaces to match new API shapes:
- `ToolInstanceInfo` interface replaced by `Toolset` type from ts-client
- `ToolTypeReviewInfo.instances` now `Toolset[]`
- `ToolsetTypeResponse` replaced by `ToolsetDefinition`
- Field renames: `name` → `slug` on toolset instances

## Phase 5: Verification (pending)

```bash
make format.all
cargo check
make test.backend
cd ts-client && npm run build
cd crates/bodhi && npm run build
make test.ui
```
