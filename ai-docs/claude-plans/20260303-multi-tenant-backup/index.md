# Multi-Tenant Architecture Context Documents

> **Purpose**: Archived research, decisions, and analysis supporting BodhiApp's multi-tenancy implementation.
> These docs preserve the full reasoning behind architectural choices so future work can build on this foundation.
>
> **Created**: 2026-03-03
> **Status**: Initial research and planning complete. Implementation pending.

---

## File Index

| File | Purpose | Read when... | Update when... |
|------|---------|-------------|----------------|
| `index.md` | This file. Summary and progressive disclosure | Starting any multi-tenant work | Adding new context docs |
| `decisions.md` | All architectural decisions (D1-D19) with rationale | Making new architectural choices, reviewing tradeoffs | New decisions made during implementation |
| `settings-analysis.md` | Every BODHI_* setting categorized (global/LLM/editable) | Working on settings, deployment modes, configuration | New settings added or categorization changes |
| `table-analysis.md` | All 14 tables: current schema, tenant_id changes, index updates | Working on migrations, entity files, repository methods | Schema changes during implementation |
| `frontend-tasks.md` | Deferred frontend work items for multi-tenant UI | Starting frontend multi-tenant work | Backend changes create new frontend requirements |
| `auth-flow-analysis.md` | Current and proposed auth flows, AuthContext changes | Working on auth middleware, AuthContext, login flows | Auth flow decisions change or implementation reveals issues |
| `claude-research-multi-tenant.md` | Architecture summary (tenants, routing, RLS, provisioning) | Understanding the overall architecture vision | Research conclusions change |
| `claude-research-multi-tenancy-research-corpus.md` | Full research corpus (ADRs, Keycloak, Cloudflare, RLS, billing) | Deep-diving into specific technology choices | New research conducted |

## Quick Context

BodhiApp operates in two deployment modes:
- **`standalone`** (default): Single-tenant, SQLite/PG, local LLM inference, desktop or server
- **`multi`**: Multi-tenant, PostgreSQL only, no local inference, stateless app instances

The `tenants` table (renamed from `apps`) is the central identity. Every data table has a `tenant_id` FK. In standalone mode, one tenant row exists. In multi mode, many tenants coexist.

## Key Principles
1. **Unified schema** — both modes share the same DB schema. No conditional columns.
2. **tenant_id everywhere** — every data table has `tenant_id` FK for RLS + defense-in-depth
3. **App-layer + RLS** — auth-scoped services filter by tenant_id; PG RLS as defense-in-depth
4. **"app" = global deployment, "tenant" = scoped resource** — clear domain terminology
