# services/test_utils — CLAUDE.md
**Companion docs** (load as needed):
- `PACKAGE.md` — Implementation details and file index

## Purpose

Test infrastructure for the services layer. Provides mock services, database test fixtures, and domain object builders. Feature-gated via `test-utils` in Cargo.toml.

## Key Test Utilities

### Database Testing
- **TestDbService** (`db.rs`): Wraps `DefaultDbService` with event broadcasting (`subscribe()`) and `FrozenTimeService`. In-memory SQLite with fresh migrations.
- **FrozenTimeService** (`db.rs`): Fixed time at 2025-01-01T00:00:00Z. All tests using `test_db_service` fixture get this.
- **SeaTestContext** (`sea.rs`): Dual SQLite/PG fixture via `sea_context("sqlite")` or `sea_context("postgres")`.
- **TEST_TENANT_ID / TEST_USER_ID** (`db.rs`): Standard test constants.

### Service Composition
- **AppServiceStub** (`app.rs`): Builder-based full service composition. Default fixture: `app_service_stub` creates one with hub, db, data, and session services. Builder methods like `with_hub_service()`, `with_data_service()`, `with_session_service()`.
- **AppServiceStubBuilder** (`app.rs`): Provides default mocks for auth, cache, time, settings, hub, network, concurrency, tool, inference services.

### Auth Testing
- **AuthContext test factories** (`auth_context.rs`): `test_anonymous()`, `test_session(user_id, username, role)`, `test_api_token(user_id, role)`, `test_external_app(user_id, role, app_client_id, access_request_id)`.
- **test_auth_service** (`auth.rs`): AuthService with embedded RSA keys for JWT signing. Configurable base URL for mockito.

### Hub/Data Testing
- **OfflineHubService** (`hf.rs`): Wraps `HfHubService`, panics on download if file not local. Test data from `crates/services/tests/data/huggingface/`.
- **TestHfService** (`hf.rs`): Hybrid real/mock hub service with `allow_downloads` toggle.

### Other Utilities
- **EnvWrapperStub** (`envs.rs`): In-memory environment variables.
- **SettingServiceStub** (`envs.rs`): In-memory settings with temp dir.
- **StubNetworkService** (`network.rs`): Network service mock.
- **MockQueueProducer** (`queue.rs`): Queue producer mock.
- **Domain fixtures** (`fixtures.rs`, `model_fixtures.rs`): Test data builders for domain objects.

## Module List (20 files)

`app.rs`, `auth.rs`, `auth_context.rs`, `bodhi.rs`, `data.rs`, `db.rs`, `envs.rs`, `fixtures.rs`, `hf.rs`, `http.rs`, `io.rs`, `logs.rs`, `model_fixtures.rs`, `network.rs`, `queue.rs`, `sea.rs`, `session.rs`, `settings.rs`, `test_data.rs`, `mod.rs`

## Non-Obvious Patterns

- `test_db_service` fixture requires `#[awt]` + `#[future]` annotations when used as parameter
- `AppServiceStubBuilder::build()` is async (creates real session service with SQLite)
- `FrozenTimeService::default()` returns fixed 2025-01-01 (not `Utc::now()` as old docs claimed)
- No `SecretService` stub exists — `SecretService` was removed from the crate
- No `objs.rs` file — renamed to `fixtures.rs`
