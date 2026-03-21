# crates/ — CLAUDE.md
**Companion docs**: Load individual `crates/<crate>/CLAUDE.md` for crate-specific details.
See `MDFILES.md` in project root for documentation conventions.

## Crate Index

| Crate | Purpose |
|-------|---------|
| `errmeta_derive` | Proc macro: `#[derive(ErrorMeta)]` generating `AppError` impl |
| `errmeta` | Error foundation: `AppError` trait, `ErrorType`, `IoError`, `EntityError`, `impl_error_from!` |
| `llama_server_proc` | LLM process lifecycle: Server trait, health checks, binary resolution |
| `mcp_client` | MCP protocol client over Streamable HTTP |
| `services` | Domain types + business logic hub: AppService, DbService, SeaORM, all domain modules |
| `server_core` | HTTP infrastructure: SharedContext, SSE streaming, InferenceService |
| `routes_app` | API endpoints + auth middleware: AuthScope, ApiError, JWT, session, OpenAPI |
| `server_app` | Standalone HTTP server: ServeCommand, graceful shutdown, live integration tests |
| `lib_bodhiserver` | Embeddable server library: AppServiceBuilder, setup_app_dirs, re-exports |
| `lib_bodhiserver_napi` | NAPI bindings: @bodhiapp/app-bindings, Playwright E2E |
| `bodhi/src` | Vite + TanStack Router + TanStack Query v5 frontend: @bodhiapp/ts-client, Vitest, MSW |
| `bodhi/src-tauri` | Tauri desktop + container server: native feature flag, dual-mode |
| `ci_optims` | Docker layer caching for CI builds |
| `xtask/` | Build automation: OpenAPI spec generation, TypeScript client generation |

Sub-module docs (load when working inside these modules):
- `crates/routes_app/src/middleware/CLAUDE.md` — Auth middleware
- `crates/services/src/test_utils/CLAUDE.md` — Test utility infrastructure
- `crates/server_core/src/test_utils/CLAUDE.md` — HTTP test utilities
- `crates/lib_bodhiserver_napi/tests-js/CLAUDE.md` + `E2E.md` — Playwright E2E tests

## Shared Rust Conventions

### Module Organization
- `mod.rs` files: ONLY module declarations (`mod xxx;`) and re-exports (`pub use xxx::*;`). No logic.
- Domain modules follow `*_objs.rs` for types, `error.rs` for errors, `service.rs` for business logic.

### Re-export Rules
- `services` re-exports all `errmeta` types — downstream crates import from `services::` only, never `errmeta::` directly.
- `services` re-exports `db::*` — use `services::DbService` not `services::db::DbService`.
- `lib_bodhiserver` re-exports curated surface from `services`, `routes_app`, `server_app` for leaf crates.

## Cross-Crate Architecture Patterns

### Error Layer Separation
Three layers — never mix:
1. **errmeta**: `AppError` trait, `ErrorType`, `IoError`, `EntityError` — zero framework deps
2. **services**: Domain errors (`TokenServiceError`, `McpError`, `ToolsetError`, `AuthContextError`) — `#[derive(ErrorMeta)]`
3. **routes_app**: `ApiError` / `OpenAIApiError` / `ErrorBody` in `routes_app::shared` — HTTP responses

Flow: service error -> `AppError` -> `ApiError` (blanket `From<T: AppError>` auto-converts). Middleware uses `MiddlewareError` (also blanket `From<T: AppError>`).

**IMPORTANT**: `ApiError` is NOT in `services`.

### Role Hierarchy
`ResourceRole`: `Anonymous < Guest < User < PowerUser < Manager < Admin`. Derives `PartialOrd` — variant order matters.
- `Anonymous`: unauthenticated access (maps to `AuthContext::Anonymous`)
- `Guest`: authenticated but no assigned role (JWT with empty roles)
- `Session.role` and `MultiTenantSession.role` are `ResourceRole` (not `Option<ResourceRole>`)
- `Anonymous`/`Guest` cannot be assigned via admin APIs (input validation rejects them)
- `TokenScope` and `UserScope` do NOT have Guest/Anonymous variants

### AuthScope Extractor
Route handlers use `AuthScope` -> `AuthScopedAppService` (extracts `AuthContext` + `Arc<dyn AppService>`). Use auth-scoped sub-services (`.tokens()`, `.mcps()`, etc.), never raw service accessors. Infrastructure uses `AppService` directly. Full detail in `crates/routes_app/CLAUDE.md`.

### Route Groups and CORS
Per-group CORS — no global CorsLayer. Session-only APIs have restrictive CORS; external app APIs under `/bodhi/v1/apps/` have permissive CORS. Full group table in `crates/routes_app/CLAUDE.md`.

### CRUD Conventions
Uniform architecture across all domains. Full reference (Entity Alias Index, Request/Response types, Service/Route handler patterns) in `crates/services/CLAUDE.md`.

### Multi-Tenant Transactions
All mutating `DbService` operations use `begin_tenant_txn(tenant_id)` from `DbCore` trait. PostgreSQL sets RLS via `SET LOCAL app.current_tenant_id`. SQLite returns plain transaction.

### Multi-Tenant Isolation Test Pattern
When adding new tenant-scoped tables, add isolation tests. Reference pattern: `crates/services/src/toolsets/test_toolset_repository_isolation.rs:40`. Tests create resources in two tenants, verify list/get isolation per tenant, and cross-tenant get-by-ID returns None. Run with `#[values("sqlite", "postgres")]`. Constants: `TEST_TENANT_ID`, `TEST_TENANT_B_ID`, `TEST_USER_ID` in `test_utils/db.rs`.

### Time Handling
Never use `Utc::now()` directly. All timestamps through `TimeService`. Tests use `FrozenTimeService`.

### OpenAPI -> TypeScript Pipeline
After any API change: `cargo run --package xtask openapi` -> `make build.ts-client` -> frontend imports from `@bodhiapp/ts-client`.

## Running Tests (CLI Commands)

**Run tests ONCE per step.** Do NOT run the same test command multiple times with different `tail`/`grep` filters.

### Compile check (fast)
```bash
cargo check -p <crate> 2>&1 | tail -5
```

### Summary only
```bash
cargo test -p services --lib -p routes_app -p server_app 2>&1 | grep -E "test result|FAILED|Running |failures:"
```

### When tests fail
Re-run the **failing crate only**:
```bash
cargo test -p <failing_crate> --lib 2>&1 | grep -E "FAILED|failures:|test result" -A 5
```

### Key rules
- **NEVER** run the same `cargo test` command more than once just to see different output
- For compile errors, `cargo check` is faster than `cargo test`
- `^test ` lines are noisy with 1000+ tests — only grep for them when diagnosing a specific test

## Shared Testing Conventions

- **rstest for all Rust tests**: `#[rstest]` + `#[tokio::test]` + `#[anyhow_trace]`, return `-> anyhow::Result<()>`
- `#[case]` for parameterized, `#[values]` for combinatorial, `#[fixture]` for shared setup
- `#[awt]` only when `#[future]` fixture params are used
- **Test files**: Prefer sibling `test_*.rs` via `#[cfg(test)] #[path = "test_<name>.rs"] mod test_<name>;`. Inline `mod tests` for files under 500 lines
- **Assertions**: `assert_eq!(expected, actual)` with `pretty_assertions`. Error assertions via `.code()`, never message text
- Avoid `use super::*` in test modules — use explicit imports
- **Test boundaries**: `routes_app` = single-turn via `tower::oneshot()`, `server_app` = multi-turn real HTTP (`#[serial_test::serial(live)]`), `lib_bodhiserver_napi` = Playwright E2E
