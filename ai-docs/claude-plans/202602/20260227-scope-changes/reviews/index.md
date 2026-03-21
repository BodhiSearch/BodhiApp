# Code Review Index

## Review Scope
- **Ref**: HEAD (`8f0af5b` — feat(kc-scope-removal): layers 2–5 squashed)
- **Date**: 2026-02-27
- **Files Changed**: 95 files across 9 crates + frontend + E2E + docs
- **Crates Affected**: objs, services, auth_middleware, routes_app, server_app, lib_bodhiserver, bodhi/src, tests-js, ts-client

## Summary
- Total findings: 19
- Critical: 0 | Important: 7 | Nice-to-have: 12

## Important Issues (Should Fix)

| # | Crate | File | Location | Issue | Fix Description | Report |
|---|-------|------|----------|-------|-----------------|--------|
| 1 | routes_app | routes_apps/types.rs | `ApproveAccessRequestBody.approved_role` | Type is `String`, should be `UserScope` | Change to `UserScope` for serde-level validation, remove manual `.parse()` in handler, pass `.to_string()` to service | [routes-app](routes-app-review.md) |
| 2 | routes_app | routes_apps/test_access_request.rs | (missing test) | No test for `approved_role > requested_role` (check #1) | Add `test_approve_exceeds_requested_role`: PowerUser approver + requested=scope_user_user + approved=scope_user_power_user → 403 | [routes-app](routes-app-review.md) |
| 3 | objs | user_scope.rs | `UserScopeError::MissingUserScope` | Dead variant — only producer `from_scope()` was removed | Remove `MissingUserScope` variant from `UserScopeError` enum | [objs](objs-review.md) |
| 4 | objs | user_scope.rs | `UserScope::has_access_to()` | Reflexive case (`scope.has_access_to(&scope) == true`) lacks dedicated test | Add explicit test for equal-scope authorization | [objs](objs-review.md) |
| 5 | services | auth_service/tests.rs | `test_exchange_auth_code_success` | Duplicate `#[rstest]` attribute (copy-paste error) | Remove the extra `#[rstest]` macro | [services](services-review.md) |
| 6 | services | access_request_service/ | `DefaultAccessRequestService` | No unit tests for service-layer logic (create_draft, approve_request with role params) | Create `test_access_request_service.rs` with coverage for role threading, flow_type validation, expiry | [services](services-review.md) |
| 7 | tests-js | specs/ | (missing spec) | E2E test for role downgrade flow is missing — page objects added but no spec exercises them | Add E2E spec: submit with scope_user_power_user, review page downgrade to scope_user_user, verify token role | [ui-e2e](ui-e2e-review.md) |

## Nice-to-Have (Future)

| # | Crate | File | Location | Issue | Fix Description | Report |
|---|-------|------|----------|-------|-----------------|--------|
| 8 | objs | token_scope.rs | `TokenScope::from_scope()` | No production callers — only used in own tests | Remove for consistency with UserScope, or document retention reason | [objs](objs-review.md) |
| 9 | objs | token_scope.rs | `test_included_scopes_explicit` | `windows(2)` invariant never fires for single-element case | Minor test clarity issue | [objs](objs-review.md) |
| 10 | auth_middleware | token_service/service.rs | `handle_external_client_token` | `scopes` and `access_request_scopes` are identical after filtering — redundant re-filter | Inline the single variable or add a comment explaining intent | [auth-middleware](auth-middleware-review.md) |
| 11 | auth_middleware | token_service/service.rs | post-exchange verification | `tracing::error!` for missing KC claim should be `tracing::warn!` | Change log level — this is KC config issue, not app error | [auth-middleware](auth-middleware-review.md) |
| 12 | auth_middleware | token_service/service.rs | post-exchange verification | Uses `access_request_scopes[0]` raw index when named binding is available | Use the in-scope `access_request_scope` variable instead | [auth-middleware](auth-middleware-review.md) |
| 13 | auth_middleware | token_service/tests.rs | cache-hit path | No test for cache-hit with `role: Some(...)` — CachedExchangeResult round-trip untested | Add test exercising cached token with role | [auth-middleware](auth-middleware-review.md) |
| 14 | routes_app | routes_apps/types.rs | OpenAPI description | `createAccessRequest` description says "auto-approves" — stale after auto-approve removal | Update utoipa description to reflect draft-only behavior | [routes-app](routes-app-review.md) |
| 15 | bodhi/src | review/page.tsx | `<Select>` | Duplicate `data-testid` on wrapper and trigger — wrapper prop has no effect on Radix Select | Remove `data-testid` from `<Select>`, keep only on `<SelectTrigger>` | [ui-e2e](ui-e2e-review.md) |
| 16 | bodhi/src | review/page.tsx | `handleApprove` | Fallback `?? reviewData.requested_role` is dead code — `canApprove` requires `approvedRole !== null` | Remove the fallback for clarity | [ui-e2e](ui-e2e-review.md) |
| 17 | tests-js | pages/sections/ConfigSection.mjs | `getResourceScope()` | Dead method — reads `[data-test-resource-scope]` which no longer exists | Remove the method | [ui-e2e](ui-e2e-review.md) |
| 18 | cross-cutting | CLAUDE.md | keywords table | Typo: `{toolsets:[--]}` should be `{toolsets:[...]}` | Fix typo | [cross-cutting](cross-cutting-review.md) |
| 19 | services | db/service_access_request.rs | `update_approval` | UNIQUE constraint violation on `access_request_scope` surfaces as opaque `sqlx_error` | Add comment or domain error conversion | [services](services-review.md) |

## Missing Test Coverage

| # | Crate | What's Missing | Priority | Report |
|---|-------|----------------|----------|--------|
| 1 | routes_app | Test for `approved_role > requested_role` privilege escalation check | Important | [routes-app](routes-app-review.md) |
| 2 | services | Unit tests for `DefaultAccessRequestService` (create_draft, approve_request with role params) | Important | [services](services-review.md) |
| 3 | tests-js | E2E spec for role downgrade flow (page objects ready, spec missing) | Important | [ui-e2e](ui-e2e-review.md) |
| 4 | auth_middleware | Cache-hit path test with `role: Some(...)` | Nice-to-have | [auth-middleware](auth-middleware-review.md) |
| 5 | objs | `has_access_to()` reflexive/equal-scope test | Nice-to-have | [objs](objs-review.md) |

## Security Verification Results

All security-critical checks **PASSED**:
- External app role derived exclusively from DB `approved_role`, never from JWT scope claims
- `scope_user_*` scopes NOT forwarded during token exchange (only `scope_access_request:*`)
- Constant-time comparison used for API token hash verification
- Pre-exchange validation: status=approved, azp match, user_id match
- Post-exchange validation: access_request_id claim matches DB record
- `ExternalApp.role = None` when no validated access request → rejected by `api_auth_middleware`
- Privilege escalation guard has two checks (approved > requested, approved > max_grantable)
- CSRF state parameter handling intact in OAuth flows

## Fix Order (Layered)

When applying fixes, follow this order:
1. objs issues (#3 MissingUserScope dead variant, #4 has_access_to test, #8 from_scope cleanup) → verify: `cargo test -p objs`
2. services issues (#5 duplicate rstest, #6 service unit tests) → verify: `cargo test -p objs -p services`
3. auth_middleware issues (#10-13 cleanup) → verify: `cargo test -p objs -p services -p auth_middleware`
4. routes_app issues (#1 type change, #2 missing test, #14 stale docs) → verify: `cargo test -p objs -p services -p auth_middleware -p routes_app`
5. Full backend: `make test.backend`
6. Regenerate ts-client: `make build.ts-client` (needed after #1 type change)
7. Frontend issues (#15-16 cleanup) → verify: `cd crates/bodhi && npm test`
8. E2E issues (#7 missing spec, #17 dead method) → verify: `make build.ui-rebuild && make test.napi`
9. Documentation (#18 CLAUDE.md typo)

## Reports Generated
- [objs-review.md](objs-review.md)
- [services-review.md](services-review.md)
- [auth-middleware-review.md](auth-middleware-review.md)
- [routes-app-review.md](routes-app-review.md)
- [ui-e2e-review.md](ui-e2e-review.md)
- [cross-cutting-review.md](cross-cutting-review.md)
