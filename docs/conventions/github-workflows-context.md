# GitHub Workflows Context — CI/CD

Conventions and architecture of BodhiApp's GitHub Actions setup. The live source of truth is `.github/workflows/` and `.github/actions/` — this doc captures the *why*, not a per-file inventory (which drifts).

## Design Philosophy

1. **Makefile-first**: complex shell logic lives in Makefile targets, not inline YAML. Workflows call `make ci.*`.
2. **Reusable composite actions**: common setup steps extracted into `.github/actions/`.
3. **Fast vs comprehensive**: a fast Linux-only path for quick PR feedback; multiplatform matrices for release quality.
4. **Artifact hand-off**: build jobs upload binaries; downstream test jobs download them.

### Logic-placement hierarchy

Simple → inline YAML. Moderate → Makefile target (preferred). Complex → Python script in `scripts/` invoked from Makefile. Windows-specific → PowerShell in `scripts/`.

## Workflow Categories

Browse `.github/workflows/` for the current set. They fall into:

- **Build & test** — fast Linux build (primary CI on push/PR) and a multiplatform matrix (macOS aarch64, Linux x86_64, Windows x86_64) for `main`. Full suite: Rust + UI unit tests, NAPI bindings build, Playwright E2E against the prebuilt `bodhiserver_dev` binary, Codecov, TS-client sync check.
- **Playwright** — E2E run wiring (see `crates/lib_bodhiserver/tests-js/CLAUDE.md`).
- **Release** — desktop app (Tauri/DMG, tag `v*`), `@bodhiapp/ts-client` (tag `ts-client/v*`), `@bodhiapp/app-bindings` NAPI package (tag `bodhi-app-bindings/v*`), Docker images (standalone + multi-tenant, multiplatform), website deploy.

> Tag-trigger and platform details live in the workflow YAML headers — check `.github/workflows/<file>.yml` rather than trusting a copied list.

## Reusable Actions

All composite actions live in `.github/actions/` and follow shared patterns: `using: composite`, a `platform` input for cross-platform branches, `shell: bash` where portable. Current actions cover environment setup, model caching (HuggingFace), Rust toolchain, Node/npm, Playwright browsers, build-and-test with coverage, `bodhiserver_dev` build, NAPI build, TS-client check, and release-tag validation. Open `.github/actions/<name>/action.yml` for inputs/outputs.

## Environment & Secrets

**Env vars** (set in workflows): `CI=true`, `RUST_BACKTRACE=1`, `BODHI_EXEC_VARIANT=cpu`, `CI_DEFAULT_VARIANT=cpu`.

**Secrets**: `GH_PAT` (submodule access), `CODECOV_TOKEN`, `HF_TEST_TOKEN_*` (HuggingFace), `INTEG_TEST_*` (E2E OAuth credentials), npm publish tokens. Integration-test config: `INTEG_TEST_AUTH_URL`, `INTEG_TEST_AUTH_REALM`. The E2E credential set is documented in `crates/lib_bodhiserver/tests-js/CLAUDE.md`.

## Makefile CI Targets

| Target | Purpose |
|--------|---------|
| `ci.clean` | Clean all cargo packages |
| `ci.coverage` | Backend coverage via llvm-cov |
| `ci.ui` | Frontend tests with coverage |
| `ci.ts-client-check` | Verify TS client matches OpenAPI spec |
| `ci.build` | Tauri application build |

## Conventions

- 2-space YAML indentation; descriptive step names.
- Version-pin external actions; rely on dependabot for updates.
- Minimal secret exposure in logs; PAT only for submodule checkout; npm provenance on publish.
- Timeouts on every job; `fail-fast: false` for cross-platform matrices; `continue-on-error` only for non-critical steps (e.g. coverage upload).

## Caching

Rust via `Swatinem/rust-cache@v2`; Node via npm cache keyed on lockfile hashes; HuggingFace models with cross-OS archive; Playwright browsers keyed by version. Build artifacts retain ~1 day, test results ~7 days, release artifacts via GitHub releases.
