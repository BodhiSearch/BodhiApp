# Batch-1 follow-up: API Tokens rename + Users nav section + Access Requests redesign

## Context

Batch 1 (API Keys) shipped V2-only and its flags were retired. Dogfooding surfaced a set of
nav/labelling and design-fidelity gaps that don't fit the "API Keys" mental model and don't match
the `design/` prototypes. This follow-up corrects them:

1. The token screen is about **API Tokens**, not "App Tokens" / "New Token".
2. **User Access Requests** is mis-filed under the "API Keys" section. The design has a dedicated
   top-level **Users** section. Access requests are a user-management concern, not a key concern.
3. The current `/users/access-requests/` screen was ported as a thin list (plain rows, no avatar,
   no status chips, no detail rail) and does **not** match `design/User Access Requests.html`.
4. **Manage Users** currently lives under the **Settings** section; the design puts it under
   **Users**.

Intended outcome: the shell nav matches the prototype's information architecture (a **Users**
section holding *User Access Requests* + *Manage Users*; an **Access Tokens** section holding
*API Tokens* + *New API Token*), and the Access Requests screen matches the design — slim
avatar rows with status chips + an approve/reject role flow, a filter/search toolbar, a header
pending-count pill, and a right detail rail that mirrors the row.

This is presentation/IA only. **No backend or hook changes**; reuse the existing
`useListAllRequests` / `useApproveRequest` / `useRejectRequest` data layer and the Batch-1
shell primitives. Per the migration playbook this still honors **HARD GATE B** (live-validate in
Chrome before "done": interactions, light/dark, responsive, console clean).

### Decisions (confirmed with user)

- **Section label** → rename `api-keys` section heading **"API Keys" → "Access Tokens"** (match
  design). Sub-pages become **API Tokens** + **New API Token**. The prototype's third sub-page
  "App Tokens" is prototype-only (a model-key screen we don't have) — **not** added.
- **Role binding** → the row/rail role dropdown is an **approval input only** (feeds the existing
  approve mutation). No role is stored on a request, so decided rows show **no role**.
- **Identity** → `username` *is* the email; show `username` in the identity label and derive the
  avatar initials/color from it. No backend field added.
- **Reject wording** → use **"Rejected"** everywhere in the new screen (filter tab + decided note),
  matching the real `rejected` status (drop the design's "Denied" wording).

### Real-data reconciliation (design → our `UserAccessRequest`)

`UserAccessRequest = { id, username, user_id, reviewer?, status: pending|approved|rejected,
created_at, updated_at }`. The prototype's `email`, per-request `role`, `requestedAt`/`decidedAt`,
and "requested N ago" are sample-only. Bind to real fields:

| Design element            | Real binding |
|---------------------------|--------------|
| avatar + email            | `username` (initials + hashed color) |
| row subtitle "Requested…" | pending → `created_at`; decided → `{Approved\|Rejected} {updated_at}` |
| status chip               | `status` (pending/approved/**rejected**) |
| row Role dropdown (pending) | approval-role picker via `getAvailableRoles(userRole)`; **not** persisted |
| row Role static (decided) | **omit** (no stored role) |
| Approve / Reject          | `useApproveRequest({id, role})` / `useRejectRequest(id)` |
| rail ACCOUNT / TIMELINE    | `username`, `created_at`, `updated_at`, `reviewer` |
| header "N pending review"  | derived `counts.pending` |

## Scope

Four screens/areas, all in `crates/bodhi/src`:

1. **Shell nav** (`components/shell/shell-nav-config.tsx`) — rename + restructure sections.
2. **API Tokens** breadcrumbs (`routes/tokens/index.tsx`, `routes/tokens/new/index.tsx`) — label
   text only.
3. **User Access Requests** (`routes/users/access-requests/index.tsx` + new CSS) — full design port.
4. No change to `routes/users/index.tsx` (Manage Users) page body — only its nav location moves.

Out of scope (note, don't touch): legacy `hooks/navigation/useNavigation.tsx` menu (old chrome,
already superseded by `SHELL_NAV` in v2); `routes/users/pending/` (legacy, slated for consolidation
in the Settings/Users batch); `routes/apps/access-requests/review/` (the 3rd-party OAuth consent
flow — a bare deep-link page, NOT the nav "Access Requests" item).

---

## 1. Shell nav restructure

**File:** `crates/bodhi/src/components/shell/shell-nav-config.tsx`

Replace the `api-keys` and `settings` entries, and insert a new `users` section. Target shape
(matches `design/bodhi-app-shell.jsx` `SHELL_NAV`, minus the prototype-only "App Tokens" sub-page):

```ts
{
  id: 'api-keys', label: 'Access Tokens', icon: 'key-round', href: '/tokens/',
  subPages: [
    { id: 'api-tokens', label: 'API Tokens',     icon: 'key-round',   href: '/tokens/' },
    { id: 'new-token',  label: 'New API Token',   icon: 'plus-circle', href: '/tokens/new/' },
  ],
},
{
  id: 'users', label: 'Users', icon: 'users', href: '/users/access-requests/',
  subPages: [
    { id: 'access-requests', label: 'User Access Requests', icon: 'user-check', href: '/users/access-requests/' },
    { id: 'manage-users',    label: 'Manage Users',         icon: 'users',      href: ROUTE_USERS },
  ],
},
{
  id: 'settings', label: 'Settings', icon: 'settings', href: '/settings/',
  subPages: [
    { id: 'app-settings', label: 'App Settings', icon: 'settings', href: '/settings/' },
  ],
},
```

Changes captured:
- `api-keys` section label `API Keys → Access Tokens`; sub-page `app-tokens` label `App Tokens →
  API Tokens` (rename id `app-tokens → api-tokens` for consistency); sub-page `new-token` label
  `New Token → New API Token`; **remove** the `access-requests` sub-page from this section.
- New top-level `users` section (icon `users`) with `User Access Requests` (`user-check`) +
  `Manage Users` (`users`). Section `href` points to access-requests (the section landing).
- `settings` section loses its `manage-users` sub-page (now under Users). It keeps only
  `App Settings`. (A single-subpage settings section is fine; matches the prototype which has an
  empty settings subPages list — keeping App Settings explicit preserves the highlight on `/settings/`.)

**Why this resolves correctly:** `resolveShellRoute.ts` is fully data-driven off `SHELL_NAV` via
longest-prefix match. `/users/access-requests/` (longer) wins over `/users/` so the access-requests
sub-page highlights on that route, and `/users/` highlights "Manage Users". No code change needed
in `resolveShellRoute.ts` or `ShellNav.tsx`.

**Update the section-mapping unit test** for `resolveShellRoute` (`components/shell/
resolveShellRoute.test.ts`): the `/users/access-requests/` case must now resolve to
`section: 'users'` (was `'api-keys'`), and add/adjust a `/users/` → `section: 'users',
subPage: 'manage-users'` case.

---

## 2. API Tokens breadcrumb labels

- `crates/bodhi/src/routes/tokens/index.tsx` — `TOKEN_BREADCRUMB`: middle crumb `'API Keys' →
  'Access Tokens'`; current crumb `'App Tokens' → 'API Tokens'`.
- `crates/bodhi/src/routes/tokens/new/index.tsx` — `NEW_TOKEN_BREADCRUMB`: middle crumb
  `'API Keys' → 'Access Tokens'`; current crumb `'New App Token' → 'New API Token'`. Also update
  the page `<CardDescription>`/title copy if it says "App Token".

(The token list/rail markup, testids, hooks, and the `Switch`/status logic stay unchanged.)

---

## 3. User Access Requests — design port

**File:** `crates/bodhi/src/routes/users/access-requests/index.tsx` (rewrite the V2 content; keep
`Route`, `AppInitializer` guard `minRole="manager"`, and the existing hooks/mutations).

**New CSS:** `crates/bodhi/src/components/shell/user-access-requests.css` — port the `ua-*` classes
from `design/user-access-requests.css` (avatar, slim row cells, status chips, role select, approve/
reject buttons, pending pill, decided note, `dp-btn-approve`, `dp-role-select`). The `l-*` list and
`dp-*` rail primitives already exist in `list.css`; reuse them, only add the page-unique `ua-*`
block. Scope nothing new globally — these classes are page-local like `tokens.css`. Import it in
the route file alongside `api-keys.css` + `list.css`.

### Layout (reuse Batch-1 patterns)

Mirror `routes/tokens/index.tsx` structure exactly — it's the canonical list+rail screen:

- **Toolbar** (`l-controls` / `l-toolbar`): `ShellFilterTabs` (All / Pending / Approved /
  **Rejected**, with count badges, `testIdPrefix="requests-filter"`) + `useCollapsibleSearch`
  (placeholder "Search requests by email…", `toggleTestId`/`closeTestId`). Search filters on
  `username`. Render `searchNode.row` above the toolbar, `searchNode.toggle` in `l-tb-actions`.
- **List** (`l-scroll` → `l-listview`): a sticky `l-listhead` with cells `[avatar] User · Status ·
  Role · (actions)` and one `RequestRow` per visible request. Empty state reuses `empty-state`
  (icon `user-check`).
- **Header pending pill**: reuse existing `headerActions` (`tag tag-saffron`, `data-testid=
  "pending-pill"`) — already present, keep.
- **Selection + rail**: add `selectedId` state + `useViewTransition` wrapper (copy the
  `selectToken` pattern from tokens). On row click, select + the rail opens. Publish via
  `useShellChrome({ breadcrumb, headerActions, rail, railHeader, railDefaultOpen: false })`. Pass
  **stable memoized** `rail`/`railHeader` nodes (re-render discipline from `ShellSlotsContext`).

### Row component (`RequestRow`)

Cells (design `ua-*`): avatar (`ua-avatar`, initials+hashed color from `username`) · identity
(`ua-id`: `ua-email` = `username`, `ua-sub` = requested/decided line) · status chip (`ua-status`
+ status modifier, lucide `clock`/`check-circle-2`/`x-circle`) · role cell (`ua-role-cell`):
**pending** → role `<select>` (`ua-role-select`, options from `getAvailableRoles(userRole)`,
`stopPropagation` on click/change); **decided** → omit (no stored role) · actions (`ua-act`):
**pending** → Approve (`ua-approve`) + Reject (`ua-reject`), each `stopPropagation`; decided →
empty. Row click selects (opens rail). Preserve all existing testids verbatim:
`request-row-${username}`, `request-username`, `request-date`, `request-status-${status}`,
`request-reviewer`, `role-select-${username}`, `approve-btn-${username}`, `reject-btn-${username}`;
add `data-testid` for the avatar/row-select affordance.

### Detail rail (`RequestDetailHeader` + `RequestDetailPanel`)

Port from `design/user-access-requests-app.jsx`, bound to real data:
- **Header** (`dp-head`): avatar + `username` title + "User access request" sub + close X
  (`selectToken(null)`).
- **Panel** (`dp-panel`): `dp-status-row` (status chip + requested/decided text) → `dp-body` with
  sections: **Account** (`dp-row` Email=`username`, Requested=`created_at`); **Assign role**
  (pending → `dp-role-select` + `dp-field-hint`; decided → omit, no stored role); **Timeline**
  (`dp-row` Requested=`created_at`, + decided row Approved/Rejected=`updated_at` when not pending).
- **Footer** (`dp-foot`): pending → Approve (`dp-btn dp-btn-approve`) + Reject (`dp-btn
  dp-btn-danger`); decided → `ua-decided-note` ("Approved/Rejected {date}", icon by status).
- Add `data-testid="request-detail-rail"` (mirror `token-detail-rail`) plus testids for the rail
  approve/reject/role controls so E2E can drive them.

The role selection lifts to the parent (one `selectedRole` per selected request, default
`resource_user`) so the row dropdown and the rail dropdown stay in sync. Approve passes
`{ id, role: selectedRole }`; reject passes `id`. Keep the existing toast wiring.

**Update breadcrumb:** `REQUEST_BREADCRUMB` → `[{label:'Bodhi'}, {label:'Users', href:
'/users/access-requests/'}, {label:'User Access Requests', current:true}]` (no longer under API
Keys/tokens).

---

## 4. Manage Users (nav move only)

No change to `routes/users/index.tsx`. Moving the nav entry in §1 is the entire change; the
existing `/users/` page renders unchanged under the new Users section. (User confirmed the page
itself may be revamped in a future iteration.)

---

## Tests

Layer-by-layer, no skips (per testing-depth + no-skip conventions):

1. **Unit — nav resolution:** `components/shell/resolveShellRoute.test.ts` — update the
   `/users/access-requests/` expectation to `section:'users'`; add `/users/` → `users`/
   `manage-users`; keep `/tokens/` + `/tokens/new/` under `api-keys`.
2. **RTL — Access Requests** (`routes/users/access-requests/index.test.tsx` +
   `index.v2.test.tsx`): rewrite for the new markup. Cover: filter tabs + counts, search filter,
   pending pill, avatar/identity render, pending row shows role select + approve/reject, decided
   row shows status + no role + no action buttons, **row click publishes a rail** (assert via the
   `ShellSlotsProvider` + `useShellSlots` harness used by `tokens/index.v2.test.tsx`), rail
   approve/reject/role wired to the mutations (assert MSW calls), "Rejected" wording. Reuse the
   MSW handlers + `access-requests` fixtures (`test-fixtures/access-requests.ts`).
3. **RTL — tokens breadcrumb:** adjust any token test asserting "App Tokens"/"New App Token"/
   "API Keys" breadcrumb text to the new labels.
4. **E2E (Playwright)** in `crates/lib_bodhiserver/tests-js/`: in the access-requests spec, update
   the nav path (now under **Users**, not API Keys), assert the new row layout + status chips,
   exercise **row-click → rail opens**, approve a pending request **from the rail** and verify it
   moves to Approved, and confirm the pending-pill count. Black-box only (UI interactions, no
   `page.evaluate`/context fetch). Update any nav-label assertions for "Access Tokens" / "API
   Tokens" / "New API Token". Mind the view-transition E2E gotcha — wait for the target element
   (`waitFor({state:'visible'})`) after navigation/selection before scanning rows.

---

## Gate checks (run all before commit — no skips)

```bash
cd crates/bodhi && npm run lint && npm test          # RTL + typecheck
make build.dev-server                                # dev-server for E2E
cd crates/lib_bodhiserver && npm run test:playwright # E2E (access-requests + nav)
make format
```

No Rust/API change ⇒ no `xtask openapi` / `build.ts-client` regen needed.

## Live validation (HARD GATE B — in Claude-in-Chrome, against `make app.run.live`)

For both light and dark, desktop and a narrow viewport, console clean (0 errors):
- Nav: top-level **Users** section appears with **User Access Requests** + **Manage Users**;
  **Access Tokens** section shows **API Tokens** + **New API Token**; Settings shows only App
  Settings. Access Requests is gone from the tokens section. Each link highlights correctly.
- API Tokens + New API Token breadcrumbs read the new labels.
- Access Requests screen matches `design/User Access Requests.html`: avatar rows, status chips,
  filter tabs + counts, collapsible search, pending pill. Click a **pending** row → rail opens with
  Assign-role + Approve/Reject; click a **decided** row → rail shows static status + Timeline +
  decided note (no actions). Approve/Reject from the rail and from the row both work and update the
  list. Sidebar collapse → icon rail still navigates.
- Manage Users opens unchanged under the Users section.

## Docs to update

- `docs/claude-plans/202606/screen-v2/screen-coverage.md` — note IA correction (Users section;
  Access Requests moved out of API Keys → Access Tokens; Manage Users under Users).
- `docs/claude-plans/202606/screen-v2/tracker.md` — record this Batch-1 follow-up.
- If a `ua-*` CSS block becomes a reusable list pattern, note it in the shell CSS conventions.
