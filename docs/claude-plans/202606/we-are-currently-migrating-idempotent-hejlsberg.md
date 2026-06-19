# Screen V2 — Setup Wizard Migration (Batch "Setup")

## Context

The Screen-V2 migration is restyling every nav section to the new design (`design/*.jsx` + `*.css`,
served on `:8000`). The team paused the Models batch (3-x) to take a **detour onto the Setup
wizard** — the 6-step onboarding flow a fresh install walks through before reaching the app.

The Setup wizard was explicitly **deferred / out of scope** in the original roadmap
(`screen-v2/screen-coverage.md` line 7, `tracker.md` line 96) because it renders **bare** (outside the
three-column `AppShell`) — the V2 playbook keeps `setup/login/request-access/auth` bare so the
app-initializer redirects keep firing (`process.md` "Shell adoption boundary"). This batch migrates it.

**What this is:** a **presentation-only restyle**. Exploration confirmed the wizard is already fully
built and wired — 6 routes under `src/routes/setup/`, all hooks (`useSetupApp`, `useOAuthInitiate`,
`usePullModel`/`useListDownloads`, model catalog, `useBrowserDetection`/`useExtensionDetection`),
all redirect/gating logic in `AppInitializer`, the 6-step constants, and an existing E2E spec +
RTL tests. **Nothing about data flow, routing, or gating changes.** Only the wizard chrome
(container, stepper, cards, footer) and each step's visual body are rebuilt to match the new design.

**Intended outcome:** the live setup flow (`make app.run.live`, fresh DB) looks like the
`setup-1..6` prototypes — calm centered column, soft-wash background, breathing lotus, top stepper
with done/current/pending states, restyled feature cards / admin checklist / model recommendation
cards / extension-detection card / completion community rows — while every existing testid, redirect,
and E2E spec still passes. Ships **V2-only, no flag** (a one-time linear flow; clean wholesale swap).

### Decisions locked with the user

1. **Chrome = idiomatic React/Tailwind/tokens, NOT a verbatim CSS dump.** Do **not** copy
   `design/setup-flow.css` (815 lines of `.su-*`) into the app. Reproduce the look with Tailwind
   utilities against the existing design tokens in `globals.css` (`--primary`, `--accent`,
   `--surface-*`, `--shadow-*`, `--ease-lotus`, `--dur-*`) and the shared `ui/` primitives, the same
   way Settings/Users V2 were done. Maintainability over pixel-copy.
2. **One batch, V2-only, no flag.** Migrate all 6 steps together; no `bodhi.ui-v2.setup` flag, no
   dual path. Follow the per-batch loop (GATE A explore → plan → implement → GATE B live → retro).
3. **Step 4 reuses `ApiModelForm` as-is.** `setup-4` keeps `<ApiModelForm mode="setup" …/>`
   verbatim (already wired). Only the surrounding wizard chrome/heading/footer is restyled. **Do NOT
   port the prototype's `bf-*`/`cam-*` form CSS** — Batch 3-2's retro established this and the form is
   shared with `/models/api/new` + `/edit`. (The model-selection box was already redesigned to the
   `.cam-*` look in Batch 3-2 via `components/model-selector.css`, so setup inherits it for free.)

## Existing code — reuse, do not rebuild

| Concern | File(s) | Action |
|---|---|---|
| Routes (6) + layout | `routes/setup/route.tsx`, `routes/setup/index.tsx`, `resource-admin/index.tsx`, `download-models/index.tsx`, `api-models/index.tsx`, `browser-extension/index.tsx`, `complete/index.tsx` | Keep route defs + `AppInitializer` wrappers + all hook wiring; restyle the **content** only |
| Step constants | `routes/setup/-shared/constants.ts` (`SETUP_STEPS`, `SETUP_STEP_LABELS`, `SETUP_TOTAL_STEPS=6`) | Reuse verbatim — already matches the design's 6 labels |
| Animation presets | `routes/setup/-shared/types.ts` (`containerVariants`, `itemVariants`) | Reuse / tune to design's stagger + "rise" feel |
| Step context | `routes/setup/-components/SetupProvider.tsx` (`useSetupContext`) | Reuse — drives current step from pathname |
| All hooks | `hooks/info` (`useSetupApp`), `hooks/auth` (`useOAuthInitiate`), `hooks/models` (`useListDownloads`/`usePullModel`/catalog), `hooks/use-browser-detection`, `hooks/use-extension-detection` | Reuse verbatim |
| API-model form | `components/api-models/ApiModelForm.tsx` (`mode="setup"`) + all `form/`/`hooks/` | Reuse verbatim (decision 3) |
| Redirect/gating | `components/AppInitializer.tsx` | **Untouched** |
| Tests | `routes/setup/**/index.test.tsx`, `lib_bodhiserver/tests-js/specs/setup/*.spec.mjs` + page objects | Restructure markup but **keep every `data-testid`/ARIA** so these pass |

## Design behaviors to capture (from live walk-through on :8000)

- **Shell** (all steps): soft radial-wash background; centered column (~680px; **wide ~1040px on
  step 3 model grid**); breathing lotus logo; fixed top-right theme toggle (note: app theme is owned
  by `ThemeProvider` — wire the toggle to it, **strip** the prototype's `bodhi-theme.js`/`data-theme`
  scaffolding); top **stepper** with connectors, numbered/`✓` nodes, `is-done`/`is-current`/pending
  states + labels (the existing `SetupProgress` already emits the right testids — restyle it).
- **Step 1 Get Started:** hero (Sanskrit बोधि accent), 6 feature cards (3 with "New" badge), server
  setup card (Server Name *required min-10*, optional Description), full-width primary CTA.
- **Step 2 Login & Setup:** centered shield icon, "Admin Setup" copy, "As an admin you can" checklist
  well, primary "Continue with Login" → `useOAuthInitiate`.
- **Step 3 Local Models:** **wide** layout; Chat + Embedding sections; recommendation cards (tag
  badge, spec chips Size/Params/Context/Quant, Quality/Speed star meters, Download↔Queued button wired
  to `usePullModel` + `useListDownloads` polling); info note; Back / Continue nav.
- **Step 4 API Models:** heading + subhead, embedded `<ApiModelForm mode="setup"/>`, "no API key?"
  info note, Back / Continue(skip) nav.
- **Step 5 Extension:** puzzle icon, browser selector ("Detected" pill via `useBrowserDetection`),
  install link, Not-Found↔Connected callout with "Check Again" (`useExtensionDetection`), Back / Skip
  / Continue nav.
- **Step 6 All Done:** "Setup Complete" hero, community rows (GitHub/Discord/X/YouTube), quick
  resources, full-width "Start Using Bodhi App" → `/chat/`.

## Implementation

Work the shared chrome first, then the 6 step bodies. Presentation-only; strip every prototype idiom
on each port (`lucide.createIcons()`, `window.*`, `ReactDOM.createRoot`, `data-theme` setattr,
`Tweaks*`) per `process.md` recipe step 5 — use `lucide-react` imports + `ThemeProvider`.

### Phase 1 — Shared wizard chrome (`routes/setup/-components/`)

Rebuild these to the new design using Tailwind + tokens (keep names/exports so step files keep
importing them; keep testids):
- **`SetupContainer.tsx`** — soft-wash page background, centered column with a **`wide?` prop**
  (step 3 uses wide), breathing lotus logo, fixed theme toggle wired to `ThemeProvider`'s
  `setTheme`/`toggle`, renders `<SetupProgress>` + children with the stagger/rise motion.
- **`SetupProgress.tsx`** — restyle to the design's stepper (connectors, done=`✓`, current ring,
  pending muted; labels under nodes). **Preserve** `data-testid="setup-progress"`,
  `step-indicator-{n}`, `step-label-{n}`, `step-counter`, `progress-bar`, and the
  `data-status`/`data-completed`/`data-current` attributes (tests + page objects read them).
- **`SetupCard.tsx` / `SetupFooter.tsx`** — restyle card frame (head/title/sub/well) and the
  footer nav row (Back / spacer / Skip / primary Continue) to the design; keep prop shapes + testids.
- Small presentational pieces as needed: a `FeatureCard` (step 1; reuse/replace `BenefitCard`),
  `AdminChecklist` (step 2), `ModelRecommendationCard` + `StarMeter` (step 3),
  `ExtensionStatusCard` (step 5; restyle existing `components/setup/BrowserExtensionCard.tsx`),
  `CommunityRow` (step 6). Prefer composing `ui/card`, `ui/button`, `ui/badge`, `ui/input`,
  `ui/textarea`, `ui/checkbox` over bespoke markup.

### Phase 2 — Step bodies (one route file at a time)

For each, swap the body markup to the restyled components, keep the hook wiring + testids:
- `routes/setup/index.tsx` — hero + `FeatureCard` grid + server-setup form (`useSetupApp`).
- `routes/setup/resource-admin/index.tsx` — admin card + checklist + `useOAuthInitiate` CTA.
- `routes/setup/download-models/index.tsx` (+ `-components/ModelCard.tsx`) — wide layout, two
  sections, recommendation cards wired to `usePullModel`/`useListDownloads`/catalog.
- `routes/setup/api-models/index.tsx` — restyle chrome only; **`<ApiModelForm mode="setup"/>` stays**.
- `routes/setup/browser-extension/index.tsx` (+ `components/setup/BrowserExtensionCard.tsx`) —
  restyle detection card; keep `useBrowserDetection`/`useExtensionDetection`.
- `routes/setup/complete/index.tsx` — completion hero + community/resource rows.

### Phase 3 — Theme toggle wiring

The prototype has a fixed top-right toggle on every setup page. Wire it to the app's
`ThemeProvider` (`src/components/ThemeProvider` / `next-themes`-style) `setTheme`. **Do not** add
`design/bodhi-theme.js` or any `data-theme` setattr — `ThemeProvider` owns the `.dark` class.
Confirm both light and dark render at GATE B.

## Tests

Presentation-only ⇒ contracts unchanged; reuse fixtures/MSW handlers/page objects as-is.
- **RTL** (`cd crates/bodhi && npm run test`): update the 8 setup component tests
  (`routes/setup/**/index.test.tsx`, `-components/SetupProgress.test.tsx`,
  `download-models/-components/ModelCard.test.tsx`) **only where markup queries break** — keep every
  `data-testid`/role assertion working by preserving testids. No behavior assertions should change.
- **E2E** (`make test.e2e` from `crates/lib_bodhiserver/tests-js`): the 4 setup specs
  (`setup-flow`, `setup-api-models`, `setup-browser-extension`,
  `setup-browser-extension-with-extension-installed`) must pass **black-box** (UI only). Update page
  objects (`SetupWelcomePage`/`SetupResourceAdminPage`/`SetupDownloadModelsPage`/`SetupApiModelsPage`/
  `SetupBrowserExtensionPage`/`SetupCompletePage`) only for markup/selector drift — keep them on
  testids. Per memory: **no `test.skip` for missing env; throw in `beforeAll`**; black-box only.
- Run the full gate (RTL + E2E; no Rust/regen here) **before** commit (`feedback_run_all_gate_checks`).

## Verification (GATE B — live, in Claude-in-Chrome)

RTL/E2E are necessary but not sufficient (Batch 1 shipped an `Illegal invocation` while all tests
passed). Validate the real flow on a **fresh DB**:
1. `make app.run.live` (dev-server + Vite HMR — no ui-rebuild). Reset to a `setup`-status DB so the
   app-initializer routes to `/ui/setup/`.
2. Walk all 6 steps in Chrome: server-name form submits → step 3; model Download→Queued toggles +
   polling; step 4 `ApiModelForm` renders + Continue/skip; extension "Check Again"; step 6 → `/chat/`.
3. For each step confirm: **light AND dark** render; **responsive** (narrow viewport — feature grid +
   model grid collapse, stepper labels hide on small screens via the existing `sm:` rule); the
   app-initializer redirect still fires (bare, no AppShell); and `read_console_messages` shows
   **0 errors/exceptions** on load and each interaction.
4. Capture a GIF of the full walk-through for the retro.

## Batch artifacts (per `process.md`)

- Write `docs/claude-plans/202606/screen-v2/batch-setup-kickoff.md` (exploratory carry-forward) and,
  after approval, the canonical `batch-setup-plan.md` here in the screen-v2 folder.
- On completion: `batch-setup-retro.md` (what landed, divergences, surprises) + update
  `tracker.md`/`screen-coverage.md` to move Setup from "deferred" to ✅, and remove the
  "out of scope" note.
- Commit per batch, trunk-based on `main` (rebase onto `origin/main` first); V2-only, no flag to
  retire; delete any old chrome pieces fully replaced.

## Risks / watch-outs

- **Bare boundary:** setup must NOT be wrapped in `AppShell`; verify app-initializer + setup-status
  redirect tests still pass.
- **Theme ownership:** strip prototype `data-theme`/`bodhi-theme.js`; toggle drives `ThemeProvider`.
- **Testid preservation** is the linchpin — restructure markup freely but keep testids/ARIA verbatim.
- **Trailing slashes** (`/ui` basepath, trailing-slash-always) on any nav hrefs; no new routes needed.
- **Reduced motion:** the breathing/stagger/rise motion should respect `prefers-reduced-motion`
  (per memory, V2 view-transition/motion races) — gate framer animations on it for E2E stability.
- **Step 3 wide layout** is the only non-default column width — drive it via the `wide` prop, not a
  one-off style.
