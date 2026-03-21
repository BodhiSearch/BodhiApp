# Infrastructure Review (CI, Docker, Makefile, Cargo.toml)

## Files Reviewed
- `.github/workflows/build.yml` -- Added PostgreSQL service container for CI
- `Cargo.toml` (workspace root) -- Added `any` feature to sqlx, changed `tower-sessions-sqlx-store` to `default-features = false`
- `Cargo.lock` -- Added `serial_test` dependency
- `Makefile` -- Added `test.deps.up`/`test.deps.down` targets, `test.backend` now depends on `test.deps.up`
- `docker-compose-test-deps.yml` -- NEW file, PostgreSQL container for local development
- `crates/CLAUDE.md` -- NEW file, shared Rust conventions document
- `crates/services/Cargo.toml` -- Added `postgres` feature to sqlx and tower-sessions-sqlx-store, added `serial_test`
- `crates/services/.env.test` -- NEW file (gitignored), contains `INTEG_TEST_SESSION_PG_URL`
- `crates/ci_optims/Cargo.toml` -- Inherits workspace `tower-sessions-sqlx-store` with `default-features = false`

## Findings

### Finding 1: CI PostgreSQL service is provisioned but no tests run against it
- **Priority**: Critical
- **File**: `.github/workflows/build.yml`
- **Location**: `jobs.build-and-test.services.postgres` and `.github/actions/build-only/action.yml`
- **Issue**: The CI workflow adds a PostgreSQL service container (lines 27-39) to the `build-and-test` job. However, that job only runs `ci.build-only` (via the `build-only` action), which executes `cargo build` commands -- NOT `cargo test`. The PostgreSQL container is provisioned, consuming CI resources and adding startup time, but is never used. No `cargo test` command runs in the `build-and-test` job.
- **Recommendation**: Either (a) add a test step to the CI workflow that actually runs `cargo test` with the `INTEG_TEST_SESSION_PG_URL` environment variable set, or (b) remove the PostgreSQL service container if CI is build-only. Currently this is dead infrastructure.
- **Rationale**: Wasted CI resources. Each workflow run provisions a PostgreSQL container that sits idle. More importantly, the PostgreSQL integration tests are not validated in CI at all.

### Finding 2: INTEG_TEST_SESSION_PG_URL not set in CI environment
- **Priority**: Critical
- **File**: `.github/workflows/build.yml`
- **Location**: Entire workflow -- missing env var
- **Issue**: The test code in `crates/services/src/test_session_service.rs` reads `INTEG_TEST_SESSION_PG_URL` from either `.env.test` (local) or the environment. In CI, no `.env.test` file exists (it is gitignored) and no `INTEG_TEST_SESSION_PG_URL` environment variable is set anywhere in the workflow. If `cargo test` were added to CI, the postgres test cases would fail with a panic: `INTEG_TEST_SESSION_PG_URL must be set for postgres integration tests`.
- **Recommendation**: When tests are added to CI, set `INTEG_TEST_SESSION_PG_URL=postgres://bodhi_test:bodhi_test@localhost:54320/bodhi_sessions` as an environment variable in the test step. This can be added as a job-level or step-level `env` entry.
- **Rationale**: The `.env.test` file is gitignored (`.gitignore` line 179 only excepts `.env.test.example`, not `.env.test`). CI must provide this through environment variables.

### Finding 3: crates/services/.env.test is not tracked by git -- no .env.test.example provided
- **Priority**: Important
- **File**: `crates/services/.env.test`
- **Location**: N/A (file is gitignored)
- **Issue**: The `.env.test` file for the services crate contains `INTEG_TEST_SESSION_PG_URL` and is gitignored. Other crates in the project follow the convention of providing a `.env.test.example` file that IS tracked (e.g., `crates/auth_middleware/tests/.env.test.example`, `crates/server_app/tests/resources/.env.test.example`, `crates/lib_bodhiserver_napi/tests-js/.env.test.example`). The services crate does not have a `.env.test.example`.
- **Recommendation**: Add a `crates/services/.env.test.example` file with the template `INTEG_TEST_SESSION_PG_URL=postgres://bodhi_test:bodhi_test@localhost:54320/bodhi_sessions` so developers know what environment variables are needed. This follows the existing convention in the codebase.
- **Rationale**: New developers cloning the repo will not know this env var is needed for running postgres tests. The example file provides discoverability.

### Finding 4: Health check inconsistency between CI and docker-compose
- **Priority**: Nice-to-have
- **File**: `.github/workflows/build.yml` (lines 35-39) vs `docker-compose-test-deps.yml` (lines 9-13)
- **Location**: Health check configuration
- **Issue**: The CI health check uses a bare `pg_isready` without specifying user or database (`--health-cmd pg_isready`), while the docker-compose health check uses `pg_isready -U bodhi_test -d bodhi_sessions`. Both work because `pg_isready` defaults to checking if the server accepts connections, but the docker-compose version is more precise. The CI health check interval is 10s vs 5s in docker-compose.
- **Recommendation**: Align the health check commands for consistency. Use `pg_isready -U bodhi_test -d bodhi_sessions` in CI as well. Consider matching the interval (5s is fine for both).
- **Rationale**: Consistency reduces confusion. The more specific health check in docker-compose validates that the correct user and database are accessible, not just that PostgreSQL is running.

### Finding 5: test.backend now requires Docker -- breaking change for developers
- **Priority**: Important
- **File**: `Makefile` (line 54)
- **Location**: `test.backend: test.deps.up`
- **Issue**: `test.backend` now has `test.deps.up` as a prerequisite, which runs `docker compose -f docker-compose-test-deps.yml up -d --wait`. This means running `make test.backend` requires Docker to be installed and running. Previously, backend tests could run without Docker. This is a breaking change for the developer workflow. Developers who do not have Docker installed or running will get an error when trying to run backend tests.
- **Recommendation**: Consider one of: (a) Document the Docker requirement prominently (README, CLAUDE.md). (b) Make `test.deps.up` resilient to Docker not being available (warn and continue). (c) Provide a `test.backend.no-docker` target that skips postgres tests. (d) Gate the postgres tests behind a feature flag or env var so they are skipped when PostgreSQL is not available.
- **Rationale**: The project root CLAUDE.md documents `make test.backend` as the standard command for backend tests. Silently adding a Docker dependency changes the developer experience.

### Finding 6: test.deps.down not called after test.backend
- **Priority**: Nice-to-have
- **File**: `Makefile`
- **Location**: `test.backend` target (line 54-56)
- **Issue**: `test.backend` starts PostgreSQL via `test.deps.up` but never calls `test.deps.down` after tests complete. The PostgreSQL container will keep running indefinitely after tests finish. While the `--wait` flag on `up` is idempotent (safe to run repeatedly), leaving containers running consumes resources.
- **Recommendation**: Either (a) add cleanup as a final step in `test.backend` (but this complicates error handling), (b) add a `test.all` target that calls `test.deps.down` at the end, or (c) document that developers should run `make test.deps.down` when done. The docker-compose `down -v` is already provided, so option (c) is reasonable.
- **Rationale**: Developer machines accumulate running containers. Not critical since `test.deps.up` is idempotent and developers likely know to stop containers, but worth documenting.

### Finding 7: ci_optims crate gets tower-sessions-sqlx-store with no backend features
- **Priority**: Important
- **File**: `Cargo.toml` (workspace, line 141) and `crates/ci_optims/Cargo.toml` (line 89)
- **Location**: `tower-sessions-sqlx-store` dependency configuration
- **Issue**: The workspace `Cargo.toml` now sets `tower-sessions-sqlx-store = { version = "0.15.0", default-features = false }`. The `ci_optims` crate uses `tower-sessions-sqlx-store = { workspace = true }` without adding any features. This means `ci_optims` compiles `tower-sessions-sqlx-store` with NO backend features (no sqlite, no postgres). Previously, when default features were enabled, this crate would pre-compile the sqlite backend. The `services` crate correctly adds `features = ["sqlite", "postgres"]` in its own Cargo.toml, but `ci_optims` does not.
- **Recommendation**: Update `crates/ci_optims/Cargo.toml` to add explicit features: `tower-sessions-sqlx-store = { workspace = true, features = ["sqlite", "postgres"] }`. This ensures the CI layer cache pre-compiles both backends.
- **Rationale**: The purpose of `ci_optims` is to pre-compile all heavy dependencies for Docker layer caching. Without the features, the tower-sessions-sqlx-store compilation in `ci_optims` does not cover the sqlite or postgres backends, meaning they must be compiled from scratch when building the actual `services` crate, negating the caching benefit for this dependency.

### Finding 8: sqlx `any` feature added at workspace level affects all crates
- **Priority**: Nice-to-have
- **File**: `Cargo.toml` (workspace, line 121)
- **Location**: `sqlx = { version = "0.8.6", features = ["any"] }`
- **Issue**: The `any` feature is added to sqlx at the workspace level. Any crate that depends on `sqlx = { workspace = true }` without specifying its own features inherits the `any` feature. This includes `ci_optims`, and potentially other crates that use sqlx. The `any` feature installs the `AnyPool`/`AnyConnection` driver abstraction, which adds compile time and a small runtime overhead. Currently, only the `services` crate actually uses `AnyPool` (in `session_store.rs`).
- **Recommendation**: Consider moving the `any` feature to only the crates that need it (i.e., `crates/services/Cargo.toml` already has `sqlx = { workspace = true, features = ["chrono", "runtime-tokio", "sqlite", "postgres"] }` -- just add `"any"` there) and remove it from the workspace level. However, since all crates unify features through Cargo's feature unification anyway when built together, the practical impact is minimal.
- **Rationale**: Best practice is to declare features where they are needed. In a workspace build, Cargo unifies features across all crates, so the practical impact is zero when building the full workspace. However, building individual crates in isolation (e.g., `cargo test -p objs`) would unnecessarily compile the `any` driver.

### Finding 9: Port 54320 could conflict with existing PostgreSQL installations
- **Priority**: Nice-to-have
- **File**: `docker-compose-test-deps.yml` (line 9), `.github/workflows/build.yml` (line 34)
- **Location**: Port mapping `54320:5432`
- **Issue**: Port 54320 is chosen as the host port to avoid conflict with the standard PostgreSQL port 5432. However, if a developer runs a second PostgreSQL instance on 54320, or if another project uses the same convention, there could be a conflict. The port choice is reasonable and non-standard, making conflicts unlikely.
- **Recommendation**: No action needed. Port 54320 is a reasonable choice. The port is consistent between CI and docker-compose. If conflicts arise, developers can modify their local `docker-compose-test-deps.yml`.
- **Rationale**: Low risk. Documenting the port choice is sufficient.

### Finding 10: Hardcoded credentials acceptable for test environments
- **Priority**: Nice-to-have
- **File**: `docker-compose-test-deps.yml`, `.github/workflows/build.yml`
- **Location**: `POSTGRES_USER: bodhi_test`, `POSTGRES_PASSWORD: bodhi_test`
- **Issue**: Credentials are hardcoded in both docker-compose and CI configuration. These are test-only credentials for a database that is ephemeral (no persistent volumes, `down -v` removes data). The database is only accessible on localhost.
- **Recommendation**: Acceptable as-is for test infrastructure. No production data is ever stored. The `.env.test` file containing the URL with credentials is properly gitignored.
- **Rationale**: Test databases with ephemeral data and localhost-only access do not need secret management.

### Finding 11: serial_test is a regular workspace dependency, not dev-dependency
- **Priority**: Nice-to-have
- **File**: `Cargo.toml` (workspace, line 119)
- **Location**: `serial_test = "3.2.0"` in `[workspace.dependencies]`
- **Issue**: `serial_test` is listed in `[workspace.dependencies]` at the workspace level, which is fine -- it is a declaration, not an actual dependency. The `services` crate correctly pulls it in as a `[dev-dependencies]` entry (line 79 of `crates/services/Cargo.toml`). This is the correct pattern.
- **Recommendation**: No action needed. This is correctly structured.
- **Rationale**: Workspace dependency declarations are just templates; actual dependency scope is determined by each crate's own Cargo.toml.

### Finding 12: CLAUDE.md documentation -- good addition but narrow reference scope
- **Priority**: Nice-to-have
- **File**: `crates/CLAUDE.md`
- **Location**: Entire file
- **Issue**: The new `crates/CLAUDE.md` documents shared Rust conventions (mod.rs organization rules) and references `session_service/` as the canonical example. This is a useful addition. However, it only covers one convention (module organization). The file could be expanded over time to cover other shared conventions.
- **Recommendation**: The current content is good. Consider expanding over time with other cross-crate conventions as they emerge. The reference to `session_service/` as the canonical multi-file module layout is appropriate.
- **Rationale**: Documentation of conventions improves codebase consistency.

### Finding 13: Postgres test cases will panic-fail if PostgreSQL is unavailable
- **Priority**: Important
- **File**: `crates/services/src/test_session_service.rs` (lines 18-19)
- **Location**: `pg_url()` function
- **Issue**: The `pg_url()` function calls `.expect("INTEG_TEST_SESSION_PG_URL must be set for postgres integration tests")`, which will panic and fail the test suite if the env var is not set and `.env.test` is missing. Every test function uses `#[case::postgres("postgres")]` alongside `#[case::sqlite("sqlite")]`. When running `cargo test -p services` without PostgreSQL running, all 7 postgres test cases will panic with a hard failure. The sqlite test cases are self-contained (use temp directories) and will pass, but the postgres failures will cause the overall `cargo test` to report failures.
- **Recommendation**: Consider making the postgres test cases conditional -- either (a) skip them when `INTEG_TEST_SESSION_PG_URL` is not set (return early or use `#[ignore]` with a runtime check), or (b) clearly document that PostgreSQL must be running before running tests. Option (a) is more developer-friendly and allows `cargo test -p services` to work without Docker.
- **Rationale**: This is the root cause of Finding 5. If postgres tests could gracefully skip when PostgreSQL is unavailable, `test.backend` would not strictly require Docker. Developers could opt-in to postgres tests.

## Summary
- Total findings: 13 (Critical: 2, Important: 4, Nice-to-have: 7)

### Critical Issues
1. **Finding 1**: PostgreSQL service in CI is dead infrastructure -- provisioned but never used (no tests run in that job)
2. **Finding 2**: `INTEG_TEST_SESSION_PG_URL` not set in CI -- if tests are added, postgres tests will fail

### Important Issues
3. **Finding 3**: No `.env.test.example` for discoverability
4. **Finding 5**: `make test.backend` now requires Docker -- undocumented breaking change
5. **Finding 7**: `ci_optims` crate compiles `tower-sessions-sqlx-store` with no backend features, defeating cache purpose
6. **Finding 13**: Postgres tests panic-fail when PostgreSQL is unavailable -- no graceful skip

### Recommended Priority Actions
1. Either add `cargo test` to CI with proper env vars, or remove the unused PostgreSQL service from `build.yml`
2. Add `features = ["sqlite", "postgres"]` to `tower-sessions-sqlx-store` in `crates/ci_optims/Cargo.toml`
3. Make postgres tests skip gracefully when `INTEG_TEST_SESSION_PG_URL` is not set
4. Add `crates/services/.env.test.example` following the existing codebase convention
