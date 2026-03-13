# App Access Request — Implementation Reference

> **Purpose**: Central context for AI coding assistants working on the app access request feature.
> All content derived from source code as of 2026-03-13. Plans/docs used only for direction.
>
> **Scope**: 3rd-party OAuth app access requests only (not user role elevation requests).

## Reading Order

1. **[data-model.md](./data-model.md)** — Start here. Table schema, domain objects, JSON shapes, status FSM, indexes. Referenced by all other files.

2. **[draft-creation-flow.md](./draft-creation-flow.md)** — How external apps create access request drafts. Unauthenticated endpoint, service layer, status polling.

3. **[review-approval-flow.md](./review-approval-flow.md)** — User reviews and approves/denies requests. Role dropdown, privilege escalation guard, frontend review page, KC consent registration.

4. **[token-validation-flow.md](./token-validation-flow.md)** — Runtime: how external app tokens are validated. Pre/post KC token exchange, scope extraction, AuthContext::ExternalApp construction, caching.

5. **[entity-enforcement-flow.md](./entity-enforcement-flow.md)** — Runtime: per-request entity-level validation. access_request_auth_middleware, Toolset/MCP validators, approved JSON checking.

6. **[keycloak-integration.md](./keycloak-integration.md)** — Full KC round-trip. SPI endpoints, dynamic scope registration, consent, token exchange, test-oauth-app reference implementation.

## Quick Architecture

```
External App                    BodhiApp                         Keycloak
    |                              |                                |
    |-- POST /apps/request-access ->|                                |
    |<- 201 {id, review_url}       |                                |
    |                              |                                |
    |   [User opens review_url in browser]                          |
    |                              |-- GET /access-requests/{id}/review
    |                              |-- PUT /access-requests/{id}/approve
    |                              |---------- register_consent ------->|
    |                              |<--------- {access_request_scope} --|
    |                              |                                |
    |-- GET /apps/access-requests/{id} (poll) -->|                  |
    |<- {status: approved, access_request_scope} |                  |
    |                              |                                |
    |-- OAuth authorize (scope includes access_request_scope) ----->|
    |<- access_token (with access_request_id claim) ---------------|
    |                              |                                |
    |-- API call with Bearer token ->|                              |
    |                              |-- token exchange -------------->|
    |                              |<- scoped token ----------------|
    |                              |-- validate access_request_id   |
    |                              |-- validate entity in approved  |
    |<- API response               |                                |
```

## Key Design Facts

- **No auto-approve**: Every access request requires user review via the UI
- **Global at creation**: Draft requests have `tenant_id = NULL`, `user_id = NULL` — bound at approval
- **Role via access request**: `requested_role`/`approved_role` stored on the request, not via OAuth scopes
- **Non-auth-scoped service**: `AccessRequestService` is intentionally not auth-scoped (handles both anonymous and authenticated callers)
- **10-minute expiry**: Draft requests auto-expire; checked lazily on read
- **Entity enforcement**: Toolsets and MCPs are the only entity types with access request validation

## File Locations (Quick Reference)

| Layer | Path |
|-------|------|
| Domain objects | `crates/services/src/app_access_requests/access_request_objs.rs` |
| Entity | `crates/services/src/app_access_requests/app_access_request_entity.rs` |
| Repository | `crates/services/src/app_access_requests/access_request_repository.rs` |
| Service | `crates/services/src/app_access_requests/access_request_service.rs` |
| Service errors | `crates/services/src/app_access_requests/error.rs` |
| Migration | `crates/services/src/db/sea_migrations/m20250101_000009_app_access_requests.rs` |
| Route handlers | `crates/routes_app/src/apps/routes_apps.rs` |
| Route schemas | `crates/routes_app/src/apps/apps_api_schemas.rs` |
| Route errors | `crates/routes_app/src/apps/error.rs` |
| Auth middleware | `crates/routes_app/src/middleware/auth/auth_middleware.rs` |
| Token service | `crates/routes_app/src/middleware/token_service/token_service.rs` |
| Entity middleware | `crates/routes_app/src/middleware/access_requests/access_request_middleware.rs` |
| Entity validators | `crates/routes_app/src/middleware/access_requests/access_request_validator.rs` |
| Entity errors | `crates/routes_app/src/middleware/access_requests/error.rs` |
| Frontend review | `crates/bodhi/src/app/ui/apps/access-requests/review/page.tsx` |
| Frontend hooks | `crates/bodhi/src/hooks/useAppAccessRequests.ts` |
| Auth service (KC) | `crates/services/src/auth/auth_service.rs` |
| AuthContext | `crates/services/src/auth/auth_context.rs` |
| Test oauth app | `crates/lib_bodhiserver_napi/test-oauth-app/` |
