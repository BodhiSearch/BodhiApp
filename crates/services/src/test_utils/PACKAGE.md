# PACKAGE.md - services/test_utils

*For architecture overview, see [CLAUDE.md](CLAUDE.md)*

## Module Organization

Entry point: `src/test_utils/mod.rs` — declares 20 sub-modules with `pub use *` re-exports.

Also re-exports: `MockAccessRequestService`, `MockInferenceService`, `MockSettingsRepository`, `ModelMetadataEntityBuilder`.

Static constant: `SNAPSHOT` — test snapshot hash.

Feature-gated via `test-utils` feature in `crates/services/Cargo.toml`.

## File Index

| File | Key Exports | Purpose |
|------|-------------|---------|
| `app.rs` | `AppServiceStub`, `AppServiceStubBuilder`, `app_service_stub` fixture, `app_service_stub_builder` fixture | Full service composition for integration tests |
| `auth.rs` | `test_auth_service`, embedded RSA keys, `access_token_claims()`, `build_token()` | AuthService with deterministic JWT signing |
| `auth_context.rs` | `AuthContext::test_anonymous()`, `test_session()`, `test_api_token()`, `test_external_app()` | AuthContext factory methods for tests |
| `bodhi.rs` | `temp_bodhi_home` fixture | Isolated bodhi home directory |
| `data.rs` | Data service helpers | Temp directory fixtures for alias/model tests |
| `db.rs` | `TestDbService`, `FrozenTimeService`, `test_db_service` fixture, `TEST_TENANT_ID`, `TEST_USER_ID` | Database testing with event broadcasting and frozen time |
| `envs.rs` | `EnvWrapperStub`, `SettingServiceStub` | In-memory environment and settings |
| `fixtures.rs` | Domain object builders | Test data construction (renamed from `objs.rs`) |
| `model_fixtures.rs` | Model fixture builders | Model-specific test data |
| `hf.rs` | `TestHfService`, `OfflineHubService` | HuggingFace mock with real/mock modes |
| `http.rs` | HTTP test utilities | Request/response helpers |
| `io.rs` | IO helpers | File system test utilities |
| `logs.rs` | Log capture | Tracing subscriber for assertions |
| `network.rs` | `StubNetworkService` | Network service stub |
| `queue.rs` | Queue helpers | Queue service test infrastructure |
| `sea.rs` | `SeaTestContext`, `sea_context()` | Dual SQLite/PG database fixture |
| `session.rs` | Session mocks, `SessionTestExt` | Session service test helpers |
| `settings.rs` | `bodhi_home_setting` | Setting service test configuration |
| `test_data.rs` | Static constants | Test data values |

## AppServiceStub Details

`AppServiceStub` (in `app.rs`) implements `AppService` trait. Builder fields include all 18 services as `Option<Arc<dyn Trait>>`.

Default fixture chain:
1. `test_db_service` → creates in-memory SQLite with migrations + FrozenTimeService
2. `app_service_stub_builder` → builds with hub, db, data, session services
3. `app_service_stub` → calls `build().await`

Builder methods:
- `with_hub_service()` — creates `OfflineHubService` with test data from `tests/data/huggingface/`
- `with_data_service()` — creates `LocalDataService` with temp dir (async)
- `with_session_service()` — creates `DefaultSessionService` with SQLite (async)

Default services provided by builder: `MockAuthService`, `FrozenTimeService`, `SettingServiceStub`, `MokaCacheService`, `OfflineHubService`, `StubNetworkService`, `LocalConcurrencyService`, `DefaultToolService`, `MockInferenceService`, `MockQueueProducer`.

## TestDbService Details

`TestDbService` (in `db.rs`) wraps `DefaultDbService` with:
- `event_sender: Sender<String>` — broadcast channel for operation tracking
- `now: DateTime<Utc>` — frozen time from `FrozenTimeService`
- `encryption_key: Vec<u8>` — deterministic key for encrypted field testing

Methods:
- `subscribe()` → `Receiver<String>` for reactive test validation
- All repository traits delegated to inner `DefaultDbService`

## FrozenTimeService Details

`FrozenTimeService` (in `db.rs`):
- `Default` returns fixed 2025-01-01T00:00:00Z (NOT `Utc::now()`)
- `utc_now()` always returns the frozen time
- `created_at()` always returns 0

## AuthContext Test Factories

`auth_context.rs` provides `impl AuthContext` factory methods. All use `DEFAULT_CLIENT_ID = "test-client-id"` and `TEST_TENANT_ID`.

Key methods:
- `test_anonymous(deployment)` — Anonymous with given DeploymentMode
- `test_session(user_id, username, role)` — Session with ResourceRole
- `test_session_no_role(user_id, username)` — Session without role
- `test_api_token(user_id, role)` — ApiToken with TokenScope
- `test_external_app(user_id, role, app_client_id, access_request_id)` — ExternalApp with UserScope

## Commands

```bash
cargo test -p services                     # All tests
cargo test -p services --features test-utils  # With test utilities
cargo test -p services -- --nocapture      # Show output
```
