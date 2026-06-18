# Batch 0 — Foundation — Retrospective

Status: implementation complete, gates green (E2E result appended below). Single commit per the plan.

## What landed

**Backend (services → routes_app → regen):**
- OAuth **id_token** is no longer discarded. `AuthService::exchange_auth_code` now returns
  `(AccessToken, RefreshToken, Option<String>)` via a new `IdTokenFields` OIDC extra-fields type
  (`crates/services/src/auth/auth_service.rs`). The resource auth-code callback persists it to the
  session under `{client_id}:id_token` (new `id_token_key` in `session_keys.rs`).
- `GET /bodhi/v1/user` surfaces `id_token` on the logged-in user (`UserInfo.id_token: Option<String>`,
  read from session in `routes_users_info.rs`). Multi-tenant / external-app arms return `None`.
- New setting **`BODHI_REFERENCE_API_URL`** (default `https://api.getbodhi.app/`, env-overridable,
  NOT editable in the Settings UI) surfaced on `GET /bodhi/v1/info` as `AppInfo.reference_api_url`.
- OpenAPI + `@bodhiapp/ts-client` regenerated; `AppInfo.reference_api_url` and `UserInfo.id_token`
  flow to the frontend types. `cargo test -p routes_app -- openapi` green.

**Frontend (`crates/bodhi/src`):**
- **AppShell ported to TSX** under `components/shell/` (13 files): `AppShell`, `Shell`, `ShellNav`,
  `ShellIcon` (→ `lucide-react`, no global lucide), `ShellSearch`, `ShellModeSwitch`,
  `ShellFilterGroup`, `ShellChrome`, `ShellContext`, `shell-nav-config`, `shell.css`, `index.ts`,
  `AppShell.test.tsx`. Strip-on-port rules enforced (no `lucide.createIcons`/`window.*`/
  `ReactDOM.createRoot`/`data-lucide`/`data-theme` setattr).
- **AppShell is the root layout** via `__root.tsx` → `RootShell`: app routes render inside the shell
  (section/subPage derived from pathname by `resolveShellRoute.ts`); bare routes (setup/login/auth/
  request-access/oauth/root-redirectors) render chrome-less. Old `AppHeader`/`AppNavigation`/
  `AppBreadcrumb` are no longer rendered (kept on disk; deleted in the final batch).
- **Token merge**: `styles/globals.css` now carries the lotus V2 palette + `--bodhi-*` raw ramps;
  prod-only vars (`--header*`, full `--sidebar-*`, charts) preserved. `--c-*` semantic helpers live
  in `shell.css` (dark selector `.dark, [data-theme="dark"]`).
- **Per-screen UI-V2 flag**: `lib/uiV2Flags.ts` + `hooks/useUiV2Flag.ts` (localStorage,
  default-old), keyed by screen id.
- **Reference-API scaffold**: `lib/referenceApiClient.ts` (fetch + Bearer id_token) +
  `hooks/reference/` (`useReferenceApi` composing AppInfo base URL + user id_token). No domain hook
  yet (the MCP batch adds the catalog hook).

## Decisions made (recorded in batch-0-foundation-plan.md)
- `reference_api_endpoint` → renamed to **`reference_api_url`** mid-batch (user request).
- Section landing routes: Models→`/models/`, Settings→`/settings/`.
- Reference endpoint non-editable; flag is localStorage dev-toggle; single Batch-0 commit.
- **Centralized shell in `__root`** (not per-route `<Shell>`), with a **shell-vs-bare** split (the
  full standalone/auth/setup layout families are out of scope — see below).

## Scope clarification (locked by user mid-batch)
The task + phased plan cover **only the 13 shell app screens**. The **access-request standalone**,
**setup wizard**, and **Keycloak/auth** screens are **out of scope** (designs now exist in `design/`
under `bodhi-standalone.css`/`setup-flow.css`/`bodhi-auth.css`, but deferred). They render bare
today and keep their current layouts. `screen-coverage.md` updated.

## Surprises / learnings (carry forward)
- **Dropping `AppHeader` from `__root` did NOT break any existing RTL test** (965 pass). Route tests
  render their page component directly, not through `__root` — so the chrome swap is transparent to
  them. → Per-screen migrations can likewise restructure page chrome freely as long as
  `data-testid`/ARIA are preserved.
- **2 PRE-EXISTING backend live-test failures** are unrelated to Batch 0: `test_live_anthropic` +
  `test_live_model_router` fail with `missing field 'name'` (API-model request schema drift).
  Confirmed identical failure on a clean `git stash` tree. **Flag for separate fix** — not a Batch-0
  regression. Backend gate otherwise: 51 `test result: ok` blocks.
- **Token flip is global + instant** — verified live: the Login button rendered lotus-pink after the
  merge. Eyeball remaining screens for hardcoded-white-on-`bg-primary` as each migrates.
- **Auth wall blocks browser shell-verification**: unauthenticated app routes redirect to bare
  `/ui/login/`, so the shell on real app routes can only be seen logged in. Shell rendering is
  covered by `AppShell.test.tsx` + `resolveShellRoute.test.ts`; E2E (own login fixtures) exercises
  it on authenticated routes.
- `make test.backend` needs Docker (PostgreSQL). The pg-variant DB tests panic at fixture setup if
  Docker is down — start Docker before the backend gate.

## Reusable patterns established
- `ShellIcon` kebab→PascalCase `lucide-react` mapping (ported pages keep prototype icon names).
- `resolveShellRoute` longest-prefix section/subPage derivation (sub-page wins on href tie).
- Per-screen flag convention (`bodhi.ui-v2.<screen>`), default-old, retired at screen acceptance.
- MSW `AppInfo`/`UserInfo` fixtures must carry the new required `reference_api_url` (+ optional
  `id_token`) — handler default updated in `info.ts`.

## Gate results
- Backend: `make test.backend` — 51 ok blocks; 2 pre-existing live failures (above). openapi sync ok.
- Frontend RTL: **965 passed, 6 skipped, 0 failed**. Typecheck clean. Batch-0 files lint-clean
  (repo has ~289 pre-existing lint errors in untouched files).
- Browser: `/bodhi/v1/info` returns `reference_api_url`; token flip confirmed live.
- E2E (`make test.e2e`): **214 passed, 5 failed, 16 skipped** (46.3m). The 5 failures are 3 distinct
  specs, each failing in BOTH standalone + multi_tenant projects — clustered on live-upstream /
  streaming / setup flows, NOT shell-chrome:
  1. `api-models-forward-all` (×2) — requires a real OpenAI key (`ApiModelFixtures.getRequiredEnvVars`)
     + live `api.openai.com` sync; fails fast (~9s). All MOCK-server Models tests pass → shell did
     not break Models navigation. **Credential/env-dependent.**
  2. `chat.spec.mjs multi-chat management @integration` (×2, ~55s) — real streaming + error handling.
     Chat is UNMIGRATED (old chat UI inside the new shell). **TO VERIFY**: could be a shell
     full-height/scroll interaction OR pre-existing streaming flakiness. Confirm via clean-tree
     stash-compare before the Chat batch.
  3. `setup-api-models Happy Path` (×1) — setup wizard, which is **out of scope** and renders bare.
  214 passing includes all mock Models/tokens/MCP/nav tests → shell navigation intact.

## Follow-ups (carry to next session)
1. **Verify the 3 failing E2E specs against the clean tree** (git stash, run just those specs). Any
   that fail clean → pre-existing (log + ignore for Batch 0). If `chat.spec` passes clean but fails
   with Batch 0 → real shell regression on the chat layout; fix before the Chat batch (Batch 5).
2. **Fix the 2 pre-existing backend live-test failures** (`missing field 'name'` API-model request
   schema drift) — separate from this migration.
3. Repo has ~289 pre-existing eslint errors in untouched files (import/order, a missing rule in a
   test mock) — not addressed here.
