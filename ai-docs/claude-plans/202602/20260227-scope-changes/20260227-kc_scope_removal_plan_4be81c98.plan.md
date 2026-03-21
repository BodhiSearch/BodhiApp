---
name: KC Scope Removal Plan
overview: "Adapt BodhiApp to keycloak-bodhi-ext scope changes: remove scope_resource_*, scope_user_*, scope_token_* from OAuth flows, remove auto-approve, add role-based access requests, and simplify auth types across all crates in layered fashion."
todos:
  - id: layer1-objs
    content: "Layer 1: Update objs crate -- UserScope (remove Admin/Manager), TokenScope (remove Admin/Manager + offline_access), update AppRole mappings"
    status: completed
  - id: layer2-services
    content: "Layer 2: Update services crate -- DB schema (remove resource_scope, add requested_role/approved_role), AuthService (remove register_resource_access, update response), AccessRequestService (remove auto-approve, add role params), AppInstanceService (remove scope)"
    status: completed
  - id: layer3-auth-middleware
    content: "Layer 3: Update auth_middleware -- remove ResourceScope, rename ExternalApp.scope to role, update token_service (role from DB not JWT), update all middleware pattern matches, update test client + integration tests"
    status: completed
  - id: layer4-routes-app
    content: "Layer 4: Update routes_app -- request/response types (add role fields, remove resource_scope), handlers, API token privilege matrix, OpenAPI regen, TypeScript client regen"
    status: completed
  - id: layer5-server-app
    content: "Layer 5: Update server_app -- integration tests for external token exchange with new scope patterns"
    status: completed
  - id: layer6-frontend
    content: "Layer 6: Update frontend -- access request pages, token form, hooks, MSW handlers, component tests"
    status: completed
  - id: layer7-e2e
    content: "Layer 7: Update E2E tests -- OAuth2 fixtures, auth server client, all specs referencing scope patterns"
    status: completed
  - id: layer8-docs
    content: "Layer 8: Update documentation -- CLAUDE.md/PACKAGE.md for modified crates, authentication architecture docs"
    status: completed
isProject: false
---

# KC Scope Removal and Role-Based Access Request Migration

## Context

keycloak-bodhi-ext has removed three categories of scopes:

- `scope_resource-*` per-resource scopes (performance: O(N) -> O(1) scope loading)
- `scope_user_*` / `scope_token_*` permission tier scopes (redundant with `resource_access` JWT roles)

BodhiApp must adapt: remove auto-approve flow, switch external app permissions from JWT scope parsing to DB-based access request roles, simplify auth types.

No backwards compatibility required. No production deployment exists.

## Key Architectural Decisions

- **External app role source**: From `approved_role` on access request DB record (not JWT scope)
- **UserScope**: Repurpose -- remove Admin/Manager variants, keep `scope_user_user`/`scope_user_power_user` strings
- **TokenScope**: Remove Admin/Manager variants, remove `offline_access` requirement
- **ResourceScope**: Remove entirely
- **AppInstance.scope**: Remove field
- **DB resource_scope column**: Remove from original migration (no data migration needed)
- **Auto-approve**: Remove entirely -- all access requests go through user review
- **Access request role**: Two columns `requested_role` (NOT NULL) + `approved_role` (nullable)
- **AuthContext::ExternalApp**: Rename `scope` field to `role: UserScope`

## Implementation Process

Each layer is implemented by a specialized sub-agent following this workflow:

### Sub-Agent Inputs

1. **This plan** — full architectural context and layer-specific changes
2. **Crate CLAUDE.md + PACKAGE.md** — crate conventions, patterns, file index
3. **Previous layer summary** — changes made, decisions, deviations, downstream impact notes

### Sub-Agent Execution

1. **Load context**: Read this plan (own layer section), crate CLAUDE.md/PACKAGE.md, previous layer summary
2. **Research**: Read all referenced source files, search for usages of types/methods being changed, verify assumptions against actual code
3. **Implement**: Make code changes — update types, methods, callers. Update/add tests. Follow existing crate patterns.
4. **Gate checks** (all must pass before layer is complete):
  - `cargo check -p {crate}` — compilation
  - `cargo test --no-fail-fast -p {crate}` — crate tests pass
  - `cargo test --no-fail-fast -p {all_upstream_crates} -p {crate}` — cumulative upstream+current tests pass
  - `cargo fmt --all` — formatting clean
5. **Local commit**: Commit with message `feat(kc-scope-removal): layer N — {brief description}`
6. **Write summary**: Document changes made, decisions/deviations, downstream impact, test changes — this becomes input for next layer's sub-agent

### Layer-Specific Gate Commands


| Layer | Crate Test                      | Cumulative Test                                                   |
| ----- | ------------------------------- | ----------------------------------------------------------------- |
| 1     | `cargo test -p objs`            | (same)                                                            |
| 2     | `cargo test -p services`        | `cargo test -p objs -p services`                                  |
| 3     | `cargo test -p auth_middleware` | `cargo test -p objs -p services -p auth_middleware`               |
| 4     | `cargo test -p routes_app`      | `cargo test -p objs -p services -p auth_middleware -p routes_app` |
| 5     | `cargo test -p server_app`      | `make test.backend`                                               |
| 6     | `cd crates/bodhi && npm test`   | `cd crates/bodhi && npm test && npm run format`                   |
| 7     | (per-spec first)                | `make build.ui-rebuild && make test.napi`                         |
| 8     | —                               | —                                                                 |


### Notes

- Layer plans are **seeds** — expect to discover details during implementation. Update approach as needed.
- Always verify "dead code" claims by searching ALL source files, not just test files.
- When renaming enums, auto-generated error codes change — search tests for old codes.
- Follow existing patterns in the crate — don't invent new conventions.

---

## Layer 1: `objs` Crate

### UserScope changes (`[crates/objs/src/user_scope.rs](crates/objs/src/user_scope.rs)`)

- Remove `Manager` and `Admin` variants from `UserScope` enum
- Keep `User` (`scope_user_user`) and `PowerUser` (`scope_user_power_user`)
- Remove `from_scope()` method (parsed from space-separated scope string -- no longer needed)
- Keep `FromStr`, `has_access_to()`, `included_scopes()`, `scope_user()` methods
- Update all tests

### TokenScope changes (`[crates/objs/src/token_scope.rs](crates/objs/src/token_scope.rs)`)

- Remove `Manager` and `Admin` variants from `TokenScope` enum
- Remove `MissingOfflineAccess` error variant from `TokenScopeError`
- Remove `offline_access` check from `TokenScope::from_scope()`
- Keep `User` and `PowerUser` variants with `scope_token_user`/`scope_token_power_user` strings
- Update all tests

### Downstream impact in objs

- Update `AppRole` mapping if it references removed variants
- Update any `ResourceRole` <-> `UserScope` / `TokenScope` conversion code
- Verify `resource_role.rs` is unaffected (ResourceRole keeps all 4 variants)

### Validation

```bash
cargo test -p objs
cargo fmt
```

---

## Layer 2: `services` Crate

### DB Schema: app_access_requests (`[crates/services/migrations/0009_app_access_requests.up.sql](crates/services/migrations/0009_app_access_requests.up.sql)`)

- Remove `resource_scope` column from CREATE TABLE
- Add `requested_role TEXT NOT NULL` column
- Add `approved_role TEXT` column (nullable -- set on approval)

### AppAccessRequestRow (`[crates/services/src/db/objs.rs](crates/services/src/db/objs.rs)`)

- Remove `resource_scope: Option<String>` field
- Add `requested_role: String` field
- Add `approved_role: Option<String>` field

### DB Repository: update queries

- `[crates/services/src/db/access_request_repository.rs](crates/services/src/db/access_request_repository.rs)`: Update INSERT/SELECT/UPDATE queries
- `[crates/services/src/db/service_access_request.rs](crates/services/src/db/service_access_request.rs)`: Update `update_approval()` signature -- remove `resource_scope` param, add `approved_role` param

### AuthService (`[crates/services/src/auth_service/service.rs](crates/services/src/auth_service/service.rs)`)

- Remove `register_resource_access()` method and `RegisterResourceAccessResponse` type
- Update `RegisterAccessRequestConsentResponse`: remove `scope` field (KC no longer returns it)
- Update `register_access_request_consent()` response parsing
- Remove `get_client_access_token()` if only used by `register_resource_access`
- Update MockAuthService accordingly

### AccessRequestService (`[crates/services/src/access_request_service/service.rs](crates/services/src/access_request_service/service.rs)`)

- Add `requested_role: String` parameter to `create_draft()`
- Remove auto-approve branch entirely (the `is_auto_approve` check and register_resource_access call)
- All requests create draft status
- Update `approve_request()` to accept `approved_role` parameter, pass to DB update
- Update trait and mock

### AppInstanceService

- Remove `scope` parameter from `create_instance()` and `AppInstance` / `AppInstanceRow` structs
- Update DB schema for apps table (remove scope column from original migration)
- Update all callers

### Test updates

- Update `[crates/services/src/db/test_access_request_repository.rs](crates/services/src/db/test_access_request_repository.rs)`
- Update `[crates/services/src/test_utils/db.rs](crates/services/src/test_utils/db.rs)` (MockDbService, TestDbService)
- Update auth service tests

### Validation

```bash
cargo test -p services
cargo test -p objs -p services
cargo fmt
```

---

## Layer 3: `auth_middleware` Crate

### Remove ResourceScope (`[crates/auth_middleware/src/resource_scope.rs](crates/auth_middleware/src/resource_scope.rs)`)

- Delete file entirely
- Remove `mod resource_scope` and re-export from `[crates/auth_middleware/src/lib.rs](crates/auth_middleware/src/lib.rs)`

### AuthContext changes (`[crates/auth_middleware/src/auth_context.rs](crates/auth_middleware/src/auth_context.rs)`)

- Rename `ExternalApp { scope: UserScope }` to `ExternalApp { role: UserScope }`
- Update `app_role()` method to use `role` field
- Update test factory `test_external_app()` parameter name

### Token Service (`[crates/auth_middleware/src/token_service/service.rs](crates/auth_middleware/src/token_service/service.rs)`)

- `handle_external_client_token()` return type: change from `(String, ResourceScope, String)` to `(String, Option<UserScope>, String, Option<String>)` -- (token, role, azp, access_request_id)
- Remove `scope_user_*` filtering from exchange scope construction (line ~209-212). Only keep `scope_access_request:*`
- Derive role from validated access request record's `approved_role` instead of JWT scope parsing
- Update `validate_bearer_token()` return type and callers
- Update caching logic for exchange results

### Auth Middleware (`[crates/auth_middleware/src/auth_middleware/middleware.rs](crates/auth_middleware/src/auth_middleware/middleware.rs)`)

- Update ExternalApp construction to use `role` field from token service result

### API Auth Middleware (`[crates/auth_middleware/src/api_auth_middleware.rs](crates/auth_middleware/src/api_auth_middleware.rs)`)

- Update `ExternalApp` pattern match: `scope` -> `role`

### Access Request Auth Middleware

- Update pattern matches on ExternalApp

### Toolset Auth Middleware

- Update pattern matches on ExternalApp

### Test Utils (`[crates/auth_middleware/src/test_utils/auth_server_test_client.rs](crates/auth_middleware/src/test_utils/auth_server_test_client.rs)`)

- Remove `DynamicClients.resource_scope_name` field
- Remove `request_audience_access()` method (KC endpoint deleted)
- Update `setup_dynamic_clients()`: remove step 6 (audience access)
- Remove `scope` from `ClientInfo` (KC no longer returns it for resource clients)
- Update `ClientCreateResponse` to not expect `scope`

### Integration Tests (`[crates/auth_middleware/tests/test_live_auth_middleware.rs](crates/auth_middleware/tests/test_live_auth_middleware.rs)`)

- Update `test_cross_client_token_exchange_success`: remove `scope_user_user` and `resource_scope_name` from OAuth scopes. Use `scope_access_request:uuid` pattern. Need to create access request via KC endpoint first.
- Update scope assertions on exchanged JWT
- Update `create_test_state`: remove scope from `create_instance()` call

### Token Service Tests (`[crates/auth_middleware/src/token_service/tests.rs](crates/auth_middleware/src/token_service/tests.rs)`)

- Update all test token claims to remove `scope_user_*`
- Update mock exchange expectations (no `scope_user_*` in scopes list)
- Update `resource_scope` references in mock AppAccessRequestRow to use `requested_role`/`approved_role`
- Update assertions on ExternalApp auth context (`role` instead of `scope`)

### Validation

```bash
cargo test -p auth_middleware
cargo test -p objs -p services -p auth_middleware
cargo fmt
```

---

## Layer 4: `routes_app` Crate

### Request/Response Types (`[crates/routes_app/src/routes_apps/types.rs](crates/routes_app/src/routes_apps/types.rs)`)

- `CreateAccessRequestBody`: add `requested_role: String` (mandatory)
- `CreateAccessRequestResponse`: remove `Approved` variant entirely (no auto-approve). Only `Draft` with `{id, review_url}`
- `AccessRequestStatusResponse`: remove `resource_scope`, add `approved_role: Option<String>`
- `ApproveAccessRequestBody`: add `approved_role: String` top-level field
- `AccessRequestReviewResponse`: add `requested_role: String`

### Handlers (`[crates/routes_app/src/routes_apps/handlers.rs](crates/routes_app/src/routes_apps/handlers.rs)`)

- `create_access_request_handler`: pass `requested_role`, remove approved/auto-approve branch
- `get_access_request_status_handler`: return `approved_role` instead of `resource_scope`
- `get_access_request_review_handler`: include `requested_role` in response
- `approve_access_request_handler`: extract `approved_role` from body, pass to service

### API Token Routes (`[crates/routes_app/src/routes_api_token/route_api_token.rs](crates/routes_app/src/routes_api_token/route_api_token.rs)`)

- Update privilege escalation matrix for TokenScope with only User/PowerUser
- Update any `scope_token_manager`/`scope_token_admin` references

### Auth Routes

- Update any `ExternalApp { scope, .. }` pattern matches to `ExternalApp { role, .. }`

### OpenAPI

- Update `openapi.rs` schemas for changed types
- Regenerate: `cargo run --package xtask openapi`

### TypeScript Client

- Regenerate: `make build.ts-client`

### Test Updates

- `[crates/routes_app/src/routes_apps/test_access_request.rs](crates/routes_app/src/routes_apps/test_access_request.rs)`: update for new request/response shapes
- Update all files matching `ExternalApp { scope` pattern across routes_app

### Validation

```bash
cargo test -p routes_app
cargo test -p objs -p services -p auth_middleware -p routes_app
cargo fmt
cargo run --package xtask openapi
make build.ts-client
```

---

## Layer 5: `server_app` Crate

### Integration Tests

- `[crates/server_app/tests/test_oauth_external_token.rs](crates/server_app/tests/test_oauth_external_token.rs)`: update scope strings, ExternalApp assertions
- `[crates/server_app/tests/utils/external_token.rs](crates/server_app/tests/utils/external_token.rs)`: update token construction

### Validation

```bash
cargo test -p server_app
make test.backend
cargo fmt
```

---

## Layer 6: Frontend (`crates/bodhi/src/`)

### Components/Hooks

- Update access request hooks/types for new API shape (requested_role, approved_role, no resource_scope)
- Update token form/page for TokenScope changes (only User/PowerUser)
- Update any scope-related UI text

### Tests

- Update MSW handlers referencing scope patterns
- Update component tests

### Validation

```bash
cd crates/bodhi && npm run test && npm run format
```

---

## Layer 7: E2E Tests (`crates/lib_bodhiserver_napi/tests-js/`)

### Test Updates

- Update OAuth2 fixtures (`oauth2Fixtures.mjs`)
- Update auth server client (`auth-server-client.mjs`)
- Update specs referencing scope patterns:
  - `oauth2-token-exchange.spec.mjs`
  - `oauth-chat-streaming.spec.mjs`
  - `api-tokens.spec.mjs`
  - `toolsets-auth-restrictions.spec.mjs`
  - `mcps-auth-restrictions.spec.mjs`

### Validation

```bash
make build.ui-rebuild
make test.napi
```

---

## Layer 8: Documentation

### Update ai-docs and CLAUDE.md files

- Update crate-level CLAUDE.md/PACKAGE.md for each modified crate
- Update `ai-docs/01-architecture/authentication.md`

## File Impact Summary

- ~50+ files affected across 7 crates + frontend + E2E tests
- Core type changes: `UserScope`, `TokenScope`, `ResourceScope` (removed), `AuthContext::ExternalApp`
- DB schema change: `app_access_requests` table (modify original migration)
- API contract changes: access request create/status/approve endpoints

