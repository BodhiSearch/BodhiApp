# Phase 6: E2E & Infrastructure

Cross-cutting changes spanning Docker/CI, Playwright E2E tests (dual-DB), OpenAPI/frontend updates, server bootstrap, and workspace cleanup.

## Docker/CI

- `docker-compose-test-deps.yml` (new) -- PostgreSQL service for integration tests
- `.cargo/config.toml` (modified) -- RUST_MIN_STACK increase
- `Makefile` (modified) -- New test.deps.up/down targets, test.backend updates
- `.github/workflows/build.yml` (modified) -- CI updates for PostgreSQL

## Playwright E2E

- `playwright.config.mjs` (modified) -- Dual-DB projects (sqlite:51135, postgres:41135)
- `tests-js/fixtures.mjs` (modified) -- Shared server URL fixture with DB-aware ports
- `tests-js/scripts/start-shared-server.mjs` (modified) -- Server startup with DB config
- `tests-js/utils/db-config.mjs` (new) -- DB configuration utility for E2E tests
- `tests-js/.env.test.example` (new) -- E2E test env template
- `tests-js/E2E.md` (modified) -- Updated E2E docs for dual-DB
- `tests-js/test-helpers.mjs` (modified) -- Updated test helpers

## E2E Spec Modifications (18 files)

- `specs/api-models/*.spec.mjs` (4 files, modified) -- Updated for dual-DB fixture
- `specs/chat/*.spec.mjs` (3 files, modified) -- Updated fixtures
- `specs/mcps/*.spec.mjs` (5 files, modified) -- Updated for new MCP types
- `specs/models/*.spec.mjs` (2 files, modified) -- Updated
- `specs/oauth/*.spec.mjs` (2 files, modified) -- Updated
- `specs/tokens/*.spec.mjs` (1 file, modified) -- Updated
- `specs/toolsets/*.spec.mjs` (2 files, modified) -- Updated

## API & Frontend

- `openapi.json` (modified) -- Updated schema for new enum types
- `ts-client/src/openapi-typescript/openapi-schema.ts` (modified) -- Regenerated types
- `ts-client/src/types/types.gen.ts` (modified) -- Regenerated types
- `crates/bodhi/src/` (multiple UI files, modified) -- Updated for snake_case enum values (e.g., "pre-registered" to "pre_registered")

## Server/Lib

- `crates/lib_bodhiserver/Cargo.toml` (modified) -- Added sea-orm-migration dep
- `crates/lib_bodhiserver/src/app_service_builder.rs` (modified) -- DefaultDbService bootstrap replaces SqliteDbService
- `crates/lib_bodhiserver/src/error.rs` (modified) -- Added SeaORM error variant
- `crates/lib_bodhiserver_napi/` (multiple files, modified) -- NAPI binding updates

## Cargo Workspace

- `Cargo.toml` (root, modified) -- Workspace member updates
- `Cargo.lock` (modified) -- Dependency resolution updates

## Deleted Infrastructure

- `crates/services/migrations/` (28 files, deleted) -- Old sqlx migrations removed
