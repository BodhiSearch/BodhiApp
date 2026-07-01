# UI V2 Migration — Tracker

Per-screen status for review. **Scope:** the 13 shell app screens + the bare OAuth Access-Request
review = **14 screens** (see @screen-coverage.md). Verified against the code, not memory
(`grep useUiV2Flag`, `useShellChrome`, `BARE_PREFIXES`, `SHELL_NAV` on the date below).

> Status key: ✅ done (V2-only, flag retired, old code deleted) · 🟩 done-behind-flag (shipped V2 but
> flag kept default-off; V1 still present — see note) · 🚧 in progress · ⬜ not started ·
> ▫️ bare (renders outside AppShell).
> Last verified: **2026-06-28** (code-verified: all routes + shell config + techdebt.md).

## Batches

| Batch | Section | Status |
|---|---|---|
| 0 | Foundation (AppShell, flags, tokens merge, id_token, reference_api_url) | ✅ done |
| 1 | **API Keys → Access Tokens** (4 screens) | ✅ done |
| 1.1 | **Batch-1 follow-up** — IA correction (2026-06-18): nav section "API Keys"→"Access Tokens" (sub-pages "API Tokens"/"New API Token"); new top-level **Users** section (User Access Requests + Manage Users, moved out of Settings); Access Requests redesigned to match `design/User Access Requests.html` (avatar rows, status chips, role/approve-reject flow, detail rail). | ✅ done |
| 2 | **Settings** (App Settings) + **Manage Users** (nav under Users per 1.1) | ✅ done |
| 3 | Models — **split into sub-phases 3-1…3-7** (2026-06-19, see below) | 🚧 in progress |
| 3-1 | **My Models** list + faceted sidebar + detail rail — V2 flag **retired (2026-06-28)**, V1 `ModelsPageContent` **deleted**, `models` flag removed, `index.v2.test.tsx` renamed to `index.test.tsx`; `routes/models/index.tsx` renders `ModelsScreenV2` directly. Feature-parity gaps noted in `techdebt.md` (no delete, no chat-from-list, no metadata refresh). | ✅ done (no flag) |
| 3-2 | **New API Model** form (`api/new`+`edit`) — V2-only, **no flag** | ✅ done |
| 3-3 | **New Model Router** form (`router/new`+`edit`) — V2-only, **no flag** | ✅ done |
| 3-4 | **New Local Model** form (`alias/new`+`edit`) — richest; consumes the 3-6 `quants[]` contract. Evolved in-place over 5 phases (create-then-download backend + `BODHI_TEST_MODE`; quant selector w/ local status; context-flag catalog; request-params editor + `system_prompt`; V2 chrome). V2-only, `new-local-model` flag retired. | ✅ done (no flag) |
| 3-5 | files / pull consolidation (fold download + quant table into the local-model form) — **largely absorbed by 3-4** (create-then-download + quant table landed there); remaining `/models/files*` retirement decision | ⬜ not started |
| 3-6 | **Local Models** discovery — Explore·Local Models view wired to the live reference API (search, faceted sidebar, detail rail + quants, Pull, MultiTenant-hidden). Reference API shipped on Cloudflare; quants carry real `filename`, no split files. | ✅ done (no flag) |
| 3-7 | **API Models** discovery + **API Providers** — `routes/models/explore/api/` (Explore·API Models) + `routes/models/explore/providers/` (Explore·API Providers) both **shipped** (no flag, MultiTenant-hidden). Known gaps tracked in `techdebt-api-models.md` (server-side FTS search, relevance sort, numbered pagination, rich rail, provider-first create). | ✅ done (no flag; gaps in techdebt-api-models.md) |
| 4 | MCP | ✅ done |
| 4-1 | **Explore · MCP Servers** — new catalog view wired to the reference API (`/api/v1/mcp-servers`); near-clone of Explore·API Models (shared CatalogTable/keyboard-nav/Vimium/rail/pagination/Reset/ColumnPicker). Data-driven facets (Category hides when empty, Auth single `http` chip), client-side Verified + Installed facets, instance-join STATUS column (catalog `endpoint_url` ↔ user `mcp_server.url`), single-scroll rail (Description/Connection; provenance Metadata **dropped** — backend deletes it). V2-only, **no flag**. | ✅ done (no flag) |
| 4-2 | **MCP Screens V2** (forms + My MCPs + Explore rail) — 3 phases, V2-only, **no flag**. P1: 5 forms (New/Edit Instance +`?server=&auth=` prefill · New Server +`?url=&name=` · View/Edit Server · Playground) onto the shell via `useShellChrome`. P2: `/ui/mcps/` rewritten as a **server-centric list+detail-rail**. P3: Explore rail gains the same connect/configure/instances via the catalog→registered-server join. | ✅ done (no flag) |
| 4-3 | **MCP design-parity final pass** — match the hi-fi prototypes pixel-for-pixel using shell tokens. Rails: removed Status section; renamed Connection→**Server** spec; **Connect-with** rows gained icon tiles + **Public FIRST** ordering; unregistered-admin path → a **"Connect Server" footer**. **New Server** reads `?auth=`. **Server view rebuilt** as inline-edit hub; **`/servers/edit` redirects to `/view`**. New shared `auth-badges`. V2-only, no flag. | ✅ done (no flag) |
| 4-4 | **MCP Playground advanced** — legacy 2-pane tool-only playground replaced with **3-pane layout** covering **5 MCP capabilities** (Overview, Tools, Prompts, Resources, Templates). `InstancePicker` + `CapabilityNav` sidebar; `PlaygroundRail` with per-capability listing and `useListKeyNav`; readable-by-default `ResultPanel` (Readable/Raw/Request tabs + copy + status pill); `TemplateDetail` with RFC 6570 level-1 fill-in. `useMcpClient` extended with `listPrompts`/`listResources`/`listResourceTemplates` (guarded). Old `ToolSidebar.tsx`, `ExecutionArea.tsx`, `FormInput.tsx`, `ResultSection.tsx` deleted. V2-only, no flag. **Elicitation/Sampling/Completion deferred** (open-follow-up). | ✅ done (no flag) |
| 5 | Chat (1 screen, highest risk, last) — migrated **in-place** over 7 phases (P0 ShellSlots seam · P1 structural swap onto AppShell · P2 message+composer restyle · P3 history sidebar · P4 rail tabs Parameters+MCP · P5 MCP picker composer→rail accordion · P6 ChatTitle breadcrumb + flag cleanup). V2-only, **no flag** — chat was the last flag, so `uiV2Flags.ts`/`useUiV2Flag.ts` deleted. | ✅ done (no flag) |

> **Batch 3 split + re-sequenced (2026-06-19):** the My Models *list* lands first on the real backend
> with a server-side faceted sidebar + search (3-1). The forms follow **simplest-first**: API (3-2) →
> Fallback (3-3) → Local (3-4); then files/pull consolidation (3-5); then the reference-API discovery
> views, split Local (3-6) and API (3-7). Kick-offs: `batch-3-2…3-7-*-kickoff.md`. Rationale +
> decisions: `batch-3-1-models-retro.md`.
>
> **3-6 pulled ahead of 3-4 (2026-06-20):** Local discovery defines the reference-API "which quants
> exist for a repo + sizes" contract that 3-4's quant-file picker needs, so it goes first. The
> reference API is a **new Cloudflare repo** (Workers + D1/FTS5 + KV + Workflows/Crons);
> `…/BodhiSearch/spike-api-getbodhi-app` is a **spike** to mine, not the target.
>
> **3-1 flag retired (2026-06-28):** The `models` flag and V1 `ModelsPageContent` were removed
> post-Batch-3-4/3-6. E2E specs migrated off `ModelsListPage.mjs` (deleted) onto `ModelsListPageV2.mjs`.
> Several V1 affordances (delete, chat-from-list, metadata refresh/preview) were dropped without V2
> replacement; tracked in `techdebt.md` (Batch 3-1 parity gaps section).
>
> **3-7 shipped as two routes** (`explore/api/` + `explore/providers/`). Phase-1 (basic grid + facets +
> load-more) is live; known gaps (server-side FTS search, `sort=relevance` default, numbered pagination,
> rich rail, provider-first create) are documented in `techdebt-api-models.md`.

## Active per-screen flags (`useUiV2Flag`) — what they gate + when they retire

> **Flag system fully deleted.** `lib/uiV2Flags.ts` and `hooks/useUiV2Flag.ts` no longer exist in the
> codebase (removed when Chat shipped V2-only in Batch 5 P6). There are **zero active flags** as of
> 2026-06-28.

| Flag id | Gates | Status |
|---|---|---|
| `models` | My Models V2 list (3-1) | **Retired (2026-06-28)** — V1 list deleted, V2 always-on |

> **`ShellSlotsContext` renamed + retained (not deleted).** Investigated as removal candidate
> (2026-06-28); the persistent single `<AppShell>` structurally requires child→ancestor publish of
> dynamic rail/sidebar nodes (TanStack Router has no named outlets). Renamed to `ShellChromeContext`
> (`ShellChromeContext.tsx`); the "scaffolding" label is dropped — this is permanent architecture.
> Static section/subPage chrome now comes from route `staticData` via `useShellSection()`. The
> section-resolver (`SHELL_NAV` prefix-matching) in `resolveShellRoute.ts` was **deleted**; only the
> bare/fullscreen layout predicates (`BARE_PREFIXES`/`FULLSCREEN_PREFIXES`) remain there as an interim
> central switch (see `techdebt.md` for the optional follow-up of folding those into route `staticData`
> too).

## Screen-by-screen

| # | Section | Screen | Route | Layout | Flag | Status | Notes |
|---|---|---|---|---|---|---|---|
| 1 | Chat | Chat | `/chat/` | shell | — (deleted) | ✅ | **Batch 5** (last). Migrated **in-place** onto AppShell: chat history → `sidebar` slot (search · ⋯-menu · collapsed-rail popover via `useShell`), settings → right `rail` as **two tabs** (Parameters · MCP servers), conversation+composer restyled (real-data meta-strip), editable `ChatTitle` breadcrumb. MCP tool-picker moved composer-popover → rail accordion (`useChatMcp` shares one connection manager). Extended `ShellSlots` with layout overrides (`mainScroll`/`railScroll`/`railWidth`/…). Reused every hook/store unchanged; all `data-test*` preserved. V2-only, **no flag** (the `chat` flag + the whole `uiV2Flags` system were deleted). |
| 2 | Models | **My Models** (was "All Models") | `/models/` | shell | — (retired) | ✅ | **Batch 3-1** + refinement + **flag retired (2026-06-28)**. V2 shell list + published **faceted `sidebar`** (TYPE / CAPABILITY vision·tool-use·reasoning / SIZE dual-slider / API-FORMAT incl. **Liberty**) + **always-visible search** + selectable rows (`LinkRow`) + **read-only detail rail, 4 variants** (Local File / Model Alias / API Model w/ models list / Fallback w/ routing chain), Edit CTA → V1 form routes. **First full-stack batch:** added `size`+capability+**`search`** + **server-side facet filters** to `GET /bodhi/v1/models` + regen. `routes/models/index.tsx` renders `ModelsScreenV2` directly; V1 `ModelsPageContent`/`ModelTableRow`/`ModelPreviewModal`/`ModelActions`/`SourceBadge` **deleted**. |
| 3 | Models | New API Model | `/models/api/new/` (+ edit) | shell | — (none) | ✅ | Batch **3-2**. V2-only, **no flag**: added always-on V2 chrome (breadcrumb + centered `max-w-3xl` container) to both routes; **reused the production `ApiModelForm` unchanged**. `new-api-model` removed from `uiV2Flags.ts`. Setup wizard (`mode="setup"`) untouched. |
| 4 | Models | **New Model Router** (was "New Fallback Alias") | `/models/router/new/` (+ edit) | shell | — (none) | ✅ | Batch **3-3**. V2-only, **no flag** — a **form-body rebuild**. Decomposed `ModelRouterForm` into `-components/` (StepCard · cmdk **AliasCombobox** · RouteToModelField · StepConnector · shared **`RoutingChainPreview`**) + a published **"Routing & help" rail**. Sections IDENTITY · RESILIENCE · TARGETS (arrow ▲/▼ reorder). Renamed screen "Fallback Alias"→"Model Router". |
| 5 | Models | New Local Model | `/models/alias/new/` (+ edit) | shell | — (retired) | ✅ | Batch **3-4**, evolved **in-place** over 5 committed phases: (1) backend create-then-async-download + `BODHI_TEST_MODE`; (2) **QuantSelector** with reference-catalog quant table + per-quant local status; (3) context-params textarea + click-to-add llama-flag catalog; (4) request-params `key=value` textarea + **System Prompt** (new `OAIRequestParams.system_prompt`); (5) V2 chrome. **V2-only, `new-local-model` removed from `uiV2Flags.ts`**. |
| 5a | Models | **Explore · Local Models** | `/models/explore/local/` | shell | — (none) | ✅ | Batch **3-6** (P1–5; no flag). View wired to live reference API (search, faceted sidebar, detail rail + quants, Pull action, MultiTenant-hidden). Cloudflare reference API shipped; quants carry real `filename`. |
| 5b | Models | **Explore · API Models** | `/models/explore/api/` | shell | — (none) | ✅ | Batch **3-7**. Catalog grid + faceted sidebar (Capability/Modality/Status/Provider/Family/Open-weights/Pricing/Context + ColumnPicker) + detail rail + sort + load-more pagination. MultiTenant-hidden. Known gaps: server-side FTS, `sort=relevance` default, numbered pagination, rich multi-section rail, provider-first create — tracked in `techdebt-api-models.md`. |
| 5c | Models | **Explore · API Providers** | `/models/explore/providers/` | shell | — (none) | ✅ | Batch **3-7** (shipped together with Explore·API Models). Providers catalog (name/model-count/api_format/caps/pricing) + faceted sidebar + detail rail (catalog models from this provider). MultiTenant-hidden. Parity gaps same as 5b (Connected badge, numbered pagination, provider-first CTA, logos) — tracked in `techdebt-api-models.md`. |
| 6 | MCP | **My MCPs** (server-centric list+rail) | `/mcps/` | shell | — (none) | ✅ | Batch **4-2 P2**. V2 list of registered servers + detail rail (My Instances·Connect-with·admin Configure, all deep-links); Scope facet Configured/Connected; reuses CatalogTable/keyboard-nav/rail/search. Folds in the deleted V1 flat-instance list + My-MCP-Servers list + `McpManagementTabs`. |
| 6b | MCP | **Explore · MCP Servers** | `/mcps/explore/` | shell | — (none) | ✅ | Batch 4-1 (catalog) + **4-2 P3** (rail gains connect/configure/instances via the catalog→registered-server join; shared `McpRailSections`). |
| 7 | MCP | New/Edit Instance · New/View/Edit Server | `/mcps/new/`, `/mcps/servers/{new,view,edit}/` | shell | — (none) | ✅ | Batch **4-2 P1** + **4-3**. Forms onto the shell via `useShellChrome`. New Instance reads `?server=&auth=` prefill; New Server reads `?url=&name=&auth=`. **Server view rebuilt** as inline-edit hub (4-3); `/servers/edit` redirects to `/view`. |
| 8 | MCP | Playground | `/mcps/playground/` | shell | — (none) | ✅ | Batch **4-2 P1** shell chrome + **4-4** full rebuild. **3-pane layout:** sidebar (`InstancePicker` + `CapabilityNav` with per-feature counts) · center (`OverviewView` / `ToolDetail` / `PromptDetail` / `ResourceDetail` / `TemplateDetail`) · rail (`PlaygroundRail` with `useListKeyNav`). Readable-by-default `ResultPanel` (Readable/Raw/Request tabs). `useMcpClient` extended with `listPrompts`/`listResources`/`listResourceTemplates` (guarded). Old `ToolSidebar`/`ExecutionArea`/`FormInput`/`ResultSection` deleted. **Elicitation/Sampling/Completion deferred** (follow-up). |
| 9 | **Access Tokens** | **API Tokens (list)** | `/tokens/` | shell | — (retired) | ✅ | Nav sub-page "API Tokens". Design-faithful: table (Token/Created/Updated/Status cols), themed filter pills (All/Active/Inactive), collapsible search button, selectable rows → detail rail. **Detail-rail open/close has a view transition.** |
| 10 | **Access Tokens** | **New API Token** | `/tokens/new/` (NEW) | shell | — (retired) | ✅ | Nav sub-page "New API Token". Was a dialog → full page. Real `useCreateToken`; reuses `TokenDialog` reveal; Done → `/tokens/`. |
| 11 | **Users** | **User Access Requests (list)** | `/users/access-requests/` | shell | — (retired) | ✅ | Moved out of API-Keys into the new **Users** section (1.1). Avatar rows · status chips · pending-count header pill · filter tabs + collapsible search · pending rows show role picker + approve/reject; selectable rows → detail rail. |
| 12 | **Access Tokens** | **Access Request review** | `/apps/access-requests/review/?id=` | ▫️ bare | — (retired) | ✅ | OAuth consent (3rd-party app, deep-linked). Renders via `BareLayout` (slim topbar). |
| 13 | Settings | App Settings | `/settings/` | shell | — (retired) | ✅ | Batch 2. V2 shell list: **published `sidebar`** = settings-group scroll-spy nav + Legend; All/Modified/Env filter pills; collapsible search w/ highlight; rows → detail rail. Read-only rail by default; editor for 2 backend-editable keys only. Rail open = view transition. |
| 14 | **Users** | Manage Users | `/users/` | shell | — (retired) | ✅ | Batch 2. V2 shell list: 5 role filter pills + collapsible search; rows → **role-editor rail** (CHANGE ROLE + Save + two-click Remove). Real pagination. **Invite link** as header-action popover (multi-tenant only). |

**Retired/consolidated (no separate screen):** `/users/pending/` → folded into Access Requests
(Batch 1 migrated the target; **`/users/pending/` route + `UserManagementTabs` + the V1 `components/users/`
table/dialogs DELETED in Batch 2**). `/models/files/`, `/models/files/pull/`, `/mcps/servers/*` →
absorbed (see @screen-coverage.md §B).

**Setup wizard** — ✅ migrated (detour batch, 2026-06-19). All 6 steps restyled to the design
(`setup-1..6`): fullscreen standalone chrome (own lotus + stepper + theme toggle, NO BareLayout
topbar — added `isFullscreenRoute('/setup')` to `resolveShellRoute` + a fullscreen branch in
`__root`), idiomatic Tailwind+tokens (no `.su-*` CSS dump; one tiny `setup-wizard.css`). **V2-only, no flag.** Reused every hook + redirect + the production `ApiModelForm mode="setup"` unchanged (step 4).

**Out of scope (deferred):** access-request standalone (`/request-access/`, pending),
Keycloak/auth (@screen-coverage.md §C).

## Cross-cutting (done)
- **Reusable patterns** (Batch 1): `BareLayout` (standalone chrome) + `ShellChromeContext`/`useShellChrome`
  (screens publish breadcrumb/headerActions/rail to the root `<AppShell>`). Renamed from `ShellSlotsContext`
  (2026-06-28) — kept as permanent architecture (not temporary scaffolding).
- **`sidebar` chrome slot** (Batch 2): `ShellSlots` gained a `sidebar?: ReactNode` slot for page-body sidebars.
- **`useViewTransition` hardening** (Batch 2): swallows the **async** rejection of `ready` promise too —
  an interrupted transition rejects `ready` with `InvalidStateError`, which was surfacing as an uncaught
  console exception app-wide.
- **Mobile rail-drawer fix** (Batch 2 follow-up): at `<768px` the rail is a `position:fixed` drawer.
  `useViewTransition` skips the transition on mobile; `AppShell`'s auto-open effect opens the mobile drawer
  **whenever rail content is present**. Guarded by RTL + a live mobile E2E (`specs/users/mobile-rail.spec.mjs`).
- **View Transitions** (React-18 native): router `defaultViewTransition` + `useViewTransition()` for detail-rail open/close.
- **Theme switch in the sidebar footer** (above the user chip, always visible).
- **Reusable list-toolbar components** (`components/shell/`): `ShellFilterTabs`, `useCollapsibleSearch`, `ShellSearch`, `ShellPagination`, `LinkRow`, `useListKeyNav`, `useShellSection`.
- **`resolveShellRoute.ts` simplified (2026-06-28):** section-resolver (longest-prefix `SHELL_NAV` matching) **deleted** — section/subPage now come from route `staticData` via `useShellSection()`. Only the bare/fullscreen layout predicates (`BARE_PREFIXES`/`FULLSCREEN_PREFIXES`) remain as an interim central switch.

## Open follow-ups (carry forward)
- **MCP Playground: Elicitation / Sampling / Completion** — deferred from Batch 4-4; wire once upstream SDK lands stable surface APIs. Tracked in `batch-4-4-mcp-playground-retro.md`.
- **Explore · API Models / API Providers quality pass** — 6 frontend gaps + 10 reference-api gaps documented in `techdebt-api-models.md`. Top priorities: server-side FTS search, `sort=relevance` default, numbered pagination, rich rail, provider-first create flow. All gated on `@bodhiapp/reference-api-types` minor bump.
- **Scalable route-declared layout seam (optional)** — `BARE_PREFIXES`/`FULLSCREEN_PREFIXES` in `resolveShellRoute.ts` could eventually be replaced by `staticData.layout` / pathless `_bare` routes. Low urgency; `BareLayout` is a drop-in. Tracked in `techdebt.md`.
- **Models list: delete + chat-from-list + metadata refresh** — V1 affordances dropped without V2 replacement when the `models` flag retired. Tracked in `techdebt.md` (Batch 3-1 parity gaps).
- Run the **full E2E matrix** after any shell or shared-CSS change — the shared `useListModels`/shell/nav changes are app-wide.

> Update this tracker at the end of every batch: flip the section's screens to ✅, retire their flag
> ids, and note any new consolidations/deletions.
