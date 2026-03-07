# routes_app Integration Tests

## Prerequisites

These live integration tests require a running Keycloak instance and a PostgreSQL database.

### Keycloak Configuration

Copy `.env.test.example` to `.env.test` in `tests/resources/` and fill in the values.

#### Required Keycloak Clients

1. **Resource Client** (`INTEG_TEST_RESOURCE_CLIENT_ID`)
   - Confidential client with client secret (`INTEG_TEST_RESOURCE_CLIENT_SECRET`)
   - Used for resource-level token operations

2. **App Client** (`INTEG_TEST_APP_CLIENT_ID`)
   - Public client with **Direct Access Grants** enabled
   - **User Consent** must be turned off
   - Redirect URI: `http://localhost:5173` (not actively used by tests, but required by Keycloak client config)
   - Used by `test_live_auth_middleware` for cross-client token exchange tests

#### Test User

A test user (`INTEG_TEST_USERNAME` / `INTEG_TEST_PASSWORD`) must exist in the realm with the ID matching `INTEG_TEST_USERNAME_ID`.

### Running

```bash
cargo test -p routes_app --test test_live_auth_middleware
```
