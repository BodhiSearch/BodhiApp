# Review Plan: HEAD commit (f354b09a0) — CRUD Layer Refactor

## Context

The HEAD commit is a large CRUD layer refactor that:
1. Re-introduces `ValidatedJson` extractor replacing `WithRejection<Json<...>>`
2. Renames `*Form` → `*Request` types across all domains
3. Moves Request/Response types from `routes_app` api_schemas → `services` *_objs.rs
4. Removes `form.validate()` from service layer (validation now in routes via `ValidatedJson`)
5. Updates CLAUDE.md docs and TECHDEBT.md

**85 files changed** across services, routes_app, UI, ts-client, and docs.

## Review Parameters (from user interview)

- **Scope**: HEAD commit only, independent review (no cross-ref with prior reviews)
- **Validation bypass check**: Skip — services are only called through route handlers
- **utoipa in services**: Already decided, skip evaluation
- **Type split consistency**: Evaluate which types stayed in routes_app vs moved to services
- **From<Entity> impls**: Check consistency across ALL domains (not just new ones)
- **ValidatedJson**: Review implementation quality AND handler usage
- **Fix detail**: Descriptive only, no code snippets
- **Output**: `ai-docs/claude-plans/20260303-multi-tenant/reviews/`
- **Format**: Issues-only (no positives/compliance), optimized for claude code fixing

## Agent Flow — Sequential → Parallel (Domain-Wise)

### Architecture

```
Sequential Phase (insights cascade forward):
  Agent 1: Models + ApiModels domain (services + routes)
       ↓ summary
  Agent 2: MCPs + Toolsets domain (services + routes)
       ↓ consolidated summary (Agent 1 + Agent 2)

Parallel Phase (all receive consolidated summary):
  Agent 3: Tokens domain (services + routes)
  Agent 4: Users + Apps + Access Requests domain (services + routes)
  Agent 5: Settings + Shared + ValidatedJson impl (services + routes)
  Agent 6: UI + ts-client + cross-cutting (From<Entity> consistency, type split eval)
```

### Sequential Phase

#### Agent 1: Models + ApiModels Domain (FIRST)
**Services files**: `services/src/models/{model_objs.rs, data_service.rs, api_model_service.rs, download_service.rs, multi_tenant_data_service.rs}`, `services/src/app_service/{auth_scoped_data.rs, auth_scoped_api_models.rs, auth_scoped_downloads.rs}`
**Routes files**: `routes_app/src/models/{models_api_schemas.rs, routes_models.rs, routes_models_metadata.rs, routes_models_pull.rs, test_aliases_crud.rs, test_metadata.rs}`, `routes_app/src/api_models/{api_models_api_schemas.rs, routes_api_models.rs, mod.rs, test_api_models_*.rs, test_types.rs}`
**Output**: `models-apimodels-review.md`
**Focus**: Type moves (TestCreds, TestPromptRequest/Response, FetchModelsRequest/Response, ApiFormatsResponse, CopyAliasRequest, RefreshSource, response types), Form→Request renames (UserAliasForm→UserAliasRequest, ApiModelForm→ApiModelRequest), From<Entity> impl placement, ValidatedJson usage in handlers, removed types from api_schemas (verify nothing leftover)
**Deliverable**: Review report + concise summary of patterns/issues found for next agent

#### Agent 2: MCPs + Toolsets Domain (SECOND, receives Agent 1 summary)
**Services files**: `services/src/mcps/{mcp_objs.rs, mcp_service.rs, test_mcp_service.rs}`, `services/src/toolsets/{toolset_objs.rs, tool_service.rs, test_tool_service.rs}`, `services/src/app_service/{auth_scoped_mcps.rs, auth_scoped_tools.rs}`
**Routes files**: `routes_app/src/mcps/{routes_mcps.rs, routes_mcps_servers.rs, test_mcps.rs, test_oauth_flow.rs, test_servers.rs}`, `routes_app/src/toolsets/{routes_toolsets.rs, test_toolsets_crud.rs}`
**Output**: `mcps-toolsets-review.md`
**Focus**: Form→Request renames (McpForm→McpRequest, ToolsetForm→ToolsetRequest), Entity→Response conversion in handlers, service return types (Entity vs Response), test coverage updates
**Input**: Agent 1 summary (patterns, conventions, issues found)
**Deliverable**: Review report + consolidated summary (Agent 1 + Agent 2 patterns) for parallel agents

### Parallel Phase (all receive consolidated summary from Agents 1+2)

#### Agent 3: Tokens Domain
**Services files**: `services/src/tokens/{token_objs.rs, token_service.rs, test_token_service.rs}`, `services/src/app_service/auth_scoped_tokens.rs`
**Routes files**: `routes_app/src/tokens/{routes_tokens.rs, test_tokens_crud.rs, test_tokens_security.rs, tokens_api_schemas.rs}`
**Output**: `tokens-review.md`
**Focus**: Form→Request rename (TokenForm→TokenRequest), ValidatedJson usage, Entity→Response conversion, test updates

#### Agent 4: Users + Apps + Access Requests Domain
**Services files**: `services/src/users/user_objs.rs`, `services/src/app_access_requests/{access_request_objs.rs, access_request_service.rs}`
**Routes files**: `routes_app/src/users/{routes_users.rs, routes_users_access_request.rs, test_access_request_dto.rs, users_api_schemas.rs}`, `routes_app/src/apps/{apps_api_schemas.rs, routes_apps.rs}`
**Output**: `users-apps-review.md`
**Focus**: Type moves to user_objs.rs (ChangeRoleRequest, UserAccessStatusResponse, ApproveUserAccessRequest, UserAccessRequest, PaginatedUserAccessResponse), access request type renames (Body→Request), From<Entity> impls

#### Agent 5: Settings + Shared + ValidatedJson Implementation
**Services files**: `services/src/settings/{setting_objs.rs, constants.rs}`
**Routes files**: `routes_app/src/settings/{mod.rs, routes_settings.rs, settings_api_schemas.rs}`, `routes_app/src/shared/{mod.rs, openapi.rs, validated_json.rs, test_validated_json.rs}`, `routes_app/src/test_utils/alias_response.rs`
**Output**: `settings-shared-review.md`
**Focus**: ValidatedJson implementation quality (error format, rejection handling, OpenAI-compatible errors, edge cases), settings constants move, removed settings_api_schemas.rs cleanup, test coverage for ValidatedJson

#### Agent 6: UI + ts-client + Cross-Cutting Analysis
**Files**: `bodhi/src/hooks/{useApiModels.ts, useApiTokens.ts, useAppAccessRequests.ts, useMcps.ts, useModels.ts, useModels.test.ts, useToolsets.ts}`, `bodhi/src/schemas/{alias.ts, apiModel.ts}`, `bodhi/src/app/ui/apps/access-requests/review/page.tsx`, `openapi.json`, `ts-client/src/**`, `services/test_utils/{data.rs, sea.rs}`, all CLAUDE.md files, TECHDEBT.md
**Output**: `cross-cutting-review.md`
**Focus**:
- **From<Entity> placement consistency** across ALL domains (check every From<*Entity> for * impl in services vs routes_app)
- **Type split evaluation**: which types stayed in routes_app vs moved to services — do they follow a clear rule?
- **Type consistency**: Rust types ↔ OpenAPI spec ↔ ts-client ↔ frontend hooks
- **Frontend import updates**: verify renamed types properly updated
- **Doc consistency**: CLAUDE.md files match actual patterns

## Review Checklist (all agents)

### Convention Checks
- [ ] Form→Request rename complete (no leftover Form references in changed files)
- [ ] ValidatedJson used correctly (replaces WithRejection<Json<...>>)
- [ ] No redundant validate() calls after ValidatedJson introduction
- [ ] From<Entity> impl placement consistent across domains
- [ ] Serde annotations preserved correctly after type moves
- [ ] OpenAPI/utoipa annotations updated for moved/renamed types
- [ ] No `use super::*` in test modules

### Type Split Evaluation (Agent 6)
- [ ] Types that moved to services follow a clear rule
- [ ] Types that stayed in routes_app are genuinely presentation-only
- [ ] No types misplaced in either direction

### ValidatedJson Implementation (Agent 5)
- [ ] Error format is OpenAI-compatible
- [ ] Rejection handling covers all axum rejection types
- [ ] Edge cases (empty body, malformed JSON, validation failures)
- [ ] Test coverage adequate

## Fix Iteration Order

After review, fixes should follow layered methodology:
1. services issues → `cargo test -p services`
2. routes_app issues → `cargo test -p routes_app`
3. Full backend: `make test.backend`
4. Regenerate ts-client: `make build.ts-client`
5. Frontend issues → `cd crates/bodhi && npm test`
6. Documentation updates
