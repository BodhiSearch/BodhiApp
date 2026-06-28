# React Frontend Quality Pass v2 — Phased Plan

> **STATUS (2026-06-28): Phases 0–7 complete and committed; Phase 8 deferred as a brief.**
> Unit suite 1212 → 1252 (all new tests are real coverage), typecheck clean, lint at/below its
> pre-existing baseline. Affected E2E ran green after each UI-touching phase; full standalone E2E
> run as the final gate. Commits on `main`:
> - `28979a01` + `75cb35d9` — P0 dedups (maskApiKey/SortState/ChatSettings) + camelCase hook renames
> - `aabb10d4` — P1 drop `.v2` test suffix (9 renames + 3 merges, no coverage loss)
> - `0d54cafb` — P2 test backfill (useMcpClient, useMcpClients, RoutingChainPreview)
> - `3d16428c` — P3 `<EmptyState>` extraction (11 sites)
> - `43aafd4e` — P4 `<DetailRail>` primitives + 9 panel migrations
> - `bfff0a9d` — P5 deleted the unused ComboBoxResponsive (the "generalization" was dead code)
> - `eaa60580` + `f2d1dd7e` — P6 ShellChrome split + `useCatalogScreenState` hook (API + Providers)
> - `ec7cf9f3` — P7 E2E legacy→V2 page-object migration + settings filter-tab gap
> - `d5ca6098` — P8 deferred: non-prescriptive brief at `screen-v2/prompt-shell-remove.md`
>
> **Key scope decisions (YAGNI / clear-wins-only):** kept the 4 hand-rolled comboboxes as
> deliberate seams (no god-component); migrated only the 2 structurally-identical Explore screens to
> `useCatalogScreenState` (Local's cursor pagination + MCP's single-sort left as seams); deferred the
> mcps/new + ChatUI + AuthConfigForm large-file splits (organizational, no dedup payoff); confirmed
> the audit's token-toggle/models-column "E2E gaps" were already covered.

**Date:** 2026-06-28
**Scope:** `crates/bodhi/src` (Vite + TanStack Router + TanStack Query v5 + shadcn + Zustand) + E2E migration in `crates/lib_bodhiserver/tests-js`
**Working file:** this plan. Companion to the prior `docs/claude-plans/202606/2306-cleanup/` pass.

## Context

After the V2 screen migration (Batches 1–6: Explore Models/Providers/Local/MCP, providers parity, MCP dropdowns, autocomplete) the frontend has accrued structural duplication that the earlier `2306-cleanup` pass (comments, dead code, errorUtils, StatusButton, file splits, import-alias migration) did not cover. This pass targets **the duplication that V2 left behind**: 9 near-identical rail panels, 4 near-identical Explore catalog screens, parallel combobox implementations, and a handful of trivially-duplicated types/utils. It also adds **unit tests for genuinely complex untested logic** (MCP client state machines, routing-chain rendering), drops the now-meaningless `.v2` test suffix, finishes the E2E suite's move to V2, and ends by removing the temporary `ShellSlotsContext` scaffolding.

**Audit results that shaped the scope (already-resolved concerns):**
- **No UI feature flags remain.** `uiV2Flags`/`useUiV2Flag` were fully deleted (commit d5caac91). Nothing to remove.
- **No page is un-migrated to V2.** Every app screen uses `useShellChrome` + AppShell. `login/setup/auth/request-access/mcps/oauth/apps/access-requests/review` are intentionally bare/fullscreen, declared in `components/shell/resolveShellRoute.ts:20`. Not forgotten migrations.
- **No V1/V2 dual implementations.** Only V2 screen components exist; the `.v2` suffix on test files is now redundant.

**Guardrails (from `2306-cleanup` charter — still in force):**
- Trunk-based: focused commits straight to `main`, linear history, `make format` before each commit.
- Do NOT touch generated code (`routeTree.gen.ts`, `@bodhiapp/ts-client`) or vendored `components/ui/*` unless clearly project-customized.
- Refactor depth = **moderate, clear wins only**. Inline single-use abstractions; keep deliberate seams; conservative around stores.
- **Gate each phase by running unit tests before moving on** (regression safety). Cadence: `cd crates/bodhi && npm test` after each batch; reserve `make test.e2e` (~25 min serial) for phase ends / accumulated.
- Phases ordered **monotonic risk**: trivial dedups → test backfill (safety net for later) → component extraction → highest-risk template/structural changes.

**Not in scope (remain deferred from prior pass):** ChatMessage/AgentMessage shim removal, async-status enum, ShellFooter required-props + popover behavioral dedup, login-mode unification, chatSettings 24-setter collapse, cache-invalidation helpers.

---

## Phase 0 — Trivial single-source-of-truth dedups + hook renames (risk: very low)

Pure consolidation, no behavior change. Guarded by existing tests + typecheck.

**Batch 0a — type/util de-duplication**
- **`maskApiKey` → one source.** Identical bodies at `schemas/apiModel.ts:327` and `hooks/models/useModelsApi.ts:140`. Move to `lib/` (e.g. `lib/maskApiKey.ts`), import in both; keep the `hooks/models/index.ts:43` re-export pointing at the canonical export.
- **`SortState` → one source.** Keep `types/models.ts:3`; delete the duplicate `interface SortState` in `components/DataTable.tsx:9` and import from `@/types/models`.
- **Remove dead `ChatSettings` interface.** Live type is `stores/chatSettingsStore.ts:9` (`= PersistedChatSettings`). First grep consumers of the `Chat.settings?` field (`types/chat.ts:51`); if unused, delete the local `types/chat.ts:56` interface (and the field, or point it at the store type).

**Batch 0b — kebab→camelCase hook renames** (one batch so the build never breaks; `git mv`)
- `use-toast.ts`→`useToast.ts`, `use-mobile.tsx`→`useMobile.tsx`, `use-media-query.ts`→`useMediaQuery.ts`, `use-browser-detection.ts`→`useBrowserDetection.ts`, `use-extension-detection.ts`→`useExtensionDetection.ts`, `use-toast-messages.ts`→`useToastMessages.ts`, `use-responsive-testid.tsx`→`useResponsiveTestid.tsx`.
- **Caution:** if any vendored `components/ui/*` file imports `@/hooks/use-toast`/`use-mobile` by the kebab name, leave a thin re-export shim at the old path rather than editing the vendored file (or confirm it's project-customized first).

**Tests:** none new. **Gate:** `cd crates/bodhi && npm test && npm run test:typecheck`; `make format`.

---

## Phase 1 — `.v2.test.tsx` suffix removal (risk: very low; mechanical)

The `.v2` suffix no longer disambiguates anything (no V1 exists). Normalize names.

- **9 clean renames** (`git mv ...index.v2.test.tsx → index.test.tsx`): `mcps/explore`, `mcps/index`, `models/explore/{api,local,providers}`, `models/index`, `models/router/new`, `settings/index`, `users/index`.
- **3 collisions** — both a legacy `index.test.tsx` (distinct init/RBAC/OAuth-state concern) and `index.v2.test.tsx` exist: `tokens/`, `users/access-requests/`, `apps/access-requests/review/`. Do NOT clobber. For each: **merge** the two files into one `index.test.tsx` (preferred — fold the legacy init/RBAC/OAuth describe-blocks into the V2 file), OR if they're cleanly separable, rename the legacy file to an intent-named sibling (e.g. `index.init.test.tsx`). Verify zero coverage loss by diffing the describe/it set before deleting.

**Tests:** unchanged set, just relocated/merged. **Gate:** `cd crates/bodhi && npm test` (confirms same count of passing tests).

---

## Phase 2 — Test-before-refactor backfill (risk: low; this is the safety net for Phases 4–6)

Characterization tests that **lock current behavior** for complex untested logic that later phases touch. Nothing is refactored here. (Per user: cover MCP clients + RoutingChainPreview; do NOT backfill AutocompleteInput/AliasSelector/toolMapping.)

| Target | New test file | Cover | Why |
|---|---|---|---|
| `hooks/mcps/useMcpClient.ts` (~143 lines) | `hooks/mcps/useMcpClient.test.ts` | Status transitions disconnected→connecting→connected→error; `connect` disconnects first; `callTool` returns `{isError:true}` when not connected; `refreshTools` no-op when disconnected, refreshing→connected on success, stays connected on refresh error; unmount cleanup. Drive end-to-end via `test-utils/msw-v2/handlers/mcp-protocol.ts`. | Connection state machine, zero coverage; precondition for any chat/playground touch. |
| `hooks/mcps/useMcpClients.ts` (~219 lines) | `hooks/mcps/useMcpClients.test.ts` | `connectAll` diffing (only changed endpoints connect/disconnect, stable connections preserved); `allTools` union + memoization; `callTool(mcpId, …)` routing; disconnect of removed endpoints. | Most intricate MCP logic; multi-connection diffing easily regresses. `toolMapping.ts` is exercised transitively here — no separate test needed. |
| `routes/models/-components/RoutingChainPreview.tsx` (~45 lines, used in 4 places) | `routes/models/-components/RoutingChainPreview.test.tsx` | Empty state; alias present vs `(not selected)`; `model` arrow row; `missingModel` → "model required"; disabled step `disabledLabel` (default vs passed). | Conditional-heavy, shared by `ModelDetailRail` + `ModelRouterForm` + `RouterInfoRail`; locks behavior before DetailRail extraction (Phase 4 touches `ModelDetailRail`). |

**Gate:** `cd crates/bodhi && npm test`. All green before Phase 4.

---

## Phase 3 — `<EmptyState>` extraction (risk: low)

~10 screens inline the same `.empty-state/.empty-icon/.empty-title/.empty-sub` block (e.g. `ExploreApiScreen` ~376–383, `LocalDiscoveryScreen`, `ExploreProvidersScreen`, `ExploreMcpScreen`, `ModelsScreenV2`, `MyMcpsScreen`, `SettingsPageV2`, `McpServersPane`, `DownloadsPanel`).

- NEW `components/EmptyState.tsx` — props `icon`, `title`, `sub?`, `testId?`, `action?` slot. **Keep existing CSS classes** (zero visual change). Pass through `testId` so E2E `data-testid="cat-*-empty"` selectors survive.
- Replace inline blocks **only where structurally identical**; leave bespoke empty states (custom actions/links) inline as deliberate seams.
- This component is reused by the Phase 6 catalog work.

**Tests:** `components/EmptyState.test.tsx` (renders title/sub/testid, omits sub when absent). **Gate:** `npm test`; `make format`.

---

## Phase 4 — `<DetailRail>` primitives + 9-panel migration (risk: medium; guarded by Phase 2 + screen tests)

9 rail panels share the `.dp-panel/.dp-status-row/.dp-body/.dp-section/.dp-foot/.dp-head/.dp-row` structure and each re-define a local `Row` helper.

**Approach — composition primitives, not a config monolith.** NEW `components/detail-rail/DetailRail.tsx` exporting slot components: `<DetailRail>` (the `dp-panel` wrapper, `className`/`testId` passthrough), `<DetailRailStatusRow>`, `<DetailRailBody>`, `<DetailRailSection label?>`, `<DetailRailRow k v>` (the duplicated `Row` — renders null on empty `v`, matching current behavior), `<DetailRailFooter>`. Keeps existing CSS classes → no visual change. Slots, not a render-prop god-object — matches "keep deliberate seams".

**Migration order (low-coupling first, gate between each):**
1. `models/explore/api/-components/ExploreApiRail.tsx` (replace its local `Row`)
2. `models/explore/providers/-components/ExploreProvidersRail.tsx`
3. `models/explore/local/-components/LocalDiscoveryRail.tsx`
4. `mcps/explore/-components/ExploreMcpRail.tsx`
5. `settings/-components/SettingRailPanel.tsx` (migrate wrapper/sections; **keep the stateful save/reset footer local**)
6. `users/-components/UserRailPanel.tsx`, `users/access-requests/-components/RequestDetailPanel.tsx`
7. `models/-components/ModelDetailRail.tsx` (uses `RoutingChainPreview` — Phase 2 test guards), `models/router/-components/RouterInfoRail.tsx`
8. `mcps/-shared/McpRailSections.tsx`, `mcps/-components/MyMcpsRail.tsx`

Do NOT migrate `tokens/index.tsx`'s incidental `dp-` usage (likely a one-off — leave it).

**Tests:** `components/detail-rail/DetailRail.test.tsx` (sections/rows render, empty row omitted). Existing screen tests + rail `data-testid`s guard each migration. **Gate:** `npm test` after each panel; `make test.e2e` at phase end.

---

## Phase 5 — Combobox generalization (risk: medium; guarded by existing Combobox/RepoCombobox/AliasSelector tests)

`components/Combobox.tsx` is hardcoded to a `{value,label}` Status shape with an internal `StatusList`. Generalize without breaking existing callers.

**Approach:** `Combobox<T>` with `items: T[]`, `valueGetter`/`labelGetter` (default to `t=>t.value`/`t=>t.label` so current `{value,label}` callers compile unchanged), optional `renderItem`, and a `mode: 'constrained' | 'freetext'` flag. Preserve responsive desktop-popover / mobile-tablet-drawer behavior and ALL `data-testid`s (`combobox-trigger`, `tab-combobox-trigger`, `m-combobox-trigger`, `combobox-option-*`) — asserted in `Combobox.test.tsx` and E2E.

**Migrate call sites incrementally (one per commit, gate between), constrained family first:**
`models/explore/-shared/FacetCombobox.tsx` → `models/alias/-components/RepoCombobox.tsx` (has own test) → `models/alias/-components/QuantSelector.tsx` → `models/router/-components/AliasCombobox.tsx` → `mcps/new/-components/McpServerSelector.tsx`.

Free-text family (do LAST): `routes/chat/-components/settings/AliasSelector.tsx` — **only fold in if the generic free-text mode cleanly subsumes its effect logic** (its 227-line test is the guard). If it fights the abstraction, **leave AliasSelector as a deliberate seam.**

**Tests:** existing `Combobox.test.tsx`, `RepoCombobox.test.tsx`, `AliasSelector.test.tsx` guard. Add generic-getter/renderItem/freetext cases to `Combobox.test.tsx`. **Gate:** `npm test` after each site; `make test.e2e` at phase end.

---

## Phase 6 — Catalog-screen plumbing consolidation + large-file splits (risk: high; do last, fully guarded)

### 6a — Large-file splits FIRST (lower risk; shrinks the surface). Mechanical, no logic change.
| File (LOC) | Split into |
|---|---|
| `components/shell/ShellChrome.tsx` (406) | Move `ShellBrand`, `ShellBreadcrumb`, `ShellFooter`, `GlobalTooltip`, `AnchoredPopover` each to own file under `components/shell/`; `ShellChrome.tsx` keeps composition + barrel re-exports. **File split only — the deferred ShellFooter prop/popover behavioral rework stays deferred.** |
| `routes/mcps/new/index.tsx` (732) | Extract server-discovery section, form, and validation into `-components/` siblings; route file keeps `createFileRoute` + top-level composition. |
| `routes/chat/-components/ChatUI.tsx` (378) | Extract settings rail + composer into `-components/` siblings; conservative (chat state) — lean on existing chat tests. |
| `routes/mcps/servers/-components/AuthConfigForm.tsx` (351) | Split per-auth-type renderers (`public`/`header`/`oauth`) into siblings; keep the `McpAuthType` switch in the parent. |

**Gate:** `npm test` after each split.

### 6b — `useCatalogScreenState` hook (test-first) + 4-screen migration. **RECOMMENDED: Option B (consolidate plumbing, not a god-component).**

Two independent analyses converged on B: the 4 Explore screens (`ExploreApiScreen` 407, `LocalDiscoveryScreen` 442, `ExploreProvidersScreen` 394, `ExploreMcpScreen` 437) share **~60% screen-level orchestration** (the `commitSearch`/`onSort`/`onFacetsChange`/`onClearAllFacets`/`onReset`/`onPage`/`select` handlers + sort-resolution + `searchInput` sync + `useShellChrome` wiring) but their COLUMNS, facet-state types, sidebars, rails, and data hooks genuinely differ per domain. A single `<CatalogScreen>` render-prop component would need so many slots it'd be less readable than the duplication. So extract the **orchestration**, keep the JSX/columns/sidebars per-screen.

- NEW `routes/models/explore/-shared/useCatalogScreenState.ts` — takes config (`search`, `navigate`, `searchToFacets`, `facetsToSearch`, `hasActiveFacets`, `sortConfig: {storageKey, persistedSorts, naturalOrder}`, `pageSize`) and returns `{facets, sort, order, page, committedSearch, selectedKey, searchInput, setSearchInput, commitSearch, onSort, onFacetsChange, onClearAllFacets, onReset, resetMode, onPage, select}`. Collapses ~120 duplicated lines per screen.
- (Optional, only if clean) a small `searchModuleBuilder` to remove the field-name boilerplate in the 3 fixed-page search modules. Local Discovery's cursor pagination is divergent — **keep its `searchToParams` local**. MCP currently inlines `searchToFacets` in the screen — move it to a new `routes/mcps/explore/-shared/explore-mcp-search.ts` for parity.
- **Do NOT** unify the facet-state types under a generic `Facet<T>` — they're structurally similar but TS-unifiable only via `Record<string,unknown>`, losing type safety for little gain. Leave them per-screen.

**Test FIRST (lock behavior before extracting):** NEW `routes/models/explore/-shared/useCatalogScreenState.test.ts` — search commit sets `q`+`sort=relevance`, drops `page`/`order`; clearing search drops `q`/`sort`; `onSort` toggles active column / adopts natural order for new column + persists; `onFacetsChange` replaces facet slice keeping q/sort/order; `resetMode` precedence filters→query→none; `onPage` strips page on page 1; `select` dedup+replace. Existing guards: `catalog-query.test.ts`, `useSortPreference.test.ts`, and each screen's `index.test.tsx` (post-Phase-1 rename).

**Migrate ONE screen at a time**, reference = `ExploreApiScreen` → `LocalDiscoveryScreen` → `ExploreProvidersScreen` → `ExploreMcpScreen`. **Gate:** `npm test` per screen; `make test.e2e` after each (E2E: `api-explore.spec.mjs`, `local-discovery.spec.mjs`, `providers.spec.mjs`, `explore-mcp.spec.mjs`).

Est. ~400–500 LOC net removed; risk concentrated in the hook contract, fully covered by 4 screen test suites.

---

## Phase 7 — E2E migration to V2 + gap fill (risk: medium; wall-clock heavy)

Run from `crates/lib_bodhiserver/tests-js`. The suite is already 22/24 V2-focused; this finishes it.

**7a — Migrate the last legacy specs.** `specs/multi-tenant/multi-tenant-lifecycle.spec.mjs` (and the request-approval flow) drive the old `UsersManagementPage.mjs`. Repoint to the existing V2 `AllUsersPage.mjs` / `AllAccessRequestsPage.mjs` page-objects (set `reducedMotion:'reduce'` in their `navigateTo*`). Diff the assertions to confirm **no coverage loss** (role change, remove, approve/deny). Once no spec references `UsersManagementPage.mjs`, delete that page-object.

**7b — Fill identified V2 E2E gaps** (new test.steps on existing specs where possible):
- Settings filter tabs (`settings-filter-modified`/`-all`) + group nav — extend `app-settings-v2.spec.mjs`.
- Token status toggle (`token-status-switch`) + visibility/copy — extend `api-tokens.spec.mjs`.
- Access-request detail rail open + filter-tab switching — extend `list-users.spec.mjs` / access-request spec.
- Models column-visibility toggle (`columnsBtn`) + metadata refresh — extend `all-models-v2.spec.mjs`.

**Gate:** per-spec `npx playwright test --config=playwright.config.mjs specs/<path>`; full `make test.e2e` at phase end. Pre-sweep fixed ports (`ports kill ...`) — config uses 51135/41135/3000/55173–55178/55180/6274/6277, `workers:1`.

---

## Phase 8 — Remove `ShellSlotsContext` scaffolding via per-route layout (risk: high; structural; do last)

`components/shell/ShellSlotsContext.tsx` is self-documented temporary migration scaffolding (lines 5–20): `__root` renders ONE `<AppShell>` and screens publish chrome (breadcrumb/headerActions/sidebar/rail) up to it via `useShellChrome`. The end-state is screens owning their `<AppShell>` directly through **pathless layout routes**.

**Approach (incremental, conservative — this is a router-architecture change):**
- Introduce a pathless layout route (TanStack `_app`-style) that renders `<AppShell>` and an `<Outlet/>`; move the bare/fullscreen split (`resolveShellRoute.ts`) into route grouping (app-shell group vs bare group) rather than a runtime prefix check.
- Migrate screens off `useShellChrome(...)` to passing slot props to a per-route `<AppShell>` (or via the layout route's context), one section at a time (chat, models, settings, users, mcps, tokens), gating each with `npm test`.
- Once no screen calls `useShellChrome` and `__root` no longer consumes slots, delete `ShellSlotsContext.tsx` and the `useShellChrome` hook.

**Risk:** touches every migrated screen + root layout + routeTree generation. Heavily E2E-dependent (rail open/close, reduced-motion). **Gate:** `npm test` per section; full `make test.e2e` at the end. If mid-phase risk proves too high, stop and leave the remaining screens on the context (it's harmless) — document where the migration paused.

> Note: this phase is genuinely larger than the rest. It can be split into its own follow-up effort if the earlier phases consume the session; the dedup/test value (Phases 0–6) is independent of it.

---

## Explicitly NOT doing (decided)
- `formatBytes` extraction — single call site (`setup/.../ModelCard.tsx`); no second consumer. Inlining single-use stays.
- `toolMapping.ts` dedicated test — 15-line pure map, covered transitively by the MCP client hook tests.
- `AliasSelector` test backfill — already thoroughly tested (227-line suite).
- A monolithic `<CatalogScreen>` component — rejected for the orchestration hook (B).
- Generic `Facet<T>` type unification — loses TS type safety for little gain.
- Feature-flag removal / un-migrated-page work — **already done** (no flags, no un-migrated pages).
- The prior pass's deferred behavioral items (ChatMessage shim, async-status enum, ShellFooter props, login-mode unify, chatSettings setters, cache-invalidation helpers).

---

## Phase order & gates (monotonic risk)

| Phase | Content | Risk | Gate |
|---|---|---|---|
| 0 | maskApiKey/SortState/ChatSettings dedup; kebab→camel hook renames | very low | `npm test && test:typecheck` |
| 1 | drop `.v2` test suffix (merge 3 collisions) | very low | `npm test` (same pass count) |
| 2 | test backfill: useMcpClient, useMcpClients, RoutingChainPreview | low | `npm test` (green before P4) |
| 3 | `<EmptyState>` extraction | low | `npm test` |
| 4 | `<DetailRail>` primitives + 9 panels | medium | `npm test` per panel; `make test.e2e` end |
| 5 | Combobox generalization + call sites | medium | `npm test` per site; `make test.e2e` end |
| 6 | large-file splits + `useCatalogScreenState` (test-first) + 4 screens | high | `npm test` per item; `make test.e2e` per screen |
| 7 | E2E → V2 (migrate legacy spec + gap fill) | medium | per-spec + `make test.e2e` end |
| 8 | remove ShellSlotsContext via per-route layout | high | `npm test` per section; `make test.e2e` end |

One focused commit per item/migration; `make format` before each commit.

## Verification (end-to-end)
- **Unit:** `cd crates/bodhi && npm test` (full Vitest) green after every batch; `npm run test:typecheck` clean. Net test count should rise (Phase 2 backfill) and never fall (Phase 1 merges, Phase 6 migrations).
- **E2E:** pre-sweep ports, then `make test.e2e` from `crates/lib_bodhiserver/tests-js` at each phase end touching UI (4, 5, 6, 7, 8). Confirm the flaky `setup-browser-extension-with-extension-installed.spec.mjs` separately if it fails (known flake).
- **Visual smoke:** `make app.run.live`, drive the migrated screens in Chrome (rail open/close, combobox, explore search/sort/facets, empty states) to confirm no visual regression from DetailRail/EmptyState/Combobox class-preserving extractions.
