---
name: Refactor Access Request Types
overview: Refactor access request types to use structured deserialization, update toolset execute path to match MCP pattern, simplify test-oauth-app config to single JSON input, add comprehensive MCP restricted access e2e tests, and fix existing test failures.
todos:
  - id: phase1-objs-types
    content: "Phase 1: Move ApprovedResources + RequestedResources to objs with serde attrs, verify with cargo test -p objs"
    status: completed
  - id: phase2-middleware-structured
    content: "Phase 2: Replace raw serde_json in auth_middleware with objs::ApprovedResources, verify with cargo test -p auth_middleware"
    status: completed
  - id: phase3-routes-structured
    content: "Phase 3: Replace raw serde_json in routes_mcps/handlers, remove duplicates in routes_apps/types, verify with cargo test -p routes_app"
    status: completed
  - id: phase4-toolset-path
    content: "Phase 4: Change toolset execute path to /toolsets/{id}/tools/{tool_name}/execute, update handler+openapi, verify with cargo test -p routes_app"
    status: completed
  - id: phase5-openapi-regen
    content: "Phase 5: Regenerate OpenAPI spec + TypeScript client, update frontend refs, verify with cargo fmt + make build.ts-client"
    status: completed
  - id: phase6-frontend-tests
    content: "Phase 6: Run frontend unit tests (cd crates/bodhi && npm test), fix any breakage from type/path changes"
    status: completed
  - id: phase7-backend-full
    content: "Phase 7: Full backend validation - make test.backend, make test.ui.unit, cargo fmt"
    status: completed
  - id: phase8-config-form
    content: "Phase 8: Simplify test-oauth-app ConfigForm to single 'requested' JSON input"
    status: completed
  - id: phase9-migrate-e2e
    content: "Phase 9: Migrate ConfigSection + all e2e tests to new single input, fix .data -> .mcps in MCP tests"
    status: completed
  - id: phase10-ui-rebuild
    content: "Phase 10: make build.ui-rebuild to include all changes in NAPI build"
    status: completed
  - id: phase11-e2e-toolsets
    content: "Phase 11: Run toolsets e2e tests (playwright test tests-js/specs/toolsets/toolsets-auth-restrictions.spec.mjs), fix failures"
    status: completed
  - id: phase12-e2e-mcps-existing
    content: "Phase 12: Run existing MCP e2e tests (playwright test tests-js/specs/mcps/mcps-auth-restrictions.spec.mjs), fix failures"
    status: completed
  - id: phase13-restricted-mcp-test
    content: "Phase 13: Add new restricted-deepwiki MCP e2e test, run and verify passing"
    status: completed
  - id: phase14-e2e-full
    content: "Phase 14: Run make test.napi for full NAPI test suite, ensure all green"
    status: completed
isProject: false
---

# Refactor Access Request Types, Toolset Path, Config Form, and MCP E2E Tests

---

## Phase 1: Move types to objs crate

**Goal:** Add `ApprovedResources` and `RequestedResources` to the foundation crate so all upstream crates can use them.

**Add to [crates/objs/src/access_request.rs](crates/objs/src/access_request.rs):**

```rust
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Default)]
pub struct ApprovedResources {
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub toolsets: Vec<ToolsetApproval>,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub mcps: Vec<McpApproval>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Default)]
pub struct RequestedResources {
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub toolset_types: Vec<ToolsetTypeRequest>,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub mcp_servers: Vec<McpServerRequest>,
}
```

**Files:**

- [crates/objs/src/access_request.rs](crates/objs/src/access_request.rs) - Add both types
- [crates/objs/src/lib.rs](crates/objs/src/lib.rs) - Export `ApprovedResources`, `RequestedResources`

**Verify:** `cargo test -p objs`

---

## Phase 2: Replace raw JSON in auth_middleware

**Goal:** Replace `serde_json::Value` manual traversal with `objs::ApprovedResources` deserialization.

**File:** [crates/auth_middleware/src/access_request_auth_middleware.rs](crates/auth_middleware/src/access_request_auth_middleware.rs)

- `ToolsetAccessRequestValidator::validate_approved` (~lines 193-211): replace `serde_json::from_str::<serde_json::Value>` + `.get("toolsets")` chain with `serde_json::from_str::<ApprovedResources>` and iterate `.toolsets`
- `McpAccessRequestValidator::validate_approved` (~lines 242-260): same pattern, iterate `.mcps`

**Verify:** `cargo test -p auth_middleware`

---

## Phase 3: Replace raw JSON in routes, remove duplicates

**Goal:** Use `objs::ApprovedResources` / `objs::RequestedResources` everywhere in routes_app.

**Files:**

- [crates/routes_app/src/routes_mcps/mcps.rs](crates/routes_app/src/routes_mcps/mcps.rs) - Replace raw parsing in `extract_approved_mcp_ids()` (~lines 259-288) with `ApprovedResources` deserialization
- [crates/routes_app/src/routes_apps/handlers.rs](crates/routes_app/src/routes_apps/handlers.rs) - Change `RequestedResources` import from `crate::routes_apps` to `objs`
- [crates/routes_app/src/routes_apps/types.rs](crates/routes_app/src/routes_apps/types.rs) - Remove local `ApprovedResources` and `RequestedResources` structs, import from `objs` in all usages (e.g. `ApproveAccessRequestBody`, `AccessRequestReviewResponse`)

**Verify:** `cargo test -p routes_app`

---

## Phase 4: Update toolset execute endpoint path

**Current:** `/bodhi/v1/toolsets/{id}/execute/{method}`
**New:** `/bodhi/v1/toolsets/{id}/tools/{tool_name}/execute`

**Files:**

- [crates/routes_app/src/routes.rs](crates/routes_app/src/routes.rs) ~line 222 - Change route string to `{ENDPOINT_TOOLSETS}/{{id}}/tools/{{tool_name}}/execute`
- [crates/routes_app/src/routes_toolsets/toolsets.rs](crates/routes_app/src/routes_toolsets/toolsets.rs) - Change `Path((id, method))` to `Path((id, tool_name))`, rename all internal `method` references to `tool_name`
- [crates/routes_app/src/shared/openapi.rs](crates/routes_app/src/shared/openapi.rs) - Update `#[utoipa::path]` annotation path, change `operation_id` to `"executeToolsetTool"`

**Verify:** `cargo test -p routes_app`

---

## Phase 5: Regenerate OpenAPI + TypeScript client

**Goal:** Sync generated artifacts with backend changes.

**Commands:**

1. `cargo fmt --all`
2. `cargo run --package xtask openapi` - regenerates `openapi.json`
3. `make build.ts-client` - regenerates TS types in `ts-client/`

**Also check:** Any frontend hooks/components that reference the old toolset execute path (`/execute/`) and update to new path (`/tools/{tool_name}/execute`).

---

## Phase 6: Frontend unit tests

**Goal:** Ensure frontend compiles and passes tests with updated types and paths.

**Commands:**

1. `cd crates/bodhi && npm run test`
2. `cd crates/bodhi && npm run format`

Fix any breakage from path changes or type renames.

---

## Phase 7: Full backend + frontend validation

**Goal:** Gate check before moving to e2e changes.

**Commands:**

1. `make test.backend` (runs `cargo test` + `cargo test -p bodhi --features native`)
2. `make test.ui.unit` (runs `cd crates/bodhi && npm install && npm test`)
3. `cargo fmt --all`

All must pass before proceeding to e2e phases.

---

## Phase 8: Simplify test-oauth-app ConfigForm

**Current:** Two textareas (`input-requested-toolsets`, `input-requested-mcp-servers`)
**New:** Single textarea with `data-testid="input-requested"` accepting the full `requested` JSON

Default value: `{"toolset_types": [{"toolset_type": "builtin-exa-search"}]}`

**Files:**

- [crates/lib_bodhiserver_napi/test-oauth-app/src/components/ConfigForm.tsx](crates/lib_bodhiserver_napi/test-oauth-app/src/components/ConfigForm.tsx) - Replace two inputs with one `requested` textarea, update state/submit logic
- [crates/lib_bodhiserver_napi/test-oauth-app/src/context/AuthContext.tsx](crates/lib_bodhiserver_napi/test-oauth-app/src/context/AuthContext.tsx) - Update stored config shape (replace `requestedToolsets`+`requestedMcpServers` with `requested`)
- [crates/lib_bodhiserver_napi/test-oauth-app/src/lib/api.ts](crates/lib_bodhiserver_napi/test-oauth-app/src/lib/api.ts) - Update `requestAccess()` to pass `requested` as parsed JSON directly into the body

---

## Phase 9: Migrate e2e tests to new ConfigForm + fix response assertions

**ConfigSection page object:**

- [crates/lib_bodhiserver_napi/tests-js/pages/sections/ConfigSection.mjs](crates/lib_bodhiserver_napi/tests-js/pages/sections/ConfigSection.mjs) - Replace `requestedToolsets`/`requestedMcpServers` params with single `requested` param in `configureOAuthForm()`

**Spec files to migrate:**

- [toolsets-auth-restrictions.spec.mjs](crates/lib_bodhiserver_napi/tests-js/specs/toolsets/toolsets-auth-restrictions.spec.mjs) - Change to `requested: JSON.stringify({toolset_types: [{toolset_type: "builtin-exa-search"}]})`
- [mcps-auth-restrictions.spec.mjs](crates/lib_bodhiserver_napi/tests-js/specs/mcps/mcps-auth-restrictions.spec.mjs) - Change to `requested: JSON.stringify({mcp_servers: [{url: "..."}]})`, also fix `.data` to `.mcps` in response assertions
- Search for any other specs using `configureOAuthForm` and migrate

Also fix the existing MCP test failures: response shape is `{mcps: [...]}` not `{data: [...]}`.

---

## Phase 10: Rebuild embedded UI

**Goal:** Include all frontend + test-oauth-app changes in the NAPI build used by e2e tests.

**Command:** `make build.ui-rebuild`

This runs `build.ui-clean` then `build.ui` (Next.js build + NAPI bindings build).

---

## Phase 11: Run toolset e2e tests

**Goal:** Verify toolset execute path change works end-to-end.

**Command:**

```bash
cd crates/lib_bodhiserver_napi && HEADLESS=true npx playwright test tests-js/specs/toolsets/toolsets-auth-restrictions.spec.mjs --reporter=list
```

Fix any failures from the path change (`/execute/{method}` -> `/tools/{tool_name}/execute`).

---

## Phase 12: Run existing MCP e2e tests

**Goal:** Verify existing MCP tests pass with ConfigForm migration + response assertion fixes.

**Command:**

```bash
cd crates/lib_bodhiserver_napi && HEADLESS=true npx playwright test tests-js/specs/mcps/mcps-auth-restrictions.spec.mjs --reporter=list
```

Fix any failures.

---

## Phase 13: Add restricted-deepwiki MCP e2e test

Add to [mcps-auth-restrictions.spec.mjs](crates/lib_bodhiserver_napi/tests-js/specs/mcps/mcps-auth-restrictions.spec.mjs):

**New test: "App with MCP scope can access approved MCP but gets 401 on restricted MCP"**

1. **Phase 1 - Setup (session login):**
  - Login as admin
  - Navigate to `/ui/mcp-servers`, enable DeepWiki server URL (`https://mcp.deepwiki.com/mcp`)
  - Navigate to `/ui/mcps/new`, create "deepwiki" MCP instance, enable all tools, save
  - Navigate to `/ui/mcps/new`, create "restricted-deepwiki" MCP instance, enable all tools, save
  - Navigate to `/ui/mcps`, extract `restricted-deepwiki` UUID from table row `data-test-uuid` attribute
2. **Phase 2 - OAuth app config:**
  - Open test-oauth-app
  - Configure with `requested: JSON.stringify({mcp_servers: [{url: "https://mcp.deepwiki.com/mcp"}]})`
  - Submit access request
3. **Phase 3 - Approve with only "deepwiki" instance:**
  - On review page, select the "deepwiki" instance (not restricted-deepwiki)
  - Approve, complete OAuth flow, get token
4. **Phase 4 - REST API assertions (200 for approved):**
  - `GET /bodhi/v1/mcps/{deepwiki-id}` -> 200, verify basic MCP details
  - `POST /bodhi/v1/mcps/{deepwiki-id}/tools/refresh` -> 200
  - `POST /bodhi/v1/mcps/{deepwiki-id}/tools/read_wiki_structure/execute` with `{"repoName": "facebook/react"}` -> 200, verify basic response
5. **Phase 5 - REST API assertions (401 for restricted):**
  - `GET /bodhi/v1/mcps/{restricted-uuid}` -> 401, assert error body
  - `POST /bodhi/v1/mcps/{restricted-uuid}/tools/refresh` -> 401, assert error body
  - `POST /bodhi/v1/mcps/{restricted-uuid}/tools/read_wiki_structure/execute` with body -> 401, assert error body

**Extract shared fixtures:** Move MCP setup helpers (enable server URL, create instance, enable tools) into McpsPage methods for reuse.

**Verify:** Run the spec file again to confirm all 3 tests pass:

```bash
cd crates/lib_bodhiserver_napi && HEADLESS=true npx playwright test tests-js/specs/mcps/mcps-auth-restrictions.spec.mjs --reporter=list
```

---

## Phase 14: Full NAPI test suite

**Goal:** Final validation that all e2e and NAPI tests pass.

**Command:** `make test.napi`

This runs `cd crates/lib_bodhiserver_napi && npm install && npm run test:all` (vitest + all playwright specs).

All must be green.