# Plan — Batch 3-1: Models · All Models (list + faceted sidebar + detail rail)

> Working dir: `docs/claude-plans/202606/screen-v2/`. This plan covers **Phase 3-1 only** — the
> All Models list screen. Batch 3 was re-scoped (2026-06-19) into sub-phases; the other Models
> screens move out (see "Scope & sub-phase split"). On approval, this plan is the basis for
> `batch-3-1-models-plan.md` in the screen-v2 folder; its retro generates the 3-2…3-5 kick-offs.

## Context

We're migrating BodhiApp's 14 shell screens to the V2 left-sidebar **AppShell** design, one nav
section per batch (Batches 0–2 done: Foundation, Access Tokens, Settings/Users). Batch 3 is the
**Models** section. GATE A (interactive walk of all 4 Models prototypes on http://localhost:8000)
plus a full code map (routes/hooks/ts-client types/tests/shell infra/backend list endpoint) revealed
the Models screens are **richer and more data-divergent** than Batches 1/2 — the forms already exist
in production with most real fields, and the All Models prototype has a heavy faceted sidebar.

Per the user (2026-06-19), Batch 3 is split so the All Models **list** lands first, on the real
backend, with the faceted sidebar made **real server-side**. The form screens and the reference-API
discovery views are deferred to later sub-phases. Unlike Batches 1/2 (presentation-only), **3-1 is
full-stack**: it adds `size` + capability data and server-side filter params to `GET /bodhi/v1/models`,
regenerates OpenAPI/ts-client, then builds the V2 screen.

## Scope & sub-phase split (decided with user)

| Sub-phase | Scope | This plan? |
|---|---|---|
| **3-1** | **All Models list** + faceted sidebar (TYPE / API-FORMAT / SIZE / CAPABILITY) + detail rail (4 variants) + Edit CTA → existing form routes. **Backend**: size + capability on list rows + server-side filters. | ✅ **THIS** |
| 3-2 | New **Local** Model form (`/models/alias/new|edit/`) — "V2 styling, keep real fields" | ⬜ kick-off in 3-1 retro |
| 3-3 | Create-local-model / `files/` / `files/pull/` consolidation | ⬜ kick-off in 3-1 retro |
| 3-4 | New **Fallback** alias + New **API** Model forms (`/models/router/*`, `/models/api/*`) | ⬜ kick-off in 3-1 retro |
| 3-5 | **Local Models** + **API Models** discovery sub-views (reference-API; `api.getbodhi.app`) | ⬜ kick-off in 3-1 retro |

3-1 migrates **only** the `/models/` route behind the existing `models` flag. The 3 form routes keep
rendering V1 (their flags stay default-old); the rail's **Edit** CTA navigates to them as today.
`files/` + `files/pull/` are **untouched** (kept as-is, per user — consolidation is 3-3).

## What already exists (reuse — don't rebuild)

**Shell infra (Batches 0–2)** — `crates/bodhi/src/components/shell/`:
- `useShellChrome({ breadcrumb, headerActions, sidebar, rail, railHeader, railDefaultOpen })` publishes
  chrome to the root `<AppShell>` (`ShellSlotsContext.tsx`). **Pass stable memoized nodes.** The
  `sidebar` slot (Batch 2, App Settings scroll-spy) is the model for our faceted sidebar.
- `ShellFilterTabs` (themed pills + counts), `useCollapsibleSearch` (search button → row), `LinkRow`
  (selectable-rows-are-links anchor), `useViewTransition()` (rail/selection; mobile + reduced-motion
  hardened). `list.css` (`l-*`) primitives; `api-keys.css`/`settings.css` scoped page CSS.
- `SHELL_NAV` already declares the `models` section + 4 subPages with hrefs + testids
  (`shell-nav-models`, `shell-sub-all-models`, …) — `shell-nav-config.tsx`. **No nav edit needed.**
- `useUiV2Flag` flag `models` already exists (`lib/uiV2Flags.ts`).

**Data layer** — `crates/bodhi/src/hooks/models/`:
- `useListModels(page, pageSize, sort, sortOrder)` → `PaginatedAliasResponse` (`useModels.ts`). We
  **extend its signature** to thread filter params (see Frontend §3).
- Type guards `isApiAlias / isUserAlias / isModelAlias / isModelRouterAlias / hasLocalFileProperties`
  (`lib/utils.ts`) — drive the TYPE facet + rail variant selection.
- ts-client `AliasResponse` union (`UserAliasResponse | ModelAliasResponse | ApiAliasResponse |
  ModelRouterResponse`), `ApiFormat`, `ModelMetadata { capabilities{vision,audio,thinking,tools}, … }`.

**E2E** — `crates/lib_bodhiserver/tests-js/`:
- `pages/ModelsListPage.mjs` + `BasePage.navViaShell(section, subPage)` (clicks `shell-nav-models` →
  `shell-sub-all-models`). Specs `specs/models/*.spec.mjs`.

## Real-data decisions (from GATE A + user answers)

| Facet / element | Decision | Backing data |
|---|---|---|
| **TYPE** (Local File / Model Alias / API Model / Fallback) | Ship. Maps from `source`: `model`→Local File, `user`→Model Alias, `api`→API Model, `model_router`→Fallback. | exists (`source`) |
| **API FORMAT** (OpenAI / Responses / Anthropic / Gemini / **Liberty**) | Ship, **API-only**. Buckets: OpenAI←`openai`, Responses←`openai_responses`, Anthropic←`anthropic`+`anthropic_oauth`, Gemini←`gemini`, **Liberty←`llm_liberty_oauth`** (added per user). | exists (`api_format`) |
| **SIZE** (dual slider, local files) | Ship. Filters **local** rows by file byte-size. Non-sized rows (API/router) are **not hidden** by the slider — to hide them, the user picks TYPE=Local File (user's note). | **needs backend** `size` on list rows |
| **CAPABILITY** (vision / tool-use / reasoning) | Ship **subset** (3 of the prototype's 5). Map: vision→`capabilities.vision`, tool-use→`capabilities.tools.function_calling`, reasoning→`capabilities.thinking`. **Drop `chat` + `embeddings`** (no backing field). Local-only (API/router have no metadata → excluded). | exists (`metadata`); **needs backend filter** |
| Model-Type sub-views (My / Local / API) | **Only "My Models"** in 3-1 (the real backend list). **Local Models + API Models = reference-API discovery → 3-5.** | My=real; others=3-5 |
| Detail rail | Ship all 4 variants (read-only) + **Edit CTA** → existing (V1-flagged) form routes. | exists |
| Search | Full search bar (alias/repo/filename/base-url), client-side over fetched rows (no server search param this phase). | client-side |
| Filtering mechanism | **Server-side** filter params on `GET /bodhi/v1/models` (global filtering + accurate counts), per user. | **needs backend** |

**Dropped as prototype-only (no backing data):** capability `chat`/`embeddings` pills; the import/
download header icon (that's 3-3); any per-row "last used"/verified badges.

---

## Backend changes (Layer 1 — do first, upstream→downstream)

All in `crates/services` + `crates/routes_app`. Backend map confirmed:
`models_index` (`routes_app/src/models/routes_models.rs:69`) calls `data().list_aliases()` →
`Vec<Alias>` → in-memory sort → paginate → batch-fetch metadata for the page. Filtering today is
nonexistent; size lives only on `LocalModelResponse`/`HubFile.size`, not on the list responses.

### B1. Add `size` to local list rows
- Add `size: Option<u64>` to **`UserAliasResponse`** (`services/src/models/model_objs.rs:1951`) and
  **`ModelAliasResponse`** (`:2013`). `#[serde(skip_serializing_if = "Option::is_none")]`.
- Thread real byte-size in `models_index`: when building the User/Model response rows, resolve the
  file size. `HubService::find_local_file()` (`hub_service.rs:258`) already computes size via
  `fs::metadata`. Prefer a **batch** resolution to avoid N stats — extend `list_model_aliases()` /
  the user-alias path to carry `HubFile.size`, or stat once per (repo,filename,snapshot) for the page
  only (mirror the existing page-scoped metadata batch at `routes_models.rs:93`). Multi-tenant returns
  API-only (no local rows) → `size` naturally absent there.

### B2. Server-side filter query params
- Add a filter struct (e.g. `AliasFilterParams`) alongside `PaginationSortParams`
  (`routes_app/src/shared/pagination.rs:7`), accepted by `models_index`:
  - `type`: CSV/multi of `local_file | model_alias | api_model | fallback` → match on `Alias` variant.
  - `api_format`: CSV/multi of `openai | responses | anthropic | gemini | liberty` → bucket the
    `ApiFormat` enum (Anthropic bucket = `anthropic`+`anthropic_oauth`; Liberty = `llm_liberty_oauth`).
    Applies to API rows only.
  - `size_min` / `size_max` (bytes or GB — pick one, document it; UI slider is GB). Applies to local
    rows with a known size; rows **without** a size pass through (don't hide API/router — §SIZE).
  - `capability`: CSV/multi of `vision | tool_use | reasoning` → require the mapped
    `metadata.capabilities` field true. Applies to local rows with metadata; API/router excluded when
    a capability filter is active.
- **Apply filters server-side** in `models_index` between `list_aliases()` and sort/paginate, so
  `total` + pagination reflect the filtered set (accurate global counts). Capability filtering needs
  metadata for the **whole** list, not just the page — extend the batch-metadata fetch to cover all
  candidate local rows **when a capability filter is active** (only then, to keep the common path
  cheap). Document the cost in the plan's risks.
- Decide counts surfacing: the sidebar facet counts can be **derived client-side from the current
  filtered page** (Batch-1/2 precedent) OR returned by the backend. **Recommend**: client-side counts
  from the returned rows for v1 (simplest, consistent with shipped batches); if global per-facet
  counts are wanted, add a small aggregate to the response in a follow-up. Confirm during impl.

### B3. OpenAPI + ts-client regen
- Update utoipa annotations on `models_index` (new query params) + register the new filter struct
  (`routes_app/src/shared/openapi.rs:347`). Run `cargo run --package xtask openapi` then
  `make build.ts-client`. New `size`/filter fields flow to `@bodhiapp/ts-client`.

### B4. Backend tests
- Extend `routes_app/src/models/alias/test_aliases_index.rs` (5 existing tests): add cases for each
  filter param (type, api_format incl. Liberty + Anthropic-bucket, size range, capability), filter
  combinations, `size` present on local rows / absent on API rows, and that `total`/pagination reflect
  the filtered set. Service-layer tests if filtering logic lands in `services`. Follow
  `crates/CLAUDE.md` rstest conventions; assert via codes not message text.
- `make test.backend` (Docker up for pg variant) green before moving to the frontend.

---

## Frontend changes (Layer 2 — after regen)

New screen at the **same route** `/models/`, gated by `useUiV2Flag('models')`. Branch **inside** the
page component; keep the `AppInitializer` wrapper outside (recipe).

### F1. Page shell + chrome
- New `routes/models/-components/ModelsScreenV2.tsx` (+ co-located pieces). The route `index.tsx`
  renders `isUiV2Enabled('models') ? <ModelsScreenV2/> : <ModelsPageContent/>` inside the existing
  `AppInitializer`. Preserve `data-testid="models-content"` (and add a `data-pagestatus`), the
  `table-list-models` testid, and the row-cell testids the page objects use.
- Publish chrome via `useShellChrome`: `breadcrumb` (Bodhi › Models › My Models), `sidebar` (the
  faceted nav — memoized), `rail`/`railHeader`/`railDefaultOpen` (detail rail on selection). No
  header "New" button (nav sub-pages cover it, like Batch 1).

### F2. Faceted sidebar (published `sidebar` slot)
- Memoized node modeled on App Settings' Batch-2 sidebar. Sections: **MODEL TYPE** (My Models active;
  Local/API Models rendered **disabled/"coming soon"** or omitted — they're 3-5; **omit** to honor
  real-data-only), **TYPE** pills, **CAPABILITY** pills (vision/tool-use/reasoning), **SIZE** dual
  slider (0 → 16+ GB), **API FORMAT** pills (incl. Liberty). Selecting facets updates local filter
  state → re-queries via the extended `useListModels`. Collapsed shell → icon-rail (free from
  AppShell). Use a shadcn slider (or a small dual-range control) for SIZE; reuse pill styling from
  `list.css`/`ShellFilterTabs` where it fits (or page-scoped CSS for the facet groups).
- **CSS**: page-scoped under a `.models-screen` root (Batch-1/2 discipline — scope generic class
  names, rename collisions, drop global `:root`/`mark`). New `routes/models/-components/models.css`.

### F3. Hooks: thread filters
- Extend `useListModels` (and `modelKeys.list(...)` query key) to accept the filter object so changing
  facets refetches with the new query params and caches per filter combo (`useModels.ts`,
  `constants.ts`). Keep pagination/sort.

### F4. List rows + detail rail
- Rows render with `LinkRow` first child (selectable-rows-are-links) + type/format badges, connection
  status for API rows ("N models exposed", connected/no-key), and the fallback chain summary for
  router rows — all from real fields (`getModelDisplayRepo`/`Filename` already encode this). Selection
  is local state (no URL); open the rail via `useViewTransition`.
- **Rail variants** (read-only) keyed off the type guard:
  - **Local File / Model Alias**: FILE (repo / filename / snapshot) + the "auto-discovered / read-only"
    or "user-created alias" note + (optional) capabilities/size from `metadata`/`size`.
  - **API Model**: CONNECTION (base URL, provider badge, models N exposed) + MODELS list.
  - **Fallback**: ROUTING CHAIN (numbered steps, ON-ERROR arrows, disabled badge) + BEHAVIOR +
    **Edit fallback alias** CTA → `/models/router/edit/?id=`.
  - Each rail's **Edit** CTA → the matching existing form route (`alias/edit`, `api/edit`,
    `router/edit`) — those stay V1 this phase.
- Reuse existing `ModelPreviewModal` only if needed; the rail replaces the modal for the V2 path.

### F5. Frontend tests (RTL)
- New `routes/models/index.v2.test.tsx` using the **`ShellSlotsProvider` + slots-consumer harness**
  (Batch-1/2 pattern in `tokens/index.v2.test.tsx`) to assert published `sidebar`/`rail`/`breadcrumb`.
  Cover: facet filtering (TYPE/API-FORMAT/CAPABILITY/SIZE → correct query params via MSW assert), row
  → rail (each of the 4 variants), Edit CTA navigation, empty/error states, light path. Extend
  `test-utils/msw-v2/handlers/models.ts` `mockModels` to carry `size`/`metadata` and to assert/accept
  the new filter query params. Keep all reused `data-testid`s.

### F6. E2E
- Update `pages/ModelsListPage.mjs`: nav via `navViaShell('models', 'all-models')`; add helpers for
  the facet sidebar (select TYPE/API-FORMAT/CAPABILITY pill, drag/ set SIZE slider) and rail assertions
  (the 4 variants + Edit CTA). Set `reducedMotion: 'reduce'` in the page object (Batch-2 VT-detach fix)
  and wait for mutation/transition settle before asserting. Black-box only (no `page.evaluate`).
- Spec(s) under `specs/models/`: list renders, each facet filters the list (server-side), row → rail
  per type, Edit CTA lands on the right (V1) form route. Set the `models` flag via `addInitScript`
  until retired. **Run the full E2E matrix once** (shared shell/CSS app-wide). Pre-existing
  chat-resize + backend live-test failures are not ours (@techdebt.md).

---

## Critical files

**Backend**
- `crates/routes_app/src/models/routes_models.rs` (`models_index`, ~:69) — filters + size threading.
- `crates/routes_app/src/shared/pagination.rs` (~:7) — new `AliasFilterParams`.
- `crates/services/src/models/model_objs.rs` (`UserAliasResponse` ~:1951, `ModelAliasResponse` ~:2013)
  — add `size`. `data_service.rs` (`list_aliases` ~:91) / `hub_service.rs` (`find_local_file` ~:258,
  `list_model_aliases` ~:352) — size source.
- `crates/routes_app/src/shared/openapi.rs` (~:347) — register params/struct.
- `crates/routes_app/src/models/alias/test_aliases_index.rs` — extend tests.

**Frontend**
- `crates/bodhi/src/routes/models/index.tsx` (flag branch) + new `-components/ModelsScreenV2.tsx`,
  rail components, `models.css`.
- `crates/bodhi/src/hooks/models/useModels.ts` + `constants.ts` (filter params + query key).
- `crates/bodhi/src/test-utils/msw-v2/handlers/models.ts` (size/metadata/filter-aware `mockModels`).
- `crates/bodhi/src/routes/models/index.v2.test.tsx` (new RTL).
- `crates/lib_bodhiserver/tests-js/pages/ModelsListPage.mjs` + `specs/models/*.spec.mjs`.

**Reuse (no change beyond signature):** `components/shell/*` (`useShellChrome`, `ShellFilterTabs`,
`useCollapsibleSearch`, `LinkRow`, `useViewTransition`), `lib/utils.ts` type guards, `lib/uiV2Flags.ts`
(`models` flag exists), `shell-nav-config.tsx` (`models` nav exists).

## Risks / watch-outs

- **Full-stack batch** (unlike 1/2): backend + regen must land + be green before frontend. Work
  upstream→downstream (`services` → `routes_app` → regen → frontend), per root CLAUDE.md.
- **Capability filter needs whole-list metadata** — extend the batch-metadata fetch to all candidate
  local rows **only when a capability filter is active**; keep the default path page-scoped. Log/note
  the cost; don't silently scan everything every request.
- **Multi-tenant returns API-only** → SIZE/CAPABILITY facets are empty there (local rows don't exist);
  ensure the UI degrades cleanly (facets present but filter nothing) and tests cover both deployments.
- **Don't transition `grid-template-columns`** (carry-forward risk #1); `useViewTransition` only for
  in-page rail/selection. Honor reduced-motion + mobile skip (Batch-2 fixes).
- **Scope generic CSS** under `.models-screen`; strip prototype idioms on port (no `lucide.createIcons`/
  `window.*`/`ReactDOM.createRoot`/`data-theme`/`TweaksPanel`).
- **Dual-range slider**: no shadcn dual-thumb out of the box — pick a small accessible control
  (two-thumb) and give it real labels/ARIA (web-design-guidelines).
- **GATE B is blocking**: RTL+E2E necessary but not sufficient (Batch-2 caught dup-key + async VT
  `InvalidStateError` only live). Validate each interaction in **light + dark + responsive**, console
  clean (0 errors) on load + every facet/row interaction.

## Verification (end-to-end)

1. **Backend**: `make test.backend` (Docker up) green; manually hit `GET /bodhi/v1/models?type=…&
   api_format=liberty&size_min=…&capability=vision` and confirm filtered `total` + `size` on local rows.
2. **Regen**: `cargo run --package xtask openapi && make build.ts-client`; `make ci.ts-client-check`.
3. **RTL**: `cd crates/bodhi && npm run test` (new v2 test + existing green); typecheck + lint touched.
4. **E2E**: Docker up; `make test.e2e` from `crates/lib_bodhiserver` — `specs/models/*` + **full
   matrix once** (shared shell/CSS). `reducedMotion: 'reduce'`; re-run on a healthy auth server if
   OAuth-login flake.
5. **GATE B (live)**: `make app.run.live`, log in, open `/models/`. Walk every facet (TYPE / API-FORMAT
   incl. Liberty / CAPABILITY / SIZE slider), each rail variant + Edit CTA, **light + dark +
   responsive**, **console clean** throughout (`read_console_messages`).
6. **Done** → retire the `models` flag + delete the V1 All-Models path (keep the V1 **form** routes —
   3-2/3-4) → commit (trunk-based, rebase on `origin/main`) → write `batch-3-1-models-retro.md` +
   the **3-2 / 3-3 / 3-4 / 3-5 kick-offs** → update `tracker.md` (All Models ✅; note the sub-phase split).
