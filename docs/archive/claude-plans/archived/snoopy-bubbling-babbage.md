# Plan: SeaORM Post-Migration Cleanup & Documentation Consolidation

## Context

The SeaORM migration (commit d49357535) is complete and merged. All changes are squashed in HEAD. Before launching the next review cycle (full SeaORM diff review), we need to:

1. **Clean up code**: Remove `_sea` suffix from service files to improve git diff readability when re-squashed
2. **Clean up documentation**: Remove stale reviews/plans, consolidate into clean phase plans
3. **Update context.md**: Rewrite as a snapshot of current code state (full commit scope)

---

## Part 1: Code Cleanup

### Step 1.1 — Rename 9 `service_*_sea.rs` → `service_*.rs`

Rename files in `crates/services/src/db/`:

| Current Name | New Name |
|---|---|
| `service_access_request_sea.rs` | `service_access_request.rs` |
| `service_access_sea.rs` | `service_access.rs` |
| `service_app_instance_sea.rs` | `service_app_instance.rs` |
| `service_mcp_sea.rs` | `service_mcp.rs` |
| `service_model_sea.rs` | `service_model.rs` |
| `service_settings_sea.rs` | `service_settings.rs` |
| `service_token_sea.rs` | `service_token.rs` |
| `service_toolset_sea.rs` | `service_toolset.rs` |
| `service_user_alias_sea.rs` | `service_user_alias.rs` |

**Why**: Old sqlx files had identical names (`service_model.rs`, etc.). Renaming makes git detect these as modifications instead of delete+add, producing a much more readable squashed diff.

Use `git mv` for each file to preserve git history tracking.

### Step 1.2 — Update module declarations in `crates/services/src/db/mod.rs`

Lines 14-22: Update 9 `mod` declarations to drop `_sea` suffix:
```rust
mod service_access_request;  // was: mod service_access_request_sea;
mod service_access;          // was: mod service_access_sea;
mod service_app_instance;    // was: mod service_app_instance_sea;
mod service_mcp;             // was: mod service_mcp_sea;
mod service_model;           // was: mod service_model_sea;
mod service_settings;        // was: mod service_settings_sea;
mod service_token;           // was: mod service_token_sea;
mod service_toolset;         // was: mod service_toolset_sea;
mod service_user_alias;      // was: mod service_user_alias_sea;
```

No import changes needed — these modules are private (no `pub use`), accessed only implicitly through `DefaultDbService` trait impls.

### Step 1.3 — Verify `SqlxError` usage, remove if dead

File: `crates/services/src/db/error.rs`

The `SqlxError` struct wraps `sqlx::Error`. Need to verify it's not used by `session_service/error.rs` (which has its own `SessionServiceError`). If `SqlxError` is truly unused in code:
- Remove `SqlxError` struct definition
- Remove the `impl_error_from!(::sqlx::Error, SqlxError, ...)` macro call
- Keep `SeaOrmDbError` and `DbError` enum as-is

### Step 1.4 — Update stale documentation references

1. **`crates/services/PACKAGE.md`** (line 54): Change `service_*_sea.rs` → `service_*.rs`
2. **`crates/services/PACKAGE.md`** (line 59): Change `service_*_sea.rs` → `service_*.rs`
3. **`crates/services/src/test_utils/PACKAGE.md`** (lines 117, 123): Change `SqliteDbService` → `DefaultDbService`
4. **`crates/services/CLAUDE.md`**: Check and update any references to `SqliteDbService` or `_sea` naming

### Step 1.5 — Full consistency audit

Scan all crates for:
- References to `_sea` file names or module names
- References to `SqliteDbService` (should only be in historical docs, not code)
- Stale comments mentioning old sqlx patterns
- Unused imports or dead code in `crates/services/src/db/` files

---

## Part 2: Documentation Cleanup (`ai-docs/claude-plans/20260227-sea-orm/`)

### Step 2.1 — Delete stale files

Remove ALL of the following:
- `prompt.md`
- `20260227-sea-orm-iter-1.md`
- `20260227-orm-conventions.md`
- `20260227-sea-orm-mid-review.md`
- `20260227-plan-mid-review-fix.md`
- `20260227-sea-orm-e2e.md`
- `20260227-sea-orm-mcp-e2e-impls.md`
- `20260227-sea-orm-final-review.md`
- `20260228-final-test-fix.md`
- `mid-review-feedback.md`
- `mid-review-findings.md`
- `e2e-dual-db-prompt.md`
- `seaorm_exploration_migration_d00adb61.plan.md`
- `seaorm_full_migration_6cebce11.plan.md`
- `updated_seaorm_migration_plan_79a76649.plan.md`
- `reviews/` (entire directory: index.md + 7 review files)

### Step 2.2 — Create phase plans (by crate layer)

Create reverse-engineered phase plans from the git diff, organized by crate layer. Each plan lists every modified/added/deleted file with 1-line description. Semi-technical, functional, core impls only.

**Files to create:**

#### `phase-1-objs.md` — Foundation Layer Changes
Covers `crates/objs/`:
- `db_enums.rs` (new) — Domain enums (DownloadStatus, TokenStatus, AppStatus) with DeriveValueType
- `json_vec.rs` (new) — JsonVec wrapper with private inner field, FromJsonQueryResult
- `access_request.rs` (modified) — Added DeriveValueType to FlowType, UserAccessRequestStatus, AppAccessRequestStatus
- `mcp.rs` (modified) — Added DeriveValueType to McpAuthType, RegistrationType (snake_case)
- `model_metadata.rs` (modified) — FromJsonQueryResult for ModelArchitecture
- `api_model_alias.rs` (modified) — DeriveValueType additions
- `user_alias.rs` (modified) — DeriveValueType additions
- `Cargo.toml` (modified) — Added `sea-orm`, `ulid` deps; removed `uuid`
- Files that changed UUID→ULID patterns

#### `phase-2-services-entities-migrations.md` — SeaORM Schema Layer
Covers `crates/services/src/db/entities/` and `crates/services/src/db/sea_migrations/`:
- 17 entity files (list each with table it maps to)
- 14 migration files (list each with what it creates)
- CASCADE FK constraints on MCP tables
- `mod.rs` files for both modules
- `Cargo.toml` changes (sea-orm, sea-orm-migration deps)

#### `phase-3-services-repositories.md` — Repository Implementations
Covers `crates/services/src/db/`:
- 9 trait definition files (`*_repository.rs`)
- 9 SeaORM implementation files (`service_*.rs`) — what each implements
- `default_service.rs` — DefaultDbService struct, DbCore impl, seed_toolset_configs
- `db_core.rs` — DbCore trait (migrate, now, encryption_key, reset)
- `service.rs` — DbService super-trait, blanket impl
- `error.rs` — SeaOrmDbError, DbError enum
- `time_service.rs` — TimeService trait, DefaultTimeService, FrozenTimeService
- Deleted old sqlx files (list each)

#### `phase-4-services-test-infra.md` — Test Infrastructure
Covers `crates/services/src/test_utils/` and test files:
- `sea.rs` (new) — SeaTestContext fixture (dual SQLite/PostgreSQL)
- `db.rs` (modified) — TestDbService wrapping DefaultDbService
- 9 test files (`test_*_repository.rs`) — what each tests
- FrozenTimeService integration
- `.env.test.example` update

#### `phase-5-auth-middleware-routes.md` — Upstream Consumer Changes
Covers `crates/auth_middleware/` and `crates/routes_app/`:
- Token service changes
- Auth context changes
- Route handler changes for new domain types
- Error handling updates

#### `phase-6-e2e-infra.md` — E2E & Infrastructure
Covers E2E test infrastructure and cross-cutting:
- `docker-compose-test-deps.yml` — PostgreSQL service
- `playwright.config.mjs` — Dual-DB projects (sqlite:51135, postgres:41135)
- `fixtures.mjs` — Shared server URL fixture
- `scripts/start-shared-server.mjs` — Server startup changes
- `utils/db-config.mjs` (new) — DB configuration utility
- E2E spec modifications (18 files)
- `openapi.json` changes
- `ts-client/` type regeneration
- UI component changes (`crates/bodhi/src/`)
- `.cargo/config.toml` — RUST_MIN_STACK increase
- `Makefile` updates

### Step 2.3 — Update `context.md`

Rewrite as a post-implementation snapshot of current code state (full commit scope). Reference actual file paths. Sections:

1. **Migration Outcome** — What was replaced (sqlx → SeaORM), key stats
2. **Persistence Architecture** — DefaultDbService, DbService trait, 9 repository traits, entity definitions, migrations
3. **Dual Database Support** — SQLite/PostgreSQL, SeaTestContext, configuration
4. **Key Patterns** — DeriveValueType, FromJsonQueryResult, TimeService, ULID, CASCADE FKs, error handling (SeaOrmDbError)
5. **Test Infrastructure** — TestDbService, SeaTestContext, FrozenTimeService, test file organization
6. **E2E Infrastructure** — Dual-DB Playwright, shared server, port conventions
7. **API & Frontend Changes** — OpenAPI, TS client, UI component updates
8. **Session Service** — Still uses sqlx directly (not SeaORM), separate from main migration

Remove: migration 0014 reference, kebab-case mention. Update: enum serialization is snake_case via strum attributes.

### Step 2.4 — Update `index.md`

Rewrite to reflect new folder structure:
```
- context.md          — Post-implementation snapshot of current state
- index.md            — This file (folder navigation)
- migration-decisions.md   — Architectural decisions (ULID, DateTime, CITEXT, encryption)
- implementation-conventions.md — Coding patterns (Entity A/B, tests, errors, migrations)
- phase-1-objs.md     — Foundation layer changes
- phase-2-services-entities-migrations.md — Schema layer
- phase-3-services-repositories.md — Repository implementations
- phase-4-services-test-infra.md — Test infrastructure
- phase-5-auth-middleware-routes.md — Consumer crate changes
- phase-6-e2e-infra.md — E2E & infrastructure
```

### Step 2.5 — Update `migration-decisions.md` and `implementation-conventions.md`

Review and update both files to reflect final code state:
- Remove any decisions that were reversed (e.g., migration 0014)
- Update any conventions that evolved during implementation
- Ensure file path references are current (no `_sea` suffixes)

---

## Part 3: Verification

### 3.1 — Code verification
```bash
# Compile check after renames
cargo check -p services

# Run services tests
cargo test -p services

# Run full backend tests
make test.backend
```

### 3.2 — Documentation verification
- Verify no references to `_sea` suffix remain in crates/ source code
- Verify no references to `SqliteDbService` remain in code (only historical docs acceptable)
- Verify `index.md` links match actual files in folder
- Verify `context.md` file paths reference actual existing files

### 3.3 — Git diff verification
After renaming, check that git detects the changes as renames:
```bash
git diff --stat -M  # -M enables rename detection
```
Expect to see `service_model_sea.rs => service_model.rs` style entries for 8 of 9 files.

---

## Execution Order & Sub-Agent Orchestration

### Stage 1: Code Cleanup (Main Agent)

**Step 1.1-1.2**: Rename 9 files + update `mod.rs` (atomic)
**Step 1.3**: Verify and remove `SqlxError` if dead
**Step 1.4-1.5**: Update stale doc references, consistency audit

### Gate 1: Code Verification (Sub-Agent)

**Agent: `code-verify`**
- Run `cargo check -p services` — must pass
- Run `cargo test -p services` — must pass
- Run `make test.backend` — full backend regression
- Run `git diff --stat -M` to confirm renames detected
- Grep for stale `_sea` references, `SqliteDbService` in code
- **Gate passes only if ALL checks succeed**

### Stage 2: Documentation Cleanup (3 Parallel Sub-Agents)

Launched only after Gate 1 passes.

**Agent A: `phase-plans-writer`** (writes 6 phase plan files)
- Analyze `git diff HEAD~1..HEAD -- crates/` for file-level changes
- Write `phase-1-objs.md` through `phase-6-e2e-infra.md`
- Each file: crate-layer organized, file-level detail, 1-line descriptions
- Semi-technical, functional, core impls only
- Reference post-rename file names (no `_sea`)

**Agent B: `context-rewriter`** (rewrites context.md)
- Read current code state across all crates
- Rewrite `context.md` as post-implementation snapshot (full commit scope)
- Sections: Migration Outcome, Persistence Architecture, Dual DB, Key Patterns, Test Infra, E2E Infra, API/Frontend, Session Service
- Reference actual file paths, use current code as source of truth
- Remove migration 0014 reference, update enum serialization to snake_case

**Agent C: `doc-cleanup`** (delete stale files, update reference docs)
- Delete all 15 stale plan/review files + `reviews/` directory
- Update `index.md` to reflect new folder structure
- Update `migration-decisions.md` — remove reversed decisions, update file paths
- Update `implementation-conventions.md` — update file paths, ensure current patterns

### Gate 2: Documentation Verification (Sub-Agent)

**Agent: `doc-verify`**
- Verify all file paths in `context.md` reference existing files
- Verify all file paths in phase plans reference existing files (post-rename names)
- Verify `index.md` entries match actual files in folder
- Verify no references to deleted files remain in kept documents
- Verify no `_sea` suffix references in any documentation
- **Gate passes only if ALL checks succeed**

### Stage 3: Final Verification (Sub-Agent)

**Agent: `final-verify`**
- Run `make test.backend` one final time (full regression after all changes)
- Verify git status is clean (all changes staged/committed)
- Run `git diff --stat -M` to confirm final diff shape is optimal
