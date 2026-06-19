# Batch 3-1 — Models · All Models — Retrospective

Status: implementation complete; backend (816) + RTL (979) green; the V2 E2E spec passes; GATE B
(live) passed. **Batch 3 was re-scoped (2026-06-19) into sub-phases 3-1…3-5** — this batch lands
**only the All Models list** (the first full-stack screen-v2 batch).

> **Flag NOT retired this sub-phase (deliberate deviation from the per-batch rule).** The `models`
> flag stays **default-off**; V2 ships behind it and V1 `ModelsPageContent` is **kept**. Reason: the
> existing model/api-model E2E specs (`model-alias`, `model-router`, `model-metadata`, `api-models/*`)
> drive **create/edit/delete via the V1 list↔form flow** (preview modal, `edit-button`/`delete-button`
> row controls, `verifyPreviewCapability` ×36). The V2 list intentionally replaces those with a rail,
> and the **forms are still V1** (3-2/3-4). Making V2 the default now would break that suite before its
> forms migrate. **The `models` flag + the V1 list are retired in the final Models sub-phase (3-4/3-5),
> when the forms are V2 and those specs migrate together.** This keeps `main` green and is reversible.

## What landed

**One screen migrated to V2 (`/models/`), behind the `models` flag:**
- **All Models** — V2 AppShell list with a **published faceted `sidebar`** (TYPE / CAPABILITY / SIZE
  dual-slider / API-FORMAT incl. the new **Liberty** bucket), a toolbar TYPE quick-tab row
  (`ShellFilterTabs`), a collapsible search, selectable rows (`LinkRow`) with type/connection badges,
  and a read-only **detail rail with 4 variants** (Local File / Model Alias / API Model / Fallback)
  whose **Edit CTA** navigates to the still-V1 form routes (`alias/edit`, `api/edit`, `router/edit`).
  Real-data-only; the prototype's discovery sub-views (Local/API Models) and the 3 forms are out of
  scope (→ 3-2…3-7; see the re-sequenced sub-phase list below).

**First full-stack screen-v2 batch — backend changes (unlike Batches 0–2, which were presentation-only):**
- Added `size: Option<u64>` to `UserAliasResponse` + `ModelAliasResponse`; `models_index` resolves
  on-disk file size **page-scoped** (mirrors the existing page-scoped metadata batch) via
  `hub_service.find_local_file`.
- Added **server-side facet filtering** to `GET /bodhi/v1/models` via a new `AliasFilterParams`
  (`type` / `api_format` / `size_min` / `size_max` / `capability`, all comma-separated multi-value).
  Filters apply **before** sort + pagination so `total` + the page reflect the filtered set. Capability
  filtering fetches whole-list metadata **only when a capability facet is active** (cheap common path).
- API-FORMAT facet buckets: `openai`←OpenAI, `responses`←OpenAIResponses, `anthropic`←Anthropic +
  AnthropicOAuth, `gemini`←Gemini, **`liberty`←LlmLibertyOauth** (added per user).
- Capability facet maps to existing GGUF metadata: `vision`→capabilities.vision,
  `tool_use`→tools.function_calling, `reasoning`→thinking (chat/embeddings dropped — no field).
- OpenAPI + ts-client regenerated; `size` + the filter params flow to `@bodhiapp/ts-client`.

**Frontend infra:**
- `useListModels(page, pageSize, sort, sortOrder, filter?)` — optional `ModelsFilter` threaded into a
  CSV query-param serializer (`buildModelsFilterParams`) + a sorted filter cache-key. Added
  `placeholderData: keepPreviousData` so the list doesn't flash empty on facet/page change.
- New `routes/models/-components/`: `ModelsScreenV2.tsx`, `ModelSidebarFacets.tsx`, `ModelDetailRail.tsx`,
  `models.css` (scoped under `.models-screen` / `m-*` prefixes; reuses `list.css` `l-*`/`dp-*`).
- `routes/models/index.tsx` flag-branches inside the page component (V2 ↔ existing V1
  `ModelsPageContent`), `AppInitializer` stays outside.

## Decisions made (with the user)
1. **Batch 3 split into 3-1…3-5** — 3-1 = All Models list (+ detail rail) only; Local/API discovery →
   3-5 (reference API); the 3 forms → 3-2 (local) / 3-3 (files/pull consolidation) / 3-4 (fallback + API).
2. **Server-side filtering + backend `size`/capability** — the user chose accurate global filtering
   over Batch-1/2-style client-side-per-page, making this a full-stack batch.
3. **API-FORMAT adds Liberty**; **CAPABILITY ships the 3 mapped fields** (vision/tool-use/reasoning),
   dropping chat/embeddings (no backing data).
4. **SIZE slider only constrains local rows**; API/router rows (no file) are never hidden by size — to
   hide them, pick TYPE. (User: "if user needs to hide other rows, can select type as local.")
5. **Detail rail in scope**; the Edit CTA navigates to the still-V1 form routes.

## Surprises / learnings (carry forward)
- **GATE B earned its keep again** — the live walk confirmed server-side filtering against the *real*
  backend (the running `make app.run.live` binary had to be rebuilt; a stale binary silently ignores
  the new query params and returns all rows — automated tests would still pass but live filtering
  wouldn't work). **When a batch changes the backend, the live GATE-B server must run the NEW binary.**
- **The one live console exception is the pre-existing router-nav view-transition `InvalidStateError`**
  (`:0:0`, no app stack — techdebt.md risk #1), fired on route entry, NOT from the screen. In-page
  rail/facet interactions were **console-clean** (`useViewTransition` Batch-2 hardening holds).
- **E2E facet flake was a refetch race, fixed two ways:** (1) the page object waits for the list to
  settle (a row OR the empty state) after a facet click before reading counts; (2) `keepPreviousData`
  on `useListModels` keeps the prior page visible during the refetch (also a real UX win — no
  empty-flash on facet change). *Server-side-filtered lists need a settle-wait in E2E + keepPreviousData.*
- **E2E nav: prefer direct `navigate('/ui/models/')` over `navViaShell`** for the list page object —
  the shell-nav dropdown trigger can be intercepted by the main scroll-area viewport (30s click
  timeout). Mirrors what the V1 `ModelsListPage` already did.
- **Dedup rows by id** (Batch-2 carry-forward) — guards against the backend returning an alias twice.

## Reusable patterns to carry to later batches
- **Full-stack screen-v2 recipe**: extend the backend list endpoint (response field + filter params) →
  regen OpenAPI/ts-client → thread an optional filter object through the list hook (CSV serializer +
  sorted cache-key + `keepPreviousData`) → publish a faceted `sidebar` slot → server-side filter.
- **Faceted sidebar** as a published `useShellChrome({ sidebar })` node (memoized on filter state):
  multi-select pill groups (`m-facet-pill`) + a two-thumb native `<input type=range>` dual-slider with
  ARIA labels. Reuse `ModelSidebarFacets` shape for MCP/other faceted lists.
- **Type-discriminated read-only rail with N variants** keyed off `source` guards
  (`isApiAlias`/`isModelRouterAlias`/`isUserAlias`), composing `dp-*` primitives + an Edit CTA.

## Gate results (after the design-refinement follow-up)
- **Backend:** `cargo test -p services -p routes_app --lib` — **817 passed / 0 failed** (incl. 7 new
  list tests: type / api_format incl. Liberty+Anthropic-bucket / size / capability / size-on-rows /
  **search**).
- **Frontend RTL:** full suite **986 passed / 5 skipped / 0 failed** (incl. `routes/models/index.v2.test.tsx`
  — 14 tests: 4 row types, faceted sidebar, facets→query-params, no-top-bar-tabs, search-on-Enter,
  search-clear-resets, 3 rail variants, Edit CTA, empty, V1 fallback). Typecheck clean; touched files
  lint-clean.
- **E2E:** `specs/models/all-models-v2.spec.mjs` (V2 list + faceted sidebar + server-side TYPE filter +
  server-side **search** + detail rail + Edit CTA) — **2/2 pass (standalone)**. Full standalone matrix
  was run; the only failures were pre-existing/environmental (chat-resize techdebt, MCP-OAuth,
  `api-live-upstream` live-provider timeouts that passed in the prior baseline, browser-extension) —
  none touch Models (verified: no E2E references the renamed `all-models` testid).
- **GATE B (live):** My Models V2 on `make app.run.live` (**rebuilt binary** — required for the backend
  search/filter params): live server-side filtering (TYPE=API → API rows; API-FORMAT=Anthropic → the
  anthropic_oauth row), live `?search=llama` → 1 row (highlighted), search-clear restores; colorful
  treatment (per-type icon tiles, provider badge, active-row left-accent); API detail rail; **light +
  dark + responsive (414px) + mobile rail drawer**; **console-clean** on all in-page interactions (only
  the known router-nav VT exception on route entry).

## Follow-up refinements (2026-06-19, post-3-1, design update)
After the initial 3-1 commit the design was refined; four changes landed on top:
1. **Nav rename** `all-models` → **`my-models`** (label "My Models", testid `shell-sub-my-models`) —
   `shell-nav-config.tsx` + `AppShell.test.tsx`/`resolveShellRoute.test.ts`/`screen-coverage.md`.
2. **Top-bar TYPE quick-tabs removed** — TYPE lives only in the sidebar facet now; the toolbar is just
   the **always-visible** search (was the collapsible `useCollapsibleSearch`).
3. **Search moved server-side** — new `search` field on `AliasFilterParams` (case-insensitive
   substring over alias/repo/filename for local rows, id/name/base_url for API, alias for routers),
   applied before size/capability (pure, no I/O). Frontend submits on **Enter** (clearing live-resets);
   `ModelsFilter.search` threads through `useListModels`. Regen flows the param to ts-client.
4. **More colorful, per the design** — color-coded **icon tiles** per type
   (`m-icon-local/alias/api/fallback` = saffron/lotus/indigo/teal), API rows show the **provider** as a
   green `m-provider-badge` (api_format uppercased) + connection status with icon, fallback badge
   leaf→**teal**, and an **active-row left-accent** (`inset 3px var(--c-lotus-text)`).
- Validated: backend +1 search test (22 list tests), RTL +3 (search-on-Enter, clear-resets, no-top-bar
  tabs; 14 V2 tests; full suite 986 pass), V2 E2E +1 search test (2/2 pass), GATE B live (search
  filters server-side, colorful treatment, light+dark, console-clean).

## Follow-ups
1. **Run the FULL E2E matrix** (both projects) at commit time — the shared `useListModels`
   `keepPreviousData` + the backend list changes are app-wide.
2. The **router-level navigation `InvalidStateError`** remains the deferred cross-cutting item (techdebt.md).
3. **Sub-phases re-sequenced + kickoffs written** (2026-06-19, simplest-first): `batch-3-2-new-api-model`
   → `3-3-new-fallback-model` → `3-4-new-local-model` → `3-5-files-pull-consolidation` →
   `3-6-local-discovery` → `3-7-api-discovery`. **Next: 3-2 (New API Model form).**
4. Carry the temporary-scaffolding removal (`ShellSlotsContext` sidebar slot, `useUiV2Flag`) to the
   end-of-migration cleanup (techdebt.md).
