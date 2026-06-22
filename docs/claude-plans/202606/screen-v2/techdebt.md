# UI V2 Migration — Tech Debt / Failing Tests

Tracked failing tests + deferred fixes surfaced during the migration. Update as items are resolved
(remove the entry) or as new ones appear in later batches.

## Failing / Deferred

### Chat `chat.spec.mjs:72` "multi-chat management and error handling" — Batch-0 shell regression
Confirmed (not env/streaming flakiness): the chat-history **delete buttons are unclickable** because
the new AppShell's left **resize handle** floats on top of them.

Root cause, measured live (1280×720, standalone):
- `.shell-resize.left` is a 16px-wide, full-height, **`z-index:40`** strip centered on the shell
  sidebar's 240px boundary → occupies `x≈232–248`, full height.
- The chat page is an unmigrated screen that renders its **own** `fixed`, **`z-10`** history sidebar
  (width 260px, starting at `x=0`) inside the shell. Its delete buttons sit at `x≈227–247`, directly
  under the higher-z resize handle. `document.elementFromPoint(deleteCenter)` returns
  `DIV.shell-resize left`, so Playwright's click never lands → 30s actionability timeout.
- `z-index` on the delete button does not help: the chat sidebar (`z-10`) and the handle (`z-40`)
  live in different stacking contexts, so the button can't rise above the handle from inside its own.

**Decision: defer.** Fix when chat migrates to screen v2 (the unmigrated chat sidebar overlapping the
shell sidebar is the real source of the collision; it goes away once chat adopts the shell chrome).
Candidate fixes when we get there: suppress the shell's left/right resize handles on `flush`
coexistence screens that render their own sidebar, or narrow the handle's pointer-capture area.
Fails identically in `standalone` + `multi_tenant`.

## Other deferred items
- **Router view-transition `InvalidStateError` on navigation.** `main.tsx` sets
  `defaultViewTransition: true`; on a shell-route navigation the browser can log one
  `InvalidStateError: Transition was aborted because of invalid state` (at `:0:0`, no app stack —
  from TanStack Router's own transition, which the app can't catch from `useViewTransition`).
  Screen-agnostic: reproduces identically on shipped screens. Functionally harmless (DOM updates
  correctly). **Batch 2 hardened `useViewTransition.startViewTransition` further**: it now swallows the
  **async** rejection of the transition's `ready` promise (not just `finished`) in addition to the
  synchronous try/catch fallback — so the app's own in-page selection/rail/theme transitions are now
  console-clean even when interrupted by an overlapping navigation cross-fade. The remaining
  navigation-time exception (when it occurs) is router-internal. Real fix = revisiting the router-level
  `defaultViewTransition` config (e.g. gating/awaiting in-flight transitions) — cross-cutting, deferred.
  This is carry-forward risk #1 from the Batch-1 retro.
- Repo-wide ESLint: ~289 pre-existing errors in files untouched by the migration (import/order in
  `stores/initStores.ts`, `types/chat.ts`; a missing `react/display-name` rule in
  `tests/mocks/framer-motion.tsx`; etc.). Batch-0 files are lint-clean. Not addressed here.
- `make test.backend` requires Docker (PostgreSQL); pg-variant DB tests panic at fixture setup if
  Docker is down. Start Docker before the backend gate.
- **Manage Users V2 role filter + counts are per-page only** (Batch 2). `useListUsers` is
  server-paginated; the role filter tabs + their counts filter only the **current page** (mirrors the
  already-shipped Access Requests screen — there's no server-side role-filter query param). Accepted
  limitation; a real fix needs a backend `role` query param on `GET /bodhi/v1/users`.

## Models V1-list retirement + E2E-spec migration — DONE (Batch 3-1 flag removed)
The `models` flag and the V1 `ModelsPageContent` list have been **removed**; the V2 My-Models list is
now the only Models screen. `routes/models/index.tsx` renders `ModelsScreenV2` directly; the
legacy-only components (`ModelTableRow`, `ModelPreviewModal`, `ModelActions`, `SourceBadge`) and the
legacy component test were deleted; `index.v2.test.tsx` became `index.test.tsx` (no flag opt-in). The
E2E suite migrated off the legacy `ModelsListPage.mjs` (deleted) onto the extended
`ModelsListPageV2.mjs`. See the feature-parity gaps below for what the V2 list does NOT yet do.

### Batch 3-1 Models V2 — feature parity deferred (flag removed)
The legacy `/ui/models/` table had inline affordances the V2 master-detail screen does not yet
reproduce. When the `models` flag was removed (V1 list deleted), the E2E coverage that depended on
those affordances was deleted with it. Each should come back as a V2 feature, then have black-box E2E
re-added:
- **Delete from the list** — the V2 rail is read-only (Edit CTA only); there is no delete on the
  Models screen. All `deleteModel`/`deleteLocalModel` E2E steps (~18 call-sites across api-models /
  chat / oauth / tokens / model-router specs) were removed. Backend delete is still covered at the
  `routes_app` / `server_app` layers; only the UI delete path is now uncovered by E2E.
- **Chat-from-list** — the per-row "chat with model" button is gone. Specs that drove chat from the
  list now select the model on the chat screen (`chatPage.navigateToChat()` + `selectModel`). Re-add a
  list→chat shortcut if the product wants it back.
- **Metadata preview + refresh** — V2 has no preview modal or refresh-metadata button. The V2 Local
  rail shows capability chips but does not verify or refresh metadata. `specs/models/model-metadata.spec.mjs`
  (per-GGUF-fixture capability verification + modal/per-row refresh) was **deleted** — no V2 home.
- **Local-alias lifecycle list affordances** — `specs/models/model-alias.spec.mjs` leaned on inline
  delete, chat-from-list, external-link, and source badges; it was **deleted**. Local-alias
  create/edit smoke coverage remains via `all-models.spec` (rail + Edit CTA) and the alias form's own
  component tests.
- **Deleted specs:** `specs/models/model-alias.spec.mjs`, `specs/models/model-metadata.spec.mjs`.
  **Trimmed:** delete + chat-from-list + legacy `getModelRow` cell-assertions removed from the
  migrated api-models / chat / oauth / tokens / model-router specs. Specifics worth re-adding when
  delete returns to the UI:
  - `api-models-forward-all` lost the "prefixed models disappear from chat after deletion"
    assertion (it depended on deleting the alias from the list). The `getModelRow` `{prefix,
    forward_all, …}` list-cell snapshot it also had was re-expressed as a row-presence check plus the
    existing edit-form verification (`verifyForwardAllModeSelected` + `verifyPrefixValue`).
  - `api-sdk-compat` lost its dedicated `[teardown] delete provisioned aliases` test (delete-only).
  - `api-live-upstream` lost the `deleteAllModels` teardown for both the API-token and
    OAuth-app-token paths.
  - All removed cleanup relied on `autoResetDb` resetting the DB between tests, so dropping it does
    not cross-contaminate the suite.

## Migration scaffolding to REMOVE when the whole migration completes (added Batch 1)
Temporary structures introduced to enable in-place, flag-gated coexistence. **Delete these once
every screen is migrated** (tracked here so they don't become permanent non-obvious cruft):
- **`ShellSlotsContext` + `useShellChrome`** (`components/shell/ShellSlotsContext.tsx`) — lets a
  migrated screen publish breadcrumb/headerActions/**sidebar**/rail up to the single root `<AppShell>`
  during coexistence (the `sidebar` slot was added in Batch 2 for App Settings' group nav). Once all
  screens are migrated, screens pass props to a per-route `<AppShell>` directly (or we adopt pathless
  `_layout` routes) and this context is deleted.
- **Per-screen `useUiV2Flag` machinery** (`lib/uiV2Flags.ts`, `hooks/useUiV2Flag.ts`) — removed when
  the last batch lands and every screen is V2-only.

## Deferred architectural improvement (planned follow-up, NOT temporary) (added Batch 1)
- **Scalable route-declared layout seam.** Batch 1 makes the bare review screen work the minimal way:
  add `/apps/access-requests/review` to `resolveShellRoute.ts`'s `BARE_PREFIXES` + render bare routes
  through the new reusable `components/shell/BareLayout.tsx`. The **central `BARE_PREFIXES` pathname
  switch should eventually be replaced** by each route declaring its layout — TanStack Router
  `staticData.layout: 'shell' | 'bare'` read in `__root` via `useMatches()`, converging to idiomatic
  pathless `_shell/`/`_bare/` layout routes. Deferred to a dedicated routing step (out of scope for
  the API-Keys batch); `BareLayout` is built so this lands as a drop-in.
