# SeaORM Migration — Documentation Index

## Overview

This folder documents the SeaORM migration (commit d49357535) that replaced BodhiApp's raw sqlx persistence layer with SeaORM, adding dual-database support (SQLite + PostgreSQL).

## Documents

| File | Description |
|------|-------------|
| `context.md` | Post-implementation snapshot of current code state |
| `index.md` | This file (folder navigation) |
| `migration-decisions.md` | Architectural decisions (ULID, DateTime, CITEXT, encryption, entity patterns) |
| `implementation-conventions.md` | Coding patterns (Entity A/B, tests, errors, migrations) |
| `phase-1-objs.md` | Foundation layer changes (objs crate) |
| `phase-2-services-entities-migrations.md` | SeaORM schema layer (entities, migrations) |
| `phase-3-services-repositories.md` | Repository implementations (9 traits, 9 impls) |
| `phase-4-services-test-infra.md` | Test infrastructure (SeaTestContext, TestDbService, 9 test files) |
| `phase-5-auth-middleware-routes.md` | Upstream consumer changes (auth, routes, services) |
| `phase-6-e2e-infra.md` | E2E & infrastructure (Playwright dual-DB, Docker, CI, frontend) |
