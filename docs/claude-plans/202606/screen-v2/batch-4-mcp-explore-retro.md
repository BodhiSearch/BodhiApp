# Batch 4-1 — Explore · MCP Servers (catalog) — Retrospective

Status: implementation complete. **Shipped V2-only — NO flag.** Built as 4 thin evolutionary slices
(list → rail → facets → instance-join), each verified live in Claude-in-Chrome, unit + E2E tested,
styling-reviewed, then committed. Plan: `../we-are-continiuing-docs-claude-plans-202-cheeky-crab.md`.
Gaps/deferrals: `mcp-techdebt.md`.

Commits: `53986411` (P1 list) · `a2e6a690` (P2 rail) · `2c776048` (metadata-removal) · `ca2f6594`
(P3 facets) · `7e170e40` (P4 instance-join).

RTL: full UI suite **1200 pass / 5 skip / 0 fail** (16 MCP-explore tests + 4 hook tests added). E2E:
`specs/mcps/explore-mcp.spec.mjs` green in **both** standalone AND multi_tenant; the shared-catalog-kit
specs (api-explore, providers, local-discovery) re-ran green (no regression from reuse). GATE B (live)
passed: light + dark + responsive (rail→drawer <768px, STATUS column drops), console clean apart from the
pre-existing app-wide view-transition `InvalidStateError` (swallowed by the shell's `useViewTransition`).

## This was a near-clone, by design

The Explore · MCP page is the same shape as the shipped Explore · API Models page, so the whole catalog
kit was reused **unchanged**: `CatalogTable` + `LinkRow` (Vimium hints), `useListKeyNav`, `ShellSearch`,
`ShellPagination`, `ResetButton` (3-state), `ColumnPicker`/`useHiddenColumns`, `useViewTransition`,
`catalog-format` (monogram/tint), `catalog.css`. Per-page code is just the hook, the columns adapter,
the sidebar, the rail, and the instance-join. No new design primitives.

## The data drove the UI (not the mock)

The hi-fi mock predates the v1 catalog API and depicts a far richer server (Status/Tools/Category/
Auth-taxonomy/Publisher/Stats/tabs). Per the user's rule — shipped page wins on layout, **reference API
wins on data** — every unsupported mock field was logged in `mcp-techdebt.md` and either dropped or
derived, never built as dead UI:
- **Facets are data-driven** from the response `facets` arrays (Category group hides while empty in v1;
  Auth renders the single `http` chip) — no hard-coded taxonomy, so they light up as enrichment lands.
- **Per-user status is a client-side join** (`useListMcps` → `mcp_server.url` ↔ catalog `endpoint_url`,
  normalized for trailing-slash + case) → enabled/disabled/none. The catalog API carries zero per-user
  state by design.
- `verified` + `installed` are **client-side cuts** (no API param); `category` + `auth` are server-side.

## Decisions worth recording

- **Naming collision avoided:** the reference-catalog hook is `hooks/reference/useMcpCatalog.ts`
  (`useMcpServers`/`useMcpServerDetail`), distinct from the pre-existing `hooks/mcps/useMcpServers.ts`
  (the user's MCP-server allowlist). Different domains, different concepts.
- **Types bump:** `@bodhiapp/reference-api-types ^0.0.11 → ^0.0.12` (the version that ships the MCP wire
  types). Installed 0.0.11 lacked them — verified the published 0.0.12 `.d.ts` before bumping.
- **Provenance Metadata removed mid-batch** (user request): `source`/`sources`/`first_seen_at`/
  `last_scraped_at` are scrape-internal and are being **deleted from the backend** (don't leak). The rail
  is now Description + Connection only; the UI reads none of those fields, so the upcoming type drop is a
  no-op here. Stub + assertions updated to not carry them.
- **Logo proxy:** the backend now serves `logo_url` as a relative path (`/api/v1/mcp-servers/logos/{id}`)
  from the reference-API domain. `McpServerLogo` resolves relative URLs against `reference_api_url` (and
  passes absolute CDN URLs through, which is what the deployed API still returns today).
- **Multi-tenant:** no `MultiTenantGuard` — the catalog is public-readable and MCP isn't deployment-gated
  (matches `/ui/mcps/`). The E2E spec passes in both projects.

## Deferred (later MCP batches — see `mcp-techdebt.md`)

- Migrate **My MCPs** (`/ui/mcps/`, still V1) to the V2 shell + a real instance-management screen.
- **V2 new-instance / connect flow** (OAuth, API-key, approval). The rail's "Add to My MCPs" CTA is a
  plain deep-link to the existing V1 `/mcps/new/` until then (that route takes no prefill params).
- **Re-enable enrichment UI** (Category rail, auth taxonomy, tools/publisher/license/repo) — all coded
  data-driven, so they activate automatically once the reference API populates those fields.

## Next

Batch 4 continues with the My MCPs migration + new-instance V2 flow, or hand off to Batch 5 (Chat).
Update `tracker.md` accordingly (Batch 4 = 🚧; 4-1 Explore = ✅).
