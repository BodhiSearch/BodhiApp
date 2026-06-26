# Tech Debt & Gap Analysis — Explore · MCP Servers

**Status:** analysis complete, feeds the Batch-4 (MCP · Explore) plan. **Date:** 2026-06-26.
**Scope:** the new catalog page `/ui/models/explore/...` → actually a **new MCP nav section**
`/ui/mcps/explore/`, fed by the Bodhi Reference API `/api/v1/mcp-servers`, joined client-side with the
user's own instances from BodhiApp's `/bodhi/v1/mcps`.

## How this was produced

Side-by-side of the **hi-fi mock** (`http://localhost:8000/Bodhi MCP Discover v2.html`) against the
**shipped, precedent-setting** Explore · API Models page (`http://localhost:1135/ui/models/explore/api/`)
and a source read of the **reference-API** MCP surface (`BodhiSearch/api-getbodhi-app`:
`worker/routes/mcp.*.ts`, `worker/schemas/mcp.ts`, `worker/services/real/RealMcpReader.ts`,
`drizzle/0004_mcp_servers.sql`, `packages/api-types/src/index.ts`) plus the published
`@bodhiapp/reference-api-types@0.0.12` `.d.ts`. **Decision rule:** the shipped Explore · API Models page
wins on layout/components; the **reference API wins on data** — where the mock shows a field the v1 API
doesn't serve, it's tech debt logged here, not built.

> The mock was drawn before the v1 catalog API existed, so it depicts a **much richer** server than the
> scraped v1 data supports. Most of the mock's "richness" is either (a) future reference-API enrichment
> (`category`, `tools`, `publisher`, `license`, `repo`, the auth taxonomy) or (b) **per-user app state**
> that is *not* reference data at all (connection status, approval, instance counts, usage stats).

---

## What the v1 reference API actually serves

`GET /api/v1/mcp-servers` → `{ items: McpServerSummary[], facets: { category, auth }, page, page_size,
total }`. `GET /api/v1/mcp-servers/{id}` → `McpServerDetail`. Query: `q` (substring over name+desc),
`category` (repeatable OR), `auth` (repeatable OR), `sort=name`, `order`, `page`, `page_size` (≤100).

- **`McpServerSummary`**: `id, slug, name, description, logo_url, endpoint_url, transport, auth_type,
  category, external_link, verified, featured`.
- **`McpServerDetail`** adds: `details, publisher, tools[], license, repo, source, sources[],
  first_seen_at, last_scraped_at`.
- **v1 realities (per `docs/functional/mcp.md`):** `transport` is **always** `"streamable-http"`;
  `auth_type` is **always** `"http"` (placeholder); `category` is **null** and `facets.category` is
  **empty**; `publisher/tools/license/repo` are **null**. Live data = 198 servers today.

---

## Gap table — mock feature → reality → decision

| Mock element | v1 API reality | Decision |
|---|---|---|
| **Status column** + Connect / Request Approval / My instances buttons | Not catalog data | **Join `/bodhi/v1/mcps`** (user instances) client-side on `endpoint_url`↔`mcp_server.url`; render a derived status badge. *Connect/approval/instance flows themselves are deferred* (no create-instance form in this batch). |
| **Tools** count column + rail "Tools (N)" list | `tools` is **null** in v1 | Hide the column + rail section (show nothing, not "0"). Re-enable when enrichment lands. |
| **Category** column + Category facet rail (Productivity/Dev Tools/…) | `category` null, `facets.category` `[]` | Render the rail **data-driven from `facets.category`** → empty ⇒ hidden today; auto-activates when data arrives. No hard-coded category list. |
| **Auth Type** facet = OAuth / API Key / No auth | `auth_type` always `"http"`; `facets.auth` = `["http"]` | Render data-driven from `facets.auth` → single "http" chip today. Do **not** hard-code the 4-value taxonomy. |
| **Availability** facet (Admin-approved / Available) | App state, not reference data | **Derive** from the joined user-instance state as a client-side facet (Installed / Not installed). Drop "admin-approved" (no admin-approval backend in scope). |
| **Publisher** facet (Verified / Official / Community) | only `verified: boolean` + null `publisher` | Keep a single **Verified** pill (data-backed); drop Official/Community. |
| Rail **"Connected"** badge + install count "7.4k" | Connection = joined state; counts not served | Connected/Enabled badge from the join; **drop the install count.** |
| Rail tabs **About / Capabilities / Connection / Connect / Metadata** | Only About-ish + Metadata data exist | Single-scroll rail (matches API-Models rail), not tabs. Sections: Description (`details`), Connection (`endpoint_url`/`transport`/`external_link`), Metadata (`source`/`sources`/timestamps/`verified`/`featured`). Hide null sections. |
| Rail **Stats (30d)**: installs / calls / uptime / p50 | Not served anywhere | **Drop entirely.** |
| Rail **Manage Instances** / "Connect" CTA | Needs a new-instance flow | Rail CTA deep-links to the **existing** `/ui/mcps/new/` create flow (prefilled where possible); building a V2 new-instance form is **out of scope** for this batch. |
| Topbar **"Role: User"** switcher | Demo control | Drop. |
| **My MCPs** nav entry | Existing V1 page at `/ui/mcps/` | Keep the existing V1 page as-is; **do not migrate** it this batch (deferred to a later MCP batch). |

---

## Carry-forward / follow-ups (later MCP batches)

- **Migrate "My MCPs"** (`/ui/mcps/`) to the V2 shell + a real instance-management V2 screen.
- **V2 new-instance / connect flow** (OAuth, API-key, approval) — the catalog's "Add"/"Connect" CTA is a
  deep-link stub until then.
- **Re-enable enrichment UI** (category rail, auth taxonomy, tools list, publisher/license/repo rail
  sections) when the reference API populates those fields — all already coded data-driven so they
  light up automatically; only the `tools`/`publisher`/`license`/`repo` rail sections need un-hiding.
- **Reference-API `sort`** is `name`-only today; a relevance/featured sort would need a backend change +
  types bump (mirror the API-Models `techdebt-api-models.md` R3/R7 pattern).
