# Prompt: BodhiApp Structural Analysis & Domain-Driven Consolidation

## Objective

Analyze BodhiApp's Rust workspace holistically and bottom-up to produce a consolidation plan that removes structural debt, makes code organization uniform and predictable, and aligns crate/module/type boundaries with the application's natural domain boundaries.

This is NOT just about errors — it covers types, traits, modules, naming, re-exports, service boundaries, and code placement.

## Prior Art

The error handling refactoring (see `ai-docs/claude-plans/20260207-error-cleanup.md`, `ai-docs/claude-plans/20260207-error-cleanup-2.md`) established uniform error patterns. This analysis extends that thinking to the entire codebase.

## Analysis Structure

### Step 1: Identify Broad Domains

Analyze the full application and identify 5-8 natural domain areas. Expected domains include (but discover from the code, don't assume):
- **Model Management** — aliases, repos, GGUF files, HuggingFace integration, model metadata
- **Inference** — llama.cpp process management, request forwarding, streaming, health checks
- **Authentication & Authorization** — OAuth2 flows, JWT, sessions, roles, scopes, API tokens
- **User Management** — user CRUD, access requests, role assignment
- **API Compatibility** — OpenAI endpoint compatibility, Ollama endpoint compatibility, format translation
- **Configuration & Settings** — app settings, environment, secrets, keyring
- **Toolsets** — tool definitions, tool instances, tool execution
- **Application Lifecycle** — setup, serve, desktop integration, Docker

For each domain, note which crates currently own pieces of it and whether the ownership is clean or fragmented.

### Step 2: Bottom-Up Component Analysis

For each crate (`objs`, `services`, `server_core`, `llama_server_proc`, `auth_middleware`, `commands`, `routes_oai`, `routes_app`, `routes_all`, `server_app`, `bodhi/src-tauri`), analyze:

#### 2a. Types & Domain Objects (`objs` focus)
- Are domain types grouped by domain or scattered?
- Are there types that belong to a specific domain but live in `objs` because of dependency convenience?
- Are there redundant or near-duplicate types? (e.g., multiple ways to represent a model alias)
- Naming consistency: do related types follow a naming convention?
- Are `pub use` re-exports from `objs/src/lib.rs` organized or a flat dump?

#### 2b. Service Layer (`services` focus)
- Does each service map cleanly to one domain?
- Are there services doing work for multiple unrelated domains?
- Are there cross-service dependencies that suggest a service should be split or merged?
- Are service trait methods well-scoped or do they have grab-bag interfaces?
- Is the `db/` module a clean data access layer or does it mix business logic?
- Are mock boundaries well-defined for testing?

#### 2c. Route Layer (`routes_oai`, `routes_app` focus)
- Does each route module map to one domain?
- Are there route modules that are too large and mix domains?
- Are there route modules that are too small and should merge?
- Is DTO/request/response type placement consistent? (some in route files, some in `*_dto.rs`, some in `objs`)
- Are handler signatures consistent in style?
- Is the error-to-response translation consistent across routes?

#### 2d. Infrastructure Layer (`server_core`, `auth_middleware`, `llama_server_proc`)
- Are infrastructure concerns cleanly separated from business logic?
- Does `server_core` contain business logic that should be in services?
- Is `auth_middleware` focused solely on auth or does it leak into other concerns?

#### 2e. Cross-Cutting Concerns
- **Naming conventions**: Are modules, types, functions named consistently across crates? Look for inconsistencies like `FooService` vs `FooManager`, `*Error` vs `*Failure`, `*Request` vs `*Params`, `*_handler` vs `*_endpoint`.
- **Module organization**: Are modules organized by domain or by technical layer within each crate?
- **Re-export hygiene**: Are `lib.rs` files organized with sections/comments or flat `pub use *`?
- **Test organization**: Are test utilities and fixtures well-organized or scattered?
- **Feature flags**: Are feature flags used consistently?

### Step 3: Identify Structural Debt Patterns

For each issue found, classify it:

1. **Misplaced code** — type/function lives in wrong crate or module
2. **Fragmented domain** — one domain's code scattered across multiple unexpected locations
3. **Naming inconsistency** — similar things named differently
4. **Redundant abstractions** — unnecessary indirection or wrapper types
5. **Missing abstractions** — repeated patterns that should be unified
6. **Oversized modules** — files doing too much, should be split
7. **Undersized modules** — files too small, should merge into neighbors
8. **Leaky boundaries** — crate/module exposes internals or depends on things it shouldn't
9. **Inconsistent patterns** — same task done differently in different places

### Step 4: Produce Consolidation Plan

Group findings into phases ordered by:
1. **Foundation first** — changes to `objs` and `services` before route-level changes
2. **Risk-adjusted** — smaller, safer changes before large restructurings
3. **Domain-coherent** — changes that make one domain fully clean before moving to the next

For each phase, specify:
- What changes
- Why (which debt pattern it addresses)
- What files move/merge/split
- What renames occur
- What tests need updating
- Verification command

## Constraints

- Do NOT propose changes to the frontend (`crates/bodhi/src/`)
- Do NOT propose changes to build system, CI, or Docker
- Do NOT propose new crates — prefer reorganizing within existing crates
- Do NOT propose trait redesigns unless the current design causes concrete problems
- Preserve all public API behavior (HTTP endpoints, request/response formats)
- Each phase must be independently compilable and testable
- Prefer moves and renames over rewrites — the code works, it just needs reorganization

## Output Format

Produce the analysis as a plan document with:
1. Domain map (diagram or table showing domains → crates → key types)
2. Findings table (issue, classification, severity, affected files)
3. Phased consolidation plan (as described in Step 4)
4. Summary metrics (number of file moves, renames, merges, splits per phase)

## How to Execute This Analysis

Read the following files first to build context:
- Every `CLAUDE.md` in the crates (symlinked in `ai-docs/`)
- Every `lib.rs` to understand re-exports and module structure
- Every `error.rs` or `error/mod.rs` to understand error boundaries
- `Cargo.toml` for each crate to understand dependency graph
- The completed error handling plan (`ai-docs/claude-plans/serene-toasting-parrot.md`) for context on what was already fixed

Then do targeted reads of specific modules where the CLAUDE.md hints at complexity or fragmentation.
