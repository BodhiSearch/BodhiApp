# Batch 0 — Foundation — Plan

> Status: **DRAFT for approval**. Grounded in a fresh read of the current code (2026-06).
> Loop ref: @process.md §"The per-batch loop" / §"Batch 0 — Foundation". Context: @context.md,
> @architecture.md, @reference-api.md, @screen-coverage.md.

## Goal of this batch

Lay the foundation every later batch depends on — **no screen is redesigned**. After Batch 0:
AppShell is the app's root layout (left sidebar replaces the top header **app-wide**), each
nav-section route renders its **existing** content inside the shell behind a default-old per-screen
flag, the brand tokens are refreshed, the backend surfaces the OAuth `id_token` + a configurable
`reference_api_url`, and all existing tests still pass.

> Decision (confirmed): the shell goes app-wide in Batch 0 — the one visible change this batch ships
> is top-header → left-sidebar chrome; screen interiors migrate in later batches.

## Confirmed current state (from fresh exploration — no drift)

- `crates/bodhi/src/routes/__root.tsx`: `ThemeProvider → ClientProviders → NavigationProvider →
  <div.flex.min-h-screen><AppHeader/><main><Outlet/></main><Toaster/></div>`. Single shared wrapper,
  no per-route layout.
- `components/navigation/AppHeader.tsx` hides on `pathname.startsWith('/setup/')`; renders
  `AppNavigation` + `AppBreadcrumb` + a GitHub button.
- `ThemeProvider` toggles `.dark`/`.light` on `<html>` (storage `bodhi-ui-theme`). Design dark
  selector is `.dark, [data-theme="dark"]` → **no ThemeProvider change needed**.
- `styles/globals.css`: `--primary: 250 95% 64%` (blue) today → flips to lotus-pink. Has prod-only
  `--header*` + full `--sidebar-*` to preserve.
- **No feature-flag pattern exists.** `hooks/useLocalStorage.ts` exists to reuse. Build the flag fresh.
- `lib/constants.ts`: trailing-slashed route constants (`ROUTE_CHAT='/chat/'`, `ROUTE_MCPS`, etc.);
  some sections (Models/Settings) have no top-level route today (dropdown only) — shell will give
  them stable section landing routes.
- Hooks: `useGetAppInfo()` → `AppInfo`; `useGetUser()` → `UserInfoEnvelope`. `lucide-react@^0.515.0`.
- `AppInitializer` is applied per-route (guards status/role); **stays outside `<Shell>`**.
- Backend: `exchange_auth_code` → `(AccessToken, RefreshToken)` via `EmptyExtraTokenFields`
  (**id_token discarded**); `session_keys.rs` has `access_token_key`/`refresh_token_key`, no id_token
  key; `AppInfo` + `UserInfo` structs as documented; settings recipe = constant + `ensure_default!`
  + metadata + `SETTING_VARS` (+ `EDIT_SETTINGS_ALLOWED` if editable).

## Prerequisites

None external (reference APIs are per-batch; Batch 0 only scaffolds the client). Batch 0's outputs
ARE the prerequisites for Batches 1–5.

## Work breakdown (ordered, upstream → downstream)

### Step 1 — Backend: surface OAuth `id_token` (services → routes_app)
- `crates/services/src/auth/auth_service.rs`: introduce an `IdTokenFields { id_token: Option<String> }`
  OIDC extra-fields type; parse the token response with it; extend `exchange_auth_code` to return the
  id_token (e.g. `(AccessToken, RefreshToken, Option<String>)`). Update the trait + the `MockAuthService`/test double + all callers.
- `crates/services/src/session_keys.rs`: add `id_token_key(client_id)` mirroring `access_token_key`.
- `crates/routes_app/src/auth/routes_auth.rs`: in `auth_callback`, persist the id_token to the
  session under the new key alongside access/refresh.
- **Tests:** extend `crates/routes_app/src/auth/test_login_callback.rs` (id_token persisted) and the
  auth_service unit tests (extra-field parsed). `cargo test -p services -p routes_app`.

### Step 2 — Backend: current-user returns `id_token`
- `crates/services` `UserInfo` (the struct behind `UserResponse::LoggedIn`): add `id_token: Option<String>`.
- `crates/routes_app/src/users/routes_users_info.rs`: read id_token from session and populate it on
  the `LoggedIn` arm. Keep `#[serde(skip_serializing_if = "Option::is_none")]`.
- **Tests:** `crates/routes_app/src/users/test_user_info.rs` — assert id_token present for a logged-in
  session, absent otherwise.

### Step 3 — Backend: `reference_api_url` setting + AppInfo
- `crates/services/src/settings/constants.rs`: add `BODHI_REFERENCE_API_URL` to `SETTING_VARS`
  (and `EDIT_SETTINGS_ALLOWED` if it should be UI-editable — propose **not** editable for now).
- `crates/services/src/settings/default_service.rs`: `ensure_default!(BODHI_REFERENCE_API_URL,
  Value::String("https://api.getbodhi.app/".into()))`; metadata `=> SettingMetadata::String`; import
  the constant. Add a `reference_api_url()` accessor mirroring `public_server_url()`.
- `crates/routes_app/src/setup/setup_api_schemas.rs`: add `reference_api_url: String` to
  `AppInfo`; populate in `routes_setup.rs` `setup_show` from the new accessor.
- **Tests:** `crates/routes_app/src/setup/test_setup.rs` (field present, default value) +
  settings service test for default + env override.

### Step 4 — Regen OpenAPI + ts-client
- `cargo run --package xtask openapi` → `make build.ts-client`. Verify `AppInfo` gains
  `reference_api_url` and `UserInfo` gains `id_token` in `ts-client`. Run
  `cargo test -p routes_app -- openapi` (sync check). Commit regenerated client.

### Step 5 — Frontend: port AppShell to TSX
- Create `crates/bodhi/src/components/shell/` from `design/bodhi-app-shell.jsx`:
  `AppShell.tsx`, `ShellContext.tsx` (`useShell`), `ShellNav.tsx`, `ShellIcon.tsx`,
  `ShellSearch.tsx`, `ShellModeSwitch.tsx`, `ShellFilterGroup.tsx`, `ShellChrome.tsx`
  (brand/footer/breadcrumb/tooltip/popover), `shell-nav-config.tsx`, `Shell.tsx`, `index.ts`.
- `ShellIcon` → `lucide-react` (kebab→PascalCase registry lookup; keeps prototype name strings).
- `ShellFooter` → real `useGetUser()` + the existing logout flow.
- `shell-nav-config.tsx` = NEW 5-section config (Chat/Models/MCP/API Keys/Settings as siblings),
  hrefs from `lib/constants.ts` (trailing-slash), nav via TanStack `<Link>`, highlight prop-driven.
- Port CSS by side-effect import: `shell.css` (← `bodhi-app-shell.css`, incl. `--c-*` helpers),
  `bodhi-form.css`, `api-keys-shell.css`, `model-access-picker.css`.
- **Strip on port:** `lucide.createIcons()`, `window.*`, `ReactDOM.createRoot`, `data-theme` setattr,
  TweaksPanel/EDITMODE.
- **Tests:** `components/shell/AppShell.test.tsx`, `ShellNav.test.tsx`, `ShellIcon.test.tsx` (renders
  nav, highlights active section/subPage, collapse→icon-rail, mobile drawer).

### Step 6 — Frontend: per-screen flag mechanism
- Build a small, greppable flag util (e.g. `lib/uiV2Flags.ts` + a `useUiV2Flag(screenId)` hook over
  `useLocalStorage`, optionally seeded by `import.meta.env`). Default **old** for every screen.
  Document the screen ids. (Reuse `useLocalStorage`; no new dep.)
- **Tests:** unit test the flag default + override.

### Step 7 — Frontend: AppShell as root + wrap existing screens
- `__root.tsx`: drop the `min-h-screen` flex + `<AppHeader/>`; render providers + `<Outlet/>` +
  `<Toaster/>`. (Keep `NavigationProvider` for now if `document.title` depends on it; port title into
  the shell/a small hook before its eventual deletion in Batch 5.)
- Each of the 5 nav-section route trees renders `<Shell section subPage>` around its **existing**
  content (default-old flag → old interior, new chrome). `contentClass="flush"` so old content is
  undisturbed. Correct `section`/`subPage` per route for nav highlight.
- **Adoption boundary:** `setup/*`, `login/`, `request-access/`, `auth/*`, `mcps/oauth/callback/`
  render **bare** (no `<Shell>`). `AppInitializer` stays outside `<Shell>`.
- **Tests:** existing route tests must still pass; extend the `@tanstack/react-router` test mock if
  `ShellNav` needs it (it shouldn't — highlight is prop-driven). Add a render-through-Shell helper.

### Step 8 — Token merge
- Replace the `:root`/`.dark` token bodies in `styles/globals.css` with `design/colors_and_type.css`
  bodies; **preserve** prod-only `--header*` + full `--sidebar-*` (still referenced until Batch 5).
  `tailwind.config.ts` unchanged. ⚠ `--primary` flips blue→lotus-pink **app-wide instantly** — do it
  deliberately; eyeball every screen once (esp. hardcoded white-on-`bg-primary`; new
  `--primary-foreground` is dark).

### Step 9 — Reference-API client scaffold (no real API)
- `crates/bodhi/src/lib/referenceApiClient.ts`: `fetch` wrapper, `new URL(path, baseUrl)`,
  `Authorization: Bearer <idToken>`, typed error.
- `crates/bodhi/src/hooks/reference/`: `useReferenceApi.ts` composing `useGetAppInfo()` (base URL) +
  `useGetUser()` (id_token) into a memoized, `enabled`-gated client; `constants.ts` for keys. No
  domain hook yet (the MCP batch adds the catalog hook).
- **Tests:** unit test the client builds the right URL + header; (external-origin MSW stub harness
  lands with the first consumer in the MCP batch — note it, don't build the handlers yet).

## Test / gate plan (all must pass before commit)
- Backend: `make test.backend` (or scoped `cargo test -p services -p routes_app` during dev, full
  before commit). OpenAPI sync: `cargo test -p routes_app -- openapi`.
- ts-client: `make build.ts-client` succeeds; committed.
- Frontend RTL: `cd crates/bodhi && npm run test` — new shell tests + **all existing tests green**
  (shell wrapper transparent to old testids).
- E2E: `make test.e2e` — existing specs green; update page objects only where the nav chrome moved
  (top-nav → shell nav). Black-box, no `page.evaluate`.
- Manual (`make app.run.live`): every section loads inside the shell with correct highlight;
  collapse-to-icon-rail, hover-resize, mobile drawer; light/dark; setup/login/auth render bare;
  `AppInitializer` redirects still fire.

## Exit criteria (from @process.md)
- Every nav-section route renders inside AppShell, correct `section`/`subPage`, old content behind a
  default-old flag. Light/dark works. Backend id_token + `reference_api_url` land with regen.
  **All existing RTL + e2e tests pass.** Adoption boundary respected.

## Risks (this batch) — see @process.md §"Carry-forward risks"
- Brand `--primary` flip is global+instant (eyeball pass). Strip-on-port discipline (grep each ported
  file). Don't transition `grid-template-columns`. Trailing-slash on any new section landing routes +
  regen `routeTree.gen.ts`. `NavigationProvider`/`document.title` coupling — keep until title is
  ported. id_token is sensitive: it's returned to the authenticated user's own session only (no new
  exposure surface) — confirm it's not logged.

## Resolved decisions (user-approved, 2026-06)
1. **Section landing routes** — shell nav points **Models → `/models/`** and **Settings → `/settings/`**
   (existing list pages; no new routes).
2. **`reference_api_url` editability** — **non-editable** (env/default only; NOT in
   `EDIT_SETTINGS_ALLOWED`).
3. **Flag source** — **localStorage dev toggle** via `useLocalStorage`, default-old. No env seed for now.
4. **Commit granularity** — **single commit** for the whole Batch 0.
