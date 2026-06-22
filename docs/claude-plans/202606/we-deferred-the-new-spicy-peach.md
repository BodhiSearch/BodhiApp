# Plan — Batch 3-4: New Local Model form, evolved in-place (phase-wise)

## Context

The **New Local Model form** (`/models/alias/new` + `/edit`, component
`crates/bodhi/src/routes/models/alias/-components/AliasForm.tsx`) was deferred during the screen-v2
Models migration because the backend had no way to enumerate a repo's **quantisations** — the form
could only offer files **already downloaded** (cascading `repo → filename → snapshot` comboboxes from
`useListModelFiles`).

Two things changed:
1. **Batch 3-6 (Explore · Local Models) shipped** against the live reference API. The single-model
   endpoint returns `quants: [{ name, filename, size, bits, method, recommended }]` where **`filename`
   is the real repo-relative `.gguf`** (`@bodhiapp/reference-api-types` `Quant`).
2. The user decided to **relax the backend** so an alias can be created for a model that is **not yet
   downloaded** — the backend creates the alias *and* kicks off the async download (reusing the pull
   path), so progress shows in the Downloads panel.

**Approach (decided): evolve the existing form in place, phase by phase — do NOT rebuild from
scratch.** The form has extensive component + E2E coverage we must preserve. Each phase makes one
focused change, **updates the related component + E2E tests in the same phase**, runs
`make test.ui.unit` **and** the local-model/user-alias E2E specs, fixes failures, then **commits**
before the next phase. Styling/theming is the LAST phase. (Memory: evolve-form-keep-tests,
phased-vertical-slice-dev, layered-refactors → feature rollout commits per phase.)

## Key decisions (carry into every phase)

- **Backend relax:** `create_alias_from_form` no longer rejects a missing local file; if absent it
  creates the alias **and** spawns the same async pull the Downloads panel shows.
- **E2E test-mode flag:** real downloads must not run in CI. Add a **test-mode setting** (new
  `BODHI_*` key, e.g. `BODHI_TEST_MODE`, set by the E2E harness via `app_settings` like
  `BODHI_DEV_PROXY_UI`). When on, the alias-create download path creates the `DownloadRequest` row
  **directly in `completed` state** (no real fetch). Production/dev unchanged.
- **System prompt:** stored inside `request_params` (new `OAIRequestParams.system_prompt`), injected
  at request-forward time by prepending a system message to `messages[]`
  (`OAIRequestParams::apply_to_value`, applied in `server_core/src/shared_rw.rs:221`).
- **Form is frontend sugar over JSON:** textareas render `request_params` as `key=value\n` (system
  prompt in its own textarea) and `context_params` as `key value\n`; convert to/from the typed JSON
  on load/submit. Backend contract stays JSON (`UserAliasRequest`).
- **Quant selector** correlates reference-API quants with local state → Downloaded / Downloading /
  Not-downloaded per quant.
- **Repo = free-text only.** Catalog autocomplete deferred → `screen-v2/techdebt.md`.
- **Drop unbacked design bits:** Presets (Default/Coding/…) → `techdebt.md`; BPW column and the
  distrusted `recommended` badge dropped; request-param click-to-add lists only the **8 real** params
  + `system_prompt`.
- **V2-only, no flag.** The form already lives on the real routes; remove `'new-local-model'` from
  `lib/uiV2Flags.ts` when the work is done (or in the styling phase).

---

## Phases

> Each phase: implement → update component tests + E2E spec → run `make test.ui.unit` and the
> local-model/user-alias E2E (`make build.dev-server` if backend changed, then the alias specs) →
> fix → **commit**. Order below is the user's; adjust only with reason.

### Phase 1 — Backend: create-then-download + E2E test-mode flag

Make the API accept an alias for a not-yet-downloaded file and start the download; add the test-mode
short-circuit so E2E never fetches real files.

- **`services`** (`crates/services/src/models/data_service.rs`, `create_alias_from_form` ~L271-286):
  drop the hard `local_file_exists` error. If the file is present → keep resolving the real snapshot;
  if absent → create the alias (`snapshot = form.snapshot.unwrap_or(main)`) **and enqueue a
  download**.
  - **Download seam (preferred):** factor "create a `DownloadRequest` row + `spawn_pull` background
    task" out of `routes_app/src/models/files/routes_files_pull.rs` (`spawn_pull` /
    `execute_pull_by_repo_file`) into a reusable downloads-service method that both the pull route and
    the alias-create path call (dedup an in-flight request via `find_by_repo_filename`). One download
    code path (memory: generic/evolvable). If lifting is too invasive this batch, orchestrate in the
    `models_create` route handler instead — keep the single POST → alias + download UX.
  - **Test-mode:** when the new test setting is on, create the `DownloadRequest` directly in
    `completed` status and skip `spawn_pull`. Read the setting via `SettingService` (mirror
    `is_production()` in `setting_service.rs`).
  - **MultiTenant:** HubService rejects downloads there — creating an alias for an undownloaded file
    must error clearly (surface the download-unsupported error), not create a dangling alias.
- **Test-mode setting plumbing:** add the `BODHI_*` key in `services/src/settings/setting_objs.rs`
  + accessor; have the E2E bootstrap set it via `app_settings` (`lib_bodhiserver` app_options /
  the `bodhiserver_dev` launch in the tests-js harness).
- **Tests (Rust):** `cargo test -p services -p routes_app --lib` — create-when-absent creates alias +
  enqueues download; test-mode → download is `completed` immediately; create-when-present resolves the
  real snapshot; multi-tenant create-undownloaded errors. Existing
  `test_create_alias_handler_non_existent_repo` changes meaning (valid repo + absent file now
  succeeds; an *invalid* repo string still 400s) — update it.
- **E2E:** add a `test.step` (or new case) to the local-model alias spec: create an alias for an
  undownloaded repo/filename → alias appears and the Downloads panel shows it **completed** (test-mode).
- **Gate:** `make test.backend` (or scoped crates) + `make test.ui.unit` + local-alias E2E → commit.

### Phase 2 — Quant file selector (replace the filename combobox)

Swap the `filename` combobox for a quant selector driven by the reference API + local status. Keep the
`repo` input free-text; on (debounced) repo change, fetch quants via `useModelDetail`
(`hooks/reference/useDiscoverModels.ts`; split `repo` into `namespace`/`repo`).

- New sub-component under `routes/models/alias/-components/` (e.g. `QuantSelector`) listing
  `detail.quants` (name · size · select), setting `form.filename` on select.
- **Status correlation:** mark each quant by matching its `filename` against `useListModelFiles`
  (downloaded) + `useListDownloads({enablePolling})` (pending → "Downloading"); else "Not downloaded".
  Info note mirrors the design ("will download automatically after save" / "already downloaded").
- Preserve the `?repo&filename&snapshot` deep-link prefill the `new` route already supports (arrive
  from Explore "Add to Bodhi"). Keep snapshot input (default `main`). MultiTenant: restrict to
  already-downloaded files (no catalog-pull affordance).
- **Tests:** component tests for quant fetch/empty/error + status correlation + filename set on
  select; MSW for the reference detail + `/models/files` + `/models/files/pull`. Update the E2E to
  drive the new selector. **Watch the cmdk multi-combobox strict-mode gotcha** if comboboxes share
  options (3-3 retro).
- **Gate:** `make test.ui.unit` + local-alias E2E → commit.

### Phase 3 — Context params: textarea + click-to-add catalog

Evolve `context_params` from the current input into the design's editor: a `key value\n` textarea with
a click-to-add catalog of real llama-server flags.

- Render existing `context_params` ↔ textarea (`schemas/alias.ts` `convertApiToForm`/`convertFormToApi`
  already split/join lines — extend for the textarea shape).
- Click-to-add catalog of llama-server flags (real flags only). **Remove presets** (Default/Coding/…)
  → add a `techdebt.md` entry for preset support later.
- **Tests:** component tests for add-flag + text⇄array round-trip; E2E step editing flags. 
- **Gate:** `make test.ui.unit` + local-alias E2E → commit.

### Phase 4 — Request params + system prompt

Evolve `request_params` into the design's editor (`key=value\n` textarea + click-to-add catalog) and
add the **System prompt** textarea — both backed by `request_params` JSON.

- **Backend:** add `OAIRequestParams.system_prompt` (`services/src/models/model_objs.rs`) +
  inject in `apply_to_value` (prepend a `{role:"system"}` message to `messages[]`, only if not
  already present; mirror the "only set if absent" convention). Regen:
  `cargo run --package xtask openapi && make build.ts-client`.
- **Frontend:** `schemas/alias.ts` parses `key=value\n` → typed `OAIRequestParams` (validate keys
  against the 8 real params via `requestParamsSchema`; ignore/flag unknowns) and folds `system_prompt`
  in; inverse on load (system prompt to its own field). Click-to-add catalog = 8 real params only.
- **Tests:** Rust — `apply_to_value` prepends system message (no-op when one already leads);
  round-trip. Component — text⇄JSON incl. system prompt, add-param. E2E — set a system prompt + params,
  create, then (where feasible) assert it's applied on forward.
- **Gate:** `make test.backend` (scoped) + regen + `make test.ui.unit` + local-alias E2E → commit.

### Phase 5 — Styling & theming (no migration, swap to V2 look)

Restyle the now-feature-complete form to the V2 theme, matching sibling migrated forms
(`components/api-models/ApiModelForm.tsx` — 3-2, `routes/models/router/-components/ModelRouterForm.tsx`
— 3-3): shadcn/Tailwind + design tokens, V2 shell chrome via `useShellChrome` (breadcrumb + optional
help rail, **one publisher per screen**). Reference `design/Create New Local Model v4.html` +
`design/models/local-model-app.jsx` for layout/feel only (don't copy raw CSS; scope any ported rules).
Remove old styles outright (no compat). Remove `'new-local-model'` from `lib/uiV2Flags.ts` + the
`UiV2Screen` union (keep the nav sub-page entry). Update `screen-v2/tracker.md` (3-4 → ✅).

- **Tests:** update any component test asserting old markup/testids; E2E reducedMotion + selectors.
- **Gate:** `make test.ui.unit` + full local-alias E2E + GATE B (live walk) → commit → retro.

---

## Cross-phase verification

- **Backend (Phases 1, 4):** `cargo test -p services -p routes_app --lib`; `make test.backend` before
  the dependent commit. Regen ts-client after the `system_prompt` field lands (Phase 4).
- **Frontend (every phase):** `cd crates/bodhi && npm run format && npm run lint`; **`make
  test.ui.unit`** (component/unit) each phase.
- **E2E (every phase):** `make build.dev-server` (when backend changed) then run the **local-model /
  user-alias alias specs** in `crates/lib_bodhiserver/tests-js/` and fix before committing. Black-box
  only; `reducedMotion:'reduce'`; throw on missing env (memory carry-forwards). ⚠ Backend-changing
  phases need the **rebuilt** binary live for GATE B (memory: gateb-rebuild).
- **GATE B (final, Phase 5):** `make app.run.live`, `/ui/models/alias/new/`: type an undownloaded repo
  → quants load with correct per-quant status; select a not-downloaded quant + name + system prompt +
  flags → **Create** → alias created and Downloads panel shows it downloading (live) / completed
  (test-mode); chat against the alias and confirm the **system prompt is applied**. Edit an alias
  (alias locked, params hydrate). MultiTenant hides/blocks undownloaded-create. Light + dark +
  responsive; console clean.

## Files (representative)

- **Backend:** `services/src/models/data_service.rs` (relax + download trigger),
  `services/src/models/model_objs.rs` (`system_prompt` + `apply_to_value`),
  `services/src/settings/setting_objs.rs` (test-mode key),
  `routes_app/src/models/files/routes_files_pull.rs` (factor pull-enqueue) and/or
  `routes_app/src/models/alias/routes_alias.rs` (`models_create`),
  `lib_bodhiserver/src/app_options.rs` + tests-js harness (set test-mode in E2E).
- **Frontend:** `routes/models/alias/-components/AliasForm.tsx` (+ new `-components/` sub-pieces),
  `schemas/alias.ts`, `routes/models/alias/{new,edit}/index.tsx`, `lib/uiV2Flags.ts`,
  hooks `useModels`/`useModelFiles`/`useDownloads`, `hooks/reference/useDiscoverModels.ts`.
- **Docs:** `screen-v2/techdebt.md` (repo autocomplete + presets), `screen-v2/tracker.md`,
  a `batch-3-4-...-retro.md`.
