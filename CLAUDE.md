# BodhiApp — CLAUDE.md

**Companion docs** (load as needed):
- `crates/CLAUDE.md` — Crate index, shared Rust conventions, cross-crate patterns
- `MDFILES.md` — Documentation conventions for all *.md files
- Individual crate docs: `crates/<crate>/CLAUDE.md` for crate-specific details

## Development Commands

### Testing
- `make test` — Run all tests (backend, UI, NAPI)
- `make test.backend` — Rust backend tests (requires Docker for PostgreSQL; runs `cargo test` and `cargo test -p bodhi --features native`)
- `make test.ui` — Frontend tests (`cd crates/bodhi && npm install && npm test`)
- `make test.napi` — NAPI/Playwright E2E tests (`cd crates/lib_bodhiserver_napi && npm install && npm run test`)

### Building & Packaging
- `make ci.build` — Build Tauri desktop application
- `make build.ts-client` — Build TypeScript client package with tests
- `cd crates/bodhi && npm run build` — Build Next.js frontend
- `cd crates/lib_bodhiserver_napi && npm run build:release` — Build NAPI bindings

### Code Quality
- `make format` — Format all code (Rust, Node.js, Python)
- `make format.all` — Format and run Clippy fixes
- `cargo fmt --all` — Format Rust code only
- `cd crates/bodhi && npm run format` — Format frontend code
- `cd crates/bodhi && npm run lint` — Lint frontend code

### OpenAPI & Client Generation
- `cargo run --package xtask openapi` — Generate OpenAPI specification
- `cd ts-client && npm run generate` — Generate TypeScript client types
- Always regenerate after API changes: `cargo run --package xtask openapi && cd ts-client && npm run generate`

### Running the Application
- `cd crates/bodhi && npm run dev` — Start Next.js dev server (hot reload)
- `cd crates/bodhi/src-tauri && cargo tauri dev` — Run Tauri desktop app in dev mode
- `make run.app` — Run standalone HTTP server with dev configuration
- `cargo run --bin bodhi -- serve --port 1135` — Run server directly

### Docker
- `make docker.dev.cpu` — Build CPU Docker image (multi-platform AMD64/ARM64)
- `make docker.dev.cuda` — Build NVIDIA CUDA GPU image
- `make docker.dev.cpu.amd64` / `make docker.dev.cpu.arm64` — Platform-specific images
- `make docker.run.amd64` / `make docker.run.arm64` — Run locally built images
- `make docker.list` / `make docker.clean` — List/remove local images

### Release Management
- `make release.ts-client` — Tag release for @bodhiapp/ts-client
- `make release.app-bindings` — Tag release for @bodhiapp/app-bindings
- `make release.docker` / `make release.docker-dev` — Tag Docker image releases
- `make ci.ts-client-check` — Verify TypeScript client is in sync with OpenAPI spec
- `make docs.context-update` — Update AI documentation context symlinks

## Technology Stack

- **Backend**: Rust + Axum + SeaORM (SQLite dev/desktop, PostgreSQL production/Docker)
- **Frontend**: React + TypeScript + Next.js 14 + TailwindCSS + Shadcn UI
- **Desktop**: Tauri | **LLM**: llama.cpp | **Auth**: OAuth2 + JWT
- **API**: OpenAI-compatible endpoints | **Types**: OpenAPI -> TypeScript auto-generation

## Crate Dependency Chain

```
errmeta_derive (proc-macro)
       |
    errmeta (AppError, ErrorType, IoError, EntityError, impl_error_from!)
    /      \
llama_server_proc    mcp_client
       \                 /
        \               /
         services (ALL domain types + business logic)
        /          \
server_core         |
        \          /
         routes_app (ApiError, OpenAIApiError, middleware here)
             |
         server_app
             |
      lib_bodhiserver
      /             \
lib_bodhiserver_napi  bodhi/src-tauri
```

For crate purposes, keywords, and detailed index: see `crates/CLAUDE.md`.

## Layered Development Methodology

When implementing a feature spanning multiple crates, always work upstream-to-downstream:

1. **Upstream Rust crate first**: Change the most upstream crate affected. Run `cargo test -p <crate>`. Verify no regressions in upstream crates.
2. **Repeat downstream**: Move to next crate in the chain. Run cumulative tests.
3. **Continue through the chain**: `services` -> `server_core` -> `routes_app` -> `server_app` -> leaf crates.
4. **Full backend validation**: `make test.backend` after all Rust changes.
5. **Regenerate TypeScript types**: `make build.ts-client` to update frontend types.
6. **Frontend component tests**: Change UI in `crates/bodhi/src/`, using `@bodhiapp/ts-client` types. Run `cd crates/bodhi && npm run test`.
7. **E2E tests**: `make build.ui-rebuild` then update tests in `crates/lib_bodhiserver_napi/tests-js/`. Run `make test.napi`.
8. **Documentation**: Update crate-level `CLAUDE.md` / `PACKAGE.md` for each modified crate.

## Important Notes

### Development Guidelines
- Run `make test` before making changes to verify baseline
- Use `make format.all` to format and fix linting across all languages
- Frontend uses strict TypeScript — avoid `any` types, import types from `@bodhiapp/ts-client`
- NAPI bindings require Node.js >= 22

### Architectural Patterns
- **Time handling**: Use `TimeService` (never `Utc::now()`) — see `crates/services/src/db/time_service.rs`
- **Error handling**: service errors -> `ApiError` (in `routes_app::shared`) -> OpenAI-compatible responses. Auth context errors use `AuthContextError` (in `services`). Middleware errors use `MiddlewareError` (in `routes_app::middleware`). `ApiError` is NOT in `services`.
- **AuthScope extractor**: All route handlers use `AuthScopedAppService` via `AuthScope`. Infrastructure uses `AppService` directly. See `crates/CLAUDE.md` for details.
- **Imports**: Avoid `use super::*` in `#[cfg(test)]` modules — use explicit imports
- **Multi-tenant**: Mutating DB ops use `begin_tenant_txn(tenant_id)` for RLS on PostgreSQL
- **CRUD conventions**: See `crates/CLAUDE.md` "CRUD Convention Reference" for entity aliases, Request types, ValidatedJson, route handler patterns

### Testing Practices
- **rstest for all Rust tests**: `#[case]` for parameterized, `#[values]` for combinatorial, `#[fixture]` for shared setup
- **Test file organization**: Prefer `test_*.rs` sibling files via `#[cfg(test)] #[path = "test_<name>.rs"] mod test_<name>;`. Inline `mod tests` for files under 500 lines. Reference: `crates/routes_app/src/mcps/`
- **Assertions**: `assert_eq!(expected, actual)` with `pretty_assertions`. Error assertions via `.code()`, never message text
- **Determinism**: No if-else logic or try-catch in test code
- **UI tests**: Use `data-testid` with `getByTestId`. Do NOT add inline timeouts in component tests or Playwright tests (except ChatPage for model warm-up)

## Critical UI Development Workflow

**IMPORTANT: After UI changes, rebuild the embedded UI before testing:**

1. `make build.ui-clean` — Clean embedded UI build
2. `make build.ui` — Build Next.js + NAPI bindings
3. Or: `make build.ui-rebuild` — Combined clean + build

The application embeds the UI build. Changes to `crates/bodhi/src/` are NOT visible until rebuilt. For active development, use `cd crates/bodhi && npm run dev` for hot reload.

## Backwards Compatibility
- Do not plan for backwards compatibility unless specifically mentioned — BodhiApp prioritizes architectural improvement
- If you make changes to `crates/bodhi/src/`, run `make build.ui-rebuild` for Playwright tests to pick up UI updates
- Do not add inline timeouts in component tests — rely on defaults or fix the root cause
