# Implementation Phases

> Status: Planning | Created: 2026-01-14

## Phase Order (by dependency)

```
Phase domain-objects → Phase database → Phase exa-service → Phase tool-service
    → Phase routes → Phase auth-middleware → Phase ui-pages → Phase integration
```

---

## Phase domain-objects: Domain Types in `objs` crate

**Files to create:**
- `crates/objs/src/tool_definition.rs`
- `crates/objs/src/tool_scope.rs`
- `crates/objs/src/tool_config.rs`
- `crates/objs/src/tool_execution.rs`

**Changes:**
- `crates/objs/src/lib.rs` - add module exports

**Tests:**
- Unit tests for ToolScope parsing, serialization
- Unit tests for ToolDefinition JSON schema

---

## Phase database: Database Migration & DbService Extension

**Files to create:**
- `crates/services/migrations/0007_tools_config.up.sql`
- `crates/services/migrations/0007_tools_config.down.sql`

**Files to modify:**
- `crates/services/src/db/service.rs` - add tool config CRUD methods
- `crates/services/src/db/objs.rs` - add `UserToolConfigRow` struct

**Methods to add to DbService:**
```rust
async fn get_user_tool_config(&self, user_id: &str, tool_id: &str) -> Result<Option<UserToolConfigRow>, DbError>;
async fn upsert_user_tool_config(&self, config: &UserToolConfigRow) -> Result<UserToolConfigRow, DbError>;
async fn list_user_tool_configs(&self, user_id: &str) -> Result<Vec<UserToolConfigRow>, DbError>;
```

**Tests:**
- Migration up/down roundtrip
- CRUD operations with encryption

---

## Phase exa-service: Exa API Integration

**Files to create:**
- `crates/services/src/exa_service.rs`
- `crates/services/src/exa_error.rs`

**Files to modify:**
- `crates/services/src/lib.rs` - add module exports

**Tests:**
- Unit tests with mockito for HTTP responses
- Error mapping tests (401 → InvalidApiKey, 429 → RateLimited)

---

## Phase tool-service: Tool Service Implementation

**Files to create:**
- `crates/services/src/tool_service.rs`
- `crates/services/src/tool_error.rs`

**Files to modify:**
- `crates/services/src/lib.rs` - add module exports
- `crates/services/src/app_service.rs` - add `tool_service()` method

**Tests:**
- Unit tests with MockDbService, MockExaService
- Tool execution flow tests

---

## Phase routes: API Routes

**Files to create:**
- `crates/routes_app/src/routes_tools.rs`
- `crates/routes_app/src/tools_dto.rs`

**Files to modify:**
- `crates/routes_app/src/lib.rs` - add module exports
- `crates/routes_app/src/openapi.rs` - add OpenAPI paths/schemas
- `crates/routes_all/src/routes.rs` - register tool routes

**Tests:**
- Handler unit tests with axum-test
- Request/response validation tests

---

## Phase auth-middleware: Tool Authorization

**Files to create:**
- `crates/auth_middleware/src/tool_auth_middleware.rs`

**Files to modify:**
- `crates/auth_middleware/src/lib.rs` - add module export

**Tests:**
- Session auth → config check
- bodhiapp_ token → config check
- OAuth token → scope check + config check
- Missing scope → 403

---

## Phase ui-pages: Frontend UI

**Files to create:**
- `crates/bodhi/src/app/ui/tools/page.tsx`
- `crates/bodhi/src/app/ui/tools/[toolId]/page.tsx`

**Files to modify:**
- `crates/bodhi/src/components/sidebar.tsx` (or nav component) - add Tools link
- `crates/bodhi/src/mocks/handlers.ts` - add tool MSW handlers

**Tests:**
- Component tests with MSW mocks
- Navigation test

---

## Phase integration: E2E Testing

**Files to modify:**
- `crates/integration-tests/tests/` - add tool integration tests

**Test scenarios:**
1. Configure tool with API key
2. Enable tool
3. Execute tool via chat flow
4. OAuth scope rejection

**E2E Requirements:**
- `EXA_API_KEY` environment variable for real API tests
- MSW for frontend-only tests

---

## Verification Checklist

After implementation:

1. **Backend unit tests**: `make test.backend`
2. **Frontend tests**: `make test.ui`
3. **Format**: `make format.all`
4. **OpenAPI regenerate**: `cargo run --package xtask openapi`
5. **TypeScript client**: `cd ts-client && npm run generate`
6. **Manual verification**:
   - Navigate to /ui/tools
   - Configure Exa API key
   - Enable tool
   - Start chat, verify tool appears in tool picker
   - Execute search via LLM tool call
