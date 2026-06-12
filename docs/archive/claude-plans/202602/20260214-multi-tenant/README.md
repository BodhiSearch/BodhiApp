# Multi-Tenancy Plan for BodhiApp

## Overview

Transform BodhiApp from a single-tenant local application into a horizontally-scalable, multi-tenant hosted SaaS platform while maintaining backwards compatibility for self-hosted single-tenant deployments.

### Key Architecture Decisions
- **Data isolation**: Row-level with `org_id` column (PostgreSQL RLS for defense-in-depth)
- **Keycloak**: KC 26+ Organizations, single realm, per-org clients with client-scoped roles
- **Database**: `sqlx::Any` for dual SQLite/PostgreSQL support in single implementation
- **Sessions**: PostgreSQL-backed with separate connection pool (configurable)
- **Cache**: Generic trait with Redis (multi-tenant) / in-memory (single-tenant) implementations
- **Runtime mode**: `BODHI_MULTI_TENANT` env var, same binary, no Cargo feature flags
- **Org resolution**: Traefik injects `X-BodhiApp-Org` from subdomain; app resolves via cached DB lookup
- **Org provisioning**: External service (`new.getbodhi.app`), app is consumer only
- **SecretService**: Removed entirely; secrets move to organizations table + env vars

### What's NOT in Scope
- Rate limiting (deferred)
- Audit logging implementation (interface defined, NATS JetStream deferred)
- Org management endpoints (external service handles provisioning)
- Billing/tier system
- Per-org settings

---

## Phase Summary

| Phase | Name | Depends On | Deliverable |
|-------|------|------------|-------------|
| 1 | Session PostgreSQL Migration | - | tower-sessions on PostgreSQL with separate pool |
| 2 | DB Abstraction (sqlx::Any) | Phase 1 | DbServiceImpl with AnyPool, dual SQLite/PG support |
| 3 | Org Threading | Phase 2 | organizations table, org_id in all tables, auth middleware, DataService refactor |
| 4 | Cache + Redis | Phase 3 | Generic CacheService, Redis impl, org config caching |
| 5 | Docker & Deployment | Phase 4 | Multi-tenant Docker, Traefik config, docker-compose |
| 6 | Frontend | Phase 5 | Org switcher, org-aware API calls, login per org |

Each phase produces a **working, testable application**.

---

## Context Files

| File | Description |
|------|-------------|
| [decisions-ctx.md](./decisions-ctx.md) | All Q&A decisions from planning sessions |
| [current-arch-ctx.md](./current-arch-ctx.md) | Current architecture analysis |
| [db-migration-ctx.md](./db-migration-ctx.md) | Database schema changes, sqlx::Any, migrations |
| [auth-keycloak-ctx.md](./auth-keycloak-ctx.md) | KC Organizations, per-org auth, SecretService removal |
| [service-layer-ctx.md](./service-layer-ctx.md) | Service trait changes, org propagation pattern |
| [middleware-ctx.md](./middleware-ctx.md) | Org resolution middleware, header injection |
| [config-deployment-ctx.md](./config-deployment-ctx.md) | Env vars, Docker, Traefik, startup flow |
| [testing-ctx.md](./testing-ctx.md) | Test strategy, fixtures, migration of existing tests |
| [org-lifecycle-ctx.md](./org-lifecycle-ctx.md) | Org provisioning, audit logging, frontend org features |

## Phase Plan Files

| File | Phase |
|------|-------|
| [phase-1-session-pg.md](./phase-1-session-pg.md) | Session PostgreSQL migration |
| [phase-2-db-abstraction.md](./phase-2-db-abstraction.md) | sqlx::Any DB abstraction |
| [phase-3-org-threading.md](./phase-3-org-threading.md) | Org table + org_id threading + auth + DataService |
| [phase-4-cache-redis.md](./phase-4-cache-redis.md) | Generic CacheService + Redis |
| [phase-5-docker-deployment.md](./phase-5-docker-deployment.md) | Docker + Traefik deployment |
| [phase-6-frontend.md](./phase-6-frontend.md) | Frontend multi-tenant changes |
