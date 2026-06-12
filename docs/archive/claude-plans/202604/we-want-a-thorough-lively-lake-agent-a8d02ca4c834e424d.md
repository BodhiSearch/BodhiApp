# ai-docs/01-architecture/ — Staleness Audit

25 files. README claims a "January 2025 content recovery" but the corpus reflects a
**reverted-to-Next.js** era. Live truth (per root CLAUDE.md, crates/CLAUDE.md,
crates/bodhi/src/CLAUDE.md): **Vite + React + TanStack Router + TanStack Query v5**,
backend on **SeaORM** (not SQLx), `objs` crate **no longer exists** (merged into `services`).

## Headline staleness
- 13/25 files reference Next.js; README lists "Frontend Next.js (frontend-react.md)".
- README tech table: Next.js v14.2.6, React Query v3.39.3, App Router, SQLx, next-pwa — all wrong.
- README links `roadmap.md` which does not exist (broken link).
- `ai-ide-memories.md` actively asserts "React Query v3.39.3 (not TanStack Query)" — directly false now.
- 10 files reference the removed `objs` crate; ARCHITECTURE_SUMMARY references `routes_oai`/`routes_all` layering.
- Backend deep-dives (error-l10n, settings, openapi-utoipa, testing-utils) are good content but overlap crate CLAUDE.md and carry `objs`/SQLx staleness.

## Recommendation by group
- **Frontend group → trash/superseded** by crates/bodhi/src/CLAUDE.md + TESTING.md:
  frontend-react, frontend-query, api-integration, frontend-testing, ui-design-system,
  TESTING_GUIDE, ai-ide-memories.
- **Meta/index → trash**: README (broken+stale), ARCHITECTURE_SUMMARY, testing-strategy, development-conventions.
- **Backend deep-dives → heavy rewrite or research-archive**: backend-error-l10n, backend-settings-service,
  backend-openapi-utoipa, backend-testing-utils, backend-testing, backend-development-conventions, rust-backend.
- **Cross-cutting → cleanup**: system-overview, bodhi-platform, architectural-decisions, authentication,
  app-status, build-config, tauri-desktop.
