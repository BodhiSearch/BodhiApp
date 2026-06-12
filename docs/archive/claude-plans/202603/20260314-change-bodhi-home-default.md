# Plan: Change BODHI_HOME default from `~/.cache/bodhi` to `~/.bodhi`

## Context

The default BODHI_HOME directory is currently `~/.cache/bodhi` (production) / `~/.cache/bodhi-dev` (development). This should change to `~/.bodhi` / `~/.bodhi-dev`. The change only affects the hardcoded default fallback — when BODHI_HOME is set via env var or `defaults.yaml` (as in all Docker deployments), nothing changes.

No data migration needed — no production deployment uses the default path.

## Changes

### Step 1: `services` crate — default_service.rs

**File:** `crates/services/src/settings/default_service.rs`

Two locations compute the default BODHI_HOME for the settings UI "default_value" display:

- **Line 438:** `home_dir.join(".cache").join("bodhi")` → `home_dir.join(".bodhi")`
- **Line 448:** `home_dir.join(".cache").join("bodhi")` → `home_dir.join(".bodhi")` (used for BODHI_LOGS fallback)

Run: `cargo check -p services`

### Step 2: `services` crate — test fixtures

**File:** `crates/services/src/test_utils/envs.rs`

- **Line 127:** `home.join(".cache").join("bodhi")` → `home.join(".bodhi")`

Note: Line 129 (`home.join(".cache").join("huggingface")`) is unrelated — leave it alone.

**File:** `crates/services/src/settings/test_setting_service.rs`

- **Line 192:** `home_dir.join(".cache").join("bodhi")` → `home_dir.join(".bodhi")`
- **Lines 197-198:** `home_dir.join(".cache").join("bodhi").join("logs")` → `home_dir.join(".bodhi").join("logs")`

Run: `cargo test -p services --lib 2>&1 | grep -E "test result|FAILED|failures:"`

### Step 3: `lib_bodhiserver` crate — app_dirs_builder.rs

**File:** `crates/lib_bodhiserver/src/app_dirs_builder.rs`

- **Line 205:** `home_dir.join(".cache").join(path)` → `home_dir.join(format!(".{}", path))`

This produces `~/.bodhi` (prod) / `~/.bodhi-dev` (dev).

### Step 4: `lib_bodhiserver` crate — tests

**File:** `crates/lib_bodhiserver/src/test_app_dirs_builder.rs`

- **Line 44:** `temp_dir.path().join(".cache").join("bodhi-dev")` → `temp_dir.path().join(".bodhi-dev")`
- **Line 165:** `temp_dir.path().join(".cache").join("bodhi-dev")` → `temp_dir.path().join(".bodhi-dev")`

Run: `cargo test -p lib_bodhiserver --lib 2>&1 | grep -E "test result|FAILED|failures:"`

### Step 5: Makefile

**File:** `Makefile`

- **Line 130:** `~/.cache/bodhi-dev-makefile` → `~/.bodhi-dev-makefile`
- **Line 139:** `BODHI_HOME=~/.cache/bodhi-dev-makefile` → `BODHI_HOME=~/.bodhi-dev-makefile`
- **Line 146:** `BODHI_HOME=~/.cache/bodhi-dev-makefile` → `BODHI_HOME=~/.bodhi-dev-makefile`

### Step 6: Frontend mock data

**File:** `crates/bodhi/src/test-utils/msw-v2/handlers/settings.ts`

- **Line 37:** `'/home/user/.cache/bodhi'` → `'/home/user/.bodhi'`

**File:** `crates/bodhi/src/app/ui/settings/page.test.tsx`

- **Line 32:** `'/home/user/.cache/bodhi'` → `'/home/user/.bodhi'`

Run: `cd crates/bodhi && npm test 2>&1 | tail -20`

### NOT changed (confirmed safe)

- **Dockerfiles** (`devops/*.Dockerfile`): All set `BODHI_HOME: /data/bodhi_home` in `defaults.yaml` — this is an explicit path, not using the `.cache` default. No change needed.
- **NAPI bindings** (`lib_bodhiserver_napi`): Reference `BODHI_HOME` as a string constant name only, not the path. No change needed.
- **BODHI_HOME constant** (`services/src/settings/constants.rs`): This is the env var key `"BODHI_HOME"`, not a path. No change needed.

## Verification

1. `cargo test -p services --lib` — settings + test fixture changes
2. `cargo test -p lib_bodhiserver --lib` — app_dirs_builder changes
3. `make test.backend` — full backend test suite
4. `cd crates/bodhi && npm test` — frontend tests with updated mock data
5. `make test.napi` — E2E tests (should be unaffected since they set BODHI_HOME explicitly)
