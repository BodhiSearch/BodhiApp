# Fix auth_middleware integration test 404 failures

## Context

After updating the Keycloak deployment on `test-id.getbodhi.app`, three `auth_middleware` integration tests fail with:
```
Audience access request failed: {"error":"HTTP 404 Not Found"}
```

The Keycloak Bodhi extension renamed the resource-initiated audience access endpoint from `/resources/request-access` to `/resources/apps/request-access` (commit `fe32233`). The `services` crate already uses the correct path (`crates/services/src/auth_service/service.rs:802`), but the `auth_middleware` test client was never updated.

## Fix

**File**: `crates/auth_middleware/src/test_utils/auth_server_test_client.rs`, line 315

Change:
```
"{}/realms/{}/bodhi/resources/request-access"
```
to:
```
"{}/realms/{}/bodhi/resources/apps/request-access"
```

## Verification

```bash
cargo test -p auth_middleware --test test_live_auth_middleware -- test_cross_client_token_exchange
```
