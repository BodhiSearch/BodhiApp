# Remove the `models` UI-V2 flag, delete the legacy Models page, migrate the E2E suite

## Context

The V2 "My Models" screen (`ModelsScreenV2`) has shipped behind the per-screen flag
`bodhi.ui-v2.models` (default off). The route `/ui/models/` renders V2 when the flag is on and the
legacy table (`ModelsPageContent`) when off. We now want V2 to be the **only** Models screen:
retire the flag, delete the legacy page + its legacy-only components, and bring the E2E suite onto
the V2 page object.

The catch surfaced during exploration: the legacy `ModelsListPage.mjs` page object is **not** used
only by the three models specs — it is a shared test fixture for creating/editing/deleting/verifying
API models across **18 specs** (api-models, chat, oauth, tokens, models). With the flag gone, the
legacy list no longer renders, so every legacy selector (`alias-cell-*`, `repo-cell-*`,
`editButton`, `deleteButton`, preview modal, chat-from-list, refresh-metadata) disappears. The V2
screen intentionally **dropped** several of those affordances (delete, chat-from-list,
preview/refresh-metadata modal) — the rail is read-only with a single Edit CTA, and creation moved
to sidebar-nav subpages.

Per the user's decision: features that V2 doesn't yet have (delete, chat-from-list, metadata
preview/refresh, capability verification) are **deferred feature-parity work** — record them in
`screen-v2/techdebt.md`, delete the E2E coverage that depended on them, and note that deletion in
techdebt.md too. The V2 page object (`ModelsListPageV2.mjs`) is extended into the full-featured one
and the legacy `ModelsListPage.mjs` is deleted.

## Scope summary

**Frontend (flag + legacy page removal)**
- Remove `'models'` from the `UiV2Screen` enum and update the module doc comment.
- Collapse `routes/models/index.tsx` to render `ModelsScreenV2` directly (delete `ModelsPageContent`,
  `ModelsPageInner`, the flag read, and legacy-only imports).
- Delete legacy-only components no longer referenced after the collapse.
- Delete/replace the legacy component test `index.test.tsx`; fold the "flag off → V1" V2-component
  test into the always-V2 reality.

**E2E (page-object swap + spec migration/deletion)**
- Extend `ModelsListPageV2.mjs` with the create/edit/verify-in-list helpers the surviving specs need;
  remove `enableV2Flag()` (no longer needed); delete `ModelsListPage.mjs`.
- Migrate the 15 non-models specs (api-models, chat, oauth, tokens) that use ModelsListPage only as a
  create/edit/verify fixture onto the V2 page object.
- Delete the models-domain specs whose coverage is gone-in-V2 (`model-alias`, `model-metadata`, and
  the list-affordance parts of `model-router`); record in techdebt.md.

## Phase 1 — Frontend: retire the flag + delete the legacy page

**`crates/bodhi/src/lib/uiV2Flags.ts`**
- Remove `'models'` from `UiV2Screen`. Update the header comment to note Models (Batch 3-1) shipped
  V2-only, mirroring the existing API-Model/Router note.

**`crates/bodhi/src/routes/models/index.tsx`** (currently 351 lines)
- Delete `ModelsPageContent` (lines 68–337), the `columns` const, `ModelsPageInner`, and the
  `useUiV2Flag('models')` read. Make `ModelsPage` render `ModelsScreenV2` directly inside
  `AppInitializer`:
  ```tsx
  export default function ModelsPage() {
    return (
      <AppInitializer allowedStatus="ready" authenticated={true}>
        <ModelsScreenV2 />
      </AppInitializer>
    );
  }
  ```
- Remove now-unused imports (`useState`, `useEffect`, `DataTable`, `Pagination`,
  `DeleteConfirmDialog`, `Dialog*`, `UserOnboarding`, `useDeleteApiModel`, `useDeleteModelRouter`,
  `useListModels`, `useToast`, `useUiV2Flag`, `ModelPreviewModal`, `ModelTableRow`, the alias-type
  guards, `formatPrefixedModel`, `SortState`, etc.). Keep `createFileRoute`, `AppInitializer`,
  `ModelsScreenV2`.

**Delete legacy-only components** (verify each has no other importer first — grep before delete):
- `routes/models/-components/ModelTableRow.tsx` (legacy table row; V2 uses inline `ModelRow`).
- `routes/models/-components/ModelPreviewModal.tsx` (preview modal; V2 has no preview).
- `routes/models/-components/ModelActions.tsx` — **only if** unreferenced after the collapse
  (it was listed as legacy-only; confirm V2 doesn't import it).
- Keep `ModelsScreenV2.tsx`, `ModelDetailRail.tsx`, `ModelSidebarFacets.tsx`, `ModelRailHeader.tsx`
  (in ModelDetailRail), `RoutingChainPreview.tsx`, `SourceBadge.tsx`, `models.css`, `tooltips.ts` —
  these belong to V2 or are shared (e.g. RoutingChainPreview is shared with the router form).

**Component tests**
- Delete the legacy `routes/models/index.test.tsx` (tests `ModelsPageContent`, which is gone).
- Rename `routes/models/index.v2.test.tsx` → `index.test.tsx`. Remove the
  `localStorage.setItem('bodhi.ui-v2.models', 'true')` opt-in (no longer a flag) and delete the
  "falls back to the V1 screen when the models flag is off" test (lines 246–261) — there is no V1
  screen anymore.

**Reuse note:** the V2 creation/edit entry points already exist and need no new code — sidebar nav
subpages (`shell-sub-new-api-model`, `shell-sub-new-local-model`, `shell-sub-new-fallback-model` in
`components/shell/shell-nav-config.tsx`) and the standalone form routes
(`/models/{api,alias,router}/new/` + edit) reachable by direct URL.

## Phase 2 — E2E: extend the V2 page object, retire the legacy one

**`crates/lib_bodhiserver/tests-js/pages/ModelsListPageV2.mjs`** — grow into the full fixture:
- Remove `enableV2Flag()` and its call sites (flag is gone; V2 is the only screen).
- `navigateToModels()` stays (already correct — `/ui/models/`, reduced-motion, waits for
  `data-pagestatus=ready`).
- Add **create entry points** via the V2 nav subpages (or direct URL — see note):
  - `clickNewApiModel()` → navigate to `/ui/models/api/new/` (via `shell-sub-new-api-model` nav item
    or `this.navigate('/ui/models/api/new/')`), wait for SPA ready.
  - `clickNewModelAlias()` → `/ui/models/alias/new/`.
  - `clickNewModelRouter()` → `/ui/models/router/new/`.
  - Recommendation: drive these through the **sidebar nav** (`BasePage.navViaShell`) where practical
    so the nav wiring stays covered; fall back to direct URL for setup-only specs to keep them fast.
- Add **list verification** against V2 row testids (replacing `verifyApiModelInList` /
  `verifyModelRouterInList` / `verifyLocalModelInList`):
  - `expectModelInList(id)` → `model-row-${id}` visible.
  - `verifyApiModelInList(id, ...)` → assert `model-row-${id}` + `model-type-${id}` (provider badge).
    Note: V2 row shows title + subtitle (`base_url · N models`) + provider badge, **not** the
    legacy alias/repo/filename cells — assertions adapt to the row shape.
  - `verifyModelRouterInList(alias)` → router rows key on `id`; the `all-models-v2` spec already
    proves the Router badge. Where specs only have the alias name, expose a helper that finds the row
    by visible title.
- Add **edit** via the rail: `editModel(id)` → `openRow(id)` → `clickRailEdit()` →
  `waitForUrl('/ui/models/api/edit/')` (router/alias variants by source). The rail Edit CTA
  (`model-detail-edit`) already navigates to the correct edit route per source.
- Add `createModelAndCaptureId` parity helpers if needed by `ApiModelFormPage.mjs` (see below).
- **Do NOT add** `deleteModel`, `clickChatWithModel`, preview-modal, or refresh-metadata helpers —
  those affordances don't exist in V2 (deferred; see Phase 4).

**`crates/lib_bodhiserver/tests-js/pages/ApiModelFormPage.mjs`** (lines 43–48)
- It dynamically imports `ModelsListPage` for `getLatestModel()` / `getModelIdFromRow()`. Repoint to
  `ModelsListPageV2` and reimplement those two helpers against V2 row testids (or capture the id
  from the create response/URL as `createModelAndCaptureId` already does). `expectToBeOnModelsListPage`
  (line 70) stays — it asserts the `/ui/models/` URL + `models-content`, both still valid.

**Delete `crates/lib_bodhiserver/tests-js/pages/ModelsListPage.mjs`** once no spec/page imports it.

## Phase 3 — E2E: migrate the surviving specs, delete the gone-in-V2 ones

**Migrate (swap `ModelsListPage` → `ModelsListPageV2`, drop delete/chat-from-list steps):**
These 15 specs use the legacy page only as a create/edit/verify fixture. For each: change the import
+ `new ModelsListPage(...)` → `ModelsListPageV2`, swap method calls to the V2 equivalents, and
remove any `deleteModel`/`clickChatWithModel` cleanup/assertions (replace chat coverage, where a spec
genuinely needs it, by selecting the model directly via `ChatPage.selectModel` — the model already
exists from the create step).
- `specs/api-models/*.spec.mjs` (8): `api-llm-liberty-codex`, `api-live-upstream`, `api-models-extras`,
  `api-models-prefix`, `api-models-forward-all`, `api-sdk-compat`, `api-models-no-key`,
  `api-llm-liberty-anthropic`, `api-gemini-embeddings`.
- `specs/chat/*.spec.mjs` (4): `chat`, `chat-gemini`, `chat-mcps`, `local-models`.
- `specs/oauth/oauth-chat-streaming.spec.mjs`.
- `specs/tokens/api-tokens.spec.mjs`.
- `specs/models/model-router.spec.mjs`: **migrate the create-router-then-chat journeys** (creation via
  nav/URL + form, list verification via V2 row, chat via ChatPage). The router specs' core value
  (fallback / health-aware routing through chat) survives on V2 — only the legacy list-affordance
  glue changes. Keep this spec.

> The `multi_tenant` Playwright project already ignores `**/models/**`; specs under `specs/models/`
> stay standalone-only. The api-models/chat/oauth/tokens specs run in both projects — verify the V2
> page object works without GGUF-only assumptions (it navigates by URL and asserts on row testids).

**Delete (coverage is gone-in-V2; record in techdebt.md):**
- `specs/models/model-alias.spec.mjs` — local-alias lifecycle leans on inline delete, chat-from-list,
  external-link, source badges, and the create buttons; create/edit survive but delete + chat-from-list
  do not. Per the user, delete the spec and defer parity. (Local-alias **create/edit** smoke coverage
  remains via `all-models-v2.spec` rail + the alias form's own component tests.)
- `specs/models/model-metadata.spec.mjs` — entirely built on the preview modal + refresh-metadata +
  per-capability verification, none of which exist in V2. Delete; defer parity.
- `specs/models/all-models-v2.spec.mjs` — **keep**; remove the now-redundant `enableV2Flag()` call in
  `beforeEach` and consider renaming to `all-models.spec.mjs` (no longer "v2"-specific).
- `specs/models/local-discovery.spec.mjs` — unaffected (uses `LocalDiscoveryPage`); leave as-is.

## Phase 4 — Document deferred feature-parity in techdebt.md

Append a new section to `docs/claude-plans/202606/screen-v2/techdebt.md` capturing the V2 Models
feature gaps and the E2E coverage deleted for them:

> ### Batch 3-1 Models V2 — feature parity deferred (flag removed)
> The legacy `/ui/models/` table had inline affordances the V2 master-detail screen does not yet
> reproduce. When the `models` flag was removed (legacy page deleted), the E2E coverage for these was
> deleted with it. Restore each as a V2 feature, then re-add black-box E2E:
> - **Delete from the list** — V2 rail is read-only (Edit CTA only). `model-alias`/`api-models`/
>   `model-router` deletes (`deleteModel`/`deleteLocalModel`, ~18 call-sites) were dropped. Backend
>   delete is still covered at the routes_app/server_app layers; only the UI path is uncovered.
> - **Chat-from-list** — the per-row "chat with model" button is gone; chat coverage now selects the
>   model via the chat screen. Re-add a list→chat shortcut if the product wants it back.
> - **Metadata preview + refresh** — V2 has no preview modal or refresh-metadata button; deleted
>   `model-metadata.spec.mjs` (capability verification per GGUF fixture, modal/per-row refresh). The
>   V2 Local rail shows capability chips but does not verify/refresh metadata.
> - **Deleted specs:** `specs/models/model-alias.spec.mjs`, `specs/models/model-metadata.spec.mjs`.
>   **Trimmed:** delete/chat-from-list steps removed from migrated api-models/chat/tokens/oauth specs.

## Verification

1. **Frontend unit/component tests**
   `cd crates/bodhi && npm run test` — the renamed `routes/models/index.test.tsx` (ex-`index.v2.test`)
   passes without the flag opt-in; no test references `bodhi.ui-v2.models`. Lint + typecheck:
   `npm run lint` (no unused imports/dead exports from the legacy deletion).
2. **Grep gates** (must return nothing):
   - `grep -rn "ui-v2.models\|useUiV2Flag('models')\|'models'" crates/bodhi/src/lib/uiV2Flags.ts`
   - `grep -rn "ModelsListPage.mjs\|new ModelsListPage(" crates/lib_bodhiserver/tests-js`
   - `grep -rn "ModelsPageContent\|enableV2Flag" crates/bodhi crates/lib_bodhiserver/tests-js`
3. **Live smoke (Chrome)** — `make app.run.live`, open `/ui/models/`, confirm the V2 list renders
   directly (no flag), the faceted sidebar + rail work, and sidebar-nav "New API Model / New Model
   Alias / New Model Router" reach the forms. Confirm a created API model appears as a V2 row and the
   rail Edit CTA opens the edit form.
4. **E2E** (from `crates/lib_bodhiserver`, `make build.dev-server` first if backend untouched it's not
   needed): `make test.e2e` — run the migrated `specs/api-models`, `specs/chat`, `specs/oauth`,
   `specs/tokens`, and `specs/models/{all-models,model-router,local-discovery}.spec.mjs`. Set
   `reducedMotion:'reduce'` is already handled in `navigateToModels()`. Confirm the deleted specs are
   gone and the suite is green in both `standalone` and (for non-models specs) `multi_tenant`.
5. **Full gate before commit** — `make format`, frontend tests, and the E2E subset above. Commit as a
   focused change on `main` (trunk-based; no branch/PR).
