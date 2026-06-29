# API Token Screen — Redesign to New Theme

## Context

The implemented **New API Token** screen (`/ui/tokens/new/`) diverges from the approved
prototype (`Tokens-New-API.html`). The current form is functionally complete — the Phase 4/5/6
backend grants work is done and the form already wires `list_models` / `models` / `list_mcps` /
`mcps` correctly — but it uses plain shadcn `Select` dropdowns and an inline checkbox `MultiSelect`,
not the prototype's richer UX:

- **4 titled sections** (Token Identity · Model Access · MCP Access · Token Scope) inside a card with
  a sticky footer (Cancel / Generate).
- **Model/MCP access** = a "List all …" permission toggle (`/v1/models`, `/v1/mcps`) **plus** an
  `All / Specific` radio with a **slide-in side panel** picker (search, type filter, grouped list,
  selected-rows-with-remove).
- **Token Scope** = two **role cards** (User / Power User) with scope-code badges, not a dropdown.

This redesign ports that prototype into BodhiApp's existing design system (shadcn + Tailwind + the
`.api-keys-screen` scoped-CSS pattern, the same `hsl(var(--…))` tokens the prototype already uses),
builds the access picker as **one reusable component shared by models and MCPs** (App Tokens will
reuse it next — see `project_api_token_grants`), restyles the **token list + detail rail** to follow
the Explore API Models catalog pattern, and removes the now-unused `McpGrant::None` variant so MCP
grants mirror model grants. Tests are migrated/extended at every layer, with new **E2E coverage for
models & MCP access selection** (the current gap).

## Locked decisions (from interview)

1. **Token reveal** — keep the modal `TokenDialog` (not an inline banner), but re-theme it to match
   the prototype's reveal visual. On close → `navigate('/tokens/')`.
2. **Scope** — redesign the **New form**, the **detail rail**, and **rebuild the token list** to
   follow the Explore API Models page: semantic table, ↑/↓ keyboard nav, auto-open rail on select,
   URL-driven selection (browser back/forward).
3. **MCP `none`** — **remove `McpGrant::None` from the backend.** MCP grants mirror models:
   `All | Specific { ids }`, where empty `ids` ⇒ no access. Propagate through enforcement, OpenAPI,
   ts-client, and frontend. `ResourceAccess::None` is removed too (symmetry).
4. **Picker** — build **one** reusable slide-in-panel access picker, used by both models and MCPs,
   ported (not copied) into the app's shadcn/Tailwind system.

---

## Phase 0 — Backend: remove `McpGrant::None` (upstream-first)

Make `McpGrant` symmetric with `ModelGrant`. Empty `Specific { ids: [] }` is the new "no MCP access".

**`crates/services/src/tokens/token_objs.rs`**
- `McpGrant` enum → drop `None`; becomes `#[default] All` + `Specific { ids }` (mirror `ModelGrant`).
- `allows_mcp_connect()` → `match { All => true, Specific { ids } => ids.contains(mcp_id) }`
  (empty ⇒ false, preserving old `None` semantics). `mcp_listable()` unchanged (delegates).
- Fix the serialization rstest case (line ~268) that uses `McpGrant::None`
  → `McpGrant::Specific { ids: vec![] }`.

**`crates/routes_app/src/users/users_api_schemas.rs`**
- `ResourceAccess` enum → drop `None { list }`; keep `All { list }` + `Specific { list, ids }`.
- `ResourceAccess::mcps()` → map `McpGrant::Specific { ids }` (incl. empty) → `Specific`.

**Rust test fixtures referencing `McpGrant::None` / `ResourceAccess::None`** (replace with empty
`Specific`): `routes_app/src/shared/test_token_grants.rs` (2 cases), `users/test_user_info.rs`
(fixture + expectation), `middleware/token_service/test_token_service.rs`,
`oai/test_chat_completions.rs`, `oai/test_models.rs`, `tokens/test_tokens_crud.rs`.
Enforcement code in `routes_app/src/shared/token_grants.rs` and `mcps/routes_mcps.rs` needs no logic
change (it only calls the delegating methods).

**Regenerate types** (mandatory after the schema change):
```
cargo run --package xtask openapi && make build.ts-client
```
This drops `{ type: 'none' }` from the generated `McpGrant` and `ResourceAccess` in `@bodhiapp/ts-client`.

**Gate:** `cargo test -p services -p routes_app` (and `make test.backend` before commit). **Commit.**

---

## Phase 1 — Shared `AccessPicker` component (reusable: models + MCPs)

New co-located component family under **`crates/bodhi/src/components/access-picker/`** (a shared
location since App Tokens and App Access Request will reuse it). Port the prototype's
`model-access-picker.jsx` behavior into shadcn/Tailwind. Generic over "resource" (model or MCP):

- **`AccessPicker.tsx`** — the `ModelAccessPicker` analog. Props:
  `{ mode: 'all'|'specific', onModeChange, items: AccessItem[], selectedIds, onToggle,
     noun, panelTitle, panelSubtitle, allLabel/allDesc, specificLabel/specificDesc, readOnly? }`.
  Renders the `All / Specific` radio (two `map-radio-option`-style cards) and, when `specific`,
  the selected-rows list (name + meta + remove `x`) plus a dashed **"Select / Add more {noun}s"**
  button that opens the panel. Empty-specific shows the "no access will be granted" hint.
- **`AccessPickerPanel.tsx`** — slide-in side panel built on the existing **shadcn `Sheet`**
  (`components/ui/sheet.tsx`, `side="right"`), porting `ModelPickerPanel`: search input, optional
  type filter (`all/local/api`, shown only when items carry `type`), grouped list (Local / API /
  ungrouped), checkbox rows, footer "{n} selected" + Done.
- **`ListingToggle.tsx`** — the "List all {models|MCPs}" permission row: checkbox-style box, title +
  `/v1/models` code chip + description, with the `redundant` soft-hint when mode = `all`.
- **`AccessItem` type** — `{ id: string; label: string; type?: 'local'|'api'; meta?: string }`.
  A small adapter maps API models (`grantableModelIds`) and MCP instances (`{id, name}`) into it.
- **`access-picker.css`** — port the prototype's `.map-*` / `.panel-*` rules, **scoped** under a
  wrapper class (e.g. `.access-picker`) per the `api-keys.css` convention; reuse the app's
  `hsl(var(--…))` tokens (already what the prototype uses). Drop prototype-only affordances we don't
  need: drag-reorder/rank badges, `SingleModelCombo`, and the granted/new upgrade pills (those belong
  to the future App-Access-Request token-exchange flow, not token creation — leave the prop seam but
  no dead UI).

**Tests:** `AccessPicker.test.tsx` — radio switch, opening the Sheet, search-filter, toggle
select/deselect, remove from selected-rows, empty-specific hint; `ListingToggle.test.tsx` — toggle +
redundant hint. **Commit** (component + tests, not yet wired).

---

## Phase 2 — New API Token form redesign

Rewrite **`crates/bodhi/src/routes/tokens/-components/TokenForm.tsx`** and its page
**`routes/tokens/new/index.tsx`** to the prototype layout, keeping react-hook-form + zod and all
existing hooks (`useListModels`, `useListMcps`, `useGetUser`, `useCreateToken`, `useToastMessages`).

- **Schema** — `mcpMode: z.enum(['all','specific'])` (drop `'none'`). `toCreateTokenRequest` maps
  both models and mcps as `all` → `{type:'all'}`, `specific` → `{type:'specific', ids}` (empty ok).
- **Layout** — card head (title + subtitle) → 4 `bf-section`-style blocks separated by dividers →
  footer with **Cancel** (`navigate('/tokens/')`) + **Generate Token**. Port the section/footer/
  divider styling into `tokens.css` (or a `new-token.css`) scoped under `.api-keys-screen`.
  - **§1 Token Identity** — optional name input (`token-name-input`).
  - **§2 Model Access** — `<ListingToggle>` for `listModels` (`/v1/models`) + `<AccessPicker noun="model">`
    fed by `grantableModelIds(modelsData)` with `type`/`meta` where available.
  - **§3 MCP Access** — `<ListingToggle>` for `listMcps` (`/v1/mcps`) + `<AccessPicker noun="MCP">`
    fed by `mcpsData.mcps` (`{id, label:name, meta}`).
  - **§4 Token Scope** — two **role cards** (User / Power User) replacing the scope `Select`; keep the
    role gating (Power User card hidden/disabled unless `userInfo.role !== 'resource_user'`); each card
    shows the scope-code badge (`scope_token_user` / `scope_token_power_user`).
- **Reveal** — keep `TokenDialog` but re-theme `TokenDialog.tsx` to the prototype's reveal visual
  (success header, mono token value, copy, "won't be shown again" warning). `onClose` already calls
  `handleDone` → `navigate('/tokens/')`. Preserve testids: `token-dialog`, `token-value-input*`,
  `token-dialog-done`, `copy-content`/`toggle-show-content`.
- **Preserve form testids** the page/E2E rely on: `token-form`, `token-name-input`,
  `generate-token-button`; add stable testids on the new controls: `list-models-switch`,
  `list-mcps-switch`, `model-mode-{all,specific}`, `mcp-mode-{all,specific}`,
  `access-picker-add-{model,mcp}`, `access-panel-{model,mcp}`, `access-panel-item-{id}`,
  `access-selected-{id}`, `scope-card-{scope_token_user,scope_token_power_user}`.

**Tests:** rewrite `TokenForm.test.tsx` — keep the `toCreateTokenRequest` unit cases (drop the
`none` case, add empty-specific), and rewrite the interaction tests for the new picker/role-card UI
(select specific models + specific MCPs via the panel, switch role card, submit → `onTokenCreated`).
Update `TokenDialog.test.tsx` for the re-themed markup. Update MSW/fixtures only if shapes changed.

**Verify in Chrome** (`make app.run.live`, rebuilt binary — Phase 0 changed the API):
create a token with specific models + specific MCPs, confirm payload + reveal. **Commit.**

---

## Phase 3 — Token list + detail rail (Explore API Models pattern)

Rebuild **`crates/bodhi/src/routes/tokens/index.tsx`** to mirror the catalog reference
(`routes/models/explore/api/index.tsx` + `-shared/`), reusing existing infra rather than new code:

- **URL-driven selection** — add `validateSearch` (Zod): `{ q?, select?, sort?, order?, page? }`.
  Selecting a row writes `?select={id}` with `replace: true`; the rail derives from `select`, so
  **browser back/forward** restores it. (Port the `select()` pattern; the full `useCatalogScreenState`
  facet machinery is overkill here — a thin local hook is fine, but reuse `useSortPreference` for
  sort.)
- **Table** — render rows via the shared **`CatalogTable`** (`-shared/catalog-table.tsx`) or a
  semantic `<table>` using the same `.l-listrow`/`.cat-row`/`.active` classes. Columns: name+scope,
  model-grant summary, mcp-grant summary, status (switch), created/last-used. Keep status filter tabs
  (`ShellFilterTabs`) + search reveal.
- **Keyboard nav** — call **`useListKeyNav()`** (drop-in: ↑/↓ eager-select, Home/End, no-wrap).
- **Auto-open rail on select** — publish `rail` / `railHeader` via `useShellChrome`; `AppShell`
  auto-opens when rail content appears (`railDefaultOpen: false`).
- **Detail rail** — rebuild `TokenDetailPanel` on the **`DetailRail` / DetailRailSection / Row**
  components, re-themed: status + scope, Models section (`list_models` + inference label + specific
  chips `token-model-grant-{id}`), MCP section (`list_mcps` + connect label + chips
  `token-mcp-grant-{id}`), Details (id/scope/dates), Delete (two-step confirm) + status toggle.
  Update `mcpGrantLabel()` to the `all | specific(n) | specific-empty⇒"No MCPs"` mapping (no `none`).
- **Preserve testids**: `tokens-page`, `token-row-{id}`, `token-name-{id}`, `token-scope-{id}`,
  `token-status-switch-{id}`, `token-detail-rail`, `tokens-filter-*`, `tokens-search-toggle`.

**Tests:** rewrite `routes/tokens/index.test.tsx` for the table + URL-select rail + keyboard nav +
grant summary/chips + delete. Keep `useTokens.test.ts` (CRUD unchanged). **Verify in Chrome**
(keyboard nav, select→rail, back/forward). **Commit.**

---

## Phase 4 — E2E (grow the existing spec)

Extend **`crates/lib_bodhiserver/tests-js/specs/tokens/api-tokens.spec.mjs`** and
**`pages/TokensPage.mjs`** (black-box only — UI interactions, no `page.evaluate`/context fetch, per
`feedback_blackbox_e2e`). Add `test.step`s to the existing lifecycle test:

- **Models & MCP access selection (the key new coverage):** register a model + an MCP instance in
  setup; on the New Token form choose **Specific** models, open the slide-in panel, search, select a
  model; choose **Specific** MCPs, select an MCP; generate; open the new token's detail rail and
  assert the model/mcp grant chips (`token-model-grant-*`, `token-mcp-grant-*`). Add a second case:
  **List-all toggles** on with **All** access.
- **List behaviors:** row select auto-opens the rail; **↑/↓ keyboard** moves selection; **browser
  back/forward** restores/clears the rail (`page.goBack()`/`goForward()`).
- **Page object** — add methods: `createTokenWithGrants({name, scope, models[], mcps[], listModels,
  listMcps})`, `selectSpecificModelsInPanel(...)`, `selectSpecificMcpsInPanel(...)`,
  `expectModelGrantChip(id)`, `expectMcpGrantChip(id)`, `selectRowByKeyboard(dir)`. Set
  `reducedMotion: 'reduce'` for the rail/view-transition screens (`feedback_e2e_reduced_motion…`);
  throw in `beforeAll` if required env is missing (`feedback_no_skip_for_missing_env`).

**Run:** `make build.dev-server` then `make test.e2e` from `crates/lib_bodhiserver/tests-js`.

---

## Verification (end-to-end)

1. **Backend:** `make test.backend` green; OpenAPI + ts-client regenerated and committed (no `none`
   in `McpGrant`/`ResourceAccess`).
2. **Frontend unit:** `cd crates/bodhi && npm test` — AccessPicker/ListingToggle, TokenForm,
   TokenDialog, tokens list, useTokens all green.
3. **Chrome (`make app.run.live`, rebuilt binary):** New Token form matches the prototype (sections,
   listing toggles, All/Specific + slide-in panel, role cards, footer); create token with specific
   models + MCPs; reveal dialog; list shows table + grant summaries; row select auto-opens themed
   rail; ↑/↓ nav; back/forward.
4. **E2E:** `make test.e2e` green including the new models/MCP-access and list-behavior steps.
5. **Gates before each commit:** `make format`, relevant unit/backend tests, and E2E when touched
   (`feedback_run_all_gate_checks`). Commit per phase (feature rollout — `feedback_layered_refactors`).

## Critical files

- Backend: `crates/services/src/tokens/token_objs.rs`,
  `crates/routes_app/src/users/users_api_schemas.rs`, `crates/routes_app/src/shared/token_grants.rs`,
  `crates/routes_app/src/mcps/routes_mcps.rs` (+ the 6 Rust test files listed in Phase 0).
- Shared component: `crates/bodhi/src/components/access-picker/` (new:
  `AccessPicker.tsx`, `AccessPickerPanel.tsx`, `ListingToggle.tsx`, `access-picker.css`, tests),
  reusing `components/ui/sheet.tsx`.
- Form: `crates/bodhi/src/routes/tokens/-components/TokenForm.tsx`, `TokenDialog.tsx`,
  `routes/tokens/new/index.tsx`, `components/shell/tokens.css`.
- List/rail: `crates/bodhi/src/routes/tokens/index.tsx`, reusing
  `components/shell/useListKeyNav.ts`, `routes/models/explore/-shared/catalog-table.tsx` +
  `useSortPreference.ts`, `components/detail-rail/*`.
- Tests: `routes/tokens/**/*.test.tsx`, `test-utils/msw-v2/handlers/tokens.ts`,
  `test-fixtures/tokens.ts`, `lib_bodhiserver/tests-js/specs/tokens/api-tokens.spec.mjs`,
  `pages/TokensPage.mjs`.
