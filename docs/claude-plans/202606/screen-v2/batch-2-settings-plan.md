# Batch 2 — Settings — Plan

> Status: **DRAFT for approval**. Working folder: `docs/claude-plans/202606/screen-v2/`.
> Loop ref: `process.md` §"The per-batch loop". Kickoff: `batch-2-settings-kickoff.md`.
> Grounded in: GATE-A live interactive walk of both prototypes on http://localhost:8000
> (light + dark + responsive, console-clean), a fresh read of the current code (2026-06), the
> already-migrated **access-requests v2** sibling, and the `tanstack-router` / `tanstack-query` /
> `vercel-react-best-practices` / `web-design-guidelines` skills.
>
> **On approval, copy this file to `screen-v2/batch-2-settings-plan.md`** (the canonical per-batch
> plan location). It lives at the plan-mode path only because that's the one file editable while
> planning.

## Context — why this batch

Batch 0 made AppShell the root layout; Batch 1 migrated **API Keys** (4 screens) to V2, retired its
flags, and built the reusable list/chrome primitives (`useShellChrome`, `ShellFilterTabs`,
`useCollapsibleSearch`, `BareLayout`, `list.css`/`api-keys.css`). **Batch 2 migrates the Settings
section's two screens** — **App Settings** and **Manage Users** — to the V2 design behind their
per-screen flags, then retires the flags and deletes the old code (incl. the now-dead
`/users/pending/` route + `UserManagementTabs`). It is the **first batch to publish a shell
`sidebar`** (App Settings' scroll-spy group nav), so it adds one small reusable slot to the chrome
scaffolding; everything else reuses Batch 1.

## Scope (2 screens) — decisions locked with the user

| Screen | prod route | nav (section / subPage) | flag id | layout |
|---|---|---|---|---|
| **App Settings** | `/settings/` | `settings` / `app-settings` | `app-settings` | shell (+ published sidebar + rail) |
| **Manage Users** | `/users/` | **`users` / `manage-users`** | `manage-users` | shell (+ rail) |

> **Doc drift fixed by this batch:** the kickoff table (`batch-2-settings-kickoff.md` line 29) and
> its loop steps (line 82) say Manage Users is `settings / manage-users` and nav via
> `navViaShell('settings','manage-users')`. The **actual `SHELL_NAV` nests `manage-users` under the
> `users` section** (sibling of the Batch-1 Access Requests). `screen-coverage.md` and `tracker.md`
> are already correct. **Trust the code:** Manage Users is `users / manage-users`; the **kickoff is
> the only stale doc** and we fix it. (User directive: "update the doc that are conflicting.")

### Key user directives baked in
1. **Real data only — no scope increase** (Batch-1 rule). Prototype elements with no backing data
   are **out**: App Settings' **restart banner + "Needs Restart" tab** (no `requiresRestart` field,
   no restart endpoint in `routes_settings.rs`), and Manage Users' **multi-tenant org-switcher /
   TweaksPanel** (prototype scaffolding).
2. **App Settings read-only rail:** only **2 settings are editable** (`BODHI_EXEC_VARIANT`,
   `BODHI_KEEP_ALIVE_SECS` — the backend `EDIT_SETTINGS_ALLOWED` list). **Every** row opens the rail
   to *view* details; the **editor (NEW VALUE + Save/Cancel/Reset) renders only for those 2**;
   read-only rows show a "read-only (set via *source*)" note. `source==='system'` hides the current
   value (existing rule, preserved).
3. **App Settings filters:** **All / Modified / Env** only (counts derived during render). No restart.
4. **Manage Users invite link:** **keep it, multi-tenant deployments only.** Port the existing
   `InviteLinkSection` (already gated on `appInfo.deployment === 'multi_tenant'`) into the shell
   **header actions** as a self-contained Invite button/popover. Preserve `invite-url-input` +
   `invite-copy-button` testids and the multi-tenant gate.
5. `reference_api_url` is **NOT a `SettingInfo`** (it's `AppInfo`) — the kickoff's "not editable"
   note is **moot**; nothing to do.

## Prerequisites (build before the screens)

### P0 — Add a `sidebar` slot to the chrome scaffolding *(the one shared change)*
The App Settings design's distinctive piece is a **settings-group scroll-spy nav in the shell
sidebar**. Today `ShellSlots` carries only `breadcrumb | headerActions | rail | railHeader |
railDefaultOpen` — **no `sidebar`**. `AppShell` *already* accepts a `sidebar` prop (renders it into
both the expanded `.shell-nav-body` and the collapsed `.shell-iconrail`), and `__root.tsx` already
spreads `<AppShell {...slots}>` — so the fix is **additive and needs no `__root` edit**:
- `components/shell/ShellSlotsContext.tsx`: add `sidebar?: ReactNode` to `ShellSlots`; thread it
  through the `useShellChrome` memo + publish effect (keep the split setter/value contexts).
- Update the doc comment (it enumerates the slots).
- **Unit test**: `ShellSlotsContext.test.tsx` asserts a published `sidebar` reaches `useShellSlots()`
  and clears on unmount. (Follows `feedback_generic_evolvable_design` — a reusable seam later
  Models/MCP batches can use, not dead code.)

> Why not also publish `mainScroll`/`contentClass`? They're fixed by `__root`'s single `<AppShell>`
> and can't be slot-driven. We **don't need them**: the proven `.l-page` (height:100%) → inner
> `.l-scroll` (`overflow-y:auto`, `@container`) pattern is fully self-contained — the outer shell
> body has nothing to scroll. The prototype's `mainScroll={false}` is unnecessary under our root
> shell (Access Requests already proves this).

### P1 — Port the design CSS (scoped; strip prototype idioms)
- `design/bodhi-app-settings.css` → `components/shell/settings.css`, **every rule scoped under
  `.settings-screen`**. Reuses `list.css` `l-*`/`dp-*` globally; ports only the settings-unique
  classes: `.section-nav`/`.snav-*` (group nav + legend), `.section-header*`, `.setting-row` + cells
  (`.row-key`/`.key-name`/`.row-value`/`.row-actions`/`.row-edit-btn`/`.type-badge`), the
  `.dp-restart-warn`/`.dp-foot-row` rail bits, `.field-input`/`.field-select`.
  ⚠ **Collision guard:** the design sheet ships **global** `.badge*`, `.empty-state`, `mark`,
  `.field-*`, and a `:root` var block. **Rename `.badge` → `.s-badge`** (or scope it), drop the
  global `:root`/`mark`, and scope everything under `.settings-screen`. Grep before importing.
- `design/manage-users.css` → `components/shell/manage-users.css`, scoped under
  `.manage-users-screen`. Most list/rail styling comes from `list.css`; port only `mu-*` cells/badges
  the rows/rail need. (Or reuse the `user-access-requests.css` patterns — decide during port; prefer
  reuse.) Same `.mu-badge`/global-collision discipline.

### P2 — Shared setting-input logic (extract, don't duplicate)
New `routes/settings/-shared/settingInput.tsx`:
- `SettingValueInput` — renders the type-specific control (string `text` / number `min..max` /
  `option` select / `boolean` switch), mirroring `EditSettingDialog.renderInput`. In the **rail** use
  the design's raw `.field-input`/`.field-select`; the surviving V1 dialog keeps shadcn.
- `parseSettingValue(metadata, raw)` — the number coercion + min/max + boolean parsing from
  `EditSettingDialog.handleSubmit`, returning `{ value | error }`.
- Both the (still-flagged-on-default) V1 `EditSettingDialog` and the new V2 rail import these, so
  validation can't diverge. **Unit test** `settingInput.test.tsx` (parse + render per type).

**No backend / OpenAPI / ts-client changes** — all hooks + endpoints already exist
(`useListSettings`/`useUpdateSetting`/`useDeleteSetting`, `useListUsers`/`useChangeUserRole`/
`useRemoveUser`/`useGetAuthenticatedUser`, `useGetAppInfo`). Confirmed.

## Per-screen implementation (behind flags; recipe = `process.md` §"Per-screen migration recipe")

Both screens keep their `AppInitializer` wrapper **outside** the shell (redirects must fire), branch
**old vs new** via `useUiV2Flag('<id>')` (default-old) **inside the page component** (Batch 2 is the
**first** flag consumer — no in-repo example; pattern: `const [v2] = useUiV2Flag('app-settings');
return <AppInitializer …>{v2 ? <V2/> : <V1/>}</AppInitializer>`). Hooks/queries/mutations/fixtures
**unchanged**. Slot nodes **memoized** (vercel `rerender-no-inline-components`).

### 1. App Settings (`/settings/`, flag `app-settings`)
New `SettingsPageV2` (colocated in `routes/settings/index.tsx`; keep `SETTINGS_CONFIG` + V1):
- **Derive groups from the existing `SETTINGS_CONFIG`** (App / Model / Llama.cpp Exec / Server /
  Public Server / Logging / Development / Auth) **plus** the dynamic *Server Args by Variant*
  (`BODHI_LLAMACPP_ARGS_*`) and *Miscellaneous* (ungrouped) groups — **do NOT drop the real groups
  the 4-group prototype omits.** A setting's group/desc come from config; its value/source/metadata
  from `useListSettings()`.
- **Published `sidebar`** = settings-group nav: per-group counts, **scroll-spy** (click → smooth
  `scrollTo` the section; active group follows scroll via a **rAF-throttled `onScroll`** with a
  change-guard) + the **Legend** card. Collapsed shell → icon-rail buttons come free from
  `AppShell`'s `sidebar` rendering. Memoize on `[counts, activeSection]` (activeSection is data).
- **Body** (`.settings-screen l-page`): `l-controls` (searchNode.row + `l-toolbar` with
  `ShellFilterTabs` All/Modified/Env + `l-tb-actions`{searchNode.toggle}); `l-scroll` →
  per-group `.section-group` (sticky `.section-header`) of `.setting-row`s. Empty state on no match.
  - **Filter logic (real source data):** Modified = `current_value !== default_value`; Env =
    `source === 'environment'`. Counts derived in a `useMemo`.
  - **Search** filters key+desc+value, highlights matches (`useCollapsibleSearch`).
  - **Row**: key (mono) + value (or `—`; hidden when `source==='system'`) + **`.s-badge` source** +
    type badge + edit pencil (only for editable) + description. Click anywhere → select + open rail.
- **Rail** (`useShellChrome` rail/railHeader): header (key + group + close); body always =
  status badges (source/type), description, **Current / Default** rows. Then **conditionally**:
  - editable (`EDITABLE_KEYS = {BODHI_EXEC_VARIANT, BODHI_KEEP_ALIVE_SECS}`): NEW VALUE
    `SettingValueInput`, **Save** (`useUpdateSetting`, disabled until dirty), **Cancel**, **Reset**
    (`useDeleteSetting` = delete→default). Gate on the explicit set — **never infer from `source`.**
  - read-only: a `.dp-field-hint` "This setting is read-only (set via *source*)." — no input.
  - Use `useViewTransition()` for select (matches App Tokens rail). `source==='system'` hides
    current value in the rail too.
- **Breadcrumb** `Bodhi › App Settings`. **Testids to add** (current code has only skeletons):
  page root `data-testid="settings-page"` + `data-pagestatus`, `setting-row-{key}`,
  `setting-key-{key}`, `setting-value-{key}`, `setting-source-{key}`, `setting-edit-{key}`,
  `settings-filter-{all|modified|env}` (via `ShellFilterTabs testIdPrefix`),
  `settings-search-toggle`/`-close`, rail: `setting-detail-{key}`/`-close`, `setting-new-value`,
  `setting-save`/`-cancel`/`-reset`, `setting-readonly-note`. Keep `settings-skeleton*` for loading.

### 2. Manage Users (`/users/`, flag `manage-users`) — mirror access-requests v2 exactly
New `UsersPageV2` (colocated in `routes/users/index.tsx`; keep V1 + `InviteLinkSection`). Structure
**mirrors `routes/users/access-requests/index.tsx`** (same `users` section, proven under the root
shell):
- `<div className="manage-users-screen l-page" data-testid="users-page" data-pagestatus=…>` with
  `l-controls` (search row + `l-toolbar`{`ShellFilterTabs` + `l-tb-actions`}) and `l-scroll`
  (loading skeleton / empty-state / `l-listview` with `l-listhead` + rows) + `Pagination` when
  `total > pageSize`.
- **Hooks unchanged:** `useListUsers(page,pageSize)` (real pagination kept), `useChangeUserRole`,
  `useRemoveUser`, `useGetAuthenticatedUser`, `useGetAppInfo`.
- **Filter tabs = 5** (All / Admin / Manager / Power User / User) from `ROLE_OPTIONS` (**4 real
  roles** — the prototype's 3 was a simplification). **Client-filter the current page only** (like
  access-requests; no server-side role param — would be a backend change). Counts are **per-page**;
  comment the limitation. Collapsible **search by username**.
- **Row**: Username (mono, `user-username` + `user-username-{username}`) + Role badge
  (`getRoleLabel`/`getRoleBadgeVariant`, `user-role` + `user-role-{username}`) + "You" on self.
  Click → select + open rail.
- **Rail** = role editor, reproducing `UserActionsCell`'s hierarchy **verbatim** (`getRoleLevel`/
  `getAvailableRoles` + `useGetAuthenticatedUser`):
  - `isCurrentUser` → read-only note + `current-user-indicator` ("You").
  - target outranks actor (`!canModifyUser`) → read-only note + `restricted-user-indicator`.
  - else: ACCOUNT (username, current role), **CHANGE ROLE** native select
    (`role-select-{username}`, options from `getAvailableRoles`), **Save** (`useChangeUserRole`,
    disabled until dirty), **Remove** two-click confirm (`remove-user-btn-{username}`,
    `useRemoveUser`).
- **Header actions** (`useShellChrome`): self-contained `<InviteLinkAction/>` (popover with the
  invite URL + copy), keeping the `multi_tenant` gate + `invite-url-input`/`invite-copy-button`.
  Popover open/close state lives **inside** the node so toggling doesn't re-publish the slot. (The
  search toggle lives in the toolbar `.l-tb-actions`, so "before the search" = in the header band,
  left of nothing else; the toolbar search toggle stays in the toolbar.)
- **Breadcrumb** `Bodhi › Users › Manage Users`. `AppInitializer` keeps `minRole="manager"`.

**Testid migration (Manage Users — highest-risk item):** the V2 design **replaces** the shadcn
`RoleChangeDialog`/`RemoveUserDialog` with the rail's inline select + two-click confirm (access-
requests v2 dropped its modals too).
- **Survive** (move to row/rail): `users-page`, `user-username[-{username}]`, `user-role[-{username}]`,
  `current-user-indicator`, `restricted-user-indicator`, `role-select-{username}` (now a native
  `<select>` in the rail), `remove-user-btn-{username}` (rail).
- **Retire** (shadcn-Select/AlertDialog-only — impossible on native select / inline confirm):
  `role-change-dialog`/`remove-user-dialog` families, `role-select-trigger/content-{username}`,
  `role-option-{role}-{username}`, `user-actions[-container]-{username}`, `no-actions-{username}`.
  The page object + spec are rewritten to match (below).

## Tests (all layers — `feedback_testing_depth`)

**RTL / Vitest + MSW** (handlers/fixtures reused; contract identical):
- **P0:** `ShellSlotsContext.test.tsx` — `sidebar` slot publishes/clears. **P2:**
  `settingInput.test.tsx` — parse + render per type (string/number±range/option/boolean).
- App Settings V2 (flag ON, **`ShellSlotsProvider` + slots-consumer harness extended to assert the
  published `sidebar`**): groups render from config+data; filter All/Modified/Env counts; search
  filters/highlights; click row → rail; **editor present only for the 2 editable keys**, read-only
  note for others; Save→`useUpdateSetting`, Reset→`useDeleteSetting`; `source==='system'` hides value.
- Manage Users V2 (flag ON, harness): list + role filter (5 tabs) + search; **role-hierarchy in the
  rail** (self → "You"; outranked → "Restricted"; modifiable → select+Save+Remove); two-click
  remove; invite action renders **only** when `deployment==='multi_tenant'`; pagination.
- Keep both default-old V1 paths green until flags retire (V1 tests untouched).

**server_app integration (Rust):** none — presentation-only, no route/contract change (confirmed).

**Playwright E2E** (black-box; `make test.e2e` from `lib_bodhiserver/tests-js`; flag set via
`page.addInitScript` **before first nav**, never `page.evaluate`):
- **New** `pages/SettingsPage.mjs` (none exists) + `specs/settings/app-settings-v2.spec.mjs`: nav
  `navViaShell('settings','app-settings')`; assert groups/filters/search; **edit
  `BODHI_KEEP_ALIVE_SECS`** via the rail (Save → value reflected); assert a read-only setting shows
  no editor.
- **Rewrite** `pages/AllUsersPage.mjs` + `specs/users/list-users.spec.mjs` (deeply coupled to
  `tbody tr`, shadcn comboboxes, confirm dialogs — all gone): nav
  `navViaShell('users','manage-users')`; rows are `div`s; **role change via the rail's native
  select (no dialog)**; **remove via two-click**; role availability read from the rail's `<option>`s;
  preserve the hierarchy assertions (self/restricted). Add a shared `navViaShell` helper to
  `BasePage.mjs` if not already present (Batch-1 carry-forward).
- **Run the full E2E matrix once** (both projects) — the new `sidebar` slot + shared CSS are
  app-wide. *Note: live E2E needs `tests-js/.env.test` Keycloak creds (absent locally → parse-check
  via `--list`); flag for the user.*

## Gate checks (all green before commit — `feedback_run_all_gate_checks`)
`cd crates/bodhi && npm run test` (RTL) · typecheck/lint on touched files · `make test.e2e` (Docker
up; dev-server + Vite HMR, no ui-rebuild) · **GATE B live validation** in Claude-in-Chrome
(`make app.run.live`, log in): each screen + interactions in **light + dark + responsive**, with
**`read_console_messages` = 0 errors** on load and on each key interaction (select→rail, filter,
search, scroll-spy, role change, remove, invite copy, sidebar collapse). No backend tests (no Rust
change; the 2 pre-existing live-test failures are NOT ours — `techdebt.md`).

## Retire flags + delete old code (end of batch)
Remove the 2 `useUiV2Flag` branches + V1 renders: App Settings V1 (`SettingsPageContent` +
`EditSettingDialog` — *keep the extracted `-shared/settingInput`*), Manage Users V1
(`UsersContent`/`UsersTable`/`UserRow`/`UserActionsCell`/`RoleChangeDialog`/`RemoveUserDialog`,
old `InviteLinkSection`). **Delete the now-dead `/users/pending/` route + `UserManagementTabs`**
(only consumers are `/users/` + `/users/pending/`; both handled). Drop the `app-settings`/
`manage-users` entries from `UiV2Screen`. Commit per batch (trunk-based, directly on `main`;
`git fetch && git rebase origin/main` first).

## Docs to update (this batch)
- **`batch-2-settings-kickoff.md`** — fix the **only** stale rows: line-29 table
  `settings / manage-users` → `users / manage-users`; line-82 `navViaShell('settings', …)` →
  `navViaShell('users','manage-users')`. (`screen-coverage.md` + `tracker.md` already correct.)
- **`tracker.md`** — at batch end flip screens 13/14 to ✅, retire the 2 flag ids, note the new
  `sidebar` slot + the `/users/pending/`+`UserManagementTabs` deletion.
- **`techdebt.md`** — note the `sidebar` slot is part of the temporary `ShellSlotsContext`
  scaffolding (removed with it at migration end); the per-page-only role filter on Manage Users
  (server-side role filtering would need a backend param) as an accepted limitation.
- `crates/bodhi/src/CLAUDE.md`/`PACKAGE.md` — note the `sidebar` chrome slot + the V2 settings/users
  screens as touched.

## Risks / watch-outs
- **`sidebar` slot is the only shared change** — additive, `__root` unchanged; unit-test it first.
- **Generic CSS collisions** — scope every rule under `.settings-screen`/`.manage-users-screen`;
  **rename `.badge`→`.s-badge`**, drop the design's global `:root`/`mark`. Grep before import.
- **Scroll-spy** — rAF-throttle `onScroll` + change-guard; **never transition
  `grid-template-columns`** (carry-forward #1) on the settings row grid.
- **Read-only-rail split** — gate on the **explicit `EDITABLE_KEYS` set**, never infer from `source`.
- **Manage Users testid migration is the highest risk** — page object + spec rewrite (rows→div,
  dialog→rail, shadcn-select→native). Preserve the surviving testids verbatim; retire the rest
  deliberately.
- **Per-page filter/counts** on Manage Users — label honestly; comment the limitation.
- **Stable memoized slot nodes** for `useShellChrome` (settings `sidebar` memo deps
  `[counts, activeSection]`); never inline component defs.
- **Strip prototype idioms** on every port: `lucide.createIcons`/`window.*`/`ReactDOM.createRoot`/
  `data-theme`/`TweaksPanel`/`useTweaks`/`requiresRestart`/multi-tenant org-switcher. Grep each
  ported file before "done".

## Verification (end-to-end)
1. `make app.run.live`, log in. Flag each screen on (`localStorage.setItem('bodhi.ui-v2.app-settings'
   /'…manage-users','true')`), walk both light+dark+responsive in Claude-in-Chrome; confirm:
   App Settings group-nav scroll-spy + collapse, All/Modified/Env filters, search highlight,
   read-only vs editable rail, **edit `BODHI_KEEP_ALIVE_SECS` end-to-end** (value persists);
   Manage Users 5-tab role filter, search, role change + two-click remove via rail, hierarchy
   guards (self/restricted), invite popover (multi-tenant only), pagination. **Console = 0 errors.**
2. RTL green (incl. P0 `sidebar` slot, P2 `settingInput`, both V2 screens). E2E green (new
   `SettingsPage.mjs`/spec; rewritten `AllUsersPage.mjs`/`list-users.spec.mjs`); full matrix once.
3. Flags retired + old code (+ `/users/pending/` + `UserManagementTabs`) deleted → re-run RTL + E2E
   green → commit → retro → **Batch 3 (Models)** kickoff.
