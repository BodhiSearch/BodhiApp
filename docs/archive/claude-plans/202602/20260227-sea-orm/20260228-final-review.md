# SeaORM Migration Review Plan

## Context

BodhiApp's persistence layer migrated from raw sqlx to SeaORM (commit `b33b1a22a`). This review makes the integration production-ready by thoroughly auditing 236 changed files across all layers for convention adherence, correctness, security, test coverage, and architectural consistency. No backwards compatibility needed (no production release exists).

## Review Scope

- **Commit**: HEAD (`b33b1a22a feat: SeaORM prototype migration for ModelRepository...`)
- **Output Directory**: `ai-docs/claude-plans/20260227-sea-orm/reviews/`
- **Files**: 236 changed, ~16k lines across 8+ crates
- **Reference Docs**: `ai-docs/claude-plans/20260227-sea-orm/` (context, conventions, decisions, phases 1-6)

## Key Review Decisions (from interview)

1. **`if_not_exists()`**: Flag for cleanup — no longer needed since sqlx migrations are deleted
2. **Output format**: Description + file:location (compact, navigable by fixing agent)
3. **Encrypted column exposure**: Critical — McpAuthHeaderRow, McpOAuthConfigRow, McpOAuthTokenRow must move to Pattern B
4. **Transaction atomicity**: Verify multi-step repository operations for proper wrapping
5. **Review organization**: By architectural layer, noting cross-layer fix dependencies
6. **Test gaps**: Flag both missing dual-DB parameterization AND missing test scenarios
7. **sqlx/session boundary**: Out of scope

## Layer-by-Layer Review Structure

### Review 1: `objs-review.md` — Foundation Layer (`crates/objs/`)

**Files**: `db_enums.rs`, `json_vec.rs`, `access_request.rs`, `api_model_alias.rs`, `mcp.rs`, `model_metadata.rs`, `oai.rs`, `user_alias.rs`, `lib.rs`, `Cargo.toml`, `test_mcp_types.rs`

**Checklist**:
- DeriveValueType on all enums that map to DB columns (complete set?)
- FromJsonQueryResult on types stored as JSON columns
- strum serialize_all consistency (snake_case everywhere?)
- Serde rename vs strum serialize alignment (e.g., kebab-case serde vs snake_case strum for DB)
- JsonVec correctness: Deref, serialization roundtrip, empty vec handling
- No unnecessary derives or sea_orm leaking into non-DB contexts
- Convention: `sea_orm` dependency in objs crate — is this appropriate for a "domain objects" crate?

### Review 2: `services-schema-review.md` — Entities + Migrations (`crates/services/src/db/entities/`, `sea_migrations/`)

**Files**: 17 entity files, 14 migration files, `entities/mod.rs`, `sea_migrations/mod.rs`

**Checklist**:
- Entity Pattern A vs B correctness per implementation-conventions.md
- **Critical**: McpAuthHeaderRow, McpOAuthConfigRow, McpOAuthTokenRow must be Pattern B (hiding encrypted cols)
- Migration ordering and FK dependency correctness
- `if_not_exists()` flagged for removal on all tables
- CITEXT/COLLATE NOCASE applied consistently for case-insensitive columns
- CASCADE constraints on MCP table hierarchy
- Index coverage for query patterns
- Timestamp columns all use `timestamp_with_time_zone`
- String PKs (ULID) on all tables (except settings which uses natural key)
- Relation enums complete and correct
- `json_binary()` for all JSON columns
- Migration down() drops in correct reverse order

### Review 3: `services-repos-review.md` — Repository Implementations (`crates/services/src/db/service_*.rs`)

**Files**: 9 `service_*.rs` files, `default_service.rs`, `error.rs`, `objs.rs`, `service.rs`, `mod.rs`

**Checklist**:
- **Transaction atomicity**: Multi-step operations (MCP create, toolset+config) wrapped in transactions
- TimeService injection: no direct `Utc::now()` calls
- ULID generation: `ulid::Ulid::new().to_string()` for all new records
- `Set()` for all insert fields (no `NotSet` for inserts)
- Error handling: `.map_err(DbError::from)?` default, specific matching only where needed
- Encryption/decryption pattern correctness for Pattern B entities
- DbError enum completeness for all SeaORM error scenarios
- DefaultDbService::reset_all_tables() truncation order (FK-aware?)
- seed_toolset_configs() idempotency
- Partial update pattern: `Default::default()` + set changed fields + `updated_at`
- Cross-layer: objs.rs domain object conversions (From impls) correctness

### Review 4: `services-tests-review.md` — Test Infrastructure + Test Files

**Files**: `test_utils/sea.rs`, `test_utils/db.rs`, `test_utils/mod.rs`, `test_utils/objs.rs`, `test_utils/envs.rs`, 9 `test_*_repository.rs` files, service-level test files

**Checklist**:
- All repository tests parameterized with `#[values("sqlite", "postgres")]`
- `#[serial(pg_app)]` on all dual-DB tests
- `_setup_env: ()` fixture present
- `#[anyhow_trace]` on all test functions
- `assert_eq!(expected, actual)` convention (expected first)
- No `use super::*` in test modules
- No inline timeouts
- Happy path + main error scenarios covered per repository
- SeaTestContext fixture correctness (fresh schema, FrozenTimeService)
- TestDbService wraps DefaultDbService correctly
- Encryption roundtrip tests for Pattern B entities
- Missing test scenarios (unique constraint violations, concurrent access, edge cases)

### Review 5: `upstream-review.md` — auth_middleware + routes_app + services consumers

**Files**: auth_middleware (7 files), routes_app (15+ files), services consumer files (access_request_service, mcp_service, tool_service, etc.)

**Checklist**:
- Domain type changes propagated correctly (DateTime<Utc> fields, ULID IDs)
- AuthContext handling with new types
- Route handlers: error chain intact (service error → AppError → ApiError)
- OpenAPI utoipa annotations updated for enum changes
- No direct DB access from routes (all through repository traits)
- MockDbService usage: no NEW mock usage introduced (existing OK)
- Test assertions updated for new type shapes
- snake_case enum values in API responses

### Review 6: `e2e-infra-review.md` — E2E, Frontend, Docker, CI

**Files**: Playwright config, fixtures, 18 spec files, db-config.mjs, start-shared-server.mjs, UI components (8 files), openapi.json, ts-client, lib_bodhiserver, NAPI, Docker, CI

**Checklist**:
- Dual-DB Playwright projects configured correctly (SQLite:51135, PostgreSQL:41135)
- DB config utility correctness
- Shared server startup with DB-aware config
- E2E specs use correct fixtures for dual-DB
- UI components updated for snake_case enum values
- OpenAPI schema matches Rust enum serialization
- ts-client types regenerated and consistent
- lib_bodhiserver bootstrap uses DefaultDbService correctly
- NAPI bindings expose DB config
- docker-compose-test-deps.yml PostgreSQL service config
- CI workflow PostgreSQL service setup

## Execution Plan

### Step 1: Launch 3 parallel Explore agents
- **Agent A**: Read all entity files + migration files + objs changes (Review 1 + Review 2 scope)
- **Agent B**: Read all service_*.rs repo implementations + test files (Review 3 + Review 4 scope)
- **Agent C**: Read upstream consumers + e2e/infra changes (Review 5 + Review 6 scope)

Each agent loads the relevant CLAUDE.md/PACKAGE.md for the crates it reviews.

### Step 2: Launch 3 parallel review-writing agents (background)
Based on exploration results:
- **Agent D**: Write `objs-review.md` + `services-schema-review.md`
- **Agent E**: Write `services-repos-review.md` + `services-tests-review.md`
- **Agent F**: Write `upstream-review.md` + `e2e-infra-review.md`

### Step 3: Cross-cutting analysis (main agent)
After all sub-agents complete:
- Type consistency across layers (objs ↔ entities ↔ repos ↔ routes ↔ openapi ↔ ts-client)
- Error chain tracing
- Aggregate test coverage gaps
- Dead code / unused import detection

### Step 4: Generate `index.md`
Consolidated index with:
- Finding counts by priority (Critical / Important / Nice-to-have)
- Tables linking findings to files and reports
- Fix order following layered development methodology
- Cross-layer dependency notes (e.g., "fixing entity Pattern B requires objs + services changes")

### Step 5: Present summary

## Output Files

```
ai-docs/claude-plans/20260227-sea-orm/reviews/
├── index.md                    # Consolidated findings index
├── objs-review.md              # Review 1: Foundation layer
├── services-schema-review.md   # Review 2: Entities + migrations
├── services-repos-review.md    # Review 3: Repository implementations
├── services-tests-review.md    # Review 4: Test infrastructure
├── upstream-review.md          # Review 5: auth_middleware + routes_app
└── e2e-infra-review.md         # Review 6: E2E + infrastructure
```

## Verification

After review files are generated:
1. Each review file has proper structure (Files Reviewed, Findings with Priority/File/Location/Issue/Recommendation, Summary)
2. Index.md has correct finding counts matching individual reports
3. Fix order follows layered methodology (objs → services → auth_middleware → routes_app → e2e)
4. Cross-layer dependencies are noted (e.g., Pattern B entity fixes touch both objs and services)
5. All Critical findings have clear, actionable recommendations
