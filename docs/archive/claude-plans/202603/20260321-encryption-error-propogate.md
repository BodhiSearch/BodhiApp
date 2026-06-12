# Fix: Propagate tenant decryption errors instead of silently falling back to Setup

## Context

When running the `.app` bundle, the macOS keychain returns a different encryption key than when running the bare binary (due to different code-signing identities). This causes `decrypt_tenant_row()` to fail with `DbError::EncryptionError`. Two code paths silently swallow this error via `.ok().flatten()`, defaulting to `AppStatus::Setup` — making the app appear unconfigured when it's actually a configuration/encryption error.

**Goal**: Propagate tenant loading errors as HTTP 500 responses instead of silently falling back to `AppStatus::Setup`. The frontend's existing error handling in `AppInitializer` (Shadcn Alert banner) already displays HTTP errors — no frontend changes needed.

## Layered approach (per CLAUDE.md)

1. `services` crate — add error-only logging (eprintln for bootstrap, tracing for request-time)
2. `routes_app` crate — propagate errors instead of swallowing them, update helper signature
3. Tests — update existing tests, add error propagation tests

### Logging strategy

- **Early bootstrap** (keyring, encryption key — runs before tracing subscriber is initialized in native/Tauri mode): Use `eprintln!` for **error scenarios only**. No debug/info logging.
- **Request-time** (tenant decryption, route handler — tracing subscriber is active): Use `tracing::info!` / `tracing::error!` as normal.

---

## Step 1: Add error-only logging to bootstrap code

### 1a. Keyring service — error-only eprintln
**File**: `crates/services/src/utils/keyring_service.rs`

In `SystemKeyringStore::get_password()` — add `eprintln!` only on the error path:
```rust
Err(err) => {
  eprintln!("keyring: failed to read password for service='{}' key='{}': {}", self.service_name, key, err);
  Err(err.into())
}
```

### 1b. Encryption key resolution — error-only eprintln
**File**: `crates/lib_bodhiserver/src/app_service_builder.rs`

In `build_encryption_key()` — add `eprintln!` only when keychain access fails:
```rust
.unwrap_or_else(|| {
  let result = SystemKeyringStore::new(&app_name)
    .get_or_generate(SECRET_KEY)
    .map_err(BootstrapError::from);
  if let Err(ref e) = result {
    eprintln!("build_encryption_key: failed to obtain key from keychain (app_name='{}'): {}", app_name, e);
  }
  result
})?;
```

### 1c. Tenant decryption — tracing (request-time, subscriber active)
**File**: `crates/services/src/tenants/tenant_repository.rs`

In `decrypt_tenant_row()` — add `tracing::error!` on decryption failure:
```rust
Err(e) => {
  tracing::error!(
    client_id = %model.client_id,
    error = %e,
    "decrypt_tenant_row: failed to decrypt tenant secret — encryption key mismatch?"
  );
  return Err(DbError::EncryptionError(e.to_string()));
}
```

In `get_tenant()` (standalone query) — add `tracing::info!` for tenant found + decryption attempt:
```rust
Some(model) => {
  tracing::info!(
    client_id = %model.client_id,
    app_status = %model.app_status,
    "get_tenant (standalone): found tenant, attempting decryption"
  );
  Ok(Some(self.decrypt_tenant_row(model)?))
}
```

---

## Step 2: Propagate errors in routes_app

### 2a. Fix `setup_show` handler
**File**: `crates/routes_app/src/setup/routes_setup.rs` (lines 65-74)

Change the Anonymous standalone path from:
```rust
let standalone_app = tenant_svc.get_standalone_app().await.ok().flatten();
```
To:
```rust
let standalone_app = tenant_svc.get_standalone_app().await?;
```

The `?` converts `TenantError` → `ApiError` via blanket `From<T: AppError>` impl → HTTP 500 with:
```json
{"error": {"message": "Encryption error: Decryption failed.", "type": "internal_server_error", "code": "db_error-encryption_error"}}
```

Update subsequent lines to work with `Option<Tenant>` directly (no `.ok().flatten()` needed):
```rust
let status = standalone_app.as_ref().map(|t| t.status.clone()).unwrap_or_default();
let cid = standalone_app.map(|t| t.client_id);
```

### 2b. Fix `standalone_app_status_or_default` helper
**File**: `crates/routes_app/src/middleware/utils.rs` (lines 24-34)

Change signature from `-> AppStatus` to `-> Result<AppStatus, TenantError>`:
```rust
pub async fn standalone_app_status_or_default(
  tenant_service: &AuthScopedTenantService,
) -> Result<AppStatus, TenantError> {
  Ok(
    tenant_service
      .get_standalone_app()
      .await?
      .map(|t| t.status)
      .unwrap_or_default()
  )
}
```

Add import: `use services::TenantError;`

### 2c. Update `setup_create` caller
**File**: `crates/routes_app/src/setup/routes_setup.rs` (line ~124)

Change:
```rust
let status = standalone_app_status_or_default(&tenant_svc).await;
```
To:
```rust
let status = standalone_app_status_or_default(&tenant_svc).await?;
```

---

## Step 3: Update tests

### 3a. Update existing `standalone_app_status_or_default` tests
**File**: `crates/routes_app/src/middleware/utils.rs` (test module, ~line 56)

Update assertions to handle `Result` return type — add `?` or `.unwrap()`.

### 3b. Add error propagation test for helper
**File**: `crates/routes_app/src/middleware/utils.rs` (test module)

Add test: mock `get_standalone_app()` returning `Err(TenantError::Db(DbError::EncryptionError(...)))` → assert error propagates.

### 3c. Add error propagation tests for handlers
**File**: `crates/routes_app/src/setup/test_setup.rs`

- GET `/info` as anonymous standalone with encryption error → verify HTTP 500
- POST `/setup` with encryption error → verify HTTP 500

---

## Verification

1. `cargo check -p services -p lib_bodhiserver -p routes_app` — compilation
2. `cargo test -p routes_app -- setup` — setup handler tests
3. `cargo test -p routes_app -- standalone_app_status` — helper function tests
4. `cargo test -p routes_app` — full crate tests (no regressions)
5. Manual: rebuild `.app`, verify `/bodhi/v1/info` returns HTTP 500 with error body (not `{"status":"setup"}`)
6. Manual: run bare binary, verify `/bodhi/v1/info` still returns `{"status":"ready"}`

## Files Modified

| File | Change |
|------|--------|
| `crates/services/src/utils/keyring_service.rs` | `eprintln!` on keychain read error |
| `crates/lib_bodhiserver/src/app_service_builder.rs` | `eprintln!` on keychain key build failure |
| `crates/services/src/tenants/tenant_repository.rs` | `tracing::error!` on decryption failure, `tracing::info!` on standalone tenant found |
| `crates/routes_app/src/setup/routes_setup.rs` | Propagate error in `setup_show` + `setup_create` with `?` |
| `crates/routes_app/src/middleware/utils.rs` | Change return to `Result<AppStatus, TenantError>`, update tests, add error test |
| `crates/routes_app/src/setup/test_setup.rs` | Add error propagation tests for handlers |

## No frontend changes

The existing `AppInitializer` error handling in `crates/bodhi/src/components/AppInitializer.tsx` already handles HTTP errors from `/bodhi/v1/info` via the Shadcn destructive Alert banner. The 500 error response will flow through naturally.
