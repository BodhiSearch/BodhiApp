# Kick-off — Batch 1: API Keys

> Load the shared context first via @common-prompt.md, then run the per-batch loop
> (@process.md §"The per-batch loop"). Batch 0 (foundation) is **done** — read
> @batch-0-foundation-retro.md before starting; it carries forward what's already in place and the
> gotchas. Scope reminder (@screen-coverage.md): the migration covers only the 13 shell app screens.

## What Batch 0 already gives you (don't rebuild)
- **AppShell in TSX** at `crates/bodhi/src/components/shell/` (`AppShell`, `Shell`, `ShellNav`,
  `ShellIcon`, `ShellSearch`, `ShellModeSwitch`, `ShellFilterGroup`, `ShellChrome`, `useShell`,
  `SHELL_NAV`). Import from `@/components/shell`.
- **Root shell**: `__root.tsx` already renders the shell for app routes (section/subPage derived from
  pathname). A migrated screen does NOT need to add `<Shell>` itself for chrome to appear — but it
  CAN render its own `<AppShell ...>` with richer props (breadcrumb/headerActions/sidebar/rail) when
  it needs them. **Decide per screen** whether to rely on the root shell or render an explicit
  `<AppShell>` for full control. (Open design question for this batch — see below.)
- **Per-screen flag**: `useUiV2Flag('app-tokens' | 'new-token' | 'access-requests' |
  'access-request-review')` from `@/hooks/useUiV2Flag` — render old vs new, default-old.
- **Tokens merged** (lotus palette live). **Reference-API scaffold** exists (not needed this batch).
- **Shared form/list/picker CSS** to port alongside screens: `design/bodhi-form.css`,
  `design/api-keys-shell.css`, `design/bodhi-list.css`, `design/model-access-picker.{jsx,css}`.

## This batch — API Keys section (4 screens)
| Screen | design source | prod route | notes |
|---|---|---|---|
| App Tokens (list) | `app-tokens-app.jsx` + `App Tokens.html` | `routes/tokens/index.tsx` | token cards, filter tabs, search, empty state |
| New App Token | `new-app-token-app.jsx` + `New App Token.html` | **NEW** `routes/tokens/new/index.tsx` | was a dialog → full page; 4 sections incl. Model + MCP access pickers |
| Access Requests (list) | `access-requests-app.jsx` + `Access Requests.html` | `routes/users/access-requests/` | request cards, pending-count pill in headerActions |
| Access Request review | `access-request-app.jsx` + `Bodhi Access Request.html` | `routes/apps/access-requests/review/` | uses ModelAccessPicker |

> Note: in `SHELL_NAV`, `access-requests` lives under the **api-keys** section. The admin Access
> Requests list is `/users/access-requests/`. Confirm the current route + hooks during Explore.

## Loop steps
1. **Explore** — re-read the current code for these 4 screens (routes, the `hooks/tokens` +
   `hooks/users` access-request + `hooks/apps` review hooks, existing colocated tests + page objects).
   View the prototypes visually (`design/`, server on :8000, Claude-in-Chrome).
2. **Prerequisites** — port the shared design components/CSS this section needs:
   - `components/shell/ModelAccessPicker.tsx` (← `design/model-access-picker.jsx`) — shared by New
     App Token + Access Request review. Strip-on-port rules (no global lucide/window).
   - `bodhi-form.css`, `api-keys-shell.css`, `bodhi-list.css` ports (side-effect imports).
   - No reference API needed this batch.
3. **Plan → refine → approve** — write `batch-1-api-keys-plan.md` (screens, AppShell props per
   screen, reused hooks, ModelAccessPicker, the dialog→page route, test list incl. MSW handlers +
   e2e specs, risks). Present, refine, get approval before coding.
4. **Implement** behind the flags (recipe @process.md §"Per-screen migration recipe"). Dialog→page:
   new `routes/tokens/new/index.tsx` with `createFileRoute('/tokens/new/')` (trailing slash), real
   `useCreateToken` mutation replacing the prototype's fake-token generator. The list page stops
   importing the old `CreateTokenDialog` (delete old dialog code at batch end).
5. **Migrate tests + e2e** — keep `data-testid`/ARIA across the restructure; the New App Token
   creation-flow test moves to the new route's test file; update `tests-js/pages/TokensPage.mjs`
   (nav → shell nav; dialog→page rename) + `specs/tokens/*`.
6. **All gates green** → retire flags + delete old API-Keys code → commit → retro → Batch 2 kickoff.

## Open design question to resolve in the Batch 1 plan
**Root shell vs explicit per-screen `<AppShell>`.** Batch 0 chose a centralized root shell that
derives section/subPage and passes `contentClass="flush"` only. But these screens need
`headerActions` (e.g. "New Token" button, pending-count pill), `breadcrumb`, and (for the list) a
filter sidebar slot — which the root shell doesn't supply. Decide the mechanism:
- (a) each migrated screen renders its OWN `<AppShell ...>` with full props, and `__root` skips the
  shell when the screen does so (needs a signal, e.g. the screen opts out / a context), OR
- (b) add a lightweight per-route shell-props context that a screen can populate (breadcrumb,
  headerActions, sidebar, rail) which the root shell consumes.
Recommend (b) (one root `<AppShell>`, screens contribute slots via context) — but confirm with the
user, since it sets the pattern every later batch uses. This is the first batch that needs richer
shell props, so it's the right place to establish it.

## Carry-forward gotchas (from @batch-0-foundation-retro.md + @process.md)
Strip-on-port greps; trailing-slash routes + regen `routeTree.gen.ts` for the new `/tokens/new/`;
preserve testids; the 2 pre-existing backend live-test failures are NOT yours; start Docker for
`make test.backend`.
