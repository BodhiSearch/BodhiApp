# Multi-Tenancy Plan - Context Index

## Context Files

| File | Topic | Status |
|------|-------|--------|
| [decisions-ctx.md](./decisions-ctx.md) | All Q&A decisions from planning sessions | Complete |
| [current-arch-ctx.md](./current-arch-ctx.md) | Current architecture analysis | Complete |
| [db-migration-ctx.md](./db-migration-ctx.md) | Database schema changes: sqlx::Any, org_id, dual-DB | Complete |
| [auth-keycloak-ctx.md](./auth-keycloak-ctx.md) | Keycloak Organizations, per-org clients, auth flow changes | Complete |
| [service-layer-ctx.md](./service-layer-ctx.md) | Service trait changes, org propagation pattern | Complete |
| [middleware-ctx.md](./middleware-ctx.md) | Org resolution middleware, X-BodhiApp-Org, subdomain routing | Complete |
| [config-deployment-ctx.md](./config-deployment-ctx.md) | Feature flags, Dockerfile variants, deployment modes | Complete |
| [testing-ctx.md](./testing-ctx.md) | Multi-tenant test strategy, fixtures, DB backends | Complete |
| [org-lifecycle-ctx.md](./org-lifecycle-ctx.md) | Org provisioning, audit logging, frontend org features | Complete |

## Phase Plan Files

| File | Phase | Description |
|------|-------|-------------|
| [phase-1-session-pg.md](./phase-1-session-pg.md) | 1 | Session PostgreSQL migration with separate pool |
| [phase-2-db-abstraction.md](./phase-2-db-abstraction.md) | 2 | sqlx::Any DB abstraction, SqliteDbService → DbServiceImpl |
| [phase-3-org-threading.md](./phase-3-org-threading.md) | 3 | Organizations table, org_id threading, auth middleware, DataService, SecretService removal |
| [phase-4-cache-redis.md](./phase-4-cache-redis.md) | 4 | Generic CacheService, Redis impl, org config caching |
| [phase-5-docker-deployment.md](./phase-5-docker-deployment.md) | 5 | Docker, Traefik, docker-compose, deployment |
| [phase-6-frontend.md](./phase-6-frontend.md) | 6 | Org switcher, mode detection, conditional UI |

## Overview Files

| File | Description |
|------|-------------|
| [README.md](./README.md) | Plan overview with phase summary |
| [checklist.md](./checklist.md) | Implementation progress checklist |
