# Batch "Setup" — Setup Wizard — Retrospective

Status: implementation complete; RTL green (999 pass / 5 pre-existing skips, +0 net new failures);
setup E2E green; GATE B (live) passed for steps 3–6 (steps 1–2 unreachable on an already-registered
instance — user verifies from a fresh DB). **Detour batch** off the Models 3-x sequence. **Shipped
V2-only — NO flag.**

## Design-alignment follow-up pass (post-GATE-B, user-reviewed against design)

After the user compared implemented pages to the prototypes (dark/light), a UI/UX alignment pass landed:
- **App-wide download progress bar** — flat `bg-primary` fill → higher-contrast **lotus→amber gradient**
  on a muted track. Shared `.download-progress-track`/`.download-progress-fill` in `globals.css`, used by
  the setup `ModelCard` AND `models/files/pull` (the bar was hand-rolled identically in both).
- **App-wide API-model form** — orange "Required" badge → **red `*`** after the label (new shared
  `RequiredMark` in a new `form/FormSection.tsx`, swapped into Name/BaseUrl/ApiFormat). Added labeled
  **sections** (Provider Connection / Request Routing / Model Selection) with dividers via `FormSection`,
  removed the redundant setup-mode card header (it duplicated the wizard heading), bumped card contrast.
  Same shared `ApiModelForm` so `/models/api/new` + `/edit` inherit it. (E2E page-object `welcomeTitle`
  followed: "Setup API Models" → "Set up API Models".)
- **Model cards (step 3)** denser: Quality/Speed meters left-grouped (killed the empty middle), wide
  column `max-w-5xl` → `max-w-4xl`.
- **Dark card contrast** — `border-border` (18% L) → `border-strong` (32% L) on `SetupCard` + the model
  cards so cards separate from the near-black wash like the design.
- **Setup-complete spacing** — even 24px between stacked cards via a `.setup-card + .setup-card` rule
  (design's `.su-card + .su-card`).
- Verified live (light + dark): API form, model cards, gradient progress bar (seen mid-download),
  complete spacing all match. RTL 187/187 (api+setup+files), setup E2E green. Responsive uses standard
  Tailwind `grid-cols-1 md:grid-cols-2` (the MCP window-resize couldn't shrink the CSS viewport below
  ~1280 here, so responsive is breakpoint-verified, not pixel-walked).

## The reframe — a restyle, not a build (again)

Like Batch 3-2, exploration overturned the framing. The setup wizard was **already fully built and
wired**: 6 routes under `routes/setup/`, all hooks (`useSetupApp`, `useOAuthInitiate`,
`usePullModel`/`useListDownloads`/catalog, `useBrowserDetection`/`useExtensionDetection`), the
`AppInitializer` redirect/gating, the 6-step constants (labels already matched the design verbatim),
an existing E2E spec set + RTL tests, and **step 4 already embedded the production
`ApiModelForm mode="setup"`**. So the batch was **presentation-only**: restyle the chrome + 6 step
bodies to the design, reuse everything else.

## What landed

**Shared chrome (`routes/setup/-components/`), rebuilt to the design, names/exports/testids kept:**
- `SetupContainer` — soft-wash page, centered column with a `wide` prop (step 3), breathing lotus,
  fixed theme toggle, stagger motion. `SetupProgress` — design stepper (34px nodes, connectors,
  done=✓/current-ring/pending, labels), **all testids + `data-status`/`data-completed`/`data-current`
  + the visually-hidden `progress-bar` ARIA element preserved**. `SetupCard`/`SetupFooter` restyled.
- New small pieces: `SetupThemeToggle` (drives `ThemeProvider`, reads `.dark` via MutationObserver),
  `SetupCardIcon` (centered icon tile, steps 2 & 5). `BenefitCard`/`WelcomeCard` restyled (kept
  required text + `benefit-card-*` testids + "NEW").
- One tiny `setup-wizard.css` for the **only** things awkward in Tailwind: wash gradient, breathing
  lotus, breathing halo keyframes — all reduced-motion-gated. **No `.su-*` CSS dump** (user decision).

**6 step bodies** restyled in place; hooks + testids untouched. Step 4 keeps
`<ApiModelForm mode="setup"/>` verbatim — chrome only (per Batch 3-2 carry-forward). `ModelCard`
redesigned to the recommendation card (spec chips, Quality/Speed meters, rec-highlight, Download↔state
button); dropped the benchmark rows + Specialty meter (design) — 3 RTL display tests updated to match.

**Layout: a third class — fullscreen.** The wizard is its own full-screen chrome, so it must NOT sit
inside `BareLayout`'s slim topbar (that produced a double header + double theme toggle live). Added
`isFullscreenRoute('/setup')` to `resolveShellRoute` and a fullscreen branch in `__root` that renders
the `Outlet` directly. (Login/auth/request-access still use BareLayout.)

## Surprises / fixes during GATE B

1. **Double header.** Setup rendered inside `BareLayout` (slim topbar) AND added its own lotus/toggle.
   Fix: the fullscreen layout class above. Carry-forward: the **interim** layout switches in
   `resolveShellRoute` now have THREE classes (shell / bare / fullscreen) — strengthens the case for
   the deferred route-declared layout seam (techdebt.md).
2. **Page stuck faded.** The framer entrance variants animated **opacity 0→1**; the route-level
   `defaultViewTransition` root cross-fade captured the new snapshot mid-fade and the aborted
   transition left the page at reduced opacity. Fix: made `containerVariants`/`itemVariants`
   **transform-only** (translateY), resting fully visible — exactly the design's own "capture-safe"
   note. This also helps reduced-motion/E2E. **Reusable lesson: never animate opacity from 0 on a
   route that participates in the root VT cross-fade.**
3. **`InvalidStateError: Transition was aborted`** still logs once per navigation — but it's the
   **documented, screen-agnostic router-internal** issue (techdebt.md "Other deferred items";
   reproduces identically on shipped Models/Settings). Functionally harmless; not this batch's
   regression; not fixed here (cross-cutting `main.tsx` config).
4. **No scrolling on any setup page** (user-reported after GATE B). The fullscreen branch renders
   `<Outlet/>` directly, but global `shell.css` sets `html, body { height: 100%; overflow: hidden }`
   (AppShell's internal-scroll model), so a normal-flow `min-h-screen` container clips everything past
   the viewport with no scroll. Fix: `.setup-wash` is now `position: fixed; inset: 0; overflow-y: auto`
   (mirrors `.bare-page`/`.bare-main`), and dropped the conflicting `min-h-screen` from the JSX.
   **Reusable lesson: any fullscreen route rendered outside BareLayout must own its own scroll
   container — the global `overflow: hidden` on body is the trap.**

## testid discipline (the linchpin, again)

Every page object + RTL query kept working by preserving testids through the markup rewrite. The few
that changed were **intentional copy/casing** (e.g. button "Setup Bodhi Server →" → "Set up Bodhi
Server" + arrow icon ⇒ switched the welcome page object + RTL to the stable `setup-submit` testid;
"Extension Ready" → "Extension Connected"; "Setup Complete!" → "Setup Complete"). Added
`data-testid="model-card"` to `ModelCard` and pointed `SetupDownloadModelsPage.downloadModel` at it
(replacing a fragile `.locator('..').locator('..')` DOM walk).

## Divergences from the design (folded as decisions)

- **No serif / Devanagari fonts.** The prototype loaded Cormorant Garamond + Tiro Devanagari from
  Google Fonts CDN; CSP forbids external resources (self-hosted fonts only). Hero + बोधि accent render
  in the app's Inter `sans`. Acceptable; no new font infra. (If we want the serif hero later, self-host
  the font first.)
- Feature-card icons stay **emoji** (the catalog data uses emoji); the design used lucide. Kept the
  data contract; restyled the tile only.

## For the next batch

- The fullscreen layout class is now available for any future full-screen flow.
- Steps 1–2 (welcome + resource-admin) were not live-validated (the dev instance is already
  registered/admin'd) — verify from a fresh `setup`-status DB.
