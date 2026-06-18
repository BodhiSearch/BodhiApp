# UI V2 Migration — Glossary

Shared vocabulary for this task.

- **V2 / the redesign** — the new design for all 14 app screens, captured as a hi-fidelity prototype
  in `design/`. See [design-reference.md](@design-reference.md).
- **AppShell** — the canonical V2 layout: a CSS-grid shell of sidebar / main / optional rail with a
  shared header gridline. Source: `design/bodhi-app-shell.jsx` (+ `.css`). Ported into production TSX
  as the app's root layout.
- **Section** — one of the 5 top-level nav groups: Chat, Models, MCP, API Keys, Settings. Each
  section is the unit of a **batch**.
- **subPage** — a screen within a section (e.g. App Tokens / New Token / Access Requests inside the
  API Keys section). AppShell highlights nav by `section` + `subPage`, declared per route.
- **Rail** — the optional **third column** (right detail panel) some screens use (e.g. MCP Discover
  detail, models detail). Opens as an overlay drawer on mobile.
- **Icon rail / collapse** — the sidebar can collapse to a ~60px icon-only rail (ephemeral, resets
  on reload). Distinct from the **rail** (third column) above.
- **Batch** — one nav section migrated end-to-end as a single iteration of the per-batch loop
  ([process.md](@process.md)). `Batch 0` is the foundation (no screen redesigned).
- **Per-screen flag** — the toggle that renders the old vs new version of a screen at the **same
  route** during coexistence. Default-old; flipped per screen as it's accepted; **removed** (with the
  old code) at batch end.
- **Reference API** — the external service `https://api.getbodhi.app/` that serves data the backend
  doesn't own (first: the MCP Discover catalog). Called **directly from the frontend**. See
  [reference-api.md](@reference-api.md).
- **id_token** — the user's OAuth identity token, sent to the reference API as `Bearer` for per-user
  rate-limiting/analytics. Currently discarded by the backend at login; Batch 0 surfaces it.
- **reference_api_endpoint** — the configurable reference-API base URL (a `SettingService` setting,
  env-overridable), surfaced to the frontend so dev/prod/test can point at different origins.
- **Prototype** — the static `design/` site (Babel-in-browser). **Visual reference only** — we build
  to production conventions, not by copying prototype code.
- **MSW external-origin stub** — MSW handlers registered against the reference API's (mock) absolute
  origin, used to test reference-API screens without the real service.
- **Strip-on-port** — the rule that prototype-only idioms (global `lucide`, `window.*`,
  `ReactDOM.createRoot`, `data-theme` setattr, TweaksPanel) are removed when porting a screen to
  production TSX.
- **testid discipline** — preserving `data-testid` + ARIA roles verbatim across the markup
  restructure so RTL queries and Playwright page objects keep working.
- **Working folder** — this directory (`docs/claude-plans/202606/screen-v2/`); the self-contained
  home for all task context, per-batch plans, retros, kick-offs, and reference-API specs.
