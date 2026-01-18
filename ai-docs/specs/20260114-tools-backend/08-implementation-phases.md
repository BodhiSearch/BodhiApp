# Implementation Phases

> Status: Phases 1-8.1 Complete | Phase 9 Pending | Updated: 2026-01-18

## Phase Completion Summary

| Phase | Status | Description |
|-------|--------|-------------|
| 1. Domain Objects | ✅ Complete | ToolsetScope, ToolsetDefinition, UserToolsetConfig |
| 2. Database | ✅ Complete | Migration 0009, toolset tables, CRUD with encryption |
| 3. Exa Service | ✅ Complete | 4 methods: search, find_similar, get_contents, answer |
| 4. Toolset Service | ✅ Complete | ToolsetService trait, execution logic |
| 5. AppService Integration | ✅ Complete | ToolsetService integration |
| 6. API Routes | ✅ Complete | `/toolsets` endpoints with `/execute/{method}` |
| 7. Auth Middleware | ✅ Complete | toolset_auth_middleware with 4-tier OAuth |
| 7.5. App-Level Toolset Config | ✅ Complete | Admin enable/disable |
| 7.6. External App Toolset Access | ✅ Complete | OAuth scope-based auth |
| 8. Frontend UI | ✅ Complete | `/ui/toolsets` pages, setup step 5 |
| 8.1. Chat UI Integration | ✅ Complete | Toolsets popover with per-tool selection, agentic loop |
| 9. Integration Tests | ⏳ Pending | Additional E2E tests with real Exa API |

---

## Phase 1: Domain Objects ✅ COMPLETE

**Files created:**
- `crates/objs/src/toolsets.rs` - Consolidated all toolset types

**Types implemented:**
- `ToolsetScope` enum with `BuiltinExaWebSearch`
- `ToolsetDefinition` containing `Vec<ToolDefinition>`
- `ToolDefinition` and `FunctionDefinition` (OpenAI format)
- `UserToolsetConfig` (public API model)
- `ToolsetExecutionRequest` and `ToolsetExecutionResponse`

---

## Phase 2: Database ✅ COMPLETE

**Files created:**
- `crates/services/migrations/0009_toolsets_schema.up.sql`
- `crates/services/migrations/0009_toolsets_schema.down.sql`

**Tables:**
- `user_toolset_configs` - Per-user toolset config with encrypted API keys
- `app_toolset_configs` - Admin-controlled app-level config
- `app_client_toolset_configs` - Cached external app configurations

**Implementation details:**
- Uses existing encryption: `encrypt_api_key()` / `decrypt_api_key()`
- Unique constraint on (user_id, toolset_id)
- API key stored at toolset level (one key for all tools)

---

## Phase 3: Exa Service ✅ COMPLETE

**Files created:**
- `crates/services/src/exa_service.rs`

**Implementation:**
```rust
pub trait ExaService: Debug + Send + Sync {
    async fn search(&self, api_key: &str, query: &str, num_results: Option<u32>) 
        -> Result<ExaSearchResponse, ExaError>;
    async fn find_similar(&self, api_key: &str, url: &str, num_results: Option<u32>, exclude_source_domain: Option<bool>) 
        -> Result<ExaSearchResponse, ExaError>;
    async fn get_contents(&self, api_key: &str, urls: Vec<String>, max_characters: Option<u32>) 
        -> Result<ExaContentsResponse, ExaError>;
    async fn answer(&self, api_key: &str, query: &str, num_results: Option<u32>) 
        -> Result<ExaAnswerResponse, ExaError>;
}
```

---

## Phase 4: Toolset Service ✅ COMPLETE

**Files created:**
- `crates/services/src/toolset_service.rs`

**Implementation:**
```rust
pub trait ToolsetService: Debug + Send + Sync {
    fn list_all_toolset_definitions(&self) -> Vec<ToolsetDefinition>;
    async fn get_user_toolset_config(&self, user_id: &str, toolset_id: &str) 
        -> Result<Option<UserToolsetConfig>, ToolsetError>;
    async fn update_user_toolset_config(&self, ...) -> Result<UserToolsetConfig, ToolsetError>;
    async fn execute_toolset_tool(&self, user_id: &str, toolset_id: &str, request: ToolsetExecutionRequest) 
        -> Result<ToolsetExecutionResponse, ToolsetError>;
    async fn is_toolset_available_for_user(&self, user_id: &str, toolset_id: &str) 
        -> Result<bool, ToolsetError>;
    // ... app-level and app-client methods
}
```

**Builtin toolset:**
- `builtin-exa-web-search` with 4 tools (search, find_similar, get_contents, answer)
- Tool names follow Claude MCP format: `toolset__builtin-exa-web-search__{tool_name}`

---

## Phase 5: AppService Integration ✅ COMPLETE

**Files modified:**
- `crates/services/src/app_service.rs` - Add `toolset_service()` method
- `crates/lib_bodhiserver/src/app_service_builder.rs` - Create ToolsetService

---

## Phase 6: API Routes ✅ COMPLETE

**Files created:**
- `crates/routes_app/src/routes_toolsets.rs`
- `crates/routes_app/src/toolsets_dto.rs`

**Endpoints:**
- `GET /bodhi/v1/toolsets` - List toolsets with tools
- `GET /bodhi/v1/toolsets/:toolset_id/config` - Get user config
- `PUT /bodhi/v1/toolsets/:toolset_id/config` - Update config
- `DELETE /bodhi/v1/toolsets/:toolset_id/config` - Delete config
- `POST /bodhi/v1/toolsets/:toolset_id/execute` - Execute tool (with `tool_name` in body)
- `PUT /bodhi/v1/toolsets/:toolset_id/app-config` - Admin enable
- `DELETE /bodhi/v1/toolsets/:toolset_id/app-config` - Admin disable

---

## Phase 7: Auth Middleware ✅ COMPLETE

**Files created:**
- `crates/auth_middleware/src/toolset_auth_middleware.rs`

**Authorization logic:**
1. Check app-level enabled (all auth types)
2. For OAuth: Check app-client registered
3. For OAuth: Check `scope_toolset-*` in token
4. Check user has toolset configured (API key)

---

## Phase 7.5: App-Level Toolset Config ✅ COMPLETE

**Features:**
- Two-tier auth: app_enabled AND user_enabled AND api_key
- Admin-only app-config endpoints
- Default enabled via migration seed

---

## Phase 7.6: External App Toolset Access ✅ COMPLETE

**Features:**
- Token exchange preserves `scope_toolset-*`
- Headers: `X-BodhiApp-Toolset-Scopes`, `X-BodhiApp-Azp`
- App-client config caching from Keycloak
- Four-tier authorization for OAuth

---

## Phase 8: Frontend UI ✅ COMPLETE

**Files created:**
- `crates/bodhi/src/hooks/useToolsets.ts`
- `crates/bodhi/src/app/ui/toolsets/page.tsx`
- `crates/bodhi/src/app/ui/toolsets/edit/page.tsx`
- `crates/bodhi/src/app/ui/toolsets/ToolsetConfigForm.tsx`
- `crates/bodhi/src/app/ui/setup/toolsets/page.tsx`
- `crates/bodhi/src/app/ui/setup/toolsets/SetupToolsetsForm.tsx`
- `crates/bodhi/src/test-utils/msw-v2/handlers/toolsets.ts`

**Features:**
- Toolsets list with DataTable
- Configuration page with admin controls
- Setup step 5 integration
- Shows tools within each toolset

---

## Phase 8.1: Chat UI - Toolsets Integration ✅ COMPLETE

**Goal**: Integrate toolsets with `/ui/chat` via toolsets dropdown with individual tool selection.

**Spec**: See [07.1-ui-chat-integration.md](./07.1-ui-chat-integration.md)

**Files created:**
- `crates/bodhi/src/app/ui/chat/ToolsetsPopover.tsx` - Expandable popover with nested checkboxes
- `crates/bodhi/src/app/ui/chat/ToolCallMessage.tsx` - Collapsible tool call display
- `crates/bodhi/src/hooks/use-toolset-selection.ts` - Per-chat tool selection management

**Key Features:**
- Toolsets popover with per-tool selection (tri-state parent checkbox)
- Tool selection stored as `Record<string, string[]>` (toolset_id → tool names)
- Badge shows total enabled tool count across all toolsets
- Agentic loop with parallel tool execution via `Promise.allSettled`
- Max iterations setting (default: 5) with warning injection
- Tool call display with status badges and JSON preview
- LocalStorage inheritance for new chats
- AbortController support for cancellation

**E2E Tests:**
- `chat-toolsets.spec.mjs` - UI tests for popover and settings
- `chat-agentic.spec.mjs` - Full agentic flow with Exa (requires INTEG_TEST_EXA_API_KEY)

---

## Phase 9: Integration Tests ⏳ PENDING

**Test Cases:**
1. ✅ Backend integration test: `test_live_agentic_chat_with_exa.rs` (requires INTEG_TEST_EXA_API_KEY)
2. ✅ Frontend unit tests: MSW handlers for toolsets and tool execution
3. ✅ E2E tests: `chat-toolsets.spec.mjs`, `chat-agentic.spec.mjs`
4. ⏳ Additional coverage for error scenarios and edge cases

---

## Key Files Summary

| Layer | Files |
|-------|-------|
| Domain | `crates/objs/src/toolsets.rs` |
| Database | `crates/services/migrations/0009_toolsets_schema.{up,down}.sql` |
| Service | `crates/services/src/tool_service.rs`, `exa_service.rs` |
| Routes | `crates/routes_app/src/routes_toolsets.rs`, `toolsets_dto.rs` |
| Auth | `crates/auth_middleware/src/toolset_auth_middleware.rs` |
| Frontend Hooks | `crates/bodhi/src/hooks/useToolsets.ts`, `use-toolset-selection.ts` |
| Frontend Pages | `crates/bodhi/src/app/ui/toolsets/` |
| Frontend Setup | `crates/bodhi/src/app/ui/setup/toolsets/` |
| Chat Components | `crates/bodhi/src/app/ui/chat/ToolsetsPopover.tsx`, `ToolCallMessage.tsx` |
| Chat Hooks | `crates/bodhi/src/hooks/use-chat.tsx`, `use-chat-completions.ts` |
| MSW | `crates/bodhi/src/test-utils/msw-v2/handlers/toolsets.ts` |
| E2E Tests | `crates/lib_bodhiserver_napi/tests-js/specs/chat/chat-*.spec.mjs` |
| Integration Tests | `crates/integration-tests/tests/test_live_agentic_chat_with_exa.rs` |
