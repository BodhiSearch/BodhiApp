# Implementation Phases

> Status: Phases 1-8, 7.6 Complete | Phase 8.1, 9 Pending | Updated: 2026-01-16

## Phase Completion Summary

| Phase | Status | Tests | Notes |
|-------|--------|-------|-------|
| 1. Domain Objects | ✅ Complete | 16 passing | Consolidated into `tools.rs` |
| 2. Database | ✅ Complete | 7 passing | Migration 0007, CRUD with encryption |
| 3. Exa Service | ✅ Complete | 6 passing | HTTP client, 30s timeout |
| 4. Tool Service | ✅ Complete | 10 passing | Builtin registry, execution logic, app-client checks |
| 5. AppService Integration | ✅ Complete | - | ToolService as 14th parameter |
| 6. API Routes | ✅ Complete | 6 passing | 5 endpoints in routes_app |
| 7. Auth Middleware | ✅ Complete | 10 passing | Configuration + OAuth scope checking |
| 7.5. App-Level Tool Config | ✅ Complete | 9 passing | Admin enable/disable, ~~Keycloak sync~~ |
| 7.6. External App Tool Access | ✅ Complete | 13 passing | OAuth scope-based auth, fixes 7.5 |
| 8. Frontend UI | ✅ Complete | 42+ passing | `/ui/tools` pages, setup step 5, E2E tests |
| 8.1. Chat UI Integration | ⏳ Pending | - | Agentic search in chat |
| 9. Integration Tests | ⏳ Pending | - | Additional E2E and integration tests |

**Total**: 100+ passing tests, ~4,000 lines of new/modified code

---

## Phase 1: Domain Objects ✅ COMPLETE

**Files created:**
- `crates/objs/src/tools.rs` (344 lines) - Consolidated all tool types

**Files modified:**
- `crates/objs/src/lib.rs` - Add `mod tools; pub use tools::*;`

**Types implemented:**
- `ToolScope` enum with `BuiltinExaWebSearch`
- `ToolDefinition` and `FunctionDefinition` (OpenAI format)
- `UserToolConfig` (public API model)
- `ToolExecutionRequest` and `ToolExecutionResponse`

**Tests:** 16 passing
- ToolScope parsing and serialization (kebab-case)
- ToolScope tool_id mapping
- ToolDefinition JSON schema validation
- UserToolConfig timestamp conversions

---

## Phase 2: Database ✅ COMPLETE

**Files created:**
- `crates/services/migrations/0007_tools_config.up.sql`
- `crates/services/migrations/0007_tools_config.down.sql`

**Files modified:**
- `crates/services/src/db/objs.rs` (+17 lines) - `UserToolConfigRow` struct
- `crates/services/src/db/service.rs` (+433 lines) - CRUD methods
- `crates/services/src/db/mod.rs` - Export UserToolConfigRow
- `crates/services/src/test_utils/db.rs` (+51 lines) - Test helpers

**Methods added to DbService:**
```rust
async fn get_user_tool_config(&self, user_id: &str, tool_id: &str) 
    -> Result<Option<UserToolConfigRow>, DbError>;
async fn upsert_user_tool_config(&self, config: &UserToolConfigRow) 
    -> Result<UserToolConfigRow, DbError>;
async fn list_user_tool_configs(&self, user_id: &str) 
    -> Result<Vec<UserToolConfigRow>, DbError>;
async fn delete_user_tool_config(&self, user_id: &str, tool_id: &str) 
    -> Result<(), DbError>;
```

**Implementation details:**
- Uses existing encryption: `encrypt_api_key()` / `decrypt_api_key()`
- Unique constraint on (user_id, tool_id)
- 3 indexes: user_id, tool_id, enabled

**Tests:** 7 passing
- Migration up/down roundtrip
- CRUD operations with encryption/decryption
- Unique constraint validation
- Row conversion to/from domain model

---

## Phase 3: Exa Service ✅ COMPLETE

**Files created:**
- `crates/services/src/exa_service.rs` (331 lines)

**Files modified:**
- `crates/services/src/lib.rs` - Add `mod exa_service; pub use exa_service::*;`

**Implementation:**
```rust
pub enum ExaError {
    RequestFailed(String),
    RateLimited,
    InvalidApiKey,
    Timeout,
}

#[async_trait::async_trait]
pub trait ExaService: Debug + Send + Sync {
    async fn search(&self, api_key: &str, query: &str, num_results: Option<u32>) 
        -> Result<ExaSearchResponse, ExaError>;
}

pub struct DefaultExaService {
    client: reqwest::Client,  // 30s timeout
}
```

**API details:**
- POST to `https://api.exa.ai/search`
- Headers: `x-api-key`, `Content-Type: application/json`
- Neural search with autoprompt enabled
- Error mapping: 401→InvalidApiKey, 429→RateLimited, timeout→Timeout

**Tests:** 6 passing (with mockito HTTP mocking)
- Success response parsing
- Error status codes (401, 429, 500)
- Timeout handling
- Request body validation

---

## Phase 4: Tool Service ✅ COMPLETE

**Files created:**
- `crates/services/src/tool_service.rs` (575 lines)

**Files modified:**
- `crates/services/src/lib.rs` - Add `mod tool_service; pub use tool_service::*;`

**Implementation:**
```rust
pub enum ToolError {
    ToolNotFound(String),
    ToolNotConfigured,
    ToolDisabled,
    ExecutionFailed(String),
    #[error(transparent)] DbError(#[from] DbError),
    #[error(transparent)] ExaError(#[from] ExaError),
}

#[async_trait::async_trait]
pub trait ToolService: Debug + Send + Sync {
    async fn list_tools_for_user(&self, user_id: &str) -> Result<Vec<ToolDefinition>, ToolError>;
    fn list_all_tool_definitions(&self) -> Vec<ToolDefinition>;
    async fn get_user_tool_config(&self, user_id: &str, tool_id: &str) 
        -> Result<Option<UserToolConfig>, ToolError>;
    async fn update_user_tool_config(&self, user_id: &str, tool_id: &str, 
        enabled: bool, api_key: Option<String>) -> Result<UserToolConfig, ToolError>;
    async fn delete_user_tool_config(&self, user_id: &str, tool_id: &str) 
        -> Result<(), ToolError>;
    async fn execute_tool(&self, user_id: &str, tool_id: &str, 
        request: ToolExecutionRequest) -> Result<ToolExecutionResponse, ToolError>;
    async fn is_tool_available_for_user(&self, user_id: &str, tool_id: &str) 
        -> Result<bool, ToolError>;
}
```

**Static tool registry:**
- `builtin_tool_definitions()` returns hardcoded list with "builtin-exa-web-search"
- JSON Schema for function parameters

**Tests:** 7 passing
- Tool execution flow with MockDbService and MockExaService
- Config validation (enabled + has API key)
- Error propagation from DB and Exa services
- Tool availability checking

---

## Phase 5: AppService Integration ✅ COMPLETE

**Files modified:**
- `crates/services/src/app_service.rs` (+9 lines)
  - Add `tool_service()` method to AppService trait
  - Add `tool_service: Arc<dyn ToolService>` as **14th field** to DefaultAppService
  - Update `DefaultAppService::new()` signature
  - Implement getter

- `crates/lib_bodhiserver/src/app_service_builder.rs` (+28 lines)
  - Create DefaultExaService with 30s timeout client
  - Create DefaultToolService with db, exa, time services
  - Pass to DefaultAppService::new()

- `crates/services/src/test_utils/app.rs` (+19 lines)
  - Add `with_tool_service()` builder method
  - Add MockToolService to test utilities

**Verification:**
- All service crates build successfully
- lib_bodhiserver builds successfully
- Integration verified through compilation

---

## Phase 6: API Routes ✅ COMPLETE

**Files created:**
- `crates/routes_app/src/routes_tools.rs` (516 lines)
- `crates/routes_app/src/tools_dto.rs` (181 lines)

**Files modified:**
- `crates/routes_app/src/lib.rs` - Add module exports

**Endpoints implemented:**
```rust
pub fn routes_tools(state: Arc<dyn RouterState>) -> Router {
    Router::new()
        .route("/tools", get(list_all_tools_handler))
        .route("/tools/:tool_id/config", get(get_tool_config_handler))
        .route("/tools/:tool_id/config", put(update_tool_config_handler))
        .route("/tools/:tool_id/config", delete(delete_tool_config_handler))
        .route("/tools/:tool_id/execute", post(execute_tool_handler))
        .route("/tools/:tool_id/app-config", put(enable_app_tool_handler))
        .route("/tools/:tool_id/app-config", delete(disable_app_tool_handler))
        .with_state(state)
}
```

**API-Level Validation:**
- `PUT /tools/:tool_id/config` returns 400 if app-level disabled
- `POST /tools/:tool_id/execute` returns 400 if app-level disabled
- `DELETE /tools/:tool_id/config` always allowed (cleanup)

**DTOs:**
- `ListToolsResponse` - OpenAI-compatible list format with app_enabled status
- `EnhancedToolConfigResponse` - Config with app_enabled status
- `UpdateToolConfigRequest` - Enable/disable + optional API key
- `ExecuteToolRequest` - tool_call_id + arguments
- `AppToolConfigResponse` - Admin response for app-level changes
- Re-export `ToolExecutionResponse` from objs

**Tests:** 6 passing
- All handler endpoints tested
- Request/response validation
- Error cases (not found, not configured, app disabled)

---

## Phase 7: Auth Middleware ✅ COMPLETE

**Files created:**
- `crates/auth_middleware/src/tool_auth_middleware.rs` (310 lines)

**Files modified:**
- `crates/auth_middleware/src/lib.rs` - Export tool_auth_middleware
- `crates/services/src/test_utils/app.rs` - Add `with_tool_service()` method

**Implementation:**
```rust
pub enum ToolAuthError {
    MissingUserId,
    MissingAuth,
    #[error(transparent)] ToolError(#[from] ToolError),
}

pub async fn tool_auth_middleware(
    State(state): State<Arc<dyn RouterState>>,
    Path(tool_id): Path<String>,
    req: Request,
    next: Next,
) -> Result<Response, ApiError>
```

**Authorization logic:**
1. Extract user_id from `KEY_HEADER_BODHIAPP_USER_ID`
2. Verify authentication exists (role or scope header)
3. Check tool is configured for user via `ToolService::is_tool_available_for_user()`
4. Allow if configured, reject with ToolNotConfigured otherwise

**Tests:** 7 passing
- Session auth + tool configured → pass
- Session auth + tool not configured → reject
- First-party token + configured → pass
- OAuth token + configured → pass
- OAuth token + not configured → reject
- Missing user_id → reject
- Missing auth → reject

---

## Phase 7.5: App-Level Tool Config ✅ COMPLETE

**Goal**: Add admin-controlled app-level tool enable/disable that gates user-level configuration.

**Spec**: See [05.5-app-level-tool-config.md](./05.5-app-level-tool-config.md) for full details.

**Files created/modified:**

Database:
- `crates/services/migrations/0007_tools_config.up.sql` - Added `app_tool_configs` table with seed data
- `crates/services/migrations/0007_tools_config.down.sql` - Added drop statement
- `crates/services/src/db/objs.rs` - Added `AppToolConfigRow`
- `crates/services/src/db/service.rs` - Added CRUD methods (`get_app_tool_config`, `upsert_app_tool_config`, `list_app_tool_configs`)
- `crates/services/src/db/mod.rs` - Made `encryption` module public

Domain:
- `crates/objs/src/tools.rs` - Added `AppToolConfig` struct

Tool Service:
- `crates/services/src/tool_service.rs` - Added `get_app_tool_config()`, `is_tool_enabled_for_app()`, `set_app_tool_enabled()`, `list_app_tool_configs()`, modified `is_tool_available_for_user()` to check app-level first

Routes:
- `crates/routes_app/src/routes_tools.rs` - Added admin routes, enriched existing responses with `app_enabled`
- `crates/routes_app/src/tools_dto.rs` - Added `AppToolConfigResponse`, `ToolListItem`, `UserToolConfigSummary`, `EnhancedToolConfigResponse`

**API Endpoints:**
- `PUT /tools/:tool_id/app-config` - Admin enables tool for app (requires `ResourceRole::Admin`)
- `DELETE /tools/:tool_id/app-config` - Admin disables tool for app (requires `ResourceRole::Admin`)

**Key Design Decisions:**
- Two-tier auth for session: `app_enabled AND user_enabled AND has_api_key`
- Local DB only for app-level config (no auth server sync)
- Default state: enabled via migration seed data
- Admin token passthrough (no exchange)

**Tests:** 9 passing
- `is_tool_available_returns_false_when_no_app_config`
- `is_tool_available_returns_false_when_app_disabled`
- `is_tool_available_returns_false_when_user_disabled`
- `is_tool_available_returns_false_when_no_api_key`
- `is_tool_available_returns_true_when_app_and_user_enabled`
- And 4 more covering tool service functionality

---

## Phase 7.6: External App Tool Access ✅ COMPLETE

**Completed: January 2026**

See [05.6-external-app-tool-access.md](./05.6-external-app-tool-access.md) and [phase-7.6-implementation.md](./phase-7.6-implementation.md) for details.

**Key deliverables:**
- Removed incorrect `enable_tool_scope()` / `disable_tool_scope()` from AuthService
- App-level config now local DB only (no auth server sync)
- Token exchange preserves `scope_tool-*` scopes
- New headers: `X-BodhiApp-Tool-Scopes`, `X-BodhiApp-Azp`
- `/apps/request-access` returns tools array and caches response
- Four-tier auth for OAuth: app-level → app-client → scope → user

**Files created:**
- `crates/services/migrations/0008_app_client_tool_configs.up.sql`
- `crates/services/migrations/0008_app_client_tool_configs.down.sql`

**Files modified:**
- `crates/services/src/auth_service.rs` - Removed tool scope methods, updated response types
- `crates/services/src/tool_service.rs` - Added `is_app_client_registered_for_tool`, removed auth server calls
- `crates/services/src/db/service.rs` - Added CRUD for app_client_tool_configs
- `crates/auth_middleware/src/token_service.rs` - Preserved scope_tool-* in exchange
- `crates/auth_middleware/src/auth_middleware.rs` - Added header injection
- `crates/auth_middleware/src/tool_auth_middleware.rs` - Full 4-tier auth logic
- `crates/routes_app/src/routes_login.rs` - Updated /apps/request-access with caching

**Authorization Flow:**

| Auth Type | Check 1 | Check 2 | Check 3 | Check 4 |
|-----------|---------|---------|---------|---------|
| Session/First-party | app_tool_configs | - | - | user_tool_configs |
| External OAuth | app_tool_configs | app_client_tool_configs | scope_tool-* | user_tool_configs |

**Tests:** 13 passing (3 for request-access, 10 for tool_auth_middleware)

---

## Phase 8: Frontend UI ✅ COMPLETE

**Completed: January 2026**

See [phase-8-implementation.md](./phase-8-implementation.md) for detailed implementation notes.

**Key deliverables:**
- Tools list page at `/ui/tools` with DataTable display
- Tool configuration page at `/ui/tools/edit?toolid=xxx`
- Setup flow integration as Step 5 (7-step flow)
- Separate form components: `ToolConfigForm` (config page) and `SetupToolsForm` (setup page)
- Admin controls for app-level enable/disable with confirmation dialogs
- Full test coverage (unit tests + MSW handlers + E2E tests)

**Files created:**
- `crates/bodhi/src/hooks/useTools.ts`
- `crates/bodhi/src/app/ui/tools/ToolConfigForm.tsx`
- `crates/bodhi/src/app/ui/tools/page.tsx`
- `crates/bodhi/src/app/ui/tools/edit/page.tsx`
- `crates/bodhi/src/app/ui/setup/tools/SetupToolsForm.tsx`
- `crates/bodhi/src/app/ui/setup/tools/page.tsx`
- `crates/bodhi/src/test-utils/msw-v2/handlers/tools.ts`
- E2E page objects (`SetupToolsPage.mjs`, `ToolsPage.mjs`)
- E2E specs (`setup-tools.spec.mjs`, `tools-config.spec.mjs`)

**Files modified:**
- Setup constants (7 steps), SetupProvider, navigation, api-models page routing
- All setup flow E2E tests updated for new step 5

**Tests:** 42+ passing (unit tests) + E2E tests

---

## Phase 8.1: Chat UI - Web Search Integration ⏳ PENDING

**Goal**: Integrate web search tool with `/ui/chat` for agentic search capabilities.

**Spec**: See [07.1-ui-chat-integration.md](./07.1-ui-chat-integration.md)

**Prerequisites**: Phase 8 (tool configuration) must be complete.

**Key Features:**
- Web search toggle in chat input area
- Toggle disabled with tooltip when tool unavailable
- Tool call display (collapsible JSON request/response)
- Agentic loop: detect tool calls → execute → send results back to LLM

**Files to create:**
- `crates/bodhi/src/app/ui/chat/WebSearchToggle.tsx`
- `crates/bodhi/src/app/ui/chat/ToolCallMessage.tsx`
- `crates/bodhi/src/hooks/use-tool-status.ts`

**Files to modify:**
- `crates/bodhi/src/hooks/use-chat-settings.tsx` - Add webSearch_enabled
- `crates/bodhi/src/hooks/use-chat.tsx` - Add tool call handling loop
- `crates/bodhi/src/hooks/use-chat-completions.ts` - Handle tool_calls in response
- `crates/bodhi/src/app/ui/chat/ChatUI.tsx` - Add toggle
- `crates/bodhi/src/app/ui/chat/ChatMessage.tsx` - Add tool call display
- `crates/bodhi/src/types/chat.ts` - Extend Message type for tools

**E2E Tests:**
- `crates/lib_bodhiserver_napi/tests-js/specs/tools/chat-with-web-search.spec.mjs`

---

## Phase 9: Integration Tests ⏳ PENDING

**Goal**: Additional end-to-end testing of tool features.

**Test Cases:**
1. Backend integration tests with test database
2. Additional frontend E2E tests with Playwright
3. Real Exa API testing (optional, requires API key)
4. Performance testing for tool execution
5. Chat with web search agentic loop tests

---

## Implementation Statistics

**Lines of code:**
- New files: ~3,000 lines across frontend and backend
- Modified files: ~1,000 lines across existing files
- Total: ~4,000 lines changed/added

**Test coverage:**
- 100+ passing tests across all layers
- Backend: 71+ tests
- Frontend: 42+ unit tests
- E2E: Setup flow and tools config tests

**Files created:**
1. `crates/objs/src/tools.rs` (includes `AppToolConfig`)
2. `crates/services/migrations/0007_tools_config.up.sql` (includes `app_tool_configs` table with seed)
3. `crates/services/migrations/0007_tools_config.down.sql`
4. `crates/services/migrations/0008_app_client_tool_configs.up.sql`
5. `crates/services/migrations/0008_app_client_tool_configs.down.sql`
6. `crates/services/src/exa_service.rs`
7. `crates/services/src/tool_service.rs` (expanded with app-level methods)
8. `crates/routes_app/src/tools_dto.rs` (expanded with app config DTOs)
9. `crates/routes_app/src/routes_tools.rs` (expanded with admin routes)
10. `crates/auth_middleware/src/tool_auth_middleware.rs`
11. `crates/bodhi/src/hooks/useTools.ts`
12. `crates/bodhi/src/app/ui/tools/ToolConfigForm.tsx`
13. `crates/bodhi/src/app/ui/tools/page.tsx`
14. `crates/bodhi/src/app/ui/tools/edit/page.tsx`
15. `crates/bodhi/src/app/ui/setup/tools/SetupToolsForm.tsx`
16. `crates/bodhi/src/app/ui/setup/tools/page.tsx`
17. `crates/bodhi/src/test-utils/msw-v2/handlers/tools.ts`
18. E2E page objects and specs

**Files modified:**
1. `crates/objs/src/lib.rs`
2. `crates/services/src/lib.rs`
3. `crates/services/src/db/objs.rs` (added `AppToolConfigRow`)
4. `crates/services/src/db/service.rs` (added app_tool_configs CRUD)
5. `crates/services/src/db/mod.rs` (made encryption public)
6. `crates/services/src/app_service.rs`
7. `crates/services/src/test_utils/app.rs`
8. `crates/services/src/test_utils/db.rs` (added app config helpers)
9. `crates/lib_bodhiserver/src/app_service_builder.rs`
10. `crates/routes_app/src/lib.rs`
11. `crates/auth_middleware/src/lib.rs`
12. `crates/auth_middleware/src/token_service.rs`
13. `crates/auth_middleware/src/auth_middleware.rs`
14. Setup flow frontend files (constants, provider, navigation, api-models)
15. E2E setup flow tests

---

## Future Enhancements

1. Additional builtin tools (web scraping, image search, etc.)
2. Dynamic tool registration
3. Custom user-defined tools
4. Tool usage analytics
5. Tool result caching
6. Token-level tool access restrictions (see [10-pending-items.md](./10-pending-items.md))
