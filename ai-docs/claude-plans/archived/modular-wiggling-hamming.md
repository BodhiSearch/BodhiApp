# Prompt: Add toolset_types field to /bodhi/v1/toolsets endpoint

## Task Summary
Add a `toolset_types` field to the `/bodhi/v1/toolsets` endpoint response that returns app-level toolset configurations for all `scope_toolset-*` scopes the user has access to.

## Background
Currently the `/bodhi/v1/toolsets` endpoint returns only user-configured toolsets. The frontend needs additional information about app-level toolset configurations (admin-enabled/disabled status) to display proper status messages in the UI.

## Requirements

### API Change
Modify the `ListToolsetsResponse` to include a new `toolset_types` field:

```rust
pub struct ListToolsetsResponse {
  pub toolsets: Vec<ToolsetResponse>,
  pub toolset_types: Vec<AppToolsetConfig>,  // NEW FIELD
}
```

### Logic for toolset_types
1. **For OAuth token auth**: Extract all `scope_toolset-*` scopes from the access token claims, then query `app_toolset_configs` table for those scopes
2. **For session auth**: Session users have all scopes - return ALL entries from `app_toolset_configs` where scope starts with `scope_toolset-`

### Frontend Status Derivation (for context)
The frontend will derive status from the combined data:
- If `toolset_types.find(scope).enabled == false` → "Disabled by admin"
- If `toolsets.find(t => t.scope_uuid == type.scope_uuid).enabled == false` → "Disabled by user"
- If no toolset found for that scope → "Not configured by user"

## Files to Modify

### Core Changes
- `crates/routes_app/src/toolsets_dto.rs` - Add `toolset_types` field to `ListToolsetsResponse`
- `crates/routes_app/src/routes_toolsets.rs` - Implement logic to fetch app_toolset_configs based on user scopes
- `crates/services/src/db/toolsets_service.rs` - Add method to query app_toolset_configs by scope patterns

### Test Files
- `crates/routes_app/src/routes_toolsets.rs` (route-level tests) - Test that toolset_types is returned correctly
- `crates/lib_bodhiserver_napi/tests-js/specs/toolsets/` (E2E tests) - Test the full flow

## Test Requirements

### Routes-Level Tests
1. **Session auth returns all toolset_types**: When authenticated via session, `toolset_types` should contain all `scope_toolset-*` configs
2. **OAuth auth returns scoped toolset_types**: When authenticated via OAuth token with specific scopes, only return configs for those scopes
3. **Empty scopes returns empty toolset_types**: OAuth token without any `scope_toolset-*` scopes returns empty array

### E2E Tests
1. **toolset_types in response**: Verify `/bodhi/v1/toolsets` response includes `toolset_types` array
2. **Admin disable reflected**: When admin disables a toolset type, `toolset_types[].enabled` should be `false`
3. **Frontend can derive status**: Verify frontend can correctly display "Disabled by admin" vs "Disabled by user" vs "Not configured"

## Implementation Notes

1. **AppToolsetConfig** already exists in `crates/objs/src/toolsets.rs`:
   ```rust
   pub struct AppToolsetConfig {
     pub id: i64,
     pub scope: String,
     pub scope_uuid: String,
     pub enabled: bool,
     pub updated_by: Option<String>,
     pub created_at: DateTime<Utc>,
     pub updated_at: DateTime<Utc>,
   }
   ```

2. **Scope extraction from token**: Check existing patterns in `crates/auth_middleware/src/` for how to extract scopes from JWT claims

3. **Session auth detection**: Session-authenticated users can be detected via the authentication context - they have access to all scopes

## Verification
After implementation:
1. Run route-level tests: `cargo test -p routes_app toolsets`
2. Run E2E tests: `cd crates/lib_bodhiserver_napi && npm run test:playwright -- tests-js/specs/toolsets/`
3. Manual verification:
   - Login to UI, navigate to chat page
   - Check Network tab for `/bodhi/v1/toolsets` response
   - Verify `toolset_types` field is present with correct data
