# Code Review Index — MCP Auth Schema Redesign

## Review Scope
- **Original Ref**: HEAD (566c4cf07)
- **Original Date**: 2026-03-18
- **Updated**: 2026-03-20 (post-fix pass)
- **Fix Commit**: fedcd0485

## Summary
- Original findings: 16
- Fixed: 13 | Remaining: 3

## Fixed Findings

All findings below were resolved in commit `fedcd0485`:

| # | Severity | Crate | Issue | Resolution |
|---|----------|-------|-------|------------|
| 1 | Critical | services | Zero test coverage — 3 stub files | 49 new tests: auth config CRUD, encryption round-trips, OAuth token lifecycle, cross-tenant isolation, resolve_auth_params branching |
| 2 | Critical | services | MCP create/update not atomic — 3 separate transactions | Composite repo methods (`create_mcp_with_auth`, `update_mcp_with_auth`) wrap all writes in single `with_tenant_txn()` |
| 3 | Critical | services | Instance and auth repos split — prevents atomic transactions | Merged `McpInstanceRepository` + `McpAuthRepository` into `McpRepository`; `McpServerRepository` kept separate |
| 4 | Critical | routes_app | No test for POST /mcps with oauth_token_id | `test_create_mcp_with_oauth_token_id` added in test_mcps.rs |
| 6 | Critical | routes_app | No test for PUT /mcps with credential/auth updates | Tests added: `test_update_mcp_change_credentials`, `test_update_mcp_change_oauth_token`, `test_update_mcp_clear_auth` |
| 7 | Important | services | store_oauth_token delete-then-insert not atomic | `store_oauth_token` composite method wraps delete + insert in single transaction |
| 8 | Important | bodhi | Plaintext credential values persisted to sessionStorage | Removed `credential_values` from `saveToSession()` and `restoreFromSession()` in mcpFormStore.ts |
| 9 | Important | E2E | No negative test cases for credential validation | Added: missing credentials → failure, partial credentials → failure |
| 11 | Important | routes_app | mcp_id ownership not validated in OAuth exchange | Added ownership check after CSRF validation in `mcp_oauth_token_exchange()` |
| 12 | Important | routes_app | No test for null mcp_id in OAuth exchange | `test_oauth_token_exchange_null_mcp_id` added |
| 13 | Important | routes_app | Mock `withf` closures don't assert auth fields | Explicit assertions for `credentials`, `oauth_token_id`, `auth_config_id` in all create/update mock closures |
| 15 | Nice-to-have | services | SQLite FK not enforced on mcps.auth_config_id | `delete_mcp_auth_config` now NULLs out `mcps.auth_config_id` references before deleting |
| 16 | Nice-to-have | routes_app | Unused `_header_value` parameter in test helper | Parameter removed from `create_header_auth_config_in_db()`; 8 call sites updated |

---

## Remaining Findings

### Finding 5: No test for POST /mcps/fetch-tools with oauth_token_id
- **Priority**: Critical
- **Crate**: routes_app
- **File**: `crates/routes_app/src/mcps/test_mcps.rs`
- **Location**: Fetch-tools tests (around `test_fetch_mcp_tools_with_auth_config_id`)
- **Issue**: `FetchMcpToolsRequest` has `oauth_token_id: Option<String>` but all fetch-tools tests pass `oauth_token_id: None`. The OAuth auth type relies on this field to resolve the access token for tool preview. If the handler doesn't pass it through, OAuth tool preview silently fails.
- **Recommendation**: Add `test_fetch_mcp_tools_with_oauth_token_id` — pass `oauth_token_id: Some(...)`, verify mock service's `fetch_tools_for_server` receives the token ID.

### Finding 10: No "wrong credentials" E2E test
- **Priority**: Important
- **Crate**: E2E (lib_bodhiserver_napi)
- **File**: `crates/lib_bodhiserver_napi/tests-js/specs/mcps/mcps-header-auth.spec.mjs`
- **Status**: Partially addressed — credential value verification added to test 3 (mixed auth) via `get_auth_info` tool response parsing. Negative tests added for missing and partial credentials. However, no test for providing **wrong** credential values (values filled but incorrect) and verifying the mock server returns 401.
- **Recommendation**: Add E2E test: create auth config, fill incorrect credential values, attempt fetch tools → expect failure/error. Also add a test that creates an MCP with wrong credentials, then attempts tool execution in the playground → expect error.

### Finding 14: Incomplete error path tests for MCP creation with auth
- **Priority**: Important
- **Crate**: routes_app
- **File**: `crates/routes_app/src/mcps/test_mcps.rs`
- **Status**: Partially addressed — `test_oauth_token_exchange_invalid_mcp_id` tests the ownership check error path. However, missing tests for:
  - `auth_type: Header` with `auth_config_id: None` → should error or be handled
  - `auth_config_id` referencing non-existent config → should propagate service error
  - `credentials` array with invalid param types → should be caught by validation
- **Recommendation**: Add error path tests verifying correct HTTP status codes (400 vs 404 vs 500) and error response shapes for each case.
