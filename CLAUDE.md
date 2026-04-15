# BodhiApp — CLAUDE.md

**Companion docs** (load as needed):
- `crates/CLAUDE.md` — Crate index, shared Rust conventions, cross-crate patterns
- `MDFILES.md` — Documentation conventions for all *.md files
- Individual crate docs: `crates/<crate>/CLAUDE.md` for crate-specific details

**IMPORTANT — Security testing:** Do read `ai-docs/func-specs/security/security.md` before performing any security assessment, it documents known accepted risks, by-design architectural decisions, and previously remediated vulnerabilities, to avoid repeat reporting of known issues.

## Development Commands

### Testing
- `make test` — Run all tests (backend, UI, E2E)
- `make test.backend` — Rust backend tests (requires Docker for PostgreSQL; runs `cargo test` and `cargo test -p bodhi --features native`)
- `make test.ui` — Frontend tests (`cd crates/bodhi && npm install && npm test`)
- `make test.e2e` — Playwright E2E (`make build.dev-server` then `cd crates/lib_bodhiserver && npm install && npm run test:playwright`)

### Building & Packaging
- `make ci.build` — Build Tauri desktop application
- `make build.ts-client` — Build TypeScript client package with tests
- `cd crates/bodhi && npm run build` — Build Vite frontend
- `cd crates/lib_bodhiserver_napi && npm run build:release` — Build NAPI bindings (external @bodhiapp/app-bindings npm package)
- `make build.dev-server` — Build `bodhiserver_dev` binary used by E2E suite (skips UI embed)

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
- `cd crates/bodhi && npm run dev` — Start Vite dev server (hot reload, port 3000)
- `cd crates/bodhi/src-tauri && cargo tauri dev` — Run Tauri desktop app in dev mode
- `make app.run` — Run standalone HTTP server against the embedded UI
- `make app.run.live` — Run standalone HTTP server with live Vite proxy (HMR; no UI rebuild needed)
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
- **Frontend**: React + TypeScript + Vite + TanStack Router + TanStack Query v5 + TailwindCSS + Shadcn UI
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
7. **E2E tests**: `make build.dev-server` then update tests in `crates/lib_bodhiserver/tests-js/`. Run `make test.e2e`.
8. **Documentation**: Update crate-level `CLAUDE.md` / `PACKAGE.md` for each modified crate.

## Important Notes

- Frontend uses strict TypeScript — avoid `any` types, import types from `@bodhiapp/ts-client`
- NAPI bindings require Node.js >= 22
- For architectural patterns, testing conventions, and cross-crate rules, see `crates/CLAUDE.md`

## UI Development Workflow

Three dev loops depending on what you're validating:

1. **Active UI iteration** — `cd crates/bodhi && npm run dev` (Vite on 3000; pure frontend work, MSW-mocked backend).
2. **Full stack with live UI** — `make app.run.live` or `make test.e2e` spawn `bodhiserver_dev` which proxies `/ui/*` to Vite. UI changes reload via HMR; no rebuild needed. This is the fast path for iterating on Rust + UI together and for the Playwright suite.
3. **Embedded-bundle validation** — `make build.ui-rebuild` (clean + build) is only required when you need to exercise the production embed path: Tauri desktop (`cargo tauri dev`, `make build.native`), Docker images, or the `bodhi` binary with the `native` feature. The Vite output under `crates/bodhi/out/` is baked into the binary via `include_dir!`; without a rebuild, these embedded modes serve a stale UI.

**Frontend architecture**: Vite + TanStack Router (file-based routing in `src/routes/`) + TanStack Query v5 (hooks organized in `src/hooks/<domain>/`). See `crates/bodhi/src/CLAUDE.md` for details.

## Backwards Compatibility
- Do not plan for backwards compatibility unless specifically mentioned — BodhiApp prioritizes architectural improvement
