# Plan: Versioned RequestedResources & ApprovedResources

## Context

The access_request API is heavily exposed to 3rd-party apps. Changes to `RequestedResources`/`ApprovedResources` (e.g., new resource categories, richer per-resource config) would break the API contract. We need versioned schemas so the server can support multiple versions simultaneously, and clients with older/newer versions can operate without breakage.

No production release exists yet — version field is **mandatory** (no backward-compat fallback needed).

## Design Decisions

| Decision | Choice |
|----------|--------|
| Scope | Inner payloads only (`RequestedResources` + `ApprovedResources`) |
| Version format | Integer string (`"1"`, `"2"`) |
| Serde strategy | `#[serde(tag = "version")]` internally tagged enum |
| Version mandatory | Yes — no fallback (no production release) |
| DB storage | Version inside JSON blob, no new columns, no migration |
| Version coupling | Locked 1:1 (approved version must match requested version) |
| Cross-version drafts | Preserve original version (no upconversion) |
| Review API | Pass-through versioned JSON |
| Service signature | Full enum pass-through (not decomposed Vecs) |
| Middleware | Version-aware deserialization |
| OpenAPI | Try `#[derive(ToSchema)]` first, fix manually if needed |

## Implementation

### Phase 1: `services` crate — Domain types

**File: `crates/services/src/app_access_requests/access_request_objs.rs`**

1. Rename `RequestedResources` → `RequestedResourcesV1`, remove `Default` (version field makes defaulting ambiguous)
2. Rename `ApprovedResources` → `ApprovedResourcesV1`
3. Create versioned enums:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(tag = "version")]
pub enum RequestedResources {
    #[serde(rename = "1")]
    V1(RequestedResourcesV1),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(tag = "version")]
pub enum ApprovedResources {
    #[serde(rename = "1")]
    V1(ApprovedResourcesV1),
}
```

4. Add convenience methods:

```rust
impl RequestedResources {
    pub fn version(&self) -> &str {
        match self { Self::V1(_) => "1" }
    }
    pub fn as_v1(&self) -> Option<&RequestedResourcesV1> {
        match self { Self::V1(v) => Some(v) }
    }
}
// Same for ApprovedResources
```

5. Add `Default` impl: `RequestedResources::V1(RequestedResourcesV1::default())` and same for `ApprovedResources`

6. Update `#[schema(example = ...)]` on `CreateAccessRequest` and `ApproveAccessRequest` to include `"version": "1"`:

```rust
#[schema(example = json!({
    "app_client_id": "my-app-client",
    "requested": {
        "version": "1",
        "toolset_types": [{"toolset_type": "builtin-exa-search"}]
    }
}))]
```

7. Sub-types (`ToolsetTypeRequest`, `RequestedMcpServer`, `ToolsetApproval`, `McpApproval`, etc.) — **no changes needed**. They stay as-is; only the container enums are versioned.

### Phase 2: `services` crate — Service layer

**File: `crates/services/src/app_access_requests/access_request_service.rs`**

1. Change `AccessRequestService` trait signatures:

```rust
async fn create_draft(
    &self,
    app_client_id: String,
    flow_type: FlowType,
    redirect_uri: Option<String>,
    requested: RequestedResources,  // was: two Vec params
    requested_role: UserScope,
) -> Result<AppAccessRequest>;

async fn approve_request(
    &self,
    id: &str,
    user_id: &str,
    tenant_id: &str,
    user_token: &str,
    approved: ApprovedResources,  // was: two Vec params
    approved_role: UserScope,
) -> Result<AppAccessRequest>;
```

2. Update `DefaultAccessRequestService::create_draft`:
   - Replace manual `serde_json::json!()` with `serde_json::to_string(&requested)?`
   - This automatically includes `"version": "1"` in output

3. Update `DefaultAccessRequestService::approve_request`:
   - Replace manual `serde_json::json!()` with `serde_json::to_string(&approved)?`
   - Add version match validation before Keycloak registration:

```rust
let requested_resources: RequestedResources = serde_json::from_str(&row.requested)
    .map_err(|e| AccessRequestError::InvalidStatus(format!("Invalid requested JSON: {}", e)))?;
if requested_resources.version() != approved.version() {
    return Err(AccessRequestError::VersionMismatch {
        requested_version: requested_resources.version().to_string(),
        approved_version: approved.version().to_string(),
    });
}
```

4. Update `generate_description` to accept `&ApprovedResources`:

```rust
fn generate_description(&self, approved: &ApprovedResources) -> String {
    match approved {
        ApprovedResources::V1(v1) => {
            // existing logic using v1.toolsets and v1.mcps
        }
    }
}
```

**File: `crates/services/src/app_access_requests/error.rs`**

5. Add error variant:

```rust
#[error("Approved resources version '{approved_version}' does not match requested version '{requested_version}'.")]
#[error_meta(error_type = ErrorType::BadRequest)]
VersionMismatch {
    requested_version: String,
    approved_version: String,
},
```

6. Run: `cargo test -p services --lib 2>&1 | grep -E "test result|FAILED|failures:"`

### Phase 3: `routes_app` crate — Route handlers

**File: `crates/routes_app/src/apps/routes_apps.rs`**

1. `apps_create_access_request` (line 50): Change to version-dispatch for validation, pass full enum to service:

```rust
let requested = request.requested.unwrap_or_default();

let tools = auth_scope.tools();
match &requested {
    RequestedResources::V1(v1) => {
        for tool_type_req in &v1.toolset_types {
            tools.validate_type(&tool_type_req.toolset_type)?;
        }
    }
}

let created = access_request_service
    .create_draft(
        request.app_client_id,
        request.flow_type,
        request.redirect_url,
        requested,
        request.requested_role,
    )
    .await?;
```

2. `apps_get_access_request_review` (line 187): Version-dispatch for enrichment building:

```rust
let requested: RequestedResources = serde_json::from_str(&request.requested)
    .map_err(|_| AppsRouteError::InvalidRequestedJson)?;

match &requested {
    RequestedResources::V1(v1) => {
        for tool_type_req in &v1.toolset_types { /* existing logic */ }
        for mcp_server_req in &v1.mcp_servers { /* existing logic */ }
    }
}
```

3. `apps_approve_access_request` (line 284): Version-dispatch for instance validation + pass full enum:

```rust
match &approval_input.approved {
    ApprovedResources::V1(v1) => {
        for approval in &v1.toolsets { /* existing validation */ }
        for approval in &v1.mcps { /* existing validation */ }
    }
}

let updated = access_request_service
    .approve_request(
        &id,
        user_id,
        tenant_id,
        token,
        approval_input.approved,
        approved_scope,
    )
    .await?;
```

**File: `crates/routes_app/src/apps/error.rs`**

4. Add error variant (if not already handled by service-level VersionMismatch propagation):

```rust
#[error("Failed to parse requested resources JSON.")]
#[error_meta(error_type = ErrorType::InternalServer)]
InvalidRequestedJson,
```

### Phase 4: `routes_app` crate — Middleware validators

**File: `crates/routes_app/src/middleware/access_requests/access_request_validator.rs`**

1. `ToolsetAccessRequestValidator::validate_approved` — version-dispatch:

```rust
let approvals: ApprovedResources = serde_json::from_str(approved_json).map_err(|e| {
    AccessRequestAuthError::InvalidApprovedJson { error: e.to_string() }
})?;

let instance_approved = match &approvals {
    ApprovedResources::V1(v1) => v1.toolsets.iter().any(|a| {
        a.status == ApprovalStatus::Approved
            && a.instance.as_ref().is_some_and(|i| i.id == entity_id)
    }),
};
```

2. `McpAccessRequestValidator::validate_approved` — same pattern for `v1.mcps`

### Phase 5: `routes_app` crate — Toolset/MCP list filtering

**File: `crates/routes_app/src/toolsets/routes_toolsets.rs` (line 86-95)**

```rust
.and_then(|json| serde_json::from_str::<ApprovedResources>(&json).ok())
.map(|res| match res {
    ApprovedResources::V1(v1) => v1.toolsets
        .into_iter()
        .filter(|a| a.status == ApprovalStatus::Approved)
        .filter_map(|a| a.instance.map(|i| i.id))
        .collect(),
})
```

**File: `crates/routes_app/src/mcps/routes_mcps.rs` (line 51-60)** — same pattern for `v1.mcps`

### Phase 6: OpenAPI + TypeScript client

**File: `crates/routes_app/src/shared/openapi.rs`**

1. Add `RequestedResourcesV1` and `ApprovedResourcesV1` to the `schemas(...)` list
2. Run `cargo run --package xtask openapi`
3. Inspect generated `openapi.json` — verify the enum generates proper `oneOf` with `version` discriminator
4. If utoipa output is wrong, add `#[schema(title = "RequestedResourcesV1")]` or manual schema composition
5. Run `make build.ts-client`
6. Verify generated TypeScript types in `ts-client/src/types/types.gen.ts`

### Phase 7: Frontend

**File: `crates/bodhi/src/app/ui/apps/access-requests/review/page.tsx`** (or similar path)

1. Update `handleApprove` to include `version` matching the requested version:

```typescript
const body: ApproveAccessRequest = {
    approved_role: approvedRole,
    approved: {
        version: reviewData.requested.version,  // NEW
        toolsets: [...],
        mcps: [...],
    },
};
```

2. Update any test fixtures/mocks to include `"version": "1"` in `requested` and `approved` payloads
3. Run: `cd crates/bodhi && npm test`

### Phase 8: Full validation

1. `make test.backend`
2. `make build.ui-rebuild` (if NAPI tests exist for access requests)
3. `cargo test -p routes_app -- openapi` — verify OpenAPI spec in sync

## Files to modify

| File | Change |
|------|--------|
| `crates/services/src/app_access_requests/access_request_objs.rs` | Rename structs to V1, create versioned enums, add helpers |
| `crates/services/src/app_access_requests/access_request_service.rs` | Change trait sigs to accept enums, use serde for serialization |
| `crates/services/src/app_access_requests/error.rs` | Add `VersionMismatch` variant |
| `crates/services/src/app_access_requests/test_access_request_service.rs` | Update tests for new signatures + version in JSON |
| `crates/routes_app/src/apps/routes_apps.rs` | Version-dispatch in all 3 handlers, pass enums to service |
| `crates/routes_app/src/apps/apps_api_schemas.rs` | No struct changes (types auto-updated via enum rename) |
| `crates/routes_app/src/apps/error.rs` | Add `InvalidRequestedJson` variant |
| `crates/routes_app/src/apps/test_access_request.rs` | Update tests for versioned JSON + new service sigs |
| `crates/routes_app/src/middleware/access_requests/access_request_validator.rs` | Version-dispatch in both validators |
| `crates/routes_app/src/middleware/access_requests/test_access_request_middleware.rs` | Update test JSON to include version |
| `crates/routes_app/src/toolsets/routes_toolsets.rs` | Version-dispatch in ExternalApp filter (line 86-95) |
| `crates/routes_app/src/mcps/routes_mcps.rs` | Version-dispatch in ExternalApp filter (line 51-60) |
| `crates/routes_app/src/shared/openapi.rs` | Register V1 schema types |
| `crates/bodhi/src/app/ui/apps/access-requests/review/page.tsx` | Include `version` in approve body |
| Frontend test files | Add `"version": "1"` to all access request mock data |

## Test strategy

### New unit tests (services crate)
- `test_serialize_requested_v1_includes_version` — `serde_json::to_string(RequestedResources::V1(...))` includes `"version": "1"`
- `test_deserialize_requested_v1` — JSON with `"version": "1"` deserializes correctly
- `test_deserialize_requested_missing_version_fails` — JSON without version field returns error (mandatory)
- `test_deserialize_requested_unknown_version_fails` — `"version": "99"` returns error
- `test_roundtrip_requested_v1` — serialize then deserialize is identity
- Same suite for `ApprovedResources`
- `test_version_match_v1_v1` — `requested.version() == approved.version()` is true
- `test_create_draft_stores_versioned_json` — verify DB JSON contains `"version": "1"`
- `test_approve_rejects_version_mismatch` — service returns VersionMismatch error

### Updated integration tests (routes_app crate)
- All existing access request tests updated to include `"version": "1"` in request bodies
- `test_middleware_validates_versioned_approved_json` — middleware correctly parses versioned JSON
- `test_approve_endpoint_version_mismatch` — 400 error when versions don't match

## Future V2 Extension Checklist

When adding version 2:
1. Add `RequestedResourcesV2` / `ApprovedResourcesV2` structs with new fields
2. Add `#[serde(rename = "2")] V2(RequestedResourcesV2)` variant to both enums
3. Add `"2"` match arms to `version()`, `as_v2()` methods
4. Add version-dispatch arms in all `match` statements (compiler will flag missing arms)
5. Frontend adds rendering for `version === "2"`
6. Regenerate OpenAPI + TS client
