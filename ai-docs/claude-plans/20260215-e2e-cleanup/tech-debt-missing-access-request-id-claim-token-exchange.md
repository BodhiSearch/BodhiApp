# Tech Debt: Missing `access_request_id` Claim in Keycloak Token Exchange

## Problem

Keycloak's Bodhi SPI may not be returning the `access_request_id` as a dedicated top-level JWT claim during token exchange. Empirical testing via httpyac against a real Keycloak instance showed the exchanged token lacks this claim, even when `scope_access_request:<UUID>` is included in the exchange request.

The `access_request_id` claim is **separate** from the `scope_access_request:*` scope string -- it's expected as a dedicated JWT claim that the Bodhi SPI should inject during exchange.

## Impact

The validation in `token_service.rs` (lines 327-343) explicitly checks for this claim after exchange:

```rust
// crates/auth_middleware/src/token_service.rs
let access_request_id = scope_claims.access_request_id.ok_or_else(|| {
  TokenError::AccessRequestValidation(
    AccessRequestValidationError::AccessRequestIdMismatch {
      claim: "missing".to_string(),
      expected: validated_record.id.clone(),
    },
  )
})?;
```

If the claim is missing, **any API call requiring access request authorization will fail in production** with an `AccessRequestIdMismatch` error.

## Why This Is Masked

The `ExternalTokenSimulator` (`crates/server_app/tests/utils/external_token.rs`) bypasses Keycloak entirely by:

1. Building a JWT with test claims, directly injecting `access_request_id` (line 101-103):
   ```rust
   if let Some(ar_id) = access_request_id {
     exchange_claims["access_request_id"] = serde_json::json!(ar_id);
   }
   ```
2. Seeding `MokaCacheService` with a pre-built `CachedExchangeResult`
3. Auth middleware finds the cached result and never calls Keycloak

All `server_app` integration tests (`test_oauth_toolset_auth.rs`) use this simulator, so they validate our code's behavior given an _expected_ KC response, not Keycloak's actual behavior.

## Code Paths That Consume `access_request_id`

| File | Lines | Role |
|---|---|---|
| `crates/services/src/token.rs` | 80-89 | `ScopeClaims` struct defines `access_request_id: Option<String>` |
| `crates/auth_middleware/src/token_service.rs` | 304-344 | Post-exchange validation: extracts and validates claim matches DB record |
| `crates/auth_middleware/src/auth_middleware.rs` | 168-174 | Injects `X-BodhiApp-Access-Request-Id` header if claim is present |
| `crates/auth_middleware/src/toolset_auth_middleware.rs` | 102-193 | Consumes header for toolset access authorization checks |
| `crates/auth_middleware/src/extractors.rs` | - | `MaybeAccessRequestId` extractor reads the header |

## Current Test Coverage

**Server_app tests (ExternalTokenSimulator, no real KC):**
- `test_oauth_approved_toolset_list_and_execute` -- OAuth with `access_request_id` scope
- `test_oauth_without_access_request_scope_execute_denied` -- OAuth without scope
- `test_oauth_without_toolset_scope_empty_list` -- OAuth without toolset scope

**E2E tests (real KC):**
- `toolsets-auth-restrictions.spec.mjs` Case 3 -- validates KC rejects unknown scopes (`invalid_scope` error)
- Cases 1, 2, 4 migrated to server_app -- **no E2E test validates KC returns `access_request_id` claim**

## Resolution Options

### Option A: Fix Bodhi SPI
Investigate and fix the Keycloak Bodhi SPI to ensure it adds `access_request_id` as a top-level JWT claim during token exchange. Add an E2E test to validate.

### Option B: Extract from scope string
Instead of relying on a dedicated claim, extract `access_request_id` from the `scope_access_request:<UUID>` scope string that KC does return. This removes the dependency on the SPI adding a separate claim.

### Option C: Add E2E happy path test
Add an E2E test that performs the full flow: approved access request -> OAuth with `scope_access_request:*` -> KC token exchange -> verify API call succeeds. This would immediately surface the missing claim issue against a real KC instance.

## Recommended Next Steps

1. Verify whether the Bodhi SPI is configured correctly in the dev/test KC instance
2. If SPI fix is straightforward, fix it and add E2E validation (Option A + C)
3. If SPI fix is complex, consider Option B as a pragmatic alternative
4. Regardless of approach, add at least one E2E happy path test (Option C) to prevent regression
