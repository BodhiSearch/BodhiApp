# Batch 3-2 — New API Model (form) — Plan

> Working doc for the screen-v2 migration, sub-phase 3-2. Reads with
> `docs/claude-plans/202606/screen-v2/batch-3-2-new-api-model-kickoff.md` (the kick-off) and the
> 3-1 retro. **Approved decisions are folded in** (see "Decisions locked with the user").
> On approval this content is copied to `screen-v2/batch-3-2-new-api-model-plan.md`.

> **⚠ IMPLEMENTED WITHOUT A FLAG (change after approval).** The plan below describes a flag-gated
> (`new-api-model`) chrome. During implementation the user decided the chrome was thin enough to ship
> **V2-only, always-on, with NO flag** — `useUiV2Flag`/the `if(!v2)` branch were removed from both
> routes and `new-api-model` was removed from `lib/uiV2Flags.ts`. Wherever this plan says "flag-gated"
> / "flag on/off", read "always-on". The RTL collapsed to one "publishes breadcrumb + container" test
> per route (no `localStorage`). Everything else (reuse the prod form, keep all real fields, no backend
> change, reuse existing E2E) is unchanged. See `screen-v2/batch-3-2-new-api-model-retro.md` for the
> final state.

## Context — why this change, and what it actually is

The screen-v2 migration ports every screen to the left-sidebar **AppShell**. Batch 3-1 landed the
**My Models** list (behind the default-off `models` flag). 3-2 is the **first form sub-phase**:
the **New / Edit API Model** form at `/models/api/new/` + `/models/api/edit/`.

**The key realization from exploration (and what reframed the scope):** the API-model form is
**already a production, shadcn-styled, fully-wired form-as-page**. `crates/bodhi/src/components/api-models/ApiModelForm.tsx`
already accepts `mode` (`create | edit | setup`) + `initialData`, handles every real field
(Name, the 6 real API formats, Base URL, API-Key-with-toggle, Extras, the LLM-Liberty envelope
swap, Prefix, Forward-mode, Model-selection, Test-Connection), wires the real
`useCreateApiModel` / `useUpdateApiModel` / `useGetApiModel` mutations + the `convert*` helpers, and
navigates on success. The current routes are already thin wrappers:

```tsx
// routes/models/api/new/index.tsx (today, 16 lines)
<AppInitializer allowedStatus="ready" authenticated={true}>
  <ApiModelForm mode="create" />
</AppInitializer>
```

Because `__root` **already** wraps all app routes in `<AppShell contentClass="flush" {...slots}>`,
this form **already renders inside the V2 shell today**. The screenshots confirm the only gap vs the
prototype is **chrome**:

1. **No breadcrumb** — the shell header band is empty (no `useShellChrome`); the prototype shows
   `Bodhi › Models › New API Model`.
2. **Full-width form** — today the form uses `mx-4 my-6` and stretches the whole main column; the
   prototype centers it in a calm fixed-width column.

So **this is a migration, not a rebuild**. There is **no separate "V1 form" artifact to delete** —
unlike Batch-1's dialog→page (which deleted an old dialog), the create/edit form here was never
duplicated. The V2 delta is: **add a breadcrumb + a centered container to the SAME routes, behind the
`new-api-model` flag, reusing `ApiModelForm` unchanged.** The intended outcome: the API-model
create/edit screen gets V2 chrome consistent with the rest of the shell, with **zero change** to the
form, its data layer, its edit/delete behavior, or the shared setup wizard.

The hi-fidelity prototype (`design/Create API Model.html`) is **indicative only, not constraining**
(user directive). We implement the V2 UX **using what production already uses** (shadcn `Card` + the
existing field components) — we do **not** replace production styling with the prototype's `bf-*`
system, and we do **not** restyle the shared form.

## GATE A — interactive prototype walk (done)

Walked `Create API Model.html` live on `:8000` (light + dark, scrolled, toggled model rows). Captured:
- Renders inside AppShell; breadcrumb `Bodhi › Models › New API Model`; a centered calm column on a
  muted backdrop. **No detail rail, no faceted sidebar** (it's a plain form page).
- Sections: **PROVIDER CONNECTION** (API Format / Base URL / API-Key + "Use API key" toggle + reveal),
  **REQUEST ROUTING** (Model-Prefix toggle→input, Forwarding-Mode radio cards), **MODEL SELECTION**
  (Selected chips + Clear-All, Available Models + Fetch / Select-All / search / checkbox list).
  Footer: **Test Connection** · **Cancel** · **Create API Model**.
- Dark mode + disabled-until-valid primary button render correctly in the prototype.

**Prototype-vs-production data discrepancies (surfaced + resolved with the user — keep all prod fields,
production is the superset, prototype omissions/samples ignored):**

| Aspect | Prototype (indicative) | Production (real) → action |
|---|---|---|
| API Format list | 5 samples incl. **Cohere / Ollama** (don't exist) | 6 real: `openai`, `openai_responses`, `anthropic`, `anthropic_oauth`, `gemini`, `llm_liberty_oauth` → **keep real list** |
| **Name** field | absent | present, **required** → **keep** |
| **Extras** (`extra_headers`/`extra_body`) | absent | present (shown for presets w/ defaults, e.g. `anthropic_oauth`) → **keep** |
| **LLM Liberty** envelope | absent | present (swaps base_url/key/extras when `llm_liberty_oauth`) → **keep** |

> Per `feedback_generic_evolvable_design` + the kickoff's "don't drop real fields the prototype omits":
> real-data-only cuts both ways. Production wins; nothing is dropped.

## Decisions locked with the user

1. **Reuse the production form, add V2 chrome only.** Keep the existing shadcn `ApiModelForm` (and all
   its sub-components) **unchanged**; do **not** port the prototype's `bf-*` CSS, do **not** rebuild a
   `bf-*` variant, do **not** restyle the shared form (the setup wizard shares it — out of scope).
2. **Keep the card title; no above-card page-head.** The form's existing `CardHeader`
   (CardTitle + CardDescription) stays as the only heading. **Zero form-component change.**
3. **Keep all production fields** (Name, 6 real formats, Extras, Liberty) — discrepancy table above.
4. **Flag-gate the chrome only.** One form, same routes. `useUiV2Flag('new-api-model')` gates only the
   additive V2 chrome (breadcrumb + centered container). ON → V2 chrome; OFF → today's layout.
   Edit/delete features untouched. **No routes deleted.**
5. **This is a migration, not new-feature dev — no new E2E, no Liberty E2E.** Update the route RTL for
   the chrome; reuse existing form/component RTL + existing `api-models/*` E2E specs as-is (they drive
   the same form via the same, preserved testids).
6. **Cross-Models flag-retirement is its own future iteration** (NOT this task). The `new-api-model`
   flag (and `models`, and the upcoming `new-fallback-model`/`new-local-model`) should be retired
   together when the Models section flips — captured as a **new techdebt.md item** + a **tracker.md
   flags table**.

## Scope — what changes

**Two route files get V2 chrome (flag-gated); the form component does not change.**

### 1. `routes/models/api/new/index.tsx` — add V2 chrome behind the flag
Wrap the existing `<ApiModelForm mode="create" />` so that when `useUiV2Flag('new-api-model')` is on:
- publish the breadcrumb via `useShellChrome`,
- render the form inside a centered container (Tailwind, production primitives — **no new CSS file**).

```tsx
import { useMemo } from 'react';
import { createFileRoute } from '@tanstack/react-router';
import ApiModelForm from '@/components/api-models/ApiModelForm';
import AppInitializer from '@/components/AppInitializer';
import { useShellChrome } from '@/components/shell';
import { useUiV2Flag } from '@/hooks/useUiV2Flag';

export const Route = createFileRoute('/models/api/new/')({ component: NewApiModel });

const NEW_API_MODEL_BREADCRUMB = [
  { label: 'Bodhi' },
  { label: 'Models', href: '/models/' },
  { label: 'New API Model', current: true },
];

function NewApiModelContent() {
  const [v2] = useUiV2Flag('new-api-model');
  // useShellChrome is a no-op publisher when v2 is off (empty breadcrumb)…
  useShellChrome({ breadcrumb: useMemo(() => (v2 ? NEW_API_MODEL_BREADCRUMB : undefined), [v2]) });

  if (!v2) return <ApiModelForm mode="create" />; // today's full-width layout
  return (
    <div className="container mx-auto max-w-3xl px-4 py-6" data-testid="new-api-model-page">
      <ApiModelForm mode="create" />
    </div>
  );
}

function NewApiModel() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true}>
      <NewApiModelContent />
    </AppInitializer>
  );
}
export default NewApiModel;
```

> Container width: `max-w-3xl` (~768px) approximates the prototype's calm column using a plain Tailwind
> utility — **no `bf-*` import**, consistent with the tokens/new form-as-page reference
> (`routes/tokens/new/index.tsx`, which used `container mx-auto max-w-2xl p-6`). Final width tuned at
> GATE B against the running app.

### 2. `routes/models/api/edit/index.tsx` — same chrome, same flag
Mirror the pattern: keep `useGetApiModel` + Loading/Error states; when the flag is on, publish a
breadcrumb (label `Edit API Model`, `href '/models/'` for Models) and wrap `<ApiModelForm mode="edit" …>`
in the same centered container. `api_format` stays read-only on edit (already enforced by
`ApiFormatSelector disabled={isEditMode}` + the server's `ApiFormatImmutableOnEdit`) — **no change**.

### What does NOT change (reused verbatim)
- `components/api-models/ApiModelForm.tsx` + every `form/` sub-component + `actions/` + `hooks/`.
- `schemas/apiModel.ts` (`createApiModelSchema`/`updateApiModelSchema`, `convert*`, `API_FORMAT_PRESETS`).
- All data-layer hooks (`useCreateApiModel`/`useUpdateApiModel`/`useGetApiModel`/`useTestApiModel`/
  `useFetchApiModels`/`useListApiFormats`) and every `data-testid` + ARIA role.
- The **setup wizard** (`/setup/api-models/`, `mode="setup"`) — untouched.
- **No backend change** (the create/update contract is unchanged from 3-1; confirmed). No
  OpenAPI/ts-client regen. (So GATE B does **not** require a binary rebuild — HMR is enough.)

## RTL — full form (the user asked to see this)

The existing route tests (`routes/models/api/new/index.test.tsx`,
`routes/models/api/edit/index.test.tsx`) render the page directly with `createWrapper()` and mock
`@tanstack/react-router` (`useNavigate`, `useSearch`) + `@/hooks/use-toast`. They already exercise
the full create/edit flow against MSW (`mockApiFormatsDefault`, `mockFetchApiModelsSuccess`,
`mockCreateApiModelSuccess`, `mockGetApiModel`, `mockUpdateApiModel`, …) by the preserved testids.

**They keep working unchanged** even after we add `useShellChrome`, because with no `ShellSlotsProvider`
the publish setter is the default no-op (`ShellSlotsContext` line 37). We add **one V2 structural test
per route** that wraps in the canonical `ShellSlotsProvider` + `SlotsConsumer` harness (the exact
pattern from `routes/models/index.v2.test.tsx`) and asserts the breadcrumb + container publish **with
the flag on**. The flag is `localStorage`-backed (`useUiV2Flag` → `useLocalStorage`); the test sets it
before render.

```tsx
// added to routes/models/api/new/index.test.tsx
import { ShellSlotsProvider, useShellSlots } from '@/components/shell';

function SlotsConsumer() {
  const { breadcrumb } = useShellSlots();
  const crumbs = Array.isArray(breadcrumb) ? breadcrumb.map((b) => b.label).join(' / ') : '';
  return <div data-testid="harness-breadcrumb">{crumbs}</div>;
}

function renderV2() {
  localStorage.setItem('bodhi.ui-v2.new-api-model', 'true'); // flag ON
  return render(
    <ShellSlotsProvider>
      <SlotsConsumer />
      <NewApiModel />
    </ShellSlotsProvider>,
    { wrapper: createWrapper() }
  );
}

describe('New API Model — V2 chrome (flag on)', () => {
  beforeEach(() => localStorage.clear());

  it('publishes the Models breadcrumb and renders the centered page container', async () => {
    server.use(
      ...mockAppInfoReady(),
      ...mockUserLoggedIn({ role: 'resource_user' }),
      ...mockApiFormatsDefault(),
      ...mockTestApiModelSuccess(),
      ...mockFetchApiModelsSuccess(),
      ...mockCreateApiModelSuccess()
    );

    renderV2();

    await waitFor(() => {
      expect(screen.getByTestId('create-api-model-form')).toBeInTheDocument();
    });
    // breadcrumb published to the shell
    expect(screen.getByTestId('harness-breadcrumb')).toHaveTextContent('Bodhi / Models / New API Model');
    // V2 page container present (full-width path renders the form without it)
    expect(screen.getByTestId('new-api-model-page')).toBeInTheDocument();
  });

  it('with the flag OFF publishes no breadcrumb and no V2 container', async () => {
    server.use(
      ...mockAppInfoReady(),
      ...mockUserLoggedIn({ role: 'resource_user' }),
      ...mockApiFormatsDefault()
    );
    render(
      <ShellSlotsProvider>
        <SlotsConsumer />
        <NewApiModel />
      </ShellSlotsProvider>,
      { wrapper: createWrapper() }
    );
    await waitFor(() => expect(screen.getByTestId('create-api-model-form')).toBeInTheDocument());
    expect(screen.getByTestId('harness-breadcrumb')).toHaveTextContent('');
    expect(screen.queryByTestId('new-api-model-page')).not.toBeInTheDocument();
  });
});
```

**Edit route** gets the mirror V2 test: set the flag, seed `mockGetApiModel('test-model', …)`, assert
`harness-breadcrumb` = `Bodhi / Models / Edit API Model`, `edit-api-model-page` present, and that the
`api-format-selector` is disabled (lock-on-edit) + its `-locked-hint` shows. All **existing**
edit-flow tests (prefill, update success, update error) stay as-is.

**RTL inventory for this batch:**
- `routes/models/api/new/index.test.tsx` — keep all existing; **+2** V2-chrome tests (flag on/off).
- `routes/models/api/edit/index.test.tsx` — keep all existing; **+2** V2-chrome tests (flag on/off + lock-on-edit assertion).
- `components/api-models/ApiModelForm*.test.tsx` (core + extras + llm_liberty) — **unchanged** (form not touched).
- Run the **full** `cd crates/bodhi && npm test` suite; expect prior count + 4, 0 failures, typecheck + lint clean on touched files.

## E2E — reuse, don't add

The existing Playwright specs already drive the create/edit form by the **preserved** testids via
`pages/ApiModelFormPage.mjs` + `pages/components/ApiModelFormComponent.mjs`:
`specs/api-models/api-models-extras.spec.mjs`, `…-prefix`, `…-forward-all`, `…-no-key`,
`api-llm-liberty-*`, `api-gemini-embeddings`, plus `specs/setup/setup-api-models.spec.mjs`.

- **No new specs. No Liberty E2E** (none exists for our flow today and we are not adding feature tests).
- The chrome change is **additive and behind a default-off flag**, so the existing specs (which run the
  default/flag-off path, navigating directly to `/ui/models/api/new/`) are **unaffected**. Confirm by
  running the `api-models/*` + `setup/setup-api-models` specs from `crates/lib_bodhiserver/tests-js`
  (`make test.e2e`) — they must pass unchanged before commit (per `feedback_run_all_gate_checks`).
- If GATE B is run with the flag on, navigation + container don't alter any testid, so the page objects
  keep resolving. (No page-object edits needed; if a future sub-phase turns the chrome always-on, page
  objects still match.)

## GATE B — live validation (HARD)

`make app.run.live`, log in. **No binary rebuild needed** (no backend change). Set the flag:
`localStorage.setItem('bodhi.ui-v2.new-api-model','true')`, reload.
- **Create** (`/ui/models/api/new/`): breadcrumb `Bodhi › Models › New API Model` shows; form centered
  in the calm column; OpenAI happy-path — fill Name, pick a format (watch base_url swap), toggle Use-API-key,
  Fetch Models, Test Connection, select a model, Create → toast + redirect to `/ui/models/`.
- **Edit** (`/ui/models/api/edit/?id=<id>`): breadcrumb `Edit API Model`; **`api_format` locked**
  (disabled + hint); change models / prefix → Update → success. Delete stays via the My-Models rail
  (3-1) — unchanged.
- Switch a format to `anthropic_oauth` → **Extras** appear; to `llm_liberty_oauth` → the **envelope**
  replaces base_url/key/extras (field-swap correct).
- **Light + dark + responsive** (≥414px, the mobile sidebar drawer); **console clean** on load + each
  interaction (the only allowed exception is the known router-nav VT `InvalidStateError` on route
  entry — techdebt #1, not from this screen).

## Docs to update in this batch

- **`screen-v2/tracker.md`** — flip 3-2 to 🟩 done-behind-flag; **add a flags table** (per user):
  every active `useUiV2Flag` id, what it gates, default state, and when it's planned to retire. e.g.

  | Flag | Gates | Default | Retire when |
  |---|---|---|---|
  | `models` | My Models V2 list | off | Models section flips (with forms) |
  | `new-api-model` | New/Edit API Model V2 chrome | off | Models section flips |
  | `new-fallback-model` | (3-3, not yet built) | off | Models section flips |
  | `new-local-model` | (3-4, not yet built) | off | Models section flips |
  | `chat`, `mcp-discover`, `new-mcp`, `mcp-playground` | (later batches) | off | their batch |

- **`screen-v2/techdebt.md`** — add: *"Models-section flag retirement is its own iteration. The
  `models` + `new-api-model` (+ `new-fallback-model`/`new-local-model` as they land) flags gate
  additive V2 chrome over the SAME routes; they should be flipped on (chrome always-on) + the flag
  branches removed together when the Models section is accepted — NOT per sub-phase, to keep a
  consistent V1↔V2 Models flow during the sub-phases. Tracked separately from this batch."*
- **`screen-v2/batch-3-2-new-api-model-retro.md`** — write at the end (what landed, the
  migration-not-rebuild reframe, the discrepancy resolution, gates).
- **`screen-v2/batch-3-3-new-fallback-model-kickoff.md`** — already exists; re-confirm/append the 3-2
  learning (form sub-phases are chrome-only when production already owns the form) so 3-3 re-enters the
  loop correctly.

## Commit

Trunk-based, directly on `main` (rebase onto `origin/main` first). Stage **only** the touched files
(2 route files + 2 route test files + the 3 doc files) — **never `git add -A`** (the `design/`
prototype files are user-owned working-tree changes and must not be in the commit). All gate checks
(`make format`, `npm test`, `make test.e2e` for the api-models/setup specs) green **before** commit.

## Risks / watch-outs

- **`useShellChrome` can't carry `contentClass`/`mainScroll`** (the slots seam only has
  breadcrumb/headerActions/sidebar/rail/railHeader/railDefaultOpen). The root already passes
  `contentClass="flush"`; `mainScroll` stays `true` (main column scrolls, footer scrolls with it). The
  prototype's `mainScroll={false}` sticky-footer nicety is **not reproduced** (acceptable — matches
  tokens/new). Do **not** edit `__root`/`AppShell` for it.
- **Don't restyle the shared `ApiModelForm`** — setup-wizard regression risk; out of scope.
- **Memoize the breadcrumb node** (`useMemo` on the flag) — pass stable nodes to `useShellChrome`
  (`rerender-no-inline-components`); the breadcrumb is a module-level const gated by the flag.
- **Edit route isn't in `SHELL_NAV`** → `resolveShellRoute` highlights `models` (section) with no
  subPage on `/models/api/edit/`. That's fine (the breadcrumb still reads "Edit API Model"). No change.
- **Preserve every `data-testid`** — the E2E page objects + RTL utils depend on them verbatim.
```
