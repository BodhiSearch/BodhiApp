# Fix AZP Extraction in Token Validation

## Problem
In `auth_middleware.rs`, the `azp` (authorized party) header is extracted from the **exchanged token** instead of the **original external token**. This causes the `X-BodhiApp-Azp` header to contain the resource client ID rather than the app client ID that initiated the OAuth flow.

## Solution Overview
Return `azp` from `validate_bearer_token` as a third tuple element:
- **API tokens (bodhiapp_*)**: `azp = None`
- **External OAuth tokens**: `azp = Some(original_token.azp)` extracted BEFORE exchange

---

## Phase token-service: Update token_service.rs

### File: `crates/auth_middleware/src/token_service.rs`

1. **Add cache struct** (near top of file, after imports):
```rust
#[derive(Debug, Serialize, Deserialize)]
struct CachedExchangeResult {
    token: String,
    azp: String,
}
```

2. **Change return type** of `validate_bearer_token`:
- From: `Result<(String, ResourceScope), AuthError>`
- To: `Result<(String, ResourceScope, Option<String>), AuthError>`

3. **API token path** (line ~104): Return `None` for azp:
```rust
return Ok((bearer_token.to_string(), ResourceScope::Token(token_scope), None));
```

4. **External token path** - extract azp BEFORE exchange:
- Around line 159 where `ScopeClaims` is already extracted for validation
- Store `claims.azp.clone()` before calling `handle_external_client_token`

5. **Update `handle_external_client_token`** return type:
- From: `Result<(String, ResourceScope), AuthError>`
- To: `Result<(String, ResourceScope, String), AuthError>` (azp is required)
- Return the original `azp` from the external token claims

6. **Update caching logic** (lines ~120-144):
- **Cache read**: Deserialize JSON to `CachedExchangeResult`, return `(result.token, scope, Some(result.azp))`
- **Cache write**: Serialize `CachedExchangeResult { token, azp }` as JSON

7. **Error handling**: If external token missing `azp`, deserialization already fails since `ScopeClaims.azp` is required

---

## Phase auth-middleware: Update auth_middleware.rs

### File: `crates/auth_middleware/src/auth_middleware.rs`

1. **In `auth_middleware` function** (bearer token branch, lines ~144-186):
- Update destructuring: `let (access_token, resource_scope, azp) = token_service.validate_bearer_token(header).await?;`
- Set AZP header from returned value:
```rust
if let Some(azp) = azp {
    req.headers_mut().insert(KEY_HEADER_BODHIAPP_AZP, azp.parse().unwrap());
}
```
- **Remove** the `scope_claims.azp` extraction (lines ~181-183) - now handled by return value

2. **In `inject_optional_auth_info` function** (bearer token branch, lines ~253-281):
- Update destructuring to handle new return type
- **Remove** the `KEY_HEADER_BODHIAPP_AZP` setting entirely (lines ~278-281) - not used downstream

---

## Phase tests: Update Tests

### File: `crates/auth_middleware/src/token_service.rs` (test module)

1. **Update existing tests** to handle new return type:
- All calls to `validate_bearer_token` need to destructure 3 elements
- Example: `let (access_token, scope, azp) = token_service.validate_bearer_token(...).await?;`

2. **Add azp verification tests**:
- Test API token returns `None` for azp
- Test external token exchange returns correct original azp
- Test cache hit returns correct azp

### File: `crates/auth_middleware/src/auth_middleware.rs` (test module)

1. **Update test assertions** if any check AZP header behavior

---

## Critical Files

| File | Changes |
|------|---------|
| `crates/auth_middleware/src/token_service.rs` | Add struct, change return types, update caching |
| `crates/auth_middleware/src/auth_middleware.rs` | Update both middleware functions |

---

## Verification

1. **Unit tests**: `cargo test -p auth_middleware`
2. **Run the failing test**:
```bash
cd crates/lib_bodhiserver_napi && npm run test -- specs/toolsets/toolsets-auth-restrictions.spec.mjs
```
3. **Verify AZP header** contains app client ID (not resource client ID) for OAuth token flows
