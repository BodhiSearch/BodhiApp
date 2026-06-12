# Session PostgreSQL Migration -- Re-implementation Plan

## Background

The `multi-tenant` worktree implemented session PostgreSQL migration (Phase 1 of the multi-tenant plan) but main diverged too far to merge. This plan captures the functional requirements from that work as an exploratory specification for re-implementation on main.

The implementer should discover main's current state at implementation time -- specific file paths, type names, and module structures may have changed since this plan was written.

### Source References

- Worktree: `multi-tenant` branch (uncommitted session PG changes + 19 committed auth/e2e refactors)
- Original multi-tenant plan: `ai-docs/claude-plans/20260214-multi-tenant/`
- Agent transcript: `a612e30c-83e0-46c1-a23f-176198546aae`

## Scope

This plan covers **session store PostgreSQL migration only** -- enabling tower-sessions to run on either SQLite (standalone) or PostgreSQL (cluster deployment). It does NOT cover:

- Application database migration to PostgreSQL (future Phase 2)
- Organization table / SecretService removal (future Phase 3)
- E2E test infrastructure overhaul (separate concern)
- Auth middleware refactors (archived separately in this folder)

## Key Decisions

| Decision | Choice |
|----------|--------|
| Deployment mode flag | `BODHI_DEPLOYMENT=standalone\|cluster` |
| Session DB config | `BODHI_SESSION_DB_URL` (separate from future `BODHI_DATABASE_URL`) |
| Default behavior | SQLite for standalone, PG required for cluster |
| Backend selection | Runtime via URL scheme, no Cargo feature flags |
| Naming convention | `SqliteSessionService` renamed to `DefaultSessionService` |
| Module structure | Suggested: folder module with per-backend files (implementer may adjust) |
| Testing approach | Suggested: rstest `#[values]` parameterized tests (implementer may adjust) |
| Test infrastructure | `docker-compose-test-deps.yml` for PostgreSQL |

## Crate Dependency Order

Changes propagate upstream-to-downstream:

```
services -> lib_bodhiserver -> auth_middleware -> routes_app -> server_app -> lib_bodhiserver_napi
```

Only `services` through `lib_bodhiserver_napi` are affected. `objs` and `server_core` have no session-related code.

## Phase Files

| File | Crate | Scope |
|------|-------|-------|
| [01-services.md](01-services.md) | `services` | Session store abstraction, PG backend, config, rename, docker-compose, tests |
| [02-lib-bodhiserver.md](02-lib-bodhiserver.md) | `lib_bodhiserver` | AppServiceBuilder session backend selection |
| [03-auth-middleware.md](03-auth-middleware.md) | `auth_middleware` | Import updates for DefaultSessionService |
| [04-routes-app.md](04-routes-app.md) | `routes_app` | Import updates, test utility changes |
| [05-server-app.md](05-server-app.md) | `server_app` | Live server test utility updates |
| [06-lib-bodhiserver-napi.md](06-lib-bodhiserver-napi.md) | `lib_bodhiserver_napi` | NAPI config export, Makefile targets |

## Archived References

| File | Description |
|------|-------------|
| [auth-refactors.md](auth-refactors.md) | Archive of auth refactors done in worktree (ExternalApp role optional + ResourceScope move). Nice-to-have improvements, not part of the session PG implementation flow. |

## Validation at Each Phase

Each phase follows the layered development methodology:

1. **Per-crate**: `cargo test -p <crate>` for the crate being modified
2. **Cumulative**: `cargo test -p <all-changed-crates-so-far>` to verify no regressions
3. **Full backend** (after all Rust phases): `make test.backend`
4. **Format**: `cargo fmt`
