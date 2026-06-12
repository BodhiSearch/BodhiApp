# objs Crate Review

## Files Reviewed
- `crates/objs/src/mcp.rs` (625 lines) - MCP domain types, auth type enum, discriminated union request/response types, validation functions, tests

## Findings

### Finding 1: registration_type is an unvalidated free-form String
- **Priority**: Important
- **File**: crates/objs/src/mcp.rs
- **Location**: `CreateMcpAuthConfigRequest::Oauth` (line 272), `McpAuthConfigResponse::Oauth` (line 310), `McpOAuthConfig` (line 166)
- **Issue**: The `registration_type` field is a `String` everywhere it appears (request, response, domain model). The only two valid values are `"pre-registered"` and `"dynamic-registration"`, but there is no enum or validation constraining this field. The `default_registration_type()` function (line 285) returns `"pre-registered"` as a default, but a client can submit any arbitrary string (e.g., `"foo"`) and it will be accepted and persisted.
- **Recommendation**: Either (a) create a `RegistrationType` enum with `#[serde(rename_all = "kebab-case")]` containing `PreRegistered` and `DynamicRegistration` variants, or (b) add a validation function in `create_oauth_config` / `create_auth_config` that rejects unknown values. An enum is preferable because it provides compile-time guarantees.
- **Rationale**: Unvalidated string fields lead to data inconsistency. A consumer might mistype the value and produce incorrect behavior without any error feedback. This is a domain invariant that should be enforced at the type level.

### Finding 2: No validation on authorization_endpoint and token_endpoint URLs
- **Priority**: Important
- **File**: crates/objs/src/mcp.rs
- **Location**: `CreateMcpAuthConfigRequest::Oauth` (lines 265-266)
- **Issue**: The `authorization_endpoint` and `token_endpoint` fields in the OAuth variant accept arbitrary strings. Unlike `validate_mcp_server_url` (line 451) which validates URL format using `url::Url::parse()`, the OAuth endpoints have no validation. A client could submit invalid URLs like `"not-a-url"`, which would only fail later during the OAuth flow with a confusing error.
- **Recommendation**: Add URL format validation for `authorization_endpoint` and `token_endpoint` in the `CreateMcpAuthConfigRequest::Oauth` variant, either as standalone validation functions or as part of the service-layer create method.
- **Rationale**: Fail-fast validation at creation time provides a better user experience and prevents invalid configuration data from being stored in the database. The error messages from downstream HTTP client failures are significantly less actionable than upfront validation errors.

### Finding 3: CreateMcpAuthConfigRequest enum variant fields are not pub
- **Priority**: Nice-to-have
- **File**: crates/objs/src/mcp.rs
- **Location**: `CreateMcpAuthConfigRequest` enum (lines 256-283)
- **Issue**: The fields of the `Header` and `Oauth` variants are not marked `pub`. While this is fine for serde deserialization and pattern matching (which is how they are currently consumed), it means downstream code cannot access individual fields without destructuring. The `McpAuthConfigResponse` enum (line 293) follows the same pattern. This is consistent usage but may become inconvenient if fields need to be accessed individually in the future.
- **Recommendation**: No action needed now. The current destructuring pattern works well with `match`. Consider adding `pub` visibility if direct field access becomes needed.
- **Rationale**: Consistency observation only. The current pattern is idiomatic for Rust enums used with match expressions.

### Finding 4: McpAuthConfigResponse uses duplicated field sets across variants
- **Priority**: Nice-to-have
- **File**: crates/objs/src/mcp.rs
- **Location**: `McpAuthConfigResponse` enum (lines 293-330)
- **Issue**: The `id`, `name`, `mcp_server_id`, `created_by`, `created_at`, `updated_at` fields are duplicated in both the `Header` and `Oauth` variants. The `id()` and `mcp_server_id()` accessor methods (lines 332-346) already exist to abstract over this duplication.
- **Recommendation**: This is an inherent trade-off of using `#[serde(tag = "type")]` discriminated unions in Rust -- shared fields must be duplicated in each variant. No refactoring needed, but if more accessors are needed, consider a macro or trait to reduce boilerplate.
- **Rationale**: The current approach is the standard pattern for serde-tagged enums. The duplication is acceptable and well-managed with accessor methods.

### Finding 5: Serde conventions are correctly applied
- **Priority**: Nice-to-have (positive finding)
- **File**: crates/objs/src/mcp.rs
- **Location**: Throughout
- **Issue**: None. The serde conventions are correctly and consistently applied: `skip_serializing_if = "Option::is_none"` on all optional fields, `#[serde(default)]` on optional request fields, `#[serde(tag = "type", rename_all = "kebab-case")]` on discriminated unions, `#[schema(value_type = String, format = "date-time")]` on DateTime fields for OpenAPI compatibility.
- **Recommendation**: No changes needed.
- **Rationale**: Positive observation confirming adherence to project serde conventions documented in the crate's CLAUDE.md.

### Finding 6: Secret fields correctly masked in API responses
- **Priority**: Nice-to-have (positive finding)
- **File**: crates/objs/src/mcp.rs
- **Location**: `McpAuthHeader` (line 144), `McpOAuthConfig` (lines 178-179), `McpOAuthToken` (lines 200-201), `McpAuthConfigResponse::Header` (line 299), `McpAuthConfigResponse::Oauth` (lines 322-323)
- **Issue**: None. All secret fields are properly replaced with boolean indicators (`has_header_value`, `has_client_secret`, `has_registration_access_token`, `has_access_token`, `has_refresh_token`). The plaintext values are never present in the API response structs.
- **Recommendation**: No changes needed.
- **Rationale**: Positive confirmation of the security design principle: secrets never exposed in API responses.

### Finding 7: validate_mcp_auth_config_name does not validate content quality
- **Priority**: Nice-to-have
- **File**: crates/objs/src/mcp.rs
- **Location**: `validate_mcp_auth_config_name` (lines 477-488)
- **Issue**: The validation only checks for empty and length. It does not check for whitespace-only names (e.g., `"   "` would pass). This is consistent with `validate_mcp_server_name` (line 437) which also does not check for whitespace-only, so this is a project-wide pattern rather than an MCP-auth-specific issue.
- **Recommendation**: Consider adding a `name.trim().is_empty()` check in a future pass if whitespace-only names cause UI display issues.
- **Rationale**: Whitespace-only names are technically valid per current validation but may cause confusion in the UI. Low priority since it is consistent with the existing validation pattern.
