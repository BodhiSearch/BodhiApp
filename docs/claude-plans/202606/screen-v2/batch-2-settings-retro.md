# Batch 2 — Settings — Retrospective

Status: implementation complete; RTL + targeted E2E green; GATE B (live) passed. Landed as a single
focused commit on `main` (`7d3ad736`, +3047/−3762 — net deletion from retiring V1 code).

## What landed

**Two screens migrated to V2, flags retired, old code deleted:**
- **App Settings** (`/settings/`) — V2 shell list. **First screen to publish a shell `sidebar`**: a
  settings-group **scroll-spy** nav (per-group counts, click→smooth-scroll, active follows scroll via
  a rAF-throttled `onScroll`) + a Legend card; collapsed shell → icon-rail buttons (free from
  `AppShell`). Body: **All/Modified/Env** filter pills (counts derived), collapsible search with
  match highlight, per-group sections of setting rows. Rows open a detail rail that is **read-only by
  default** — the NEW VALUE editor (Save via `useUpdateSetting`, Reset via `useDeleteSetting`) renders
  **only for the 2 backend-editable keys** (`BODHI_EXEC_VARIANT`, `BODHI_KEEP_ALIVE_SECS`); everything
  else shows a "read-only (set via *source*)" note. `source==='system'` hides the current value.
  Groups derive from a static config + **dynamic** Variant-Args (`BODHI_LLAMACPP_ARGS_*`) and
  Miscellaneous (ungrouped) groups.
- **Manage Users** (`/users/`, nav **users / manage-users**) — V2 shell list mirroring the Batch-1
  Access Requests sibling. **5 role filter pills** (All/User/Power User/Manager/Admin — 4 real roles,
  per-page counts) + collapsible search; rows open a **role-editor rail** (CHANGE ROLE native select
  + Save via `useChangeUserRole`, two-click Remove via `useRemoveUser`) that reproduces the real
  role-hierarchy guards (`canModifyUser`: self → "You" `current-user-indicator`, outranked →
  "Restricted" `restricted-user-indicator`). Real `useListUsers` pagination kept. The multi-tenant
  **invite link** ported as a header-action popover (gated on `appInfo.deployment==='multi_tenant'`,
  `invite-url-input`/`invite-copy-button` preserved).

**Shared infra:**
- **`sidebar` chrome slot** added to `ShellSlots`/`useShellChrome` — App Settings is the first
  consumer; `__root` already spreads `{...slots}`, so **no root edit**. Additive, backward-compatible,
  reusable for later batches (Models/MCP may want sidebars).
- **`routes/settings/-shared/settingInput.tsx`** — `parseSettingValue` (number coercion + min/max +
  boolean) + `SettingValueInput` (type-specific control) + `settingValueHint`. Shared by the V2 rail
  and (briefly, before deletion) the V1 dialog so validation couldn't diverge.

**Deleted (dead after migration):** `/users/pending/` route + test, `UserManagementTabs` (+ test), the
V1 `components/users/` cluster (`UsersTable`/`UserRow`/`UserActionsCell`/`RoleChangeDialog`/
`RemoveUserDialog`), `settings/-components/EditSettingDialog` (+ test), the old route tests, the
`ROUTE_ACCESS_REQUESTS_PENDING` constant, and the `app-settings`/`manage-users` flag ids.

## Decisions made (with the user)
1. **Read-only rail, no editor for read-only settings** — every row opens the rail to *view* details;
   only the 2 editable keys get the editor. (Prototype showed every row editable — prototype-only.)
2. **Drop the restart banner + "Needs Restart" tab** — `SettingInfo` has no `requiresRestart` and
   there's no restart endpoint; keep **All/Modified/Env** filters only.
3. **Invite link kept, multi-tenant only**, as a header-action popover (before/left of the toolbar
   search, which lives in `.l-tb-actions`).
4. **Manage Users is `users/manage-users`** (matches `SHELL_NAV` post-Batch-1.1), NOT
   `settings/manage-users` as the kickoff table said — fixed the stale kickoff doc.

## Surprises / learnings (carry forward)
- **GATE B earned its keep — it caught two bugs every automated test missed:**
  1. **Duplicate React key** (`BODHI_DEPLOYMENT`) — the backend returns that setting **twice**; both
     copies fell into the Misc group → duplicate keys. Fixed by deduping the dynamic group key lists
     (`[...new Set(...)]`). *Be resilient to duplicate backend keys.*
  2. **Async `InvalidStateError` from `useViewTransition`** — the hook caught the *synchronous* throw
     and the `finished` rejection, but an interrupted transition rejects **`ready`** asynchronously,
     which surfaced as an uncaught console exception **app-wide** (also on shipped Batch-1 screens).
     Fixed by also `.catch()`-ing `ready`. RTL/E2E both passed while this leaked.
- **E2E view-transition flake:** Playwright's `closeRail` raced the rail VT (the close button detaches
  mid-animation). Fix: `page.emulateMedia({ reducedMotion: 'reduce' })` in the page object's nav — the
  hook honors reduced-motion and applies updates synchronously, so the rail open/close is instant in
  tests. Clean fix; no production change.
- **OAuth login is the E2E flake source, not the screen.** The Manage Users multi-user spec passed
  cleanly in 37–41s twice, then two later runs took 7.5–15.6m and timed out on Keycloak's sign-in
  click (30s × many logins). The screen logic was never the problem — verify by re-running when the
  shared auth server recovers. (See `project_e2e_oauth_creds`.)
- **Real-data divergences from the prototype were significant** and worth the up-front analysis: 4
  prototype groups vs ~8 static + 2 dynamic real groups; 6 source values (not 4); a `boolean` metadata
  type; 4 roles (not 3); real pagination + role-hierarchy + invite-link with no prototype home.

## Reusable patterns to carry to later batches
- **`useShellChrome({ ..., sidebar })`** for any screen that needs a page-body sidebar (scroll-spy,
  sub-nav). Memoize the node; if it depends on data (e.g. `activeSection`), memo on that.
- **Read-only-vs-editable rail split** gated on an **explicit allow-set**, never inferred from data.
- **Mirror the access-requests v2 structure** for any list+rail screen under the same root shell
  (`.l-page` / `l-controls` / `ShellFilterTabs` / `useCollapsibleSearch` / `l-scroll` / `Pagination` /
  `useShellChrome` with memoized slots).
- **E2E for rail screens:** set `reducedMotion: 'reduce'` to avoid VT detach races; wait for the
  mutation to settle (Save → disabled) before asserting/reloading.

## Gate results
- **Frontend RTL:** 944 passed / 5 skipped / 0 failed (down from 1010 only because V1 settings/users
  tests were deleted; the V2 screens + P0/P2 add ~35 focused tests). Typecheck clean; touched files
  lint-clean.
- **E2E (targeted, standalone):** `settings/app-settings-v2.spec.mjs` (list, read-only-rail gating,
  edit `BODHI_KEEP_ALIVE_SECS` end-to-end, reset) — **pass (~8–11s)**. `users/list-users.spec.mjs`
  (full multi-user flow: hierarchy, role availability per actor, demote/restore via rail, cross-session
  revocation, two-click remove, final count) — **pass (~37–41s)** on a healthy auth server.
- **GATE B (live):** both screens in **light + dark + responsive**, **console clean (0 errors)** after
  the two fixes, validated in Claude-in-Chrome against `make app.run.live`.
- **Backend:** no Rust changes; backend gate not required.

## Follow-ups
1. **Run the FULL E2E matrix** (both projects, all specs) before Batch 3 — the new `sidebar` slot +
   `useViewTransition` change are app-wide. (Auth-server flakiness makes the multi-user users spec the
   one to watch; re-run when Keycloak is healthy.)
2. The **router-level navigation `InvalidStateError`** (TanStack `defaultViewTransition`) remains a
   deferred cross-cutting item — `useViewTransition` is now hardened for in-page transitions, but the
   navigation-time one is router-internal (techdebt.md).
3. Carry the temporary-scaffolding removal (the `sidebar` slot is part of `ShellSlotsContext`;
   `useUiV2Flag`) to the end-of-migration cleanup (techdebt.md).
