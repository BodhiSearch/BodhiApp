# Batch 3-3 — New / Edit Model Router (form) — Plan

> Working doc for the screen-v2 migration, sub-phase **3-3**. Reads with
> `screen-v2/batch-3-3-new-fallback-model-kickoff.md`, the **3-2 retro** (form-as-page reframe) and the
> **3-1 retro** (full-stack recipe, colorful treatment, design re-walk). On approval this content is
> copied to `screen-v2/batch-3-3-new-fallback-model-plan.md`.
>
> **⚠ Design updated mid-plan (HEAD `c6d1ee93` "Create Fallback Model.html design update", 2026-06-19
> 14:44).** Re-walked the served prototype per the 3-1 "re-read after a design ping" rule. The update
> **simplified the divergences and renamed the screen to "Model Router"** — folded in below. Net effect:
> the prototype now matches production's section set almost exactly (the heavy-divergence framing in
> the kickoff is now stale in a *good* way).

## Context — why this change, and what it actually is

The screen-v2 migration ports every screen to the left-sidebar **AppShell**. 3-1 landed the **My Models**
list, 3-2 landed the **New/Edit API Model** form (V2-only, chrome-only). 3-3 ports the **New / Edit Model
Router** form (the fallback alias) at `/models/router/new/` + `/models/router/edit/`.

**3-3 is heavier than 3-2 — a form-body rebuild, not chrome-only.** The (now-updated) prototype
(`design/create-fallback-model-app.jsx` + `.css`) is genuinely **richer** than production's plain
`ModelRouterForm`: step cards with a step-number badge + position labels (primary / final fallback /
disabled), a searchable **alias combobox** with type + provider badges, a per-step **route-to-model**
field (constrained dropdown vs free-text), **"↓ on error" connectors**, and a **published shell rail**
("Routing & help": live Routing Chain + How-it-works + Tips). GATE A (re-walked live on `:8000`)
confirmed all of these.

**The updated prototype now matches production's section set:** IDENTITY (**Name + the disabled Strategy
= Fallback select**) · **RESILIENCE** (Cooldown / Max attempts / Honor Retry-After) · TARGETS — i.e. the
resilience + strategy fields the *old* prototype omitted are now present, exactly mirroring production.
So the only real delta vs production is **richer presentation + the rail**, not a different field set.

**Decision (user, at plan time): full form-body rebuild + a published info rail, V2-only, no flag.** We
rebuild the form body to match the prototype, but keep the **real data layer and production
submit-gating unchanged**. The prototype is the visual target; production's `AliasResponse` data and
`useCreateModelRouter`/`useUpdateModelRouter` mutations are the substance. **Zero backend change.**

## GATE A — interactive prototype walk (done; re-walked after the design update)

Re-walked the served `Create Fallback Model.html` (HEAD `c6d1ee93`) — the form column + the full rail.
Captured behaviors:
- **Title "New Model Router"**; breadcrumb `Bodhi › Models › New Model Router`; desc "Chain targets into
  a priority order and route requests through them." Single centered form column; help in the **shell
  rail**.
- **IDENTITY**: **Name** input (now only strips whitespace — *no* lowercase-force / regex; hint
  "Becomes the `model` value clients send… No spaces") + **Strategy** = disabled "Fallback" select.
- **RESILIENCE**: Cooldown (number, default 30) + Max attempts per request (number, default 0) + a
  **Honor upstream Retry-After** toggle (`role="switch"`, default on) with description.
- **TARGETS (in priority order)**: step cards = header (step-num circle, "Step N" + position note,
  enable toggle, ▲/▼/✕ icon-buttons — **no drag handle anymore**) · "MODEL ALIAS" + **AliasCombobox**
  (searchable; options show **type badge** + name + **provider badge**) · alias-meta line (type badge +
  provider + `→ backing` / `· size`) · conditional **ROUTE TO MODEL** for API aliases (selected →
  `<select>` + "pre-configured only" pill; forward-all → free-text + "any model · forward-all" pill).
  `↓ ON ERROR` **connector** between cards; dashed **Add step** button below.
- **FOOTER** (inside card): **Cancel** + **Create Model Router** (button disabled until the prototype's
  `validationMsg` clears — but we keep **production** gating; see §"Status messaging"). *The inline
  footer status message was removed in the update — status now lives in the rail only.*
- **RAIL** (header **"Routing & help"** with a close ✕ via `collapseRail`, `railDefaultOpen: true`,
  width ~320): an optional **warn validation badge** + **ROUTING CHAIN** (live, numbered, type badges,
  `↓ on error`, `skipped` tags, `→ model` / `→ model required`) + **HOW IT WORKS** (now **5** items —
  adds a cooldown/Retry-After line, the copy flips on the honor toggle) + **TIPS** (now **5** — adds
  cooldown + max-attempts tips).
- **Drag-reorder is gone** (removed in the update); **arrow ▲/▼ reorder** is the only reorder.

## Decisions locked with the user (this session)

1. **Full prototype rebuild of the form body** — rebuild the step cards / combobox / route-to-model /
   connectors / footer to match the prototype, using **production primitives + a scoped `cfm-*`
   stylesheet**. Keep **arrow-based reorder** (▲/▼); **no drag-and-drop** (the design update **removed**
   the drag handle entirely — arrow reorder is the design's intended interaction).
2. **All real fields are present in BOTH the prototype and production now** — resilience
   (`cooldown_secs` / `max_attempts` / `honor_retry_after`) + the disabled **Strategy = Fallback** select
   render as their own sections (IDENTITY-Strategy + RESILIENCE), matching production 1:1. *(The earlier
   "prototype omits resilience" divergence is resolved by the design update — no judgment call needed.)*
3. **Keep PRODUCTION validation/submit-gating only** — do **NOT** adopt the prototype's stricter
   client-side *blocking* validation (no name-regex gate, no "≥2 enabled steps" gate, no per-step
   required gate). Create stays disabled **only** on `submitting || resilienceInvalid` (exactly as
   today). The rail status badge is **display-only derived UX** (see §"Status messaging").
4. **Shell published rail** — publish via `useShellChrome({ breadcrumb, rail, railHeader,
   railDefaultOpen: true })`, memoized on form state. **Not** an in-content sticky column. Default-open
   because it's a live preview, not a click-to-open detail.
5. **No flag — ship V2-only, always-on** (mirrors 3-2). Remove `new-fallback-model` from
   `lib/uiV2Flags.ts`. The chrome + rail are additive over the same `router/new|edit` routes reusing the
   real mutations; no V1 fallback needed.
6. **Full test migration** — rewrite RTL for the new markup; **rewrite the E2E page object**
   (`ModelRouterFormPage.mjs`) for the new combobox/route-to-model/toggle markup; run the live
   `model-router.spec.mjs` as a GATE.
7. **E2E always-runs prerequisite** — `model-router.spec.mjs` must always run (not be treated as
   optional) and **fail loud** if a key is missing. (It already `throw`s in `beforeAll` — no `.skip`,
   matches `feedback_no_skip_for_missing_env`. We confirm + run it green as the **baseline before**
   starting the rebuild, and drop the misleading "Requires … in .env.test" framing.)

## Real-data map (prototype field → real `AliasResponse` field → action)

The prototype's alias model (`type` / `provider` / `fwdMode` / `backing` / `size`) is mocked. Drive the
rebuild off the **real** union (`AliasResponse = ModelRouterResponse | UserAliasResponse |
ModelAliasResponse | ApiAliasResponse`), reusing the guards in `lib/utils.ts`:

| Prototype | Real source | Action |
|---|---|---|
| `type: 'api-model'` | `isApiAlias` (`source==='api'`, `ApiAliasResponse`) | indigo `at-sign` "API Model" badge |
| `type: 'model-alias'` | `isUserAlias` (`source==='user'`, `UserAliasResponse`) | lotus `tag` "Model Alias" badge |
| `type: 'local-file'` | `source==='model'` (`ModelAliasResponse`) | saffron `hard-drive` "Local File" badge |
| `provider: 'OPENAI'…` | `ApiAliasResponse.api_format` (uppercased) | leaf provider badge (reuse 3-1's treatment) |
| `fwdMode: 'selected'\|'all'` | `ApiAliasResponse.forward_all_with_prefix` (`true`=forward-all) | drives route-to-model variant |
| `models: [...]` | `apiMatchableModels(api)` (existing helper) | dropdown options for selected mode |
| `backing: 'repo:file'` | `repo` / `filename` on local/user/model aliases | `→ repo` meta text |
| `size: '18.5 GB'` | `size?: number\|null` on User/ModelAliasResponse (added in 3-1) | `· <formatSize>` meta (omit if null) |
| free provider strings (OPENROUTER, …) | only the **6 real `ApiFormat`s** | use real `api_format` only |
| `ALL_KNOWN_MODELS` autocomplete pool | **nothing** — no global model catalog for forward-all | **drop the suggestion pool**; forward-all stays a plain free-text `Input` with a `${prefix}model-name` placeholder (matches production + the E2E `fillTargetModel`). Shipping a fake list would violate real-data-only. |
| `weight` | `RouterTarget.weight` is a **reserved SEAM** (ignored by Fallback) | **no UI** (matches production) |

Targets resolve by **identity** (existing logic): API alias by `id`, local by `alias`; the referenceable
list is `useListModels(1,100,'name','asc')` filtered to `source !== 'model_router'`. **No backend
change**; **no OpenAPI/ts-client regen** (GATE B needs no binary rebuild — HMR is enough).

Two display-sugar fields the prototype shows but production can't supply per-target are **dropped
gracefully**: `backing` becomes the real `repo` (no synthetic field), and `size` renders only when
`size != null` (reuse `formatSize` from `ModelDetailRail`; never on API aliases). That the rest maps
cleanly through the existing `isApiAlias` / `aliasIdentity` / `apiMatchableModels` helpers confirms the
production data model already supports this UI.

## Status messaging — the validation split (display-only, NOT blocking — decision 3)

⚠ **Highest-risk part of the batch.** The prototype's entire footer + rail UX is driven by one
`validationMsg` memo that **includes the three gates we've banned** (name-regex, ≥2-enabled-steps,
per-step-required). Copying that memo into the disabled-prop would silently re-introduce blocking
validation and break the live E2E (which creates a router with a single enabled target and a forward-all
primary whose model is plain text). **Separate the two concepts the prototype conflates:**

1. **Submit gating — stays *exactly* production:** `disabled={submitting || resilienceInvalid}` and
   **nothing else**. Name, target count, and per-step model presence do **not** gate Create.
2. **Display-only derived status** — a tiny pure helper `deriveRailStatus(state) → { tone, message }`
   computed only for the **rail** badge (the design update removed the inline *footer* status; status
   lives in the rail now), **never** fed to `disabled`:
   - `resilienceInvalid` (the only real blocker) → **warn**: "Fix the resilience settings to continue."
   - soft **info**-tone advisories (no targets / only 1 enabled / a step missing an alias or model) may
     render in the rail as coaching, visually distinct from `warn`, and **must not** change the button.
   - otherwise the rail just shows the live chain (no badge), matching the updated prototype which only
     renders the warn badge when `validationMsg` is set.

Per-step `invalid` affordance (red border on an empty selected/forward-all model, "select a model"
hint) is rendered **display-only** — it matches production, which already lets you submit a target with
an empty model. Lock this contract with explicit RTL tests (T9 + T13) and a code-review check that
`router-submit`'s `disabled` reads `submitting || resilienceInvalid` and nothing else.

## Component decomposition (new `-components/` pieces)

Rebuild **in place** at `routes/models/router/-components/`, decomposing the 373-line monolith into
co-located sub-components (TanStack ignores `-components/`). All keep plain React state + the real
hooks/helpers; **no react-hook-form** (consistent with today + avoids a data-layer rewrite).

- `ModelRouterForm.tsx` — orchestrator: state (`alias`, `targets`, resilience knobs), the real
  `useListModels` + `aliasByIdentity`, `useCreate/UpdateModelRouter`, `handleSubmit`. Renders the
  centered container + the card (IDENTITY · RESILIENCE · FALLBACK SEQUENCE · footer). **Publishes the
  rail** via `useShellChrome` (memoized on `{alias, targets}`). Keeps `data-testid="model-router-form"`.
- `StepCard.tsx` — one target card (header controls + AliasCombobox + meta + RouteToModelField). Keeps
  `target-row-${idx}`, `target-alias-${idx}`, `target-model-${idx}`, `target-enabled-${idx}`,
  `target-up/down/remove-${idx}` verbatim.
- `AliasCombobox.tsx` — searchable alias picker built on **shadcn `Command` (cmdk) inside `Popover`**
  (the standard shadcn Combobox recipe). cmdk gives real `role="listbox"`/`role="option"` +
  `aria-selected` + keyboard nav + fuzzy search for free — satisfies `web-design-guidelines` **and**
  keeps the E2E `getByRole('option', { name })` working. Set each `CommandItem`'s `value` to a
  searchable `${display} ${type} ${provider}` (matches the prototype's search fields) while rendering
  badges + name as children. **Pin the option's accessible name to the alias *identity*** (`id` for API)
  so the existing page-object selection survives with zero spec churn (R3). Trigger keeps
  `target-alias-${idx}`. Do **not** hand-roll the prototype's div-combobox (no a11y, breaks E2E).
- `RouteToModelField.tsx` — the API-only field: shadcn `Select` (selected mode, keeps `getByRole('option')`)
  or free-text `Input` (forward-all) — keeps `target-model-${idx}`; local aliases render the disabled
  pinned-model input. Do **not** convert these to cmdk (they map 1:1 to production + the E2E methods).
- `AliasTypeBadge.tsx` (TypeBadge + ProviderBadge) — one shared presentational module off the real
  guards / `api_format`, used by `StepCard`, the combobox options, **and** the rail chain so all three
  agree.
- `StepConnector.tsx` — the `↓ on error` divider.
- **`RoutingChainPreview.tsx` (shared reuse win):** extract the chain rendering so the **My-Models detail
  rail (`FallbackRailBody`)** and this form's rail render the chain identically. `ModelDetailRail`
  already renders an almost-identical fallback chain (`m-chain-*`); the form rail adds the
  `→ model required` / `skipped` states — gate those behind props. Refactor `FallbackRailBody` to consume
  it too (per `feedback_generic_evolvable_design`: factor the shared primitive, don't duplicate).
- `RouterInfoRail.tsx` + `RouterRailHeader.tsx` — the published rail. Header = **"Routing & help"** with
  a close ✕ (the design's `CfmRailHeader`; map `collapseRail` → our rail close, via `useShellChrome`'s
  rail slot / the shell's collapse path — confirm at impl). Body = optional warn status badge (from
  `deriveRailStatus`) + `RoutingChainPreview` + **How-it-works (5 items**, the cooldown/Retry-After line
  flips on the honor toggle**)** + **Tips (5 items)**. Built on `dp-*` primitives
  (`.dp-panel`/`.dp-section`/`.dp-sec-lbl`/`.dp-foot`) from `components/shell/list.css`.
- `router-form.css` — scoped `cfm-*` styles ported from `design/create-fallback-model.css`, rooted under
  a `.router-form` (or `m-*`-style) wrapper so it can't leak. Semantic color tokens
  (`--c-lotus/saffron/leaf/indigo/teal-*`) **already exist** in `globals.css` (Batch 0 + 3-1) — reuse,
  don't re-add.

Routes (`router/new/index.tsx`, `router/edit/index.tsx`) mirror **3-2's** template: `AppInitializer`
stays outside; edit keeps `useGetModelRouter` + Loading/Error/NotFound. Breadcrumbs: `Bodhi › Models ›
New Model Router` (new) / `Edit Model Router` (edit). **Publish the breadcrumb at the route level,
*before* the edit-route guards** (in `EditModelRouterContent`), so the shell header stays stable through
Loading/Error — not only once the form mounts (R9). The **rail** is published from inside the form (it
needs live form state). Add page testid `router-form-page` (mirrors `new-api-model-page`) + a
`data-pagestatus` so the route-level v2 tests get a deterministic ready-wait (R10).

**Support changes the rebuild needs** (call them out explicitly):
- **Add `mockUpdateModelRouter`** to `test-utils/msw-v2/handlers/model-routers.ts` — only
  `mockCreateModelRouter` + `mockGetModelRouter` exist today; the edit-submit RTL (T16) needs a PUT
  handler (R6).
- **Nav-label rename (per the design update):** `shell-nav-config.tsx` line 33 currently reads
  `{ id: 'new-fallback-model', label: 'New Fallback Alias', icon: 'route', href: '/models/router/new/' }`
  — rename **label → "New Model Router"**. **Keep the nav `id` `new-fallback-model`** (it's the
  `shell-sub-{subPage}` testid + matches the design's `subPage`; renaming it would churn nav testids).
  This nav entry is **not** flag-gated, so no un-gating is needed.
- **Flag-removal sweep (R13):** remove `'new-fallback-model'` from the `lib/uiV2Flags.ts` `UiV2Screen`
  union (verified: the flag is referenced **only** in the union + the nav `id` — never consumed via
  `useUiV2Flag`, since the screen was never built behind it) and update `uiV2Flags.test.ts` if it
  asserts the flag set. Grepped all references (`feedback_plan_verification`): just those two spots.

## data-testid scheme

**Preserve verbatim** (E2E + RTL depend on them): `model-router-form`, `router-alias-input`,
`router-strategy-select`, `resilience-settings`, `cooldown-secs-input`, `cooldown-secs-error`,
`max-attempts-input`, `max-attempts-error`, `honor-retry-after-switch`, `add-target`, `no-targets`,
`target-row-${idx}`, `target-alias-${idx}`, `target-model-${idx}`, `target-enabled-${idx}`,
`target-up-${idx}`, `target-down-${idx}`, `target-remove-${idx}`, `router-cancel`, `router-submit`.

**New** (additive): `router-form-page` (route container, mirrors `new-api-model-page`), `router-rail`
(rail body root), `router-rail-status` (status badge — assert tone + message), `router-rail-chain` (live
chain container — distinct from the list rail's `model-detail-chain`), `target-alias-meta-${idx}` (the
type/provider/repo/size line — lets RTL assert the data-map), and optional `step-num-${idx}` /
`step-connector-${idx}` for ordering assertions. **No testids on cmdk option rows** —
`getByRole('option', { name })` covers them (that's why cmdk was chosen over the bespoke combobox). The
enable toggle stays a shadcn `Switch` so `aria-checked` survives for the E2E (R2).

## RTL — `routes/models/router/-components/ModelRouterForm.test.tsx` (rewritten) + route tests

**Keep all 6 existing behaviors** (re-mapped to new markup): create-with-one-local-target,
add/remove-target empty-state toggle, resilience defaults, edited-resilience-in-request-body,
invalid-cooldown-disables-submit, prefill-resilience-on-edit. **Add:**
- combobox selection pins the right model (local→alias name; API-selected→first matchable;
  API-forward-all→free-text editable) + `target-alias-meta-${idx}` shows the right type/provider badge;
- route-to-model **dropdown vs free-text** variant rendering; enable toggle; reorder ▲/▼;
- **rail routing-chain reflects the steps** (numbered, in order, `skipped` tag when a step is disabled);
  `router-rail-status` warn badge shows only when `resilienceInvalid` (otherwise no badge, per the
  updated design);
- **contract tests for decision 3 (the critical guard):** a forward-all target with an **empty model**
  leaves Create **enabled** (T9); a **0-target / empty-name** router leaves Create **enabled** — only
  invalid resilience disables it (T13). These lock the no-blocking-validation contract against the
  prototype's gates.

**Route-level V2 tests** (new `router/new/index.v2.test.tsx` + `router/edit/index.v2.test.tsx`, via the
`ShellSlotsProvider` + `SlotsConsumer` harness from `routes/models/index.v2.test.tsx`): `router/new`
publishes breadcrumb `Bodhi / Models / New Model Router` + `router-form-page` + a `router-rail`;
`router/edit` mirror with `Edit Model Router` + prefilled targets, **+ assert the breadcrumb publishes
during the Loading state and the no-id / error ErrorPage guards still render** (don't lose the edit
route's three states). Edit-submit (T16) uses the **new `mockUpdateModelRouter`** handler. Run the
**full** `cd crates/bodhi && npm test`; expect prior count + the new tests, 0 failures, typecheck +
touched-files lint clean.

## E2E — rewrite the page object, run the live spec

- **Prerequisite (GATE A):** confirm `specs/models/model-router.spec.mjs` always runs (it matches
  `**/*.spec.mjs`, standalone project, no `.skip`, `beforeAll` throws if `INTEG_TEST_OPENAI_API_KEY`
  missing — key **is** present in `.env.test`). **Run it green first** to establish the baseline, and
  drop the "Requires … in .env.test" comment framing so it reads as a required gate, not optional.
- **Rewrite `pages/ModelRouterFormPage.mjs`** for the new markup: `selectTargetAlias` drives the combobox
  (open trigger `target-alias-${idx}` → type in search → click the option by accessible name) instead of
  `getByRole('option')` on a native select; `setTargetEnabled` reads the new toggle state; keep
  `fillName`/`addTarget`/`fillTargetModel`/`selectTargetModel`/`submit`/`navigateToNew` working by the
  preserved testids.
- Re-run all 3 suites (pass-through / in-request fallback / health-aware skipping) green. **Black-box
  only** (UI interactions, no `page.evaluate`). Also run the broader standalone matrix at commit
  (shared shell/nav are app-wide; per 3-1 follow-up).

## GATE B — live validation (HARD)

`make app.run.live`, log in. **No binary rebuild needed** (no backend change). Go to
`/ui/models/router/new/`:
- Breadcrumb `Bodhi › Models › New Model Router`; centered form; **rail open** by default ("Routing &
  help") with the live ROUTING CHAIN / HOW IT WORKS / TIPS; the rail **close ✕** collapses it.
- Add steps; pick a **local** alias (pinned model, read-only), an **API-selected** alias (route-to-model
  dropdown, "pre-configured only"), an **API-forward-all** alias (free-text route-to-model); reorder
  ▲/▼; toggle a step off → it shows `skipped` in the chain.
- RESILIENCE: edit cooldown/max-attempts/honor-retry-after (the How-it-works copy flips on the honor
  toggle); invalid cooldown shows the warn badge + disables Create.
- **Create** → toast + redirect to `/ui/models/`. **Edit** (`/ui/models/router/edit/?id=<id>`):
  breadcrumb `Edit Model Router`, prefilled targets + resilience; Save → success.
- **Light + dark + responsive** (≥414px, sidebar→drawer, mobile rail-drawer); **console clean** on load
  + each interaction (only allowed exception = the known router-nav VT `InvalidStateError` on entry —
  techdebt #1). Verify `useViewTransition` is in-page-only (no route-nav VT misuse).

## Docs to update in this batch

- `screen-v2/tracker.md` — flip 3-3 to ✅ (V2-only, no flag); remove `new-fallback-model` from the flags
  table + the "No flag (shipped V2-only)" note (now lists 3-2 **and** 3-3); update the screen-by-screen
  row 4 (**rename "New Fallback Alias" → "New Model Router"** per the design update; the route stays
  `/models/router/new|edit/`).
- `screen-v2/batch-3-3-new-fallback-model-plan.md` — copy of this plan on approval.
- `screen-v2/batch-3-3-new-fallback-model-retro.md` — write at the end (rebuild scope, the
  display-only-status decision, the data-mapping, E2E page-object rewrite, gates).
- `screen-v2/batch-3-4-new-local-model-kickoff.md` — re-confirm/append the 3-3 learning (form rebuild +
  published rail recipe) so 3-4 re-enters the loop correctly.

## Commit

Trunk-based, directly on `main` (rebase onto `origin/main` first). Stage **only** the touched files
(routes + `-components/` + `router-form.css` + RTL + the E2E page object/spec + the doc files) — **never
`git add -A`** (`design/` prototype files are user-owned working-tree changes and must not be in the
commit). All gate checks (`make format`, `npm test`, the model-router E2E + standalone matrix) green
**before** commit.

## Risks / watch-outs

- **Rail thrash from per-keystroke state (R1).** The rail closes over `alias` + `targets` +
  `resilienceInvalid`, all of which change on every keystroke, so it re-publishes constantly. Acceptable
  (the slots value-context is isolated from publishers), but `useMemo` the rail node on the **minimal
  derived inputs** (a stringified chain summary + the `{tone,message}` status), **not** the raw `targets`
  array identity, so it doesn't re-publish when nothing visible changed. Keep `railHeader` +
  `railDefaultOpen` stable (module-scope / primitive) so only `rail` republishes.
- **Enable toggle must stay a shadcn `Switch` (R2).** The prototype's toggle is a bare
  `<input type=checkbox>`; the E2E reads `aria-checked`, which only Radix `Switch` (`role="switch"`)
  exposes. Style the `Switch` to look like the prototype toggle — don't swap in the bare checkbox.
- **No view transitions on the form (R11).** `useViewTransition` is in-page-only and `document.startViewTransition`
  doesn't exist in jsdom (RTL) — using it for add/remove/reorder risks console-error test flake. Arrow
  reorder is instant; the prototype has no DnD/VT. Skip view transitions here entirely.
- **Combobox accessible-name pinned to identity (R3).** Render the alias `name` as text but keep the
  cmdk option's accessible name = the **identity** (`id` for API) so `getByRole('option', { name: id })`
  in the existing spec keeps working with zero call-site churn.
- **Container width with the rail open (R7).** `railDefaultOpen: true` eats horizontal space; tune the
  form column at GATE B so `max-w-3xl` doesn't feel cramped with the rail open (My Models uses
  `railDefaultOpen: false`). Consider open-by-default only above a breakpoint. Visual-QA, not a blocker.
- **Memoize the breadcrumb** (`useMemo`) — pass stable nodes to `useShellChrome`.
- **Keep production submit-gating** (decision 3): the only thing that disables Create is
  `submitting || resilienceInvalid`. Do not let the richer status UX silently start blocking saves — RTL
  must assert a 0-target / empty-name router can still be submitted (server authority).
- **Combobox accessibility + E2E**: build on shadcn `Command`/`Popover` for real option semantics +
  keyboard nav; verify the E2E page object can select an option by its accessible name. A fully bespoke
  div-combobox would break both `getByRole('option')` and screen-reader/keyboard use.
- **Edit route breadcrumb through loading** (3-2 carry-forward): publish the breadcrumb even in the
  Loading/Error states so the shell header isn't empty mid-load.
- **`useShellChrome` can't carry `contentClass`/`mainScroll`** — the prototype's `mainScroll={false}`
  sticky footer isn't reproduced (acceptable, matches 3-2). Don't edit `__root`/`AppShell`.
- **Container width with the rail open** — tune the form column width at GATE B so it stays a calm
  centered column when the rail is open and when it's collapsed.
- **Scoped CSS** — root `router-form.css` under a wrapper class; reuse existing tokens; don't restyle
  shared shell/list CSS. After touching the rail, re-verify a sample of other rail screens (My Models,
  Settings) for regressions (shared-file rule).
- **Not a master-detail list** → no `LinkRow` (the recipe's LinkRow rule is for selectable list rows).
- **No backend/regen** → GATE B works on the existing binary (HMR); don't trigger the binary-rebuild
  step (that's only for backend-changing batches).
