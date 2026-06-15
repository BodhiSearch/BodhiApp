# BodhiApp — Models Redesign API Specs
*Audience: Backend engineers implementing api.getbodhi.app + owners of bodhi-server endpoints + frontend engineers migrating the Models/Alias/API-create screens. AI coding agents second.*

This folder captures the **API contract** the new Models-page wireframes require. Wireframe/UX intent is documented in `design/models-page/specs/`; this folder defines **what APIs the frontend will call** to drive those screens.
## Companion docs
Before reading these specs, skim these for UX intent:
- `design/models-page/specs/shared-primitives.md` — the six-entity model that drives every row shape.
- `design/models-page/specs/models.md` — Models page toolbar, filters, ranked mode.
- `design/models-page/specs/alias.md` — Create local alias flow.
- `design/models-page/specs/api.md` — Create API model flow.

## Architecture summary
Three data sources, read by the frontend directly:

1. **bodhi-server** (local) — user-owned data (aliases, api-models, files, downloads). Already implemented in `crates/routes_app/src/models/*`. These APIs **stay as-is** for this pass; any gaps surface as annotated change-requests.
2. **api.getbodhi.app** (new, public) — non-user-specific, public catalog data (provider directory, supported AI providers, curated leaderboards/rankings, trending, specialization benchmark metadata). All endpoints are **new** and must be designed from scratch.
3. **huggingface.co** (external) — HF repo browsing, search, repo/tree detail, GGUF metadata. Called directly by the frontend (CORS-safe) for browse screens; mirrored periodically into api.getbodhi.app for enrichment (cost, capability, license normalisation).

The frontend composes rows across these three sources client-side. Bodhi-server does **not** proxy HF or getbodhi.app — rejected to keep the server thin and the frontend explicit about provenance.

## File layout
1. `00-architecture.md` — three-source architecture, routing principles, provenance rules, caching expectations.
2. `10-bodhi-server-apis.md` — every bodhi-server endpoint the frontend calls, annotated `keep` / `needs-change` / `new`.
3. `20-getbodhi-public-apis.md` — new public endpoints on api.getbodhi.app, with OpenAPI schemas.
4. `30-huggingface-integration.md` — thorough analysis of HF APIs used for browse + seeding into api.getbodhi.app.
5. `90-frontend-data-flows.md` — per-page data composition: which endpoints each screen calls, how responses are joined, cache invalidation.

## Conventions
- All endpoint schemas use OpenAPI 3.1 inline YAML fragments.
- Request/response examples use realistic fixtures from `design/models-page/project/screens/primitives.jsx`.
- Status markers: **KEEP** (no change required), **CHANGE** (gap — needs extension), **NEW** (does not exist yet).
- `tenant_id` / `user_id` scoping is implicit on bodhi-server endpoints (derived from auth context), not a path/query parameter.
- Timestamps are RFC 3339 in UTC (`2026-04-21T09:24:04Z`).
- All lists return a `{ data, total, page, page_size }` envelope mirroring the existing `PaginatedAliasResponse` shape.

## Out of scope for this pass
- Auth / rate-limit headers on api.getbodhi.app (marked TBD throughout).
- Schema versioning for api.getbodhi.app (`v1` is assumed).
- OAuth lifecycle for API providers that require it (`anthropic-oauth`, `google-gemini`). Covered by existing bodhi-server MCP/OAuth machinery; out of scope for models-page APIs.
- Webhook / SSE push for catalog updates — polling only for this pass.
