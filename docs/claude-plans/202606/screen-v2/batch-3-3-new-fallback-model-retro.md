# Batch 3-3 — New / Edit Model Router (form) — Retrospective

Status: implementation complete. **Shipped V2-only — NO flag** (`new-fallback-model` removed from
`uiV2Flags.ts`). RTL green (router suite 16/16; full suite 998 pass + 1 pre-existing flaky login-timeout
that passes in isolation). model-router E2E **3/3 green against the rebuilt UI** (baseline ran green
first, then again after the rebuild). All-Models-V2 E2E 2/2 green (validates the shared chain refactor).
GATE B (live) passed. Second **form** sub-phase of Batch 3 — and the first that was a genuine
**form-body rebuild**, not chrome-only.

## The reframe vs 3-2: this WAS a rebuild

3-2 (API form) was chrome-only because the prototype matched production visually. 3-3 is the opposite:
the (updated) prototype is materially richer than the plain production `ModelRouterForm` — step cards
with a step-number badge + position labels, a searchable alias **combobox** with type/provider badges, a
per-step route-to-model field, "↓ on error" connectors, and a **published shell rail** (live routing
chain + how-it-works + tips). The user chose a **full form-body rebuild** (keeping the real data layer).
So unlike 3-2, the `ModelRouterForm` body was rewritten — but the mutations, the request contract, and
the submit-gating are untouched.

## Mid-plan design update (re-walked, per the 3-1 rule)

Partway through planning, the design was committed (HEAD `c6d1ee93`, "Create Fallback Model.html design
update"). Re-walking it (the 3-1 "re-read after a design ping" rule) showed the update **simplified the
batch and renamed the screen**:
- **Renamed "Fallback Alias" → "Model Router"** everywhere (title, breadcrumb, nav label, primary
  button) — matching production's long-standing naming (`ModelRouterForm`, `useCreateModelRouter`,
  `ENDPOINT_MODEL_ROUTERS`, `source: 'model_router'`). "Fallback Alias" was only ever the old prototype's
  name. (User confirmed: use "Model Router" consistently.)
- **Added the resilience + Strategy fields** the earlier prototype omitted → the prototype now mirrors
  production's section set 1:1 (IDENTITY incl. disabled Strategy=Fallback · RESILIENCE · TARGETS). The
  "prototype omits resilience" divergence resolved itself — no judgment call needed.
- **Moved help into the shell rail** (`rail` + `railHeader` + `railDefaultOpen`, with a close ✕ via
  `useShell().collapseRail`) — confirming the planned published-rail approach over an in-content column.
- **Removed the drag handle** — arrow ▲/▼ reorder only (we don't add DnD).

## What landed

**Form decomposed into `routes/models/router/-components/` (was one 373-line file):**
- `ModelRouterForm.tsx` — orchestrator: all state (plain `useState`, no react-hook-form), the real
  `useListModels` + `aliasByIdentity`, `useCreate/UpdateModelRouter`, `handleSubmit`. Renders the card
  (IDENTITY · RESILIENCE · TARGETS · footer) and **publishes breadcrumb + rail in ONE `useShellChrome`**.
- `StepCard.tsx` · `AliasCombobox.tsx` (cmdk `Command`+`Popover`) · `RouteToModelField.tsx` ·
  `AliasTypeBadge.tsx` (TypeBadge + ProviderBadge off the real `source`/`api_format`) · `StepConnector.tsx`
  · `RouterInfoRail.tsx` (+ `RouterRailHeader`) · `router-form.css` (scoped `rf-*`, reuses the existing
  `--c-*` tokens + the shared `m-chain-*`).
- **`routes/models/-components/RoutingChainPreview.tsx` (shared reuse win):** extracted the chain
  renderer; **`ModelDetailRail`'s `FallbackRailBody` now consumes it too** (a `disabledLabel` prop keeps
  the detail rail's "disabled" wording while the form uses "skipped").

**Routes:** `router/new` + `router/edit` wrap the form in a `container mx-auto max-w-3xl` (testid
`router-form-page` + `data-pagestatus="ready"`), passing the breadcrumb as a prop. Edit keeps
`useGetModelRouter` + Loading/Error/NotFound, with a `BreadcrumbOnly` wrapper publishing the breadcrumb
in the non-form states (R9).

**Nav + flag:** `shell-nav-config.tsx` label "New Fallback Alias" → **"New Model Router"** (nav id
`new-fallback-model` kept — it's the `shell-sub-{subPage}` testid). `new-fallback-model` removed from the
`uiV2Flags.ts` `UiV2Screen` union (verified it was never consumed via `useUiV2Flag`).

**Tests:** rewrote `ModelRouterForm.test.tsx` (6 kept behaviors + 7 new incl. the two contract tests);
added `router/new/index.v2.test.tsx` (3 route-chrome tests covering new + edit + the no-id error guard);
added `mockUpdateModelRouter` MSW handler; rewrote the `ModelRouterFormPage.mjs` E2E page object for the
combobox.

**Reused verbatim (zero behavior change):** the `useCreate/UpdateModelRouter`/`useGetModelRouter` hooks,
the request contract (`ModelRouterRequest` with `strategy:'fallback'`), every `data-testid`. **No backend
change, no OpenAPI/ts-client regen.**

## Decisions made (with the user)

1. **Full form-body rebuild** (not chrome-only) — but keep the real data layer + mutations.
2. **Keep all real fields** — resilience + Strategy; the design update made this moot (prototype has them).
3. **Keep production submit-gating ONLY** (the critical guard): Create disabled on
   `submitting || resilienceInvalid` and nothing else. The prototype's stricter name-regex / ≥2-enabled /
   per-step-required gates are rendered **display-only** (red borders, a warn rail badge) and **never**
   block submit. Locked by two explicit RTL contract tests (empty forward-all model stays submittable;
   0-target/empty-name stays submittable — only invalid resilience disables).
4. **Shell published rail**, `railDefaultOpen: true`, memoized on the minimal derived inputs.
5. **No flag — V2-only, always-on** (3-2 precedent; the chrome + rail are additive over the same routes).
6. **Full test migration** + the E2E always-runs prerequisite (below).

## Surprises / learnings (carry forward)

- **One `useShellChrome` publisher per screen.** First attempt had the route publish the breadcrumb AND
  the form publish the rail → the form's publish (no breadcrumb) **clobbered** the route's breadcrumb
  (the setter replaces the whole slots object; last writer wins). Fix: the form publishes **breadcrumb +
  rail together** (breadcrumb passed as a prop); the edit route's loading/error branches use a separate
  `BreadcrumbOnly` publisher (never mounted at the same time as the form). *Two `useShellChrome` in one
  subtree race — consolidate to one, or ensure they're mutually exclusive.* (carry to 3-4.)
- **cmdk combobox + multi-row strict-mode (the real E2E finding).** With ≥2 step cards each rendering an
  `AliasCombobox`, `getByRole('option',{name:id})` matched **2 elements** — multiple popovers' options
  coexisted in the DOM. Two-part fix: (1) **source** — lifted combobox open-state to the form so only one
  is open at a time (`openComboboxIdx`); (2) **page object** — scope to the just-opened popover
  (`[data-radix-popper-content-wrapper] [data-state="open"]`.last()), type the identity into the cmdk
  search to filter, then click the option. The single-target test passed throughout; only the multi-target
  specs caught it — **the live E2E earned its keep (RTL didn't surface it).**
- **Shared `RoutingChainPreview` needed a `disabledLabel` prop** — the 3-1 detail-rail RTL asserts
  `getByText('disabled')` on a disabled chain step; the form/prototype uses "skipped". A prop kept both
  callers' wording (and tests) intact while sharing the renderer.
- **Kept a shadcn `Switch` for the step enable toggle** (R2) so the E2E's `aria-checked` read survives;
  the Honor-Retry toggle is a styled checkbox (`role="switch"` + `aria-checked`) — RTL `.toBeChecked()`
  and the prototype look both satisfied.
- **No view transitions on the form** (R11) — arrow reorder is instant; avoided the jsdom
  `startViewTransition` console-error class. GATE B console was clean except the **one known pre-existing
  router-nav VT `InvalidStateError`** (`:0:0`, no app stack — techdebt #1), exactly as 3-1/3-2.
- **E2E port hygiene:** leftover multi_tenant/standalone/Vite servers from prior sessions block the
  Playwright `webServer` (it starts ALL configured servers regardless of `--project`). Sweep
  `ports kill 41135 51135 53000 5517x 5518x …` before a run. Also: this Playwright (1.57) parses a bare
  positional after `--project` as another **project name** — filter by `-g "<describe title>"`, not a
  path.

## E2E always-runs prerequisite (done up front)

Per the user + `feedback_no_skip_for_missing_env`: confirmed `specs/models/model-router.spec.mjs` is a
**required, always-running gate** — it matches `**/*.spec.mjs`, runs in the `standalone` project, has no
`.skip`/`@scheduled`, and each `beforeAll` **throws** if `INTEG_TEST_OPENAI_API_KEY` is missing (the key
is present in `.env.test`). Dropped the misleading "Requires … in .env.test" comment framing (reworded
to state it's a required gate). **Ran it green as the baseline BEFORE the rebuild** (3/3), then again
after (3/3).

## Gate results

- **Frontend RTL:** router suite **16/16** (`ModelRouterForm.test.tsx` 13 + `router/new/index.v2.test.tsx`
  3); full `npm test` **998 pass / 5 skip / 1 flaky** (the flaky is `login/index.test.tsx` State-B, a
  5s timeout under load — passes 21/21 in isolation; unrelated to this batch). Build clean; touched files
  prettier-clean.
- **E2E (targeted, standalone):** `model-router.spec.mjs` **3/3** (pass-through · in-request fallback ·
  health-aware skipping) against the rebuilt UI; `all-models-v2.spec.mjs` **2/2** (validates the shared
  `RoutingChainPreview` refactor via the fallback detail rail).
- **GATE B (live):** `make app.run.live` (no rebuild — no backend change). New Model Router form:
  breadcrumb · IDENTITY+Strategy · RESILIENCE · TARGETS; added a step → cmdk combobox with type/provider
  badges → selected an OPENAI API alias → alias-meta (`@ API Model` + `OPENAI`) + route-to-model dropdown
  auto-pinned `gpt-5-mini` + the **rail ROUTING CHAIN updated live**; **light + dark** both clean.
  **Console clean** except the one documented router-nav VT exception on entry.
- **Backend:** no Rust change; backend gate not required.

## Follow-ups

1. **Run the FULL E2E matrix** (both projects) at a convenient point — the shared `RoutingChainPreview`
   refactor touches `ModelDetailRail` (app-wide via My Models), though it's covered by all-models-v2.
   (Known pre-existing failures unchanged: chat-resize techdebt, MCP-OAuth, live-provider timeouts,
   browser-extension.)
2. **Models V1-list retirement remains the dedicated iteration** (the `models` flag + V1 `ModelsPageContent`
   + the V1-flow create/edit/delete E2E specs) — independent of these forms. 3-2 + 3-3 forms are done.
3. **Next: 3-4 (New Local Model form)** — the richest form (download/quant). Carry forward: the
   one-publisher-per-screen rule, the cmdk-multi-row E2E gotcha, the shared-component reuse instinct, and
   keep production submit-gating unless the user says otherwise. Kickoff: `batch-3-4-new-local-model-kickoff.md`.
