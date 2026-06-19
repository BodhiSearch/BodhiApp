# Batch 3-2 — New API Model (form) — Retrospective

Status: implementation complete; RTL green (988 pass, +2); GATE B (live) passed; targeted api-models
E2E green (25/25). **Shipped V2-only — NO flag** (the V2 chrome is always-on; `new-api-model` was
removed from `uiV2Flags.ts`). First **form** sub-phase of Batch 3.

## The reframe — this was a migration, not a rebuild

Exploration overturned the kickoff's framing. The API-model **create/edit form is already a
production, shadcn-styled, fully-wired form-as-page**: `components/api-models/ApiModelForm.tsx` accepts
`mode` (`create|edit|setup`) + `initialData`, handles every real field (Name · the 6 real API formats ·
Base URL · API-Key-with-toggle · Extras · the **LLM-Liberty envelope swap** · Prefix · Forward-mode ·
Model-selection · Test-Connection), wires the real `useCreateApiModel`/`useUpdateApiModel`/
`useGetApiModel` mutations + `convert*` helpers, and navigates on success. The routes were already
thin (`<AppInitializer><ApiModelForm mode=…/></AppInitializer>`), and `__root` already wraps every app
route in `<AppShell contentClass="flush">` — so **the form already rendered inside the V2 shell**.

The only gap vs the design was **chrome**: no breadcrumb (the shell header band was empty), and the
form stretched full-width instead of sitting in a calm centered column. So the batch became: **add a
breadcrumb (via `useShellChrome`) + a centered container to the SAME two routes, behind the flag,
reusing `ApiModelForm` unchanged.** Unlike Batch-1's dialog→page (which deleted an old dialog), there
was **no separate V1 form artifact to delete** — the form was never duplicated.

This is the carry-forward lesson for 3-3/3-4: **when production already owns the form, a "form
sub-phase" is chrome + flag, not a port.** Don't port the prototype's `bf-*` CSS, don't rebuild, don't
restyle the shared component.

## What landed

**Two routes got always-on V2 chrome (no flag); form untouched:**
- `routes/models/api/new/index.tsx` — `useShellChrome({ breadcrumb })` (`Bodhi › Models › New API
  Model`) + `<ApiModelForm mode="create" />` wrapped in `container mx-auto max-w-3xl px-4 py-6`
  (`data-testid="new-api-model-page"`).
- `routes/models/api/edit/index.tsx` — mirror: breadcrumb `Edit API Model`, container
  (`edit-api-model-page`); kept `useGetApiModel` + Loading/Error states; `api_format` stays read-only
  on edit (already enforced by `ApiFormatSelector disabled={isEditMode}` + server
  `ApiFormatImmutableOnEdit`). Breadcrumb publishes through the loading state too.
- `lib/uiV2Flags.ts` — **`new-api-model` removed** from the `UiV2Screen` union (the form ships V2-only).

**Reused verbatim (zero change):** `ApiModelForm` + every `form/`/`actions/`/`hooks/` piece,
`schemas/apiModel.ts`, all data-layer hooks, every `data-testid`/ARIA, **and the shared setup wizard**
(`/setup/api-models/`, `mode="setup"`). **No backend change**, no OpenAPI/ts-client regen.

**Model-selection component redesign (follow-up, same commit).** The user flagged that the
model-selection box didn't match the design. Rewrote `components/ModelSelector.tsx` to the design's
`.cam-*` look (ported to a scoped `components/model-selector.css` under `.model-selector-cam`, brand
indigo/lotus tokens, dark normalized to `.dark`): a grey selected-area card with **indigo monospace
chips**, an available list of **checkbox rows** (monospace names). **Behavior change (approved):**
selected models now **stay in the available list** rendered checked + pink-tinted (clicking a checked
row toggles it off via `onModelRemove`), instead of being filtered out — matches the prototype. This is
the **shared** component, so the setup wizard inherits it too. All `data-testid`s preserved verbatim;
updated the one `ModelSelector.test.tsx` case that asserted the old filter-out behavior + added a toggle
test. Re-validated: RTL full suite 989/0, GATE B live (checkboxes/pink rows/indigo chips/toggle/dark,
console-clean), **E2E 27/27** (api-models + setup-api-models).

## Decisions made (with the user)

1. **Prototype is indicative, not constraining.** Implement the V2 UX with **what production already
   uses** (shadcn `Card` + the existing field components); do **not** replace it with the prototype's
   `bf-*` system, do **not** restyle the shared form.
2. **Keep all production fields** — production is a **superset** of the (indicative) prototype:
   - prototype's API-format list shows Cohere/Ollama (don't exist) → keep the **6 real** formats;
   - prototype omits **Name** (required), **Extras**, the **Liberty envelope** → all kept (real-data
     cuts both ways).
3. **Keep the card title; no above-card page-head.** The design was updated mid-batch to move the
   heading **inside the card** — which is exactly what production's `CardHeader` already does. Confirmed
   live; no change needed.
4. **No flag — ship V2-only, always-on** (decided late, after the gates were green). The chrome is so
   thin and purely additive that a flag/V1-fallback wasn't worth it; the user asked to drop it. Removed
   `useUiV2Flag`/the `if(!v2)` branch from both routes + `new-api-model` from `uiV2Flags.ts`, collapsed
   the RTL flag-on/off pairs to one always-on test per route, re-ran the gates. *(The `models` 3-1 list
   flag is left as-is — see decision 6.)*
5. **Migration, not new-feature dev** — no new E2E, **no Liberty E2E** (none exists for our flow and we
   aren't adding feature tests); reuse the existing `api-models/*` specs (they drive the same form via
   the preserved testids).
6. **The Models V1-list retirement is its own future iteration** (separate from this form) — captured
   in @techdebt.md + the `tracker.md` flags table: retire the `models` flag + delete the V1
   `ModelsPageContent` list + migrate the V1-list↔form create/edit/**delete** E2E specs, all together,
   when the Models section is accepted. The forms (3-2 done, 3-3/3-4 later) are independent of that.

## Surprises / learnings (carry forward)

- **The design moved the heading inside the card mid-batch** (user updated `design/` at 12:32). Re-walked
  the served prototype (per the 3-1 "re-read after design updated" rule) — confirmed `bf-card-head` now
  lives at the top of `bf-card`, matching production's `CardHeader`. No code change; the locked decision
  already aligned. *Re-walking after a design ping is cheap insurance.*
- **`useShellChrome` can't carry `contentClass`/`mainScroll`** (the slots seam only has
  breadcrumb/headerActions/sidebar/rail/railHeader/railDefaultOpen). The root already passes
  `contentClass="flush"`; `mainScroll` stays `true` (the form footer scrolls with the page). The
  prototype's `mainScroll={false}` sticky-footer is **not reproduced** — acceptable, matches tokens/new.
  Did **not** touch `__root`/`AppShell`.
- **GATE B confirmed all real behaviors live** in the V2 chrome: the **Liberty field-swap** (selecting
  "LLM Liberty OAuth" replaces Base-URL/Key with the `npx @bodhiapp/llm-liberty login` envelope paste),
  **Extras** for `anthropic_oauth` (real preset headers), and **api_format lock-on-edit** (disabled +
  "Format cannot be changed…" hint). Light + dark + responsive (414px, sidebar→drawer); **console-clean**.
- **Existing route RTL keeps passing without the harness** — `useShellChrome` with no `ShellSlotsProvider`
  hits the default no-op setter, so the old tests are inert to the chrome. The **+2** V2 tests use the
  canonical `ShellSlotsProvider`+`SlotsConsumer` harness (from `models/index.v2.test.tsx`) to assert the
  breadcrumb + container publish; the edit one also asserts `api-format-selector` disabled + locked-hint.

## Gate results

- **Frontend RTL:** `npm test` — **988 passed / 5 skipped / 0 failed** (986 baseline + **2** V2-chrome
  tests: new + edit "publishes breadcrumb + container"; edit also asserts `api-format-selector` disabled
  + locked-hint). Typecheck clean (the `UiV2Screen` union shrank); the 3 source files lint-clean (test
  files are eslint-ignored by the repo config); prettier clean.
  *(Was briefly 990 with the flag-on/off pairs; collapsed to 988 when the flag was dropped.)*
- **E2E (targeted, standalone):** `api-models/{extras,prefix,forward-all,no-key}.spec.mjs` —
  **25/25 passed (9.6m)** for the chrome change; then **27/27 passed (10.0m)** re-run after the
  model-selection redesign with **`setup/setup-api-models.spec.mjs` added** (the redesign is the shared
  component, so the setup wizard is re-validated too). They drive the same create/edit/delete/chat flows
  by the preserved testids. Liberty / live-upstream / gemini / sdk-compat specs are env-gated (real
  provider creds) and untouched. Did **not** add specs.
- **GATE B (live):** `make app.run.live` (no rebuild — no backend change), flag on. Create (OpenAI
  happy-path) + Edit (`anthropic_oauth`: lock-on-edit + Extras) + Liberty field-swap; **light + dark +
  responsive (414px)**; **console-clean** (only the known router-nav VT `InvalidStateError` on entry).
- **Backend:** no Rust change; backend gate not required.

## Follow-ups

1. **Run the FULL E2E matrix** at a convenient point — the shared shell/nav changes are app-wide, but
   this batch added only flag-gated frontend chrome (no shared-component edit), so regression risk is
   minimal. (Known pre-existing failures unchanged: chat-resize techdebt, MCP-OAuth,
   `api-live-upstream`/liberty live-provider timeouts, browser-extension.)
2. **Models V1-list retirement is the dedicated iteration** (the `models` flag + the V1
   `ModelsPageContent` list + the V1-flow E2E specs — techdebt.md + tracker flags table). The forms are
   independent of it.
3. **Next: 3-3 (New Fallback alias form)** — same reframe: the prod `ModelRouterForm` likely already owns
   the form, so the port is **chrome + a published info `rail`** (the prototype adds a ROUTING-CHAIN /
   HOW-IT-WORKS / TIPS rail — heavier than 3-2's breadcrumb-only). The **flag is 3-3's call at plan
   time** (3-2's no-flag precedent applies if the rail is also purely additive). Watch the fallback
   **resilience settings** (`cooldown_secs`/`max_attempts`/`honor_retry_after`) — the prototype omits
   them, the backend has them: **keep them**. Kickoff: `batch-3-3-new-fallback-model-kickoff.md`.
