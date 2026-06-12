# Block model pull in multi-tenant deployment

## Context

In `bodhi_deployment=multi_tenant`, local model operations should be unsupported. The model pull endpoint (`POST /bodhi/v1/models/files/pull`) currently succeeds in multi-tenant mode because `HfHubService` has no deployment awareness. This allowed users to download GGUF model files onto the multi-tenant server's local storage, which is a production issue (fills up storage, no local inference exists to use them).

**Root cause**: `HfHubService` is unconditionally constructed as the real implementation regardless of deployment mode (`app_service_builder.rs:136`). Unlike `DataService` and `InferenceService` which have multi-tenant variants, `HubService` has no deployment check.

**Approach**: Minimal fix — add a deployment mode check inside `HfHubService.download()`. No polymorphism, no new service types. The proper refactor (NoopHubService, UnsupportedDownloadService) is deferred to TECHDEBT.

## Changes

### Step 1: Add `Unsupported` variant to `HubServiceError`

**File**: `crates/services/src/models/hub_service.rs`

Add new variant to `HubServiceError` enum:
```rust
#[error("operation not supported in multi-tenant deployment mode")]
#[error_meta(error_type = ErrorType::BadRequest)]
Unsupported,
```

This will automatically propagate through the existing error chain:
`HubServiceError::Unsupported` → `ModelRouteError::HubService(...)` (via `#[from]` at `routes_app/src/models/error.rs:18`) → `ApiError` → HTTP 400

### Step 2: Add `deployment_mode` field to `HfHubService`

**File**: `crates/services/src/models/hub_service.rs`

- Add `deployment_mode: DeploymentMode` field to `HfHubService` struct (line 362)
- Default to `DeploymentMode::Standalone` in all three constructors (`new`, `new_from_cache`, `new_from_hf_cache`)
- Add builder method: `pub fn with_deployment_mode(mut self, mode: DeploymentMode) -> Self`

This avoids changing the signature of existing constructors — **zero test code changes** for constructor calls.

### Step 3: Check deployment mode in `download()`

**File**: `crates/services/src/models/hub_service.rs`, in `impl HubService for HfHubService`

Add check at the top of the `download()` method (line 115):
```rust
if self.deployment_mode == DeploymentMode::MultiTenant {
    return Err(HubServiceError::Unsupported);
}
```

Only `download()` gets the check. Read-only methods (`list_local_models`, `local_file_exists`, etc.) remain unchanged — they naturally return empty/false when no local files exist.

### Step 4: Wire deployment mode in production

**File**: `crates/lib_bodhiserver/src/app_service_builder.rs`

In `build_hub_service()` (line 238-246), chain `.with_deployment_mode()`:
```rust
async fn build_hub_service(
    setting_service: &Arc<dyn SettingService>,
) -> Result<Arc<dyn HubService>, BootstrapError> {
    let hf_cache = setting_service.hf_cache().await;
    let hf_token = setting_service.get_env(HF_TOKEN).await;
    let deployment_mode = setting_service.deployment_mode().await;
    let hub_service = HfHubService::new_from_hf_cache(hf_cache, hf_token, true)?
        .with_deployment_mode(deployment_mode);
    Ok(Arc::new(hub_service))
}
```

### Step 5: Add test

**File**: `crates/services/src/models/test_hub_service.rs` (existing test file)

Add a unit test that constructs `HfHubService` with `DeploymentMode::MultiTenant` and verifies `download()` returns `HubServiceError::Unsupported`:
```rust
#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_download_unsupported_in_multi_tenant(empty_hf_home: TempDir) -> anyhow::Result<()> {
    let hub = HfHubService::new(
        empty_hf_home.path().join("huggingface").join("hub"), false, None,
    ).with_deployment_mode(DeploymentMode::MultiTenant);

    let repo = Repo::try_from("test/repo".to_string())?;
    let result = hub.download(&repo, "test.gguf", None, None).await;
    assert!(result.is_err());
    assert_eq!("hub_service_error-unsupported", result.unwrap_err().code());
    Ok(())
}
```

### Step 6: Update TECHDEBT.md

**File**: `crates/services/TECHDEBT.md`

Add entry for the proper refactor:

```markdown
## Multi-Tenant Deployment Mode Centralization

**Location**: Multiple crates — `services`, `routes_app`, `server_core`, `lib_bodhiserver`

**Issue**: Deployment mode checks are scattered across different layers:
- **Service-level polymorphism**: `DataService` (LocalDataService vs MultiTenantDataService), `InferenceService` (StandaloneInferenceService vs MultitenantInferenceService)
- **Inline service check**: `HfHubService.download()` checks `self.deployment_mode` (temporary fix)
- **Route-level if-checks**: `routes_tenants.rs`, `routes_settings.rs` (LLM_SETTINGS), `routes_dev.rs`

**Impact**: Inconsistent patterns make it hard to audit which operations are blocked in multi-tenant mode. New features may miss deployment checks.

**Proposed fix**:
1. `HubService` → `NoopHubService` for multi-tenant (empty results for reads, `Unsupported` for `download()`)
2. `DownloadService` → `UnsupportedDownloadService` for multi-tenant (all mutating ops return `Unsupported`)
3. Migrate route-level deployment if-checks (`routes_settings.rs` LLM_SETTINGS, `routes_tenants.rs`, `routes_dev.rs`) into service-level polymorphism
4. Consider a central `DeploymentPolicy` trait that services can query

**Deferred because**: Production hotfix to block downloads was sufficient for immediate release. Full centralization requires designing the polymorphism for each service and migrating all route-level checks.
```

## Files Modified

| File | Change |
|------|--------|
| `crates/services/src/models/hub_service.rs` | Add `Unsupported` variant to `HubServiceError`; add `deployment_mode` field + builder to `HfHubService`; add check in `download()` |
| `crates/lib_bodhiserver/src/app_service_builder.rs` | Wire `deployment_mode` in `build_hub_service()` |
| `crates/services/src/models/test_hub_service.rs` | Add test for multi-tenant download block |
| `crates/services/TECHDEBT.md` | Add multi-tenant centralization entry |

## Flow After Fix

1. `models_pull_create` → creates download record (Pending) → spawns background task
2. Background task calls `execute_pull_by_repo_file`
3. `hub_service.local_file_exists()` → `Ok(false)` (no deployment check on reads)
4. `hub_service.download()` → **`Err(HubServiceError::Unsupported)`** (NEW)
5. Error propagates as `ModelRouteError::HubService(HubServiceError::Unsupported)`
6. `update_download_status` captures error → download record updated to `Error` status with message

Client sees: HTTP 201 with Pending download, then on status check sees Error with "operation not supported in multi-tenant deployment mode".

## Verification

```bash
# 1. Compile check
cargo check -p services -p lib_bodhiserver 2>&1 | tail -5

# 2. Run services tests (includes new test)
cargo test -p services --lib 2>&1 | grep -E "test result|FAILED|failures:"

# 3. Run routes_app tests (verify no regressions)
cargo test -p routes_app 2>&1 | grep -E "test result|FAILED|failures:"

# 4. Full backend
make test.backend
```
