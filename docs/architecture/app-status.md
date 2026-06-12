# Application Status Guide

## Overview
Bodhi App tracks setup and operational state through the `AppStatus` enum (`crates/services/src/tenants/tenant_objs.rs`). OAuth2 is **required for all deployments** — there is no non-authenticated mode. This guide explains the states and the flow between them.

## Status States

### 1. Setup (`setup`)
- Default state on first launch — the app has no configured resource server yet.
- The setup flow (`POST /bodhi/v1/setup`) registers the server (name + optional description) and transitions to `resource_admin`.

### 2. Resource Admin (`resource_admin`)
- Intermediate state after setup: the server is registered but has no admin user yet.
- The first user to log in via OAuth becomes the admin, which transitions the app to `ready`.

### 3. Ready (`ready`)
- Final operational state — fully configured and serving requests.
- Role-based access control is enforced (`ResourceRole`: `Anonymous < Guest < User < PowerUser < Manager < Admin`).

## State Transitions

```
  ┌─────────┐   POST /bodhi/v1/setup   ┌────────────────┐   first OAuth login   ┌─────────┐
  │  setup  │ ───────────────────────► │ resource_admin │ ────────────────────► │  ready  │
  └─────────┘                          └────────────────┘                       └─────────┘
```

A single forward path — there is no branch. (`TenantSelection` was removed; multi-tenant Anonymous and sessions without an active tenant resolve to `ready`. See `crates/routes_app/CLAUDE.md`.)

## API Response (`GET /bodhi/v1/info`)
Returns the `AppInfo` struct — see `crates/routes_app/src/setup/setup_api_schemas.rs` for the canonical shape:

```json
{
  "version": "0.1.0",
  "commit_sha": "abc1234",
  "status": "ready",
  "deployment": "standalone",
  "client_id": "my-client-id",
  "url": "https://example.com"
}
```

- `status` — the `AppStatus` (`setup` | `resource_admin` | `ready`), serialized snake_case.
- `deployment` — `standalone` | `multi_tenant`.
- `client_id` — active tenant's OAuth client_id; omitted when not authenticated with an active tenant.

## Key Points
- Status is persisted across restarts and only advances forward through the setup flow.
- Authentication is unconditional — every deployment runs OAuth2; authorization headers are always honored.
- Admin is granted to the first OAuth login while in `resource_admin`.
- Role changes clear the affected user's sessions (`crates/routes_app/CLAUDE.md`).
