# Prompt: Merge routes_oai and routes_app into a Unified Routes Crate

## Objective

Analyze BodhiApp's `routes_oai`, `routes_app`, and `routes_all` crates and produce a consolidation plan that merges them into a single cohesive routing crate. The current three-crate split was an early architectural choice; now that `routes_app` has been reorganized into domain-coherent folders and `routes_oai` is small (5 files, 6 endpoints), the split creates overhead without proportional benefit.

## Prior Art

- **Route reorganization** (`ai-docs/claude-plans/20260208-cleanup-routes.md`) reorganized `routes_app` from flat files into domain folders (`routes_auth/`, `routes_users/`, `routes_models/`, `routes_api_models/`, `routes_toolsets/`).
- **Error consolidation** (`ai-docs/claude-plans/20260208-cleanup-services.md`) consolidated service error types, deleted the `commands` crate, and split `DbService` into repository sub-traits.
- The domain groupings in `routes_app` are stable and well-tested. `routes_oai` fits naturally as another domain folder.

## Current State

### routes_oai (5 files)
- `lib.rs` -- exports and endpoint path constants
- `routes_chat.rs` -- `/v1/chat/completions`, `/v1/embeddings` (OpenAI chat completions + embeddings)
- `routes_oai_models.rs` -- `/v1/models` (OpenAI models listing)
- `routes_ollama.rs` -- `/api/tags`, `/api/show`, `/api/chat` (Ollama compatibility)
- `test_utils/mod.rs` -- test helpers

Dependencies: `objs`, `services`, `auth_middleware`, `server_core`, `axum`, `utoipa`, `async-openai`, `errmeta_derive`, `thiserror`, `futures-util`, `validator`, `serde_yaml`

### routes_app (47 files)
Organized into domain folders: `routes_auth/`, `routes_users/`, `routes_models/`, `routes_api_models/`, `routes_toolsets/`, plus standalone files and shared infrastructure.

Dependencies: same as routes_oai plus `jsonwebtoken`, `oauth2`, `tower-sessions`, `uuid`, `base64`, `sha2`, `regex`. **Already depends on `routes_oai`** for `OAIRouteError` re-export.

### routes_all (4 files)
- `routes.rs` -- composes routes from both crates with middleware layers and authorization hierarchy
- `routes_proxy.rs` -- UI proxy for dev mode
- `test_utils/mod.rs` -- test helpers
- `lib.rs` -- exports

Dependencies: `routes_app`, `routes_oai`, plus `tower-http`, `utoipa-swagger-ui`, `include_dir`, `hyper-util`

### Key Observation
`routes_app` already depends on `routes_oai`. `routes_all` depends on both and exists primarily to compose them. This creates a three-layer cake where two layers could be one.

## Analysis Structure

### Step 1: Map Cross-Crate Boundaries

Before proposing changes, understand what actually crosses crate boundaries:

- What types, traits, or functions does `routes_app` import from `routes_oai`?
- What does `routes_all` import from each crate?
- Are there shared types, error enums, or test utilities that are duplicated or awkwardly split?
- What does the OpenAPI documentation generation look like across the three crates?
- How do endpoint path constants flow between crates?

### Step 2: Analyze Merge Strategy

Consider two possible merge strategies:

**Option A: Merge routes_oai into routes_app, keep routes_all**
- routes_oai becomes `routes_app/src/routes_oai/` (another domain folder)
- routes_all continues to own route composition + middleware + UI proxy
- Minimal change to route composition logic

**Option B: Merge all three into a single crate**
- routes_oai becomes a domain folder
- Route composition, middleware, UI proxy all move into the unified crate
- Eliminates the thin routes_all wrapper entirely
- server_app and bodhi (Tauri) depend directly on the unified crate

For each option, assess:
- What dependencies does the merged crate gain? Are there concerns about compile times?
- Does the route composition logic (currently in routes_all/routes.rs) belong with the handlers?
- Does the UI proxy logic belong with the API routes?
- Are there consumers that use routes_oai independently of routes_app?

### Step 3: Analyze Error Type Consolidation

Currently:
- `routes_oai` defines `OAIRouteError` (renamed from `HttpError` in recent refactoring)
- `routes_app` has domain-specific error enums per module
- Both use the same `errmeta_derive` + `AppError` pattern

Questions:
- Can `OAIRouteError` coexist alongside the routes_app domain errors in a single crate?
- Are there error type naming conflicts?
- Does the error-to-ApiError translation chain need changes?

### Step 4: Analyze OpenAPI Documentation Impact

Currently:
- `routes_oai` contributes OpenAI/Ollama endpoint documentation
- `routes_app` contributes application endpoint documentation
- `routes_all` merges them into `BodhiOpenAPIDoc`

Questions:
- How does the OpenAPI doc generation work across crate boundaries currently?
- Does merging simplify or complicate the doc generation?
- Are there shared response types or schemas that would benefit from co-location?

### Step 5: Analyze Test Infrastructure

Currently:
- `routes_oai/test_utils/` has its own test helpers
- `routes_app/test_utils/` has its own test helpers
- `routes_all/test_utils/` has its own test helpers

Questions:
- Is there duplication across these test utility modules?
- Would merging simplify test setup (shared router construction, shared mock state)?
- Do integration tests in routes_all test cross-crate behavior that would become intra-crate?

### Step 6: Produce Consolidation Plan

Group findings into phases ordered by:
1. **Foundation first** -- shared infrastructure and error types before handler moves
2. **Risk-adjusted** -- smaller, safer changes before large restructurings
3. **Domain-coherent** -- complete one logical unit before moving to the next

For each phase, specify:
- What changes
- Why (which structural issue it addresses)
- What files move/merge/split
- What renames occur
- What Cargo.toml changes are needed
- What tests need updating
- Verification command (`cargo test -p <crate>`)

## Constraints

- Do NOT propose changes to `objs`, `services`, `server_core`, or `auth_middleware` -- they are stable
- Do NOT propose changes to frontend, build system, CI, or Docker
- Do NOT propose new crates -- the goal is consolidation, not proliferation
- Preserve all public API behavior (HTTP endpoints, response formats, error codes)
- Each phase must be independently compilable and testable
- Prefer moves over rewrites -- the code works, it just needs reorganization
- Error codes must NOT change (they are part of the API contract)
- OpenAPI documentation must remain complete and correct after each phase
- The authorization hierarchy in route composition must be preserved exactly

## Output Format

Produce the analysis as a plan document with:
1. Current cross-crate boundary inventory (what crosses where)
2. Recommended merge strategy with justification
3. Phased consolidation plan (as described in Step 6)
4. Dependency impact analysis (new deps on merged crate, compile time impact)
5. File change summary (moves, renames, merges per phase)
6. Flagged issues not in scope

## How to Execute This Analysis

Read the following files first to build context:
- `crates/routes_oai/CLAUDE.md` and `crates/routes_app/CLAUDE.md` (architecture guidance)
- `crates/routes_oai/src/lib.rs` and `crates/routes_app/src/lib.rs` for module structure and re-exports
- `crates/routes_all/src/lib.rs` and `crates/routes_all/src/routes.rs` for composition logic
- `crates/routes_oai/Cargo.toml`, `crates/routes_app/Cargo.toml`, `crates/routes_all/Cargo.toml` for dependency graph
- Every module file in `routes_oai/src/` to understand handler patterns and error types
- `crates/routes_app/src/shared/openapi.rs` for OpenAPI generation
- `crates/server_app/src/` and `crates/bodhi/src-tauri/src/` to understand consumers of routes_all
- The completed route reorganization plan (`ai-docs/claude-plans/20260208-cleanup-routes.md`) for context

Then do targeted reads where complexity or unclear boundaries appear.
