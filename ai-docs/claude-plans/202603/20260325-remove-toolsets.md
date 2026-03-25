# Plan: Complete Removal of Toolset Feature

## Context

The toolset feature was an experimental capability (builtin-exa-search) that helped design the MCP feature. Now that MCP is the chosen path forward, the toolset feature must be completely removed before production release. There are production deployments (but no users/data), so compensating migrations are needed. No backwards compatibility required.

## Decisions (from interview)

- Compensating migration to drop tables (keep original creation migrations)
- Remove toolset fields from V1 structs directly (no version bump)
- Remove all 3 service traits (ToolService, AuthScopedToolService, ExaService)
- Remove ToolsetAccessRequestValidator, keep McpAccessRequestValidator
- Full chat pipeline cleanup (MCP-only)
- Remove setup toolsets step (6-step flow)
- Simplify ToolCallMessage to MCP-only
- Remove all access request toolset integration (both backend and frontend)
- Leave archived AI docs untouched
- Compiler-driven frontend cleanup via ts-client regen + typecheck

---

## Phase 1: `services` crate

**Gate**: `cargo check -p services` then `cargo test -p services --lib`

### 1.1 Add compensating migration

**Create** `crates/services/src/db/sea_migrations/m20250101_000017_drop_toolsets.rs`
- `up()`: DROP TABLE IF EXISTS `app_toolset_configs`, then `toolsets`
- PostgreSQL: drop RLS policies first (DROP POLICY IF EXISTS ... / DISABLE ROW LEVEL SECURITY)
- `down()`: no-op

**Modify** `crates/services/src/db/sea_migrations/mod.rs`
- Add `mod m20250101_000017_drop_toolsets;`
- Add to migrations vec

### 1.2 Delete entire `toolsets/` module

**Delete** `crates/services/src/toolsets/` (15 files):
- Production: `mod.rs`, `toolset_entity.rs`, `app_toolset_config_entity.rs`, `toolset_objs.rs`, `error.rs`, `tool_service.rs`, `auth_scoped.rs`, `exa_service.rs`, `execution.rs`, `toolset_repository.rs`
- Tests: `test_tool_service.rs`, `test_toolset_repository.rs`, `test_toolset_repository_isolation.rs`, `test_exa_service.rs`, `test_toolset_objs.rs`

### 1.3 Remove from `lib.rs`

**Modify** `crates/services/src/lib.rs`
- Remove `mod toolsets;` and `pub use toolsets::*;`

### 1.4 Remove from AppService trait

**Modify** `crates/services/src/app_service/app_service.rs`
- Remove `ToolService` from imports
- Remove `fn tool_service(&self) -> Arc<dyn ToolService>;` from trait
- Remove `tool_service` field from `DefaultAppService`
- Remove `fn tool_service()` impl

### 1.5 Remove from AuthScopedAppService

**Modify** `crates/services/src/app_service/auth_scoped.rs`
- Remove `AuthScopedToolService` import
- Remove `pub fn tools()` method

### 1.6 Clean access_request_objs.rs

**Note**: HEAD commit `580c6f0b` introduced versioned enums with custom `Deserialize` impls. `RequestedResources` and `ApprovedResources` are now tagged enums dispatching to V1 structs. The custom `Deserialize` impls, `version()` methods, and `Default` impls remain — they are version infrastructure, not toolset-specific.

**Modify** `crates/services/src/app_access_requests/access_request_objs.rs`
- Delete structs: `ToolsetTypeRequest`, `ToolsetApproval`, `ToolsetInstance`
- Remove `toolset_types: Vec<ToolsetTypeRequest>` field from `RequestedResourcesV1` (keep `mcp_servers`)
- Remove `toolsets: Vec<ToolsetApproval>` field from `ApprovedResourcesV1` (keep `mcps`)
- Update `#[schema(example)]` on `CreateAccessRequest` to remove `toolset_types` array (keep `version` and `mcp_servers` if present, or use MCP example)
- Update `#[schema(example)]` on `ApproveAccessRequest` to remove `toolsets` array (keep `version` and `mcps`)

### 1.6a Clean access_request_service.rs

**Modify** `crates/services/src/app_access_requests/access_request_service.rs`
- In `generate_description()`: inside `ApprovedResources::V1(v1)` match arm, remove the `for approval in &v1.toolsets { ... }` loop (keep MCP loop)
- The `create_draft()` and `approve_request()` signatures are fine — they already take `RequestedResources`/`ApprovedResources` enums directly (changed in HEAD commit)
- The version-match validation in `approve_request()` stays (it's version infrastructure, not toolset-specific)

### 1.6b Clean access_request tests

**Modify** `crates/services/src/app_access_requests/test_access_request_service.rs`
- Remove `ToolsetTypeRequest` from imports (~line 43-44)
- Remove `toolset_types` data from `RequestedResourcesV1` test fixtures (lines using `toolset_types: vec![ToolsetTypeRequest { ... }]`)
- Remove `ToolsetApproval` from imports (~line 273)
- Remove `toolsets` data from `ApprovedResourcesV1` test fixtures (lines using `toolsets: vec![ToolsetApproval { ... }]`)
- Keep MCP test data intact

### 1.7 Remove ToolsetRepository from DbService

**Modify** `crates/services/src/db/service.rs`
- Remove `use crate::toolsets::ToolsetRepository;`
- Remove `+ ToolsetRepository` from trait bound and blanket impl

### 1.8 Remove from default_service.rs

**Modify** `crates/services/src/db/default_service.rs`
- Remove `"toolsets"` and `"app_toolset_configs"` from PostgreSQL and SQLite table lists in `reset_all_tables`

### 1.9 Remove from test_rls.rs

**Modify** `crates/services/src/db/test_rls.rs`
- Remove `"toolsets"` and `"app_toolset_configs"` from RLS test table lists

### 1.10 Clean test_utils/db.rs

**Modify** `crates/services/src/test_utils/db.rs`
- Remove `use crate::toolsets::{AppToolsetConfigEntity, ToolsetEntity, ToolsetRepository};`
- Remove `impl ToolsetRepository for TestDbService { ... }` block (~lines 509-636)
- Remove mockall `impl ToolsetRepository for DbService { ... }` block (~lines 1524-1535)

### 1.11 Clean test_utils/app.rs

**Modify** `crates/services/src/test_utils/app.rs`
- Remove `MockExaService`, `DefaultToolService`, `ToolService` from imports
- Remove `tool_service` field + builder default + `with_tool_service()` method
- Remove `fn tool_service()` from `impl AppService for AppServiceStub`

---

## Phase 2: `routes_app` crate

**Gate**: `cargo check -p routes_app` then `cargo test -p routes_app --lib`

### 2.1 Delete entire `toolsets/` module

**Delete** `crates/routes_app/src/toolsets/` (8 files):
- `mod.rs`, `routes_toolsets.rs`, `toolsets_api_schemas.rs`, `error.rs`
- `test_toolsets_crud.rs`, `test_toolsets_auth.rs`, `test_toolsets_isolation.rs`, `test_toolsets_types.rs`

### 2.2 Remove from lib.rs

**Modify** `crates/routes_app/src/lib.rs`
- Remove `mod toolsets;` and `pub use toolsets::*;`

### 2.3 Clean routes.rs

**Modify** `crates/routes_app/src/routes.rs`
- Remove `ToolsetAccessRequestValidator` import
- Remove all toolset handler imports (`toolsets_*`, `apps_toolsets_*`, `toolset_types_*`)
- Remove `ENDPOINT_TOOLSETS`, `ENDPOINT_TOOLSET_TYPES`, `ENDPOINT_APPS_TOOLSETS` imports
- Remove toolset CRUD routes from `user_session_apis` (~lines 204-212)
- Remove `toolset_exec_apis` block (~lines 311-337)
- Remove toolset type admin routes from `admin_session_apis` (~lines 489-497)
- Remove `apps_toolset_exec` block (~lines 377-390)
- Remove `ENDPOINT_APPS_TOOLSETS` route from `apps_list_apis` (line 374)
- Remove `.merge(apps_toolset_exec)` and `.merge(toolset_exec_apis)` merges

### 2.4 Remove ToolsetAccessRequestValidator

**Modify** `crates/routes_app/src/middleware/access_requests/access_request_validator.rs`
- Remove `ToolsetAccessRequestValidator` struct and impl (~lines 30-69)

**Modify** `crates/routes_app/src/middleware/access_requests/mod.rs`
- Remove `ToolsetAccessRequestValidator` from exports

### 2.5 Clean access request middleware tests

**Modify** `crates/routes_app/src/middleware/access_requests/test_access_request_middleware.rs`
- Remove `ToolsetAccessRequestValidator` import
- Remove `toolset_validator` fixture and `test_router`/`test_router_with_db` helpers for toolsets
- Remove all tests using `toolset_validator` fixture (~lines 114-440)
- Keep all MCP validator tests intact

### 2.6 Clean CORS tests

**Modify** `crates/routes_app/src/test_cors.rs`
- Remove 4 toolset CORS test cases (lines 12, 14, 15, 47)

### 2.7 Remove API_TAG_TOOLSETS

**Modify** `crates/routes_app/src/shared/constants.rs`
- Remove `pub const API_TAG_TOOLSETS: &str = "toolsets";`

### 2.8 Clean openapi.rs

**Modify** `crates/routes_app/src/shared/openapi.rs`
- Remove all toolset DTO imports, `__path_*` handler symbols
- Remove `API_TAG_TOOLSETS` import and tag definition
- Remove `ENDPOINT_TOOLSETS`, `ENDPOINT_TOOLSET_TYPES`, `ENDPOINT_APPS_TOOLSETS` constants
- Remove toolset schemas from `components(schemas(...))` block
- Remove toolset path registrations
- Remove toolset types from `services::` import block (`ToolDefinition`, `Toolset`, `ToolsetApproval`, etc.)

### 2.9 Clean apps module

**Note**: HEAD commit `580c6f0b` refactored `routes_apps.rs` to use `match &requested { RequestedResources::V1(v1) => { ... } }` pattern. Toolset and MCP validation now happen inside V1 match arms. Remove only toolset parts from each arm.

**Modify** `crates/routes_app/src/apps/apps_api_schemas.rs`
- Remove `use services::Toolset;`
- Delete `ToolTypeReviewInfo` struct
- Remove `tools_info: Vec<ToolTypeReviewInfo>` from `AccessRequestReviewResponse`

**Modify** `crates/routes_app/src/apps/routes_apps.rs`
- In `apps_create_access_request`: inside `RequestedResources::V1(v1)` arm, remove the `for tool_type_req in &v1.toolset_types { tools.validate_type(...) }` loop. Remove `let tools = auth_scope.tools();` since it's no longer needed.
- In `apps_get_access_request_review`: remove `let tools_svc = auth_scope.tools();`, `let all_user_toolsets = tools_svc.list().await?;`. Inside `RequestedResources::V1(v1)` arm, remove the `for tool_type_req in &v1.toolset_types { ... }` loop building `tools_info`. Remove `tools_info` from the response struct. Remove `let mut tools_info = Vec::new();`.
- In `apps_approve_access_request`: inside `ApprovedResources::V1(v1)` arm, remove the entire `for approval in &v1.toolsets { ... }` block (keep MCP approval validation).

**Modify** `crates/routes_app/src/apps/error.rs`
- Remove `use services::ToolsetError;`
- Remove `ToolServiceError(#[from] ToolsetError)` variant

**Modify** `crates/routes_app/src/apps/test_access_request.rs`
- Remove `DefaultToolService`, `MockExaService` imports
- Remove ToolService construction from test setup helper (`let tool_service: Arc<dyn services::ToolService> = ...`)
- Remove `.with_tool_service(tool_service)` from `AppServiceStubBuilder`
- Remove `seed_toolset_instance` helper function
- Remove/update all test data: `requested` JSON strings that include `toolset_types`, `approved` JSON with `toolsets` arrays
- Update assertions that reference toolset approval data
- Keep MCP-related test data and assertions intact

### 2.10 Clean access_request_validator.rs (versioned)

**Note**: HEAD commit changed `ToolsetAccessRequestValidator.validate_approved()` to use `match &approvals { ApprovedResources::V1(v1) => ... }`. The entire `ToolsetAccessRequestValidator` struct and impl are deleted, so the versioned pattern doesn't matter — just delete it.

### 2.11 Clean test_access_request_middleware.rs (versioned)

**Note**: HEAD commit updated middleware tests to use versioned `ApprovedResources::V1(...)` in test data. Remove all tests using `toolset_validator` fixture. Keep MCP validator tests which also use the versioned pattern.

---

## Phase 3: `lib_bodhiserver` crate

**Gate**: `cargo check -p lib_bodhiserver`

### 3.1 Remove build_tool_service from AppServiceBuilder

**Modify** `crates/lib_bodhiserver/src/app_service_builder.rs`
- Remove `DefaultToolService`, `ExaService`, `ToolService` imports
- Remove `let tool_service = Self::build_tool_service(...)` call
- Remove `tool_service` from `DefaultAppService::new(...)` args
- Remove entire `fn build_tool_service(...)` method

---

## Phase 4: Backend verification

```bash
cargo check -p services -p server_core -p routes_app -p server_app -p lib_bodhiserver 2>&1 | tail -5
cargo test -p services --lib -p routes_app -p server_app 2>&1 | grep -E "test result|FAILED|failures:"
```

Fix any compile errors or test failures.

---

## Phase 5: OpenAPI + ts-client regeneration

```bash
cargo run --package xtask openapi
make build.ts-client
```

This removes all toolset types from the generated TypeScript client.

---

## Phase 6: Frontend (`bodhi/src`) — compiler-driven cleanup

### 6.1 Run typecheck to find all broken references

```bash
cd crates/bodhi && npm run test:typecheck
```

This will surface every frontend file importing now-deleted toolset types.

### 6.2 Delete toolset-specific files

**Delete directories/files**:
- `crates/bodhi/src/app/toolsets/` (entire directory)
- `crates/bodhi/src/app/setup/toolsets/` (entire directory)
- `crates/bodhi/src/app/chat/ToolsetsPopover.tsx`
- `crates/bodhi/src/app/apps/access-requests/review/ToolTypeCard.tsx`
- `crates/bodhi/src/hooks/toolsets/` (entire directory)
- `crates/bodhi/src/routes/toolsets/` (entire directory)
- `crates/bodhi/src/routes/setup/toolsets/` (directory)
- `crates/bodhi/src/lib/toolsets.ts`
- `crates/bodhi/src/test-fixtures/toolsets.ts`
- `crates/bodhi/src/test-utils/msw-v2/handlers/toolsets.ts`

### 6.3 Clean ChatUI.tsx

**Modify** `crates/bodhi/src/app/chat/ChatUI.tsx`
- Remove `ToolsetsPopover` import
- Remove `useToolsetSelection`, `useListToolsets` imports
- Remove all toolset state: `enabledTools`, `toggleTool`, `toggleToolset`, `setEnabledTools`
- Remove `toolsetsResponse`, `toolsets`, `toolsetTypes` data fetching
- Remove `scopeEnabledMap` and auto-filter `useEffect`
- Remove toolset props from `useChat()` call and `ChatUIComponent` props
- Remove `<ToolsetsPopover>` from JSX
- Keep all MCP state and `McpsPopover` intact

### 6.4 Clean useChat.tsx

**Modify** `crates/bodhi/src/hooks/chat/useChat.tsx`
- Remove `ToolsetResponse`, `ToolsetExecutionResponse` imports
- Remove `import { encodeToolName, decodeToolName } from '@/lib/toolsets';`
- Delete `executeToolCall` function
- Delete `buildToolsArray` function
- Remove `toolsetSlugToId` useMemo
- Remove `scopeEnabledMap` useMemo
- Remove `enabledTools`, `toolsets`, `toolsetTypes` from `UseChatOptions`
- Simplify `executeAllToolCalls` to MCP-only
- Simplify tools-array building in `sendMessage` to just `mcpTools`

### 6.5 Clean ToolCallMessage.tsx

**Modify** `crates/bodhi/src/app/chat/ToolCallMessage.tsx`
- Remove `import { decodeToolName } from '@/lib/toolsets';`
- Remove `const toolsetDecoded = ...`
- Simplify `toolName` and `sourceSlug` to use only `mcpDecoded`

### 6.6 Update setup flow

**Modify** `crates/bodhi/src/app/setup/constants.ts`
- Remove `TOOLS: 5`, renumber `BROWSER_EXTENSION: 5`, `COMPLETE: 6`
- Remove `'Tools'` from `SETUP_STEP_LABELS`
- Change `SETUP_TOTAL_STEPS` to 6

**Modify** `crates/bodhi/src/app/setup/components/SetupProvider.tsx`
- Remove `if (path.includes('/setup/tools')) return SETUP_STEPS.TOOLS;`

**Modify** `crates/bodhi/src/app/setup/api-models/page.tsx`
- Replace `ROUTE_SETUP_TOOLSETS` with `ROUTE_SETUP_BROWSER_EXTENSION` (already exists in constants.ts)

**Modify** `crates/bodhi/src/app/setup/api-models/page.test.tsx`
- Update navigation expectations from `/setup/toolsets` to `/setup/browser-extension`

**Modify** `crates/bodhi/src/lib/constants.ts`
- Remove `export const ROUTE_SETUP_TOOLSETS = '/setup/toolsets';`

### 6.7 Clean access request review

**Modify** `crates/bodhi/src/app/apps/access-requests/review/page.tsx`
- Remove `ToolTypeCard` import
- Remove all toolset state (`approvedTools`, `selectedInstances` for toolsets)
- Remove toolset card rendering section
- Keep MCP review cards intact

**Modify** `crates/bodhi/src/app/apps/access-requests/review/page.test.tsx`
- Remove toolset assertions and test data

### 6.8 Clean hooks/apps

**Modify** `crates/bodhi/src/hooks/apps/useAppAccessRequests.ts`
- Remove `Toolset`, `ToolsetApproval`, `ToolTypeReviewInfo` imports/re-exports

**Modify** `crates/bodhi/src/hooks/apps/useAppAccessRequests.test.ts`
- Remove toolset test data

**Modify** `crates/bodhi/src/hooks/apps/index.ts`
- Remove toolset re-exports

### 6.9 Frontend verification

```bash
cd crates/bodhi && npm run test:typecheck
cd crates/bodhi && npm test
```

---

## Phase 7: E2E tests

### 7.1 Delete toolset E2E specs

**Delete**:
- `crates/lib_bodhiserver_napi/tests-js/specs/toolsets/` (entire directory)
- `crates/lib_bodhiserver_napi/tests-js/specs/setup/setup-toolsets.spec.mjs`
- `crates/lib_bodhiserver_napi/tests-js/specs/chat/chat-toolsets.spec.mjs`

### 7.2 Fix version validation E2E test

**Modify** `crates/lib_bodhiserver_napi/tests-js/specs/request-access/request-access-version-validation.spec.mjs`
- Update test data: replace `toolset_types: [{ toolset_type: 'builtin-exa-search' }]` with `mcp_servers: [{ url: 'https://example.com/mcp' }]` (both tests use toolset data in `requested` body)

### 7.3 Search for remaining references

```bash
grep -r "toolset" crates/lib_bodhiserver_napi/tests-js/ --include="*.mjs" --include="*.js"
```

Fix any remaining references in shared helpers, fixtures, or other specs.

### 7.4 E2E verification

```bash
make build.ui-rebuild
cd crates/lib_bodhiserver_napi && npm run test:playwright
```

---

## Phase 8: Documentation

### 8.1 Update CLAUDE.md files

- `crates/CLAUDE.md` — Remove toolset from error layer example, update multi-tenant isolation test reference
- `crates/services/CLAUDE.md` — Remove `tools()` from AuthScopedAppService, remove ToolService/ExaService from init order, remove toolset module from domain layout
- `crates/routes_app/CLAUDE.md` — Remove toolset route groups, `toolset_exec_apis`, toolset CORS references
- `crates/bodhi/src/CLAUDE.md` — Update setup flow (remove toolsets step), remove toolset hook references

### 8.2 Update .claude/skills

Remove toolset references from:
- `.claude/skills/test-services/SKILL.md`
- `.claude/skills/test-routes-app/SKILL.md`
- `.claude/skills/test-routes-app/fixtures.md`
- `.claude/skills/test-routes-app/advanced.md`
- `.claude/skills/test-services/mock-patterns.md`
- `.claude/skills/test-services/db-testing.md`
- `.claude/skills/test-services/advanced.md`

---

## Phase 9: Final verification

```bash
make test.backend
make build.ts-client
cd crates/bodhi && npm run test:all
make build.ui-rebuild
cd crates/lib_bodhiserver_napi && npm run test:playwright
```

---

## Summary

| Category | Delete | Modify | Create |
|----------|--------|--------|--------|
| services (Rust) | 15 files + directory | ~8 files | 1 migration |
| routes_app (Rust) | 8 files + directory | ~10 files | — |
| lib_bodhiserver | — | 1 file | — |
| Frontend (TS/TSX) | ~18 files + 4 directories | ~12 files | — |
| E2E tests | 4 files | TBD (grep-driven) | — |
| Documentation | — | ~11 files | — |
| **Total** | **~45 files** | **~42 files** | **1 file** |

## Execution Strategy

- Phases 1-3: sequential (upstream to downstream), each with gate checks
- Phase 4: backend-wide verification pass
- Phase 5: ts-client regen (bridge between backend and frontend)
- Phase 6: compiler-driven frontend cleanup (`npm run test:typecheck` finds all broken refs)
- Phases 7-8: can run in parallel (E2E tests + docs are independent)
- Phase 9: full suite verification

Sub-agents should work one phase at a time. Within each phase, file deletions should happen first, then modifications, to let the compiler surface all remaining references.
