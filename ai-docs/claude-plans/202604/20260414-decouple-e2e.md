# Decouple E2E Tests from NAPI — `bodhiserver_dev` binary + live Vite proxy

**Status:** Shipped 2026-04-15 as a single squashed commit on `main` — see `git log` for the SHA.

## Context

Playwright E2E ran by embedding the Rust server into Node via `@bodhiapp/app-bindings` (NAPI). Each iteration meant rebuilding the Vite frontend into `crates/bodhi/out/` (so `include_dir!` had content), rebuilding NAPI bindings, and reloading the native `.node` artifact. Slow and noisy during day-to-day work on Rust or UI code.

`make app.run.live` already proved out the alternative: Vite serves the UI on `:3000`, the Rust server runs `cargo run --bin bodhi -- serve` with `BODHI_DEV_PROXY_UI=true`, and `/ui/*` is proxied to Vite (see `crates/routes_app/src/routes.rs:657-677` and `crates/routes_app/src/routes_proxy.rs`).

`lib_bodhiserver_napi` stays published on npm as `@bodhiapp/app-bindings` for external consumers, so its public API is untouched. Only our own E2E loop moves off it.

## What shipped

### 1. `lib_bodhiserver` feature flag `embed-ui` (default on)

`crates/lib_bodhiserver/Cargo.toml`:
- `default = ["embed-ui"]`, new `embed-ui = []` feature.
- New `[[bin]] bodhiserver_dev` with `required-features = ["test-utils"]` (needs `create_tenant_test`).
- Added deps for the bin: `tracing-appender`, `tracing-subscriber`, `chrono`, `anyhow`.

`crates/lib_bodhiserver/src/lib.rs`:
- `mod ui_assets` and the `pub use ui_assets::EMBEDDED_UI_ASSETS` re-export are gated behind `#[cfg(feature = "embed-ui")]`.

`crates/lib_bodhiserver/build.rs`:
- Early-return `if env::var_os("CARGO_FEATURE_EMBED_UI").is_none()` so `--no-default-features` builds skip the npm/Vite pipeline entirely. `capture_git_sha()` still runs unconditionally.

Downstream impact: `lib_bodhiserver_napi` and `bodhi/src-tauri` keep default features → keep embedding. Only the dev bin is built with `--no-default-features --features test-utils`.

### 2. `crates/lib_bodhiserver/src/bin/bodhiserver_dev.rs`

Self-contained env-var-driven binary. **No new code in `lib_bodhiserver/src/lib.rs`** per project preference (test concerns stay in the bin, library surface stays clean).

Responsibilities:
1. Require `BODHI_HOME` (fail fast if missing — tests own the dir).
2. Read host/port from `BODHI_HOST`/`BODHI_PORT`, default `localhost:1135`.
3. Force `BODHI_DEV_PROXY_UI=true` via `env::set_var` before bootstrap so the routes layer always picks the proxy router (works because `cargo build/run` keeps `debug_assertions` on, which `SettingService::get_dev_env` requires).
4. Build `AppOptionsBuilder` from env vars: system settings (`BODHI_ENV_TYPE`, `BODHI_APP_TYPE`, `BODHI_VERSION`, `BODHI_AUTH_URL`, `BODHI_AUTH_REALM`, `BODHI_DEPLOYMENT`), app settings (`BODHI_EXEC_LOOKUP_PATH`, `BODHI_LOG_LEVEL`, `BODHI_LOG_STDOUT`), optional tenant (`BODHI_CLIENT_ID` + `BODHI_CLIENT_SECRET` + `BODHI_TENANT_NAME` + `BODHI_APP_STATUS` + `BODHI_CREATED_BY`).
5. Run `setup_app_dirs` → `setup_bootstrap_service` → `setup_logs` (helper copied verbatim from the old `lib_bodhiserver_napi/src/server.rs:267-329`) → `build_app_service` → `ensure_tenant` → `ServeCommand::ByParams::get_server_handle(app_service, None)`.
6. Print `bodhiserver_dev: listening on <url>` on stdout for the JS launcher to parse, then `tokio::signal::ctrl_c()` for graceful shutdown.

### 3. `tests-js` migration

Moved with `git mv` from `lib_bodhiserver_napi/` to `lib_bodhiserver/`:
- `tests-js/` (whole tree)
- `test-mcp-oauth-server/`, `test-mcp-auth-server/`, `test-oauth-app/`
- `playwright.config.mjs`, `vitest.config.mjs`, `biome.json`, `jsconfig.json`
- `scripts/download-extension.mjs` (its `../tests-js/extension` relative path stayed valid)

`crates/lib_bodhiserver_napi/package.json` was stripped to `@napi-rs/cli` + `rimraf` + the napi build/publish scripts. `crates/lib_bodhiserver/package.json` is the new private workspace package (`@bodhiapp/lib-bodhiserver-tests`) carrying Playwright/vitest/biome and all `e2e:server:*` scripts.

JS launcher rewrites:
- `tests-js/utils/bodhi-app-server.mjs` — `BodhiAppServer` now `spawn`s the prebuilt binary at `<repo>/target/debug/bodhiserver_dev`, parses the readiness line, then pings `/ping` until ready. `stopServer` sends `SIGTERM` and waits for `exit`.
- `tests-js/test-helpers.mjs` — dropped `loadBindings()`/`createTestServer()`; added `BODHISERVER_DEV_BIN`, `buildEnvFromConfig(options)` (returns flat env map mirroring `NapiAppOptions`), `waitForListening(child)`, `pingUntilReady(url)`. Existing helpers (`createTempDir`, `getHfHomePath`, `randomPort`, `resetDatabase`, `waitForSPAReady`, `waitForRedirect`, `getCurrentPath`, `getLocalNetworkIP`, `sleep`) preserved.
- `tests-js/scripts/start-shared-server.mjs` — same flip: builds the env map via `buildEnvFromConfig`, spawns the binary, waits for the `listening on` line, forwards `SIGTERM`/`SIGINT`.

`tests-js/fixtures.mjs` and `utils/db-config.mjs` unchanged — `autoResetDb` still POSTs `/dev/db-reset`, the tenants table is preserved by `reset_all_tables` (see `crates/services/src/db/default_service.rs:90-176`), so seeded credentials survive across tests.

### 4. Vite proxy timing fix (the gotcha that ate a session)

The proxy router in `routes_app/src/routes.rs:657-677` was already correct (`build_ui_proxy_router("http://localhost:3000")` plus WebSocket support in `routes_proxy.rs`). What broke E2E was a *startup race*:

- Vite serves `index.html` with `<script type="module" src="/ui/@vite/client">` injected. The browser fetches that script first.
- Playwright's webServer treated Vite as "ready" the moment `http://localhost:3000/ui/` returned 200.
- But Vite reports HTTP-ready before it has compiled `/@vite/client` and before it has pre-bundled the SPA dep graph (react, tanstack-router, etc.).
- First test navigation → browser hits `/ui/@vite/client` → 404 → script error → React never mounts → blank page → test times out waiting for the Login button.

Two complementary, minimal fixes:

`crates/bodhi/vite.config.ts`:
```ts
server: {
  ...
  warmup: { clientFiles: ['./src/main.tsx', './src/routeTree.gen.ts'] },
},
```

`crates/lib_bodhiserver/playwright.config.mjs`:
```js
{
  command: 'npm run e2e:server:vite',
  url: 'http://localhost:3000/ui/@vite/client',  // was '/ui/'
  reuseExistingServer: false,
  timeout: 180000,
},
```

That alone made the suite reliable. Other things tried that turned out to be red herrings — **do not re-introduce them**:
- Stripping `Origin`/`Referer`/`Sec-Fetch-*` headers in the proxy (broke other endpoints by triggering Vite's SPA fallback).
- Rewriting `Origin` to the backend host in the proxy (no measurable effect once warmup was correct).
- Adding `server.cors.origin` allow-lists or `server.hmr: false` in vite.config (irrelevant once warmup was correct).

### 5. Makefile + docs

Top-level `Makefile`:
- New `build.dev-server` → `cargo build --no-default-features --features test-utils -p lib_bodhiserver --bin bodhiserver_dev`.
- New `test.e2e` / `test.e2e.standalone` / `test.e2e.multi_tenant` (depend on `build.dev-server`, then `cd crates/lib_bodhiserver && npm install && npm run test:playwright[:project]`).
- `test.napi` removed; `test` aggregates `test.backend test.ui test.e2e`.
- `format` and `setup.worktree` paths updated to `crates/lib_bodhiserver/`.
- `test.extension-download` now invokes the new `crates/lib_bodhiserver/Makefile`.

Per-crate Makefiles:
- `crates/lib_bodhiserver_napi/Makefile` reduced to npm-publish/build + `cargo test -p lib_bodhiserver_napi`.
- `crates/lib_bodhiserver/Makefile` (new) hosts `test.e2e[.project]`, `format`, `download-extension`, `clean`.

Docs touched:
- `CLAUDE.md` (root) — updated commands and the layered-development checklist.
- `crates/CLAUDE.md` — crate index entries flipped (`lib_bodhiserver` now owns the E2E suite; `lib_bodhiserver_napi` is npm-publish only). Test-boundary line updated.
- `crates/lib_bodhiserver/CLAUDE.md` — added `bodhiserver_dev` + `tests-js/` sections, feature flag table, build commands.
- `crates/lib_bodhiserver/.gitignore` (new) — keeps `**/node_modules`, `playwright-report`, `test-results`, etc. out of the repo.
- `crates/lib_bodhiserver_napi/CLAUDE.md` — note pointing readers at the new tests-js location.
- `crates/lib_bodhiserver/tests-js/CLAUDE.md` — replaced "NAPI bindings embed the Rust server" with "spawns the prebuilt `bodhiserver_dev` binary"; updated commands to run from `crates/lib_bodhiserver/`.
- `.github/PACKAGE.md` — sample test-reporter snippet now shows the Playwright JUnit path under `crates/lib_bodhiserver/test-results/`.

### 6. CI workflows

Three GitHub Actions workflows depended on `crates/lib_bodhiserver_napi` for E2E. All flipped to the new layout:

- `.github/workflows/playwright.yml` — drops the "Build NAPI bindings" + "Run NAPI binding tests" steps from the E2E job; now does `cargo build --no-default-features --features test-utils -p lib_bodhiserver --bin bodhiserver_dev`, sets up Playwright in `crates/lib_bodhiserver`, and uploads results from there.
- `.github/workflows/build.yml` — `playwright-tests` job downloads the prebuilt `bodhiserver-dev-${target}` artifact instead of NAPI bindings; npm install + Playwright run + result upload all happen in `crates/lib_bodhiserver`. Vitest "NAPI binding tests" step removed (the vitest config moved with the suite, and there are no JS unit tests for the napi crate any more).
- `.github/workflows/build-multiplatform.yml` — same playwright-tests rewrite as `build.yml`, matrix-ed across mac/linux/windows.

New composite action `.github/actions/bodhiserver-dev-build/action.yml` runs the cargo build, copies the (possibly `.exe`-suffixed) binary into a staging dir, and uploads it as artifact `bodhiserver-dev-${target}`. Both `build.yml` and `build-multiplatform.yml` invoke this action right after `napi-build` so each build job produces both artifacts.

`.github/actions/setup-playwright/action.yml` — `working-directory` default flipped from `crates/lib_bodhiserver_napi` to `crates/lib_bodhiserver`. Callers can still override.

`.github/actions/setup-node/action.yml` — npm cache key now hashes `crates/lib_bodhiserver/package-lock.json` *and* `crates/lib_bodhiserver_napi/package-lock.json` (the latter is still needed by `publish-app-bindings.yml`).

`publish-app-bindings.yml` and `.github/actions/napi-build/action.yml` are unchanged — they still build/publish `@bodhiapp/app-bindings` from `crates/lib_bodhiserver_napi/`.

## Verification (as-run)

1. `cargo check -p lib_bodhiserver --no-default-features` ✅ (no npm invoked).
2. `cargo build --no-default-features --features test-utils -p lib_bodhiserver --bin bodhiserver_dev --locked` ✅.
3. `cargo check -p lib_bodhiserver_napi` ✅ (NAPI publish path intact).
4. Manual smoke: dev binary `/ping` returned 200, proxy logged `proxying the ui to localhost:3000`, SIGTERM clean shutdown.
5. Playwright E2E (full webServer-launched stack, `reuseExistingServer: false` everywhere) — 10 diverse standalone specs (api-models-extras, api-models-prefix, request-access-version-validation) ran 10/10 green in ~2 minutes. Single-spec re-run after CI changes also green.

## Critical files (final list)

**New:**
- `crates/lib_bodhiserver/src/bin/bodhiserver_dev.rs`
- `crates/lib_bodhiserver/{Makefile, package.json, .gitignore}`
- `.github/actions/bodhiserver-dev-build/action.yml`

**Modified:**
- `crates/lib_bodhiserver/Cargo.toml` (feature flag, bin, deps)
- `crates/lib_bodhiserver/src/lib.rs` (gate ui_assets)
- `crates/lib_bodhiserver/build.rs` (gate frontend build)
- `crates/lib_bodhiserver/playwright.config.mjs` (vite readiness URL)
- `crates/bodhi/vite.config.ts` (warmup)
- `Makefile`, `CLAUDE.md`, `crates/CLAUDE.md`
- `crates/lib_bodhiserver_napi/{CLAUDE.md, Makefile, package.json}` (stripped of E2E concerns)
- `.github/workflows/{playwright.yml, build.yml, build-multiplatform.yml}`
- `.github/actions/{setup-node, setup-playwright}/action.yml`
- `.github/PACKAGE.md`

**Moved:**
- `crates/lib_bodhiserver_napi/{tests-js,test-mcp-*-server,test-oauth-app,playwright.config.mjs,vitest.config.mjs,biome.json,jsconfig.json,scripts/download-extension.mjs}` → `crates/lib_bodhiserver/...`

**Rewritten in-place under new path:**
- `tests-js/utils/bodhi-app-server.mjs` (process spawn)
- `tests-js/test-helpers.mjs` (env builder + readiness probes)
- `tests-js/scripts/start-shared-server.mjs` (cargo-spawn variant)

## Notes for future archaeologists

- The dev bin only works in debug mode because the proxy router is gated on `debug_assertions` via `SettingService::get_dev_env`. If you ever build it `--release`, proxy mode silently disables and `/ui/*` 404s.
- `create_tenant_test` lives behind `services/test-utils`. That's why the bin's `required-features = ["test-utils"]` — do not flip this to a default feature; it'd leak test-only DB methods into production builds.
- `dev_db_reset_handler` (`routes_app/src/routes_dev.rs:70`) deliberately leaves `tenants` and `tenants_users` (creator) intact across resets. The seeded OAuth tenant from `start-shared-server.mjs` survives every test, which is why the SPA stays in `Ready` instead of redirecting to `/ui/setup`.
- If the SPA ever starts depending on a *different* Vite-internal endpoint than `/@vite/client`, update the Playwright vite webServer URL to wait on whichever endpoint is now first-fetched. The pattern is: pick whatever the cold-start race-loser is.
- `bodhiserver-dev-build` action stages the binary in `bodhiserver-dev-out/` before upload because `actions/upload-artifact@v4` would otherwise drag the entire `target/debug/` tree. Don't "simplify" by uploading `target/debug/` directly.
- The cache key in `.github/actions/setup-node/action.yml` intentionally lists *both* `crates/lib_bodhiserver/package-lock.json` and `crates/lib_bodhiserver_napi/package-lock.json` — the former is the active E2E suite, the latter still drives `publish-app-bindings.yml`.
