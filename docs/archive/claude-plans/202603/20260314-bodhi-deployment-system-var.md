# Plan: Move BODHI_DEPLOYMENT from env var to system property

## Context

`BODHI_DEPLOYMENT` is currently readable/overridable at runtime via environment variables through the `SettingService` lookup chain (system > cmdline > env > db > file > defaults). This means anyone can change deployment mode by setting `BODHI_DEPLOYMENT=multi_tenant` as an env var, which is a security concern. No current build (Tauri, Docker, standalone) produces a multi_tenant deployment — only E2E tests use it.

**Goal**: Make `BODHI_DEPLOYMENT` a system property on `AppOptions`, following the same pattern as `auth_url`, `auth_realm`, `env_type`, `app_type`. System settings are immutable and always take highest priority, preventing runtime override via env vars.

**User decisions**:
- Docker: Use `defaults.yaml` to set deployment mode (same pattern as `BODHI_VERSION`)
- MT validation (tenant required for multi_tenant): Out of scope, keep current runtime check
- Service construction: Read `is_multi_tenant` directly from `BootstrapParts.system_settings` (not through `SettingService`)
- Tauri `env.rs`: Skip — Tauri is always standalone, no compile-time constant needed

---

## Step 1: Add `strum` derives to `DeploymentMode`

**File**: `crates/services/src/tenants/tenant_objs.rs:5-11`

`DeploymentMode` lacks `strum::Display` and `strum::EnumString` (needed for `to_string()` and `.parse::<DeploymentMode>()`). Add them matching the serde rename:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, ToSchema, strum::Display, strum::EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum DeploymentMode {
  #[default]
  Standalone,
  MultiTenant,
}
```

**Reference pattern**: `EnvType` at `crates/services/src/settings/setting_objs.rs:6-13` uses identical `strum::EnumString, strum::Display` with `#[strum(serialize_all = "snake_case")]`.

---

## Step 2: Add `BODHI_DEPLOYMENT` to `AppOptions` and `AppOptionsBuilder`

**File**: `crates/lib_bodhiserver/src/app_options.rs`

### 2a. Add import

Add `DeploymentMode, BODHI_DEPLOYMENT` to the `services` imports (line 3-6).

### 2b. Add field to `AppOptions`

Add after `auth_realm` (line 27):
```rust
/// Deployment mode (Standalone or MultiTenant)
pub deployment_mode: DeploymentMode,
```

### 2c. Add field to `AppOptionsBuilder`

Add to builder fields (after `auth_realm: Option<String>`, line 46):
```rust
deployment_mode: Option<DeploymentMode>,
```

### 2d. Add `set_system_setting` match arm

In `set_system_setting()` (line 69-86), add before the catch-all:
```rust
BODHI_DEPLOYMENT => {
  let deployment_mode = value.parse::<DeploymentMode>()?;
  Ok(self.deployment_mode(deployment_mode))
}
```

### 2e. Add builder method

After `auth_realm()` method (line 113-116):
```rust
pub fn deployment_mode(mut self, deployment_mode: DeploymentMode) -> Self {
  self.deployment_mode = Some(deployment_mode);
  self
}
```

### 2f. Update `build()` validation

In `build()` (line 125-152), add `deployment_mode` to `AppOptions` construction. Default to `Standalone` (not required):
```rust
deployment_mode: self.deployment_mode.unwrap_or_default(),
```

---

## Step 3: Add `BODHI_DEPLOYMENT` to `build_system_settings()`

**File**: `crates/lib_bodhiserver/src/app_dirs_builder.rs:77-137`

### 3a. Add import

Add `BODHI_DEPLOYMENT` to the `services` import on line 7.

### 3b. Extract deployment_mode from file_defaults

In `setup_bootstrap_service()` (line 48-75), extract deployment_mode from file_defaults, similar to `app_version` (lines 56-63):
```rust
let deployment_mode = file_defaults
  .get(BODHI_DEPLOYMENT)
  .map(|v| v.as_str())
  .unwrap_or(None);
```

Pass it to `build_system_settings()`.

### 3c. Add parameter and Setting entry

Update `build_system_settings()` signature to accept `deployment_mode: Option<&str>`. Add to the `vec![]`:
```rust
Setting {
  key: BODHI_DEPLOYMENT.to_string(),
  value: serde_yaml::Value::String(
    deployment_mode
      .unwrap_or(&options.deployment_mode.to_string())
      .to_string(),
  ),
  source: SettingSource::System,
  metadata: SettingMetadata::String,
},
```

This follows the same override pattern as `BODHI_VERSION` (lines 104-113): defaults.yaml value wins over AppOptions compile-time value.

---

## Step 4: Read `deployment_mode` directly from `BootstrapParts` in service builder

**File**: `crates/lib_bodhiserver/src/app_service_builder.rs:80-224`

Currently line 127: `let is_multi_tenant = setting_service.is_multi_tenant().await;`

Replace with extraction of the full `DeploymentMode` enum from `parts.system_settings` BEFORE `from_parts()` consumes `parts` (before line 120). Follow the `is_production` pattern at lines 88-90:

```rust
let deployment_mode = parts.system_settings.iter()
  .find(|s| s.key == BODHI_DEPLOYMENT)
  .and_then(|s| s.value.as_str())
  .and_then(|v| v.parse::<DeploymentMode>().ok())
  .unwrap_or_default();
let is_multi_tenant = deployment_mode == DeploymentMode::MultiTenant;
```

Store `deployment_mode` as the primary value; derive `is_multi_tenant` from it. This preserves flexibility for future deployment modes beyond standalone/multi_tenant.

Add `BODHI_DEPLOYMENT, DeploymentMode` to the imports from `services` (line 16).

Remove line 127 (`let is_multi_tenant = setting_service.is_multi_tenant().await;`).

---

## Step 5: Update `lib_bodhiserver` re-exports

**File**: `crates/lib_bodhiserver/src/lib.rs`

Ensure `DeploymentMode` is re-exported for downstream crates (bodhi/src-tauri, lib_bodhiserver_napi). Check if it's already re-exported via `services::*` or needs explicit addition.

---

## Step 6: Update `bodhi/src-tauri` entry point

**File**: `crates/bodhi/src-tauri/src/common.rs:4-12`

Add `.deployment_mode(DeploymentMode::Standalone)` to the builder chain (or rely on the `unwrap_or_default()` in `build()`). Since `DeploymentMode::default()` is `Standalone`, no explicit call needed — the current code works as-is. **No change required.**

---

## Step 7: Update Docker `defaults.yaml` in all Dockerfiles

**Files** (8 Dockerfiles in `devops/`):
- `cpu.Dockerfile` (line 34-58)
- `cuda.Dockerfile` (line 28-53)
- `rocm.Dockerfile`
- `vulkan.Dockerfile`
- `intel.Dockerfile`
- `musa.Dockerfile`
- `cann.Dockerfile`

Add `BODHI_DEPLOYMENT: standalone` to each `defaults.yaml` section, after the version information block:
```yaml
# Deployment mode
BODHI_DEPLOYMENT: standalone
```

---

## Step 8: Update E2E test setup

**File**: `crates/lib_bodhiserver_napi/tests-js/scripts/start-shared-server.mjs:79-94`

Move `BODHI_DEPLOYMENT` from `envVars` to `systemSettings` in the `serverOptions`:

```javascript
if (deployment === 'multi_tenant') {
  const mtConfig = getMultiTenantConfig();
  clientId = mtConfig.tenantId;
  clientSecret = mtConfig.tenantSecret;
  createdBy = process.env.INTEG_TEST_USERNAME_ID;
  // BODHI_DEPLOYMENT is now a system setting (not env var)
  // Dashboard credentials remain as env vars
  envVars[bindings.BODHI_MULTITENANT_CLIENT_ID] = mtConfig.dashboardClientId;
  envVars[bindings.BODHI_MULTITENANT_CLIENT_SECRET] = mtConfig.dashboardClientSecret;
}
```

Add `systemSettings` to `serverOptions` (around line 103-114):
```javascript
const systemSettings = {};
if (deployment === 'multi_tenant') {
  systemSettings[bindings.BODHI_DEPLOYMENT] = 'multi_tenant';
}
```

Pass `systemSettings` in `serverOptions`.

The `createFullTestConfig()` in `test-helpers.mjs` already handles `systemSettings` (lines 111, 143-146) — calls `bindings.setSystemSetting()` for each entry. **No change needed in test-helpers.mjs.**

---

## Step 9: Update test utilities

**File**: `crates/lib_bodhiserver/src/test_utils/mod.rs` (or wherever `AppOptionsBuilder::development()` is defined)

Verify that `development()` test helper defaults to `Standalone`. Since `deployment_mode` defaults via `unwrap_or_default()`, no explicit change needed unless there's an explicit field set.

**File**: `crates/services/src/test_utils/envs.rs:176`

Currently has `(BODHI_DEPLOYMENT.to_string(), "standalone".to_string())` in test env defaults. This can remain as a fallback in the defaults tier — system settings always win.

---

## Step 10: Keep `SettingService.deployment_mode()` and `is_multi_tenant()` methods

**File**: `crates/services/src/settings/setting_service.rs:245-254`

**No change needed.** These methods call `get_setting(BODHI_DEPLOYMENT)` which checks system settings first. After the change, `BODHI_DEPLOYMENT` will always be found in system settings (highest priority), making the env var path unreachable. The `ensure_default!` in `default_service.rs:257` also stays as a harmless fallback.

---

## Files Modified Summary

| File | Change |
|------|--------|
| `crates/services/src/tenants/tenant_objs.rs` | Add `strum::Display, strum::EnumString` to `DeploymentMode` |
| `crates/lib_bodhiserver/src/app_options.rs` | Add `deployment_mode` field to `AppOptions` + `AppOptionsBuilder`, add `set_system_setting` match arm |
| `crates/lib_bodhiserver/src/app_dirs_builder.rs` | Add `BODHI_DEPLOYMENT` to `build_system_settings()`, extract from `file_defaults` |
| `crates/lib_bodhiserver/src/app_service_builder.rs` | Read `is_multi_tenant` from `parts.system_settings` directly |
| `crates/lib_bodhiserver/src/lib.rs` | Ensure `DeploymentMode` re-exported |
| `devops/*.Dockerfile` (7 files) | Add `BODHI_DEPLOYMENT: standalone` to defaults.yaml |
| `crates/lib_bodhiserver_napi/tests-js/scripts/start-shared-server.mjs` | Move `BODHI_DEPLOYMENT` from envVars to systemSettings |

---

## Verification

### 1. Compile check
```bash
cargo check -p services && cargo check -p lib_bodhiserver && cargo check -p routes_app && cargo check -p server_app && cargo check -p bodhi
```

### 2. Backend tests
```bash
cargo test -p services --lib 2>&1 | grep -E "test result|FAILED"
cargo test -p lib_bodhiserver --lib 2>&1 | grep -E "test result|FAILED"
cargo test -p routes_app --lib 2>&1 | grep -E "test result|FAILED"
cargo test -p server_app --lib 2>&1 | grep -E "test result|FAILED"
```

### 3. Rebuild UI + NAPI bindings (clears cached artifacts)
```bash
make build.ui-rebuild
```

### 4. NAPI E2E tests
```bash
cd crates/lib_bodhiserver_napi && npm run test:playwright
```

### 5. Verify system property behavior
- Setting `BODHI_DEPLOYMENT=multi_tenant` as env var should NOT change behavior (system setting wins)
- In NAPI tests, `systemSettings[BODHI_DEPLOYMENT] = 'multi_tenant'` should work
