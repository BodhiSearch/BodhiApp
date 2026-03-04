# crates/ — CLAUDE.md
**Companion docs**: Load individual `crates/<crate>/CLAUDE.md` for crate-specific details.
See `MDFILES.md` in project root for documentation conventions.

## Crate Index

| Crate | Path | Purpose | Keywords |
|-------|------|---------|----------|
| `errmeta_derive` | `crates/errmeta_derive/` | Proc macro: `#[derive(ErrorMeta)]` generating `AppError` impl | proc-macro, error_type(), code(), args(), transparent delegation, trybuild |
| `errmeta` | `crates/errmeta/` | Minimal error foundation (zero framework deps) | AppError trait, ErrorType, IoError, EntityError, impl_error_from!, RwLockReadError |
| `llama_server_proc` | `crates/llama_server_proc/` | LLM process lifecycle management | Server trait, health checks, process spawn/kill, binary resolution |
| `mcp_client` | `crates/mcp_client/` | MCP protocol client over Streamable HTTP | McpClient trait, fetch_tools, call_tool, per-request connection |
| `services` | `crates/services/` | Domain types + business logic hub | AppService, AuthScopedAppService, DbService, TimeService, domain modules, SeaORM |
| `server_core` | `crates/server_core/` | HTTP infrastructure for LLM inference | SharedContext, DirectSse, fwd_sse, InferenceService, server args merge |
| `auth_middleware` | `crates/auth_middleware/` | Authentication/authorization middleware | AuthContext, MiddlewareError, JWT, session, CSRF, role hierarchy |
| `routes_app` | `crates/routes_app/` | API orchestration: all HTTP endpoint handlers | AuthScope extractor, ApiError, domain modules, OpenAPI/utoipa, ValidatedJson |
| `server_app` | `crates/server_app/` | Standalone HTTP server orchestration | ServeCommand, ServerHandle, graceful shutdown, live integration tests |
| `lib_bodhiserver` | `crates/lib_bodhiserver/` | Embeddable server library | AppServiceBuilder, BootstrapService, AppOptions, setup_app_dirs, re-exports |
| `lib_bodhiserver_napi` | `crates/lib_bodhiserver_napi/` | NAPI bindings for Node.js | BodhiServer class, @bodhiapp/app-bindings, Playwright E2E |
| `bodhi/src` | `crates/bodhi/src/` | Next.js 14 frontend (React) | @bodhiapp/ts-client, react-hook-form, zod, react-query, Vitest, MSW |
| `bodhi/src-tauri` | `crates/bodhi/src-tauri/` | Tauri desktop + container server | native feature flag, dual-mode, system tray, AppSetupError |
| `ci_optims` | `crates/ci_optims/` | Docker layer caching for CI | dependency pre-compilation, multi-stage Docker build |
| `xtask` | `xtask/` | Build automation and code generation | OpenAPI spec, TypeScript client generation, BodhiOpenAPIDoc |

Sub-module docs (load when working inside these modules):
- `crates/services/src/test_utils/CLAUDE.md` — Test utility infrastructure
- `crates/server_core/src/test_utils/CLAUDE.md` — HTTP test utilities
- `crates/lib_bodhiserver_napi/tests-js/CLAUDE.md` + `E2E.md` — Playwright E2E tests

## Shared Rust Conventions

### Module Organization
- `mod.rs` files: ONLY module declarations (`mod xxx;`) and re-exports (`pub use xxx::*;`). No trait definitions, error enums, structs, or implementation code.
- Domain modules follow `*_objs.rs` for types, `error.rs` for errors, `service.rs` for business logic.
- Reference implementation: `crates/services/src/auth/` module.

### Re-export Rules
- `services` re-exports all `errmeta` types — downstream crates import from `services::` only, never `errmeta::` directly.
- `services` re-exports `db::*` via `pub use db::*` — use `services::DbService` not `services::db::DbService`.
- `lib_bodhiserver` re-exports curated surface from `services`, `routes_app`, `server_app` for leaf crates.

## Cross-Crate Architecture Patterns

### Error Layer Separation
Three distinct error layers — never mix them:

1. **errmeta layer**: `AppError` trait, `ErrorType`, `IoError`, `EntityError` — zero framework deps
2. **services layer**: Domain errors (`TokenServiceError`, `McpError`, `ToolsetError`, `AuthContextError`) — all implement `AppError` via `#[derive(ErrorMeta)]`
3. **routes_app layer**: `ApiError` / `OpenAIApiError` / `ErrorBody` in `routes_app::shared` — HTTP response formatting

Flow: service error -> `AppError` -> `ApiError` (blanket `From<T: AppError>` auto-converts in handlers).

Middleware uses `MiddlewareError` (in `auth_middleware`) — has blanket `From<T: AppError>` impl.

IMPORTANT: `ApiError` is NOT in `services`. It was moved to `routes_app::shared`.

### AuthScope Extractor Flow
All API route handlers use `AuthScope` extractor (`routes_app::shared::auth_scope_extractor`):

1. Middleware populates `AuthContext` in request extensions
2. `AuthScope` extracts `AuthContext` + `Arc<dyn AppService>` from request
3. Creates `AuthScopedAppService` wrapping both
4. Handlers access auth-scoped sub-services: `.tokens()`, `.mcps()`, `.tools()`, `.users()`, `.data()`

**Rule**: Route handlers use `AuthScopedAppService`. Infrastructure (bootstrap, middleware, route composition) uses `AppService` directly.

Exception: handlers calling `RouterState::forward_request()` (LLM proxying) retain `State<Arc<dyn RouterState>>`.

### Service Initialization Order
Services must be built in dependency order (see `services/CLAUDE.md:146`):
1. TimeService -> 2. DbService -> 3. SettingService -> 4. AuthService -> 5. SessionService -> 6. TenantService -> 7. HubService/DataService/CacheService -> 8. ConcurrencyService/NetworkService -> 9-14. remaining services

### Multi-Tenant Transactions
All mutating `DbService` operations on tenant-scoped rows use `begin_tenant_txn(tenant_id)` from `DbCore` trait. On PostgreSQL this sets RLS via `SET LOCAL app.current_tenant_id`. On SQLite returns plain transaction.

### Time Handling
Never use `Utc::now()` directly. All timestamps through `TimeService`. Tests use `FrozenTimeService`.

### OpenAPI -> TypeScript Pipeline
After any API change: `cargo run --package xtask openapi` -> `make build.ts-client` -> frontend imports from `@bodhiapp/ts-client`.

## Shared Testing Conventions

### rstest Patterns
- Use `#[rstest]` for all Rust tests with `#[tokio::test]` for async
- `#[case]` for parameterized tests, `#[values]` for combinatorial, `#[fixture]` for shared setup
- Async tests: `#[rstest]` + `#[tokio::test]` + `#[anyhow_trace]`, return `-> anyhow::Result<()>`
- `#[awt]` only when `#[future]` fixture params are used

### Test File Organization
- Prefer `test_*.rs` sibling files declared via `#[cfg(test)] #[path = "test_<name>.rs"] mod test_<name>;` (Pattern A)
- Cross-handler tests use `mod.rs` declarations (Pattern B)
- For CRUD routes: `test_<handler>_crud.rs`, `test_<handler>_auth.rs`, `test_<handler>_<feature>.rs`
- Inline `#[cfg(test)] mod tests {}` acceptable for files under 500 lines
- Reference implementation: `crates/routes_app/src/mcps/` module

### Assertion Style
- `assert_eq!(expected, actual)` with `use pretty_assertions::assert_eq;`
- Error assertions via `.code()`, never message text
- Avoid `use super::*` in test modules — use explicit imports

### Test Infrastructure (services crate)
- `TestDbService`: wraps `DefaultDbService` with event broadcasting + `FrozenTimeService`
- `AppServiceStub`: builder-based full service composition for tests
- `SeaTestContext`: dual SQLite/PG test fixture
- `AuthContext` factories: `test_session()`, `test_api_token()`, `test_external_app()`
- Constants: `TEST_TENANT_ID`, `TEST_USER_ID`

### Test Boundaries
- `routes_app`: Single-turn endpoint tests via `tower::oneshot()`, no TCP listener
- `server_app`: Multi-turn workflows, real HTTP/TCP, real OAuth2 — `#[serial_test::serial(live)]`
- `lib_bodhiserver_napi/tests-js`: Playwright E2E with Page Object Model
