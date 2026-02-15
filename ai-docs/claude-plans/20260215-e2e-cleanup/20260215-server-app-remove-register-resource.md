# Plan: Migrate Live Server Tests to Pre-configured Resource Client

## Context

The OAuth2 auth server has migrated from `test-id.getbodhi.app` to `main-id.getbodhi.app`. The current test setup dynamically creates a resource client via the auth server admin API, but the new server returns clients without Direct Access Grants enabled, causing a 400 error at token acquisition (line 348 in `live_server_utils.rs`). A pre-configured resource client with Direct Access Grants enabled is now provided in `.env.test`.

Additionally: fixed port 51135 replaces random port, dev console credentials are removed, and the public URL is fixed to match the actual server address.

---

## Changes

### 1. `crates/server_app/tests/utils/live_server_utils.rs` (core change)

**a) Imports — remove auth_middleware test_utils, remove rand (lines 2-9):**
```rust
// REMOVE:
use auth_middleware::{
  test_utils::{AuthServerConfigBuilder, AuthServerTestClient},
  SESSION_KEY_ACCESS_TOKEN, SESSION_KEY_REFRESH_TOKEN,
};
use rand::Rng;

// REPLACE WITH:
use auth_middleware::{SESSION_KEY_ACCESS_TOKEN, SESSION_KEY_REFRESH_TOKEN};
```

**b) Add `BODHI_HOST` and `BODHI_PORT` to services imports (lines 22-23):**
Add `BODHI_HOST` and `BODHI_PORT` to the existing `services::` import block.

**c) Remove `test_user_id` env var read (lines 82-83):**
Delete `INTEG_TEST_USERNAME_ID` read — only used for `make_first_resource_admin()` which is being removed.

**d) Add `BODHI_HOST` and `BODHI_PORT` to env_vars (after existing env_vars, ~line 75):**
```rust
env_vars.insert(BODHI_HOST.to_string(), "127.0.0.1".to_string());
env_vars.insert(BODHI_PORT.to_string(), "51135".to_string());
```

**e) Replace dynamic OAuth client setup with env-based credentials (lines 144-178):**

Remove all `AuthServerConfigBuilder`, `AuthServerTestClient`, `create_resource_client`, `get_resource_service_token`, `make_first_resource_admin` code.

Replace with reading from env vars:
```rust
let resource_client_id = std::env::var("INTEG_TEST_RESOURCE_CLIENT_ID")
  .map_err(|_| anyhow::anyhow!("INTEG_TEST_RESOURCE_CLIENT_ID not set"))?;
let resource_client_secret = std::env::var("INTEG_TEST_RESOURCE_CLIENT_SECRET")
  .map_err(|_| anyhow::anyhow!("INTEG_TEST_RESOURCE_CLIENT_SECRET not set"))?;
let resource_client_scope = std::env::var("INTEG_TEST_RESOURCE_CLIENT_SCOPE")
  .map_err(|_| anyhow::anyhow!("INTEG_TEST_RESOURCE_CLIENT_SCOPE not set"))?;
```

Then build `AppRegInfo`:
```rust
let app_reg_info = AppRegInfoBuilder::default()
  .client_id(resource_client_id)
  .client_secret(resource_client_secret)
  .scope(resource_client_scope)
  .build()?;
```

Keep the existing `DefaultSecretService` creation and `set_app_reg_info`/`set_app_status` calls.

**f) Fixed port in `live_server` fixture (line 290):**
```rust
// FROM: let port = rand::rng().random_range(2000..60000);
// TO:
let port: u16 = 51135;
```

### 2. `crates/server_app/Cargo.toml`

**Remove `features = ["test-utils"]` from auth_middleware (line 36):**
```toml
# FROM: auth_middleware = { workspace = true, features = ["test-utils"] }
# TO:   auth_middleware = { workspace = true }
```

**Remove `rand` dev-dependency (line 48):**
No longer needed — only used for random port generation.

### 3. `crates/server_app/tests/resources/.env.test`

Remove dev console lines (lines 5-7):
```
# Dev Console Client (for creating other clients)
INTEG_TEST_DEV_CONSOLE_CLIENT_ID=client-bodhi-dev-console
INTEG_TEST_DEV_CONSOLE_CLIENT_SECRET=change-me
```

### 4. `crates/server_app/tests/resources/.env.test.example`

Update to match new structure — remove dev console vars, add resource client vars with placeholders.

### 5. No changes to test files

The 3 test files only import from `crate::utils` — no direct auth_middleware references.

---

## What stays the same

- Per-test server spawning (no `#[once]`)
- `#[serial_test::serial(live)]` for serialized execution
- `get_oauth_tokens()` function — reads from AppRegInfo (now pre-configured), uses standard scopes
- `create_authenticated_session()` and `create_session_cookie()` — unchanged
- `setup_minimal_app_service()` — all other service setup unchanged

## Verification

1. `cargo check -p server_app` — verify compilation
2. Run the failing test: `cargo test -p server_app test_live_agentic_chat_with_exa_toolset -- --nocapture`
3. Run all live tests: `cargo test -p server_app test_live -- --nocapture`

## Notes

- Redirect URI for resource client: `http://127.0.0.1:51135/ui/auth/callback` (configured in Keycloak)
- Resource scope not needed in password grant token request (only needed for third-party clients)
- `SESSION_KEY_ACCESS_TOKEN` / `SESSION_KEY_REFRESH_TOKEN` are in main auth_middleware module, not behind test-utils feature
