# MCP Screens v2 — Migration Context & Build Brief

> **Audience:** the engineer/agent (Claude Code) migrating the existing Bodhi
> app MCP pages onto the new hi‑fidelity design.
> **Companion artifacts:** the v2 prototypes in this project (`Bodhi MCP *.html`
> + `mcp/*.jsx` + `mcp/*.css`). This document captures the **interaction
> context that the static prototypes do not show** — role rules, configured /
> not‑configured states, how the right detail rail is rendered per state, and
> the query‑param contracts that make "click an action → land on a pre‑filled
> form" work.

---

## 0. How to use this document

1. The prototypes are the **visual + interaction source of truth**. Open each
   `.html`, read its `.jsx`, and match layout/behavior exactly.
2. This doc is the **decoder ring** for the hidden state. Whenever a prototype
   branches on `role`, `s.registered`, `s.disabled`, `requestStatus`, or
   `instance.status`, the matrices below tell you *every* branch — including
   ones a single screenshot can't reveal.
3. **Migrate existing pages; don't rebuild them.** Carry the current app's unit
   + e2e tests across and adapt them. Only build net‑new where there is no
   existing page to migrate (see §2).
4. Everything in the prototype's data layer (`mcp/mcp-catalog.jsx`) is **mock**.
   The real app already has the APIs, types, and tests — keep those; only the
   **UI structure, states, and flows** are migrating.

---

## 1. Why migrate, not rebuild

The current app has **UI unit tests and e2e tests** that encode real product
behavior (validation, role gating, auth flows, error paths). A from‑scratch
rebuild silently drops that coverage. So:

- **Migrate** each existing screen's markup/logic onto the new shell + design,
  and **migrate its tests in lockstep** (update selectors/DOM, keep assertions).
- **Build from scratch only the genuinely new screens** — primarily the unified
  **Explore MCPs** catalog (there is no single existing page that maps to it).
  New screens get new tests written to match the behavior specified here.
- After each screen, run its test suite. A migrated screen with red tests means
  a behavior regressed — fix the screen, don't weaken the test.

---

## 2. Page inventory & information‑architecture mapping

### 2.1 The headline IA change
The current app's separate **"My MCPs"** (a user's instances) and **"My MCP
Servers"** (the admin's server registry) list pages are **subsumed into one
unified Explore catalog**. In v2 there is a single list‑+‑detail‑rail component
(`DiscoverApp`) that renders every server — registered or not — and drives
*all* per‑server actions (connect, request, configure, manage instances) from
the **right detail rail**. "My MCPs" survives only as a **filtered mode** of
that same component, not as a separate UI.

### 2.2 Screen map

| New design file | Route intent | `window` flag | Maps from (current app) | Migrate vs new |
|---|---|---|---|---|
| `Bodhi MCP Discover v2.html` | **Explore MCPs** (full catalog + rail) | `MCP_MODE='explore'` | *(new — no 1:1 source)* | **New build** |
| `Bodhi MCP My MCPs.html` | **My MCPs** (registered‑only view of the same component) | `MCP_MODE='my-mcps'` | "My MCPs" + "My MCP Servers" list pages | **New build** (folds 2 pages → 1 mode) |
| `Bodhi MCP New Server.html` | Register a server (admin) | `MCP_SERVER_PAGE='create'` | New MCP Server form (`/ui/mcps/servers/new`) | **Migrate** (forms match closely) |
| `Bodhi MCP Server.html` | Configure server hub (admin) | `MCP_SERVER_PAGE='view'` | Server view/edit (`/ui/mcps/servers/view?id=`) | **Migrate** |
| `Bodhi MCP New Instance.html` | Create / edit a personal instance | — (URL params) | New Instance form | **Migrate** |
| `Bodhi MCP Playground.html` | Try an instance's tools | — (URL params) | Playground / tool runner | **Migrate** |

### 2.3 Navigation (`SHELL_NAV`, section `mcp`, in `shared/shell-core.jsx`)
Sub‑pages, in order: **My MCPs** (`my-mcps`) · **Explore MCPs** (`explore`) ·
**New MCP Server** (`new-server`, admin) · **New MCP Instance** (`new-mcp`).
`Configure Server` and `Playground` are **not** nav entries — they are reached
contextually (see §6). Fix nav/hrefs in `SHELL_NAV` once, never per page.

> **Decision to confirm with product:** the stated intent is "My MCPs + My MCP
> Servers → Explore." The prototype still keeps **My MCPs** as a distinct nav
> item (filtered mode). Either keep it as a convenience filter, or drop the nav
> entry and rely on Explore's Scope filter. Default: **keep both nav entries**,
> since they share one component at near‑zero cost.

---

## 3. Domain model (the data contract behind every screen)

> Mirrors the backend. In the prototype this lives in `mcp/mcp-catalog.jsx`
> (`CATALOG`), the **single source of truth** every page derives from. The real
> app already has these types — this section is so you can recognize which field
> drives which piece of UI.

```
server ──1:*── auth‑mechanism ──1:*── instance
```

### 3.1 Server
| Field | Meaning / UI effect |
|---|---|
| `id`, `name`, `publisher`, `verified` | identity; `verified` → badge‑check icon |
| `icon`, `iconBg`, `iconColor` | square avatar (publisher mark) |
| `category` + `catClass` | category tag (filterable) |
| `desc` | rail Description section |
| `url`, `transport` | rail Server spec; `transport` ∈ `streamable-http` / `sse` / `stdio` |
| `registered` | **`true` = saved in our DB** (a known/added server). Drives almost every branch. |
| `disabled` | server turned **off for the whole workspace** (admin kill‑switch). Beats everything. |
| `requestStatus` | only meaningful when **`!registered`**, for **non‑admins**: `none` · `pending` · `rejected` |
| `auth` | array of supported auth *types* for badges: `oauth` / `key` / `none` |
| `authConfigs[]` | the actual configured mechanisms (see §3.2) |
| `userInstances[]` | the current user's instances of this server (see §3.3) |

### 3.2 Auth mechanism (`authConfigs[]` item)
A server may expose **several** mechanisms, even several of one type. Union of
fields across types:
- **All:** `type` (`oauth`|`key`|`none`), `name` (internal id, e.g.
  `oauth-default`, `header-default`), `detail` (human summary string).
- **`oauth`:** `regType` (`dcr` | `pre`), `authEndpoint`, `tokenEndpoint`,
  `scopes`.
- **`key`:** `injectVia` (`header` | `query`), `keyName` (e.g. `Authorization`,
  `x-api-key`), `keyPlaceholder`.
- **Public** is special: it is **always available with no DB row**. Represented
  by the synthetic `PUBLIC_AC = { type:'none', name:'public', builtin:true }`.
  Never stored, never deletable.

**Ordering rules matter (two different orderings):**
- `availableAuth(s)` → **Public FIRST**, then explicit mechanisms. Used by the
  rail's "Connect with" list.
- `connectMechs(s)` → explicit mechanisms first, **Public LAST**. Used by the
  New‑Instance auth dropdown.

### 3.3 Instance (`userInstances[]` item)
A user's personal connection to a server via one mechanism.
| Field | UI effect |
|---|---|
| `id`, `name` | identity; `name` shown in rail + playground |
| `status` | `connected` (usable) or `pending` (OAuth not yet authorized → "Authorizing…") |
| `authType` (`oauth`/`key`/`none`) + `authName` | which mechanism it uses |
| `time` | relative "last used / created" label |

### 3.4 Derived selectors (replicate these, don't hand‑maintain copies)
- `KNOWN_SERVERS` — registered servers keyed by id → the **Configure** page.
- `CONNECTABLE_SERVERS` — `registered && !disabled` → the **New Instance**
  server picker (you can only create an instance against a live registered
  server).
- `toolsFor(id)` — tool specs per server → **Playground**.

---

## 4. Roles: `user` vs `admin`

Almost every MCP surface gates on role. **The real app derives role from the
session/authz**, not from UI.

> ⚠️ **Prototype‑only artifact — do NOT migrate:** the prototypes include a
> header **"Role: User / Admin" toggle badge** (`headerActions` in
> `bodhi-mcp-discover-app.jsx`) purely to demo both roles in one static file. In
> production, **remove the toggle**; read role from auth context and render the
> correct branch. Keep the *branches*, drop the *switch*.

Role differences, at a glance (full detail in §5–§7):
- **Admin** can register/configure/disable servers, add/delete auth mechanisms,
  and still personally connect + use instances.
- **User** can connect to and use **registered** servers, manage their own
  instances, and **request** servers that aren't registered. Users never see
  server‑registry actions.

---

## 5. Explore / My MCPs — unified list + detail rail  ⟵ the core screen

Files: `bodhi-mcp-discover-app.jsx` (root + state), `mcp-discover-cards.jsx`
(rows + sidebar filters), `mcp-discover-detail.jsx` (the rail). Three columns:
**sidebar filters · list · detail rail**, on `AppShell` (`section="mcp"`,
`resizeKey="mcp"`, `railWidth≈380`).

### 5.1 List rows are **status‑only**
A row (`McpRow`) shows: avatar · name + publisher(+verified) · category tag ·
auth badges · **status line** · chevron. **Rows contain NO actions.** Clicking a
row only selects it and opens the rail. *Every* action lives in the rail. (On
mobile, selecting a row opens the rail as a drawer via `useShell().openRail()`.)

Row left‑border accent (`statusClass`): disabled → muted; registered w/
connected instance → green; unregistered w/ `requestStatus==='pending'` → amber.

`StatusLine` text by state (this is an **indicator, not a button**):
- `disabled` → "Disabled by admin"
- `!registered` & pending → "Approval pending"; & rejected → "Request declined";
  else → **admin:** "Not configured" / **user:** "Not in this workspace"
- registered & N connected → "N instance(s)"; else if any pending →
  "Authorizing…"; else → "Available"

### 5.2 Sidebar filters
- **Explore mode:** Category (functional, single‑select) · Auth Type ·
  Availability · Publisher. (Only Category is wired in the prototype; the rest
  are visual placeholders — in the real app wire them to the list query.)
- **My MCPs mode:** **Scope** (single‑select: `all`→"Configured",
  `mine`→"Connected") · Category · Auth Type.
  - `my-mcps` base list = servers where `registered` is true.
  - Scope `mine` ("Connected") further filters to servers where
    `userInstances.length > 0`.

### 5.3 Right detail rail — **full rendering matrix** (the #1 hidden context)

The rail (`DetailPanel`) always renders **Description** + **Server spec**
(URL, transport, publisher, supported‑auth badges). Below that, the **body**
and the **footer** branch on `disabled → registered → role → requestStatus`.
**Footer is omitted entirely when there's no footer action** (see the one
"none" case).

| # | Server state | Role | Rail **body** (below spec) | Rail **footer** button(s) |
|---|---|---|---|---|
| 1 | `disabled` | any | "Disabled by admin" note | "Unavailable" (disabled). **Admin also gets** "Configure server" (ghost) |
| 2 | `!registered` | **admin** | "Not configured yet — Register this server to let users connect. The URL is pre‑filled for you." | **"Connect Server"** (lotus) → New Server, **pre‑filled** url+name |
| 3 | `!registered`, `requestStatus='none'` | user | "Not in this workspace yet" note | **"Request this server"** (lotus) → sets status `pending` |
| 4 | `!registered`, `requestStatus='pending'` | user | "Approval pending" note | "Pending approval" (disabled) |
| 5 | `!registered`, `requestStatus='rejected'` | user | "Request declined" note | **"Request again"** (lotus) → re‑request |
| 6 | `registered` (configured) | **admin** | **My Instances** + **Connect with** | **"Configure server"** (indigo) → Configure page |
| 7 | `registered` (configured) | user | **My Instances** + **Connect with** | **none** — user connects *inline* via Connect‑with rows (footer not rendered) |

Key non‑obvious points:
- **Admin on a registered server still sees My Instances + Connect with** (admin
  can personally use the server too) — the "Configure server" footer is *added*,
  it doesn't replace the connect UI.
- For an **unregistered** server, the body shows **only** the note (no instances,
  no connect) regardless of role.
- The header‑bar **"x"** (`DiscoverRailHeader`) clears selection (`setActiveId(null)`)
  and, with no active server, the rail isn't rendered at all.

### 5.4 "My Instances" block (`MyInstances`)
Rendered only when `userInstances.length > 0`. Per instance row:
- status dot (`connected` green / `pending` amber) · name · "OAuth|API Key|Public · `authName`".
- Actions: if `connected` → **Play** button → Playground (`goToPlayground`);
  if `pending` → "Authorizing" chip (no play). Always: **Edit** (pencil →
  `goToEditInstance`, server locked) and **Delete** (trash → removes instance;
  in the real app: confirm + API delete).

### 5.5 "Connect with" block (`ConnectWith`)
Lists `availableAuth(s)` = **Public first**, then each explicit mechanism. Each
row → `goToNewInstance(s.id, ac.name)` = New Instance form **pre‑selected** to
that server + mechanism. Subtitle: Public → "Always available · no setup";
others → "`name` · `detail`". This is how a **user connects without a footer
button**.

### 5.6 Request flow (non‑admin, unregistered) (`RequestPanel`)
Body note varies by `requestStatus` (none/pending/rejected, §5.3 rows 3–5).
The footer action mutates `requestStatus` (`none|rejected → pending`). In the
real app this is a **POST "request server"** that notifies an admin; reflect the
returned status and disable the button while pending.

---

## 6. Cross‑page navigation & **form pre‑fill contracts** (the #2 hidden context)

All cross‑page jumps are centralized in `mcp-catalog.jsx`. These query‑param
contracts are what make "click an action button → arrive on a pre‑filled form."
**Preserve the param names exactly** — the destination pages parse them on load.

| Helper | Navigates to | Query params | Destination pre‑fills |
|---|---|---|---|
| `goToNewServer({url,name})` | `Bodhi MCP New Server.html` | `?url=&name=` | URL + Name fields (admin registering a discovered server) |
| `goToViewServer(id)` | `Bodhi MCP Server.html` | `?server=<id>` | loads `KNOWN_SERVERS[id]` into Configure hub |
| `goToNewInstance(id, authName)` | `Bodhi MCP New Instance.html` | `?server=<id>&auth=<mechName>` | server selected + auth dropdown set to that mechanism |
| `goToEditInstance(inst, id)` | `Bodhi MCP New Instance.html` | `?instance=&edit=1&server=&name=&auth=` | **edit mode**: server **locked**, name/auth/desc loaded |
| `goToPlayground(instId, name, id)` | `Bodhi MCP Playground.html` | `?instance=&name=&server=<id>` | connects + loads that server's tools |

Destination‑side parsing to replicate:
- **New Instance** reads `server`, `auth`, `name`, `edit`, `instance`. Resolves
  `auth` against `connectMechs(server)` by **mechanism name first, then type**,
  falling back to the first mechanism. `edit=1` locks the server combobox.
- **New Server / Configure** reads `name`, `url` (create) or `server` (view).
- **Playground** reads `instance`, `name`, `server` (defaults if absent).

> When wiring to the real router, keep these as the canonical entry params even
> if the URLs change shape — the *pre‑fill semantics* are the contract, and the
> existing e2e tests almost certainly assert on them.

---

## 7. New MCP Server (create) + Configure Server (view) — **admin only**

One component file, two modes via `window.MCP_SERVER_PAGE`
(`bodhi-mcp-server-app.jsx`). Both are centered forms on `AppShell`
(`bodhi-form.css` `.bf-*` primitives).

### 7.1 Create (`MCP_SERVER_PAGE='create'`)
Fields: **URL\*** · **Name\*** · Description · **Enabled** toggle (default on) ·
a collapsible **"Authentication Configuration (Optional)"** holding one inline
`AuthBlock` with `allowNone=true` (default type **None/Public**). URL+Name
pre‑fill from `?url=&name=` (the admin "Connect Server" path, §6). Save validates
URL+Name non‑empty, then returns to Explore. Hint copy: *public is always
available; configure OAuth/key only if the server requires it; more mechanisms
can be added later from the server page.*

### 7.2 `AuthBlock` — the auth editor + **DCR discovery flow** (subtle!)
Used both in Create and in Configure's inline "add mechanism."

- **Type order is deliberate: `Header / Query Params` FIRST, then `OAuth`**
  (and `None (Public)` only when `allowNone`). Default selected type is
  **key** (or `none` when allowed) — **never OAuth by default**, because…
- **Dynamic Client Registration (DCR) fires only when OAuth is selected**
  (`useEffect` on `type==='oauth'`), never on load:
  - **discovering** → spinner "Discovering via dynamic client registration…"
  - **discovered** (host supports DCR) → success banner; auto‑filled Client ID +
    Authorization/Token endpoints; **Registration Type = Dynamic**; editable;
    link "Enter client details manually instead."
  - **manual** (DCR failed, or no URL to discover from) → warn/info banner;
    Registration Type dropdown (**Pre‑Registered** default) with Client ID,
    Client Secret (optional), Auth/Token endpoints, Scopes; "Retry dynamic
    discovery" link (hidden when there's no URL).
  - *Prototype detail:* `attemptDcr(url)` decides success by hostname regex —
    in the real app this is the **actual DCR network call**; keep the same three
    visual phases and the manual fallback.
- **Header/Query (`key`) type:** a repeatable **Key Definitions** list — each row
  = via (`Header`/`Query`) + key name; "Add Key"; delete disabled at one row.
- **None (Public):** a "Public access — no authentication required" note; no
  fields.
- The block reports a summary (`onSummary`) up to the parent for the "add
  mechanism" save.

### 7.3 Configure server (`MCP_SERVER_PAGE='view'`, `?server=<id>`)
Title "Configure server." **Two independently‑editable sections:**

1. **Basic information** — read view (Name, URL, Description, Status) with an
   **Edit** button → inline form (URL **locked** — it's the server identity;
   Name\*, Description, Enabled). Save shows a transient "Saved" check. Cancel
   reverts. Per‑section editing (not a whole‑page form).
2. **Auth mechanisms** — list of existing mechanisms (icon, label, mono `name`
   or "Built‑in" for Public, `detail`). **Mechanisms can be deleted but NOT
   edited** — delete (with a **confirm dialog** warning that existing user
   instances using it will break) and **re‑add** to change one. **Add is
   inline:** the "Add auth mechanism" button is replaced *in place* by an
   `AuthBlock` (`allowNone=false`) + Save/Cancel; Cancel restores the button.
   Public is always present and not deletable.

Footer: "Back to My MCPs" (no global save — each section saves itself).

> **Divergence to flag during migration:** the current app's server view appears
> to allow **editing** an existing mechanism inline (e.g. the `header-default`
> edit form in the screenshots). The v2 model is **delete‑and‑re‑add, no edit.**
> Confirm with product, and **update the affected tests** accordingly (an
> "edit mechanism" test must become "delete + add").

---

## 8. New / Edit MCP Instance (form)

File: `bodhi-mcp-new-instance-app.jsx`. Centered form on `AppShell`.

- **Modes:** create (`?server=&auth=`) vs **edit** (`?instance=&edit=1&server=&name=&auth=`).
  Edit **locks the server** combobox and changes titles/buttons ("Edit MCP
  Instance" / "Save Changes" vs "New MCP Instance" / "Create MCP").
- **Server combobox** (`ServerCombobox`): searchable list of
  `CONNECTABLE_SERVERS` (registered & enabled only). Shows each server's primary
  auth badge + "+N" if it has more. Locked (display‑only) in edit mode.
- **Instance Details:** **Name\*** → auto‑derives **Slug\*** (`slugify`) until
  the user edits slug manually (`slugDirty`); Description (optional); **Enable
  MCP** toggle (default on).
- **Auth Configuration** (`AuthSection`): dropdown = `connectMechs(server)`
  (**Public LAST**). Selecting a mechanism renders a detail box by type:
  - **oauth** → meta (config/type/auth‑server host) + **Connect** button that
    runs the OAuth authorize flow. *Prototype simulates* idle→connecting→connected
    with a timeout + a "Revoke" affordance; the real app does a true OAuth
    redirect and reflects the granted/again states.
  - **key** → meta (config, inject‑via) + a masked value input for `keyName`
    with show/hide toggle and the mechanism's placeholder.
  - **none** → "no authentication required" public card.
- **Validation:** Name + Slug required (and a server selected) to enable submit;
  empty name/slug flags the field. Submit (create) navigates to the new
  instance's Playground.

---

## 9. Playground

File: `bodhi-mcp-playground-app.jsx`. Reached via `goToPlayground` / "Play" on a
**connected** instance. `?instance=&name=&server=`.

- **Connection lifecycle:** mounts "connecting" → after a beat "connected"
  (status pill in header: Connecting… ↔ Connected). Header actions: **Reconnect**,
  Settings, **Execution log** toggle. The exec log is a bottom console of
  timestamped info/success/error lines.
- **Layout:** left **Tool list** (search + count, from `toolsFor(serverId)`) ·
  right **Tool detail**.
- **Tool detail:** name + description; **Form** tab (one input per param, with
  type badge + required `*` + hint) and **JSON** tab (raw args, with a generated
  placeholder of required keys). **Execute** validates required params (Form
  tab), then runs (mock latency) and fills the **Result** area.
- **Result area** tabs: **Response** (syntax‑highlighted) · **Raw JSON** ·
  **Request** (the args sent); plus copy. Empty/`running` states handled.
- Tools without params show "This tool takes no parameters" and no Clear button.

> Prototype responses are `tool.mockResponse`; wire Execute to the real MCP
> tool‑call. Keep the Form/JSON tabs, the three result tabs, and the exec‑log
> semantics — existing tests likely assert on them.

---

## 10. Status & visual vocabulary (keep consistent)

- **Auth badges** (`AUTH_META`): OAuth → lock icon, indigo; API Key → key icon,
  saffron; Public → unlock icon, leaf/green. One badge per type; row shows all
  supported types; "+N" condenses extras.
- **Category tags:** Productivity=lotus, Dev Tools=indigo, Search & Web=saffron,
  Browser=teal, Data=leaf, Comms=indigo, Memory=neutral.
- **Status semantics / accent colors:** connected = green (`--c-connected-*`),
  pending/authorizing = amber (`--c-pending-*`), rejected/declined = saffron,
  disabled = muted. These connected/pending tokens are **MCP‑page‑local** (not
  in the shared shell palette) — declare them in the page CSS as the prototype
  does.
- **Transport labels:** `streamable-http`→"Streamable HTTP", `sse`→"SSE
  (deprecated)", `stdio`→"stdio".
- Icons via Lucide (`ShellIcon`/`Ic`). Verified publisher = `badge-check`.

---

## 11. Prototype simulation → real behavior (wire these to APIs)

| Prototype shortcut | Real implementation |
|---|---|
| Header **Role: User/Admin toggle** | **Remove.** Role from session/authz; render branch accordingly. |
| `CATALOG` / `KNOWN_SERVERS` / `CONNECTABLE_SERVERS` mock data | List/detail API queries + the real domain types. |
| `attemptDcr(url)` hostname regex | Real **DCR** call to the server; same 3 phases + manual fallback. |
| OAuth **Connect** `setTimeout` → connected | Real OAuth **authorize redirect** + token exchange + revoke. |
| `requestServer` mutating `requestStatus` | POST "request server" → admin notification; reflect returned status. |
| Delete instance / delete mechanism (local splice) | API delete + confirm; mechanism delete warns about dependent instances. |
| Playground `mockResponse` + fake latency | Real MCP tool invocation; show real response/request/errors. |
| Form submits navigating after a timeout | Real create/update calls; navigate on success, surface server errors. |

---

## 12. Divergences from the current app (call out + reconcile tests)

1. **My MCPs + My MCP Servers → one Explore catalog** (§2.1). The old two list
   pages collapse into one component; admin server‑registry actions move into
   the **rail**, not a separate page. Re‑map/retire the old list‑page tests onto
   Explore + rail interactions.
2. **All row actions moved into the rail** (§5.1). Tests that clicked
   per‑row buttons must target the rail instead.
3. **Auth mechanism is delete‑and‑re‑add, not editable** (§7.3). Adjust any
   "edit mechanism" tests.
4. **DCR is auto‑attempted on selecting OAuth** rather than via an explicit
   "discover" action / manual Registration‑Type first (§7.2). Validate the
   current app's flow and update.
5. **Role is not a UI control** (§4, §11) — remove any test that toggles a
   role switch in the DOM; drive role via auth fixtures/mocks instead.

For each: prefer **adapting** the existing assertion to the new DOM over deleting
it. Only delete a test if the behavior it covers genuinely no longer exists
(and note why).

---

## 13. Migration checklist (per screen)

- [ ] New screen renders a single `<AppShell>` with correct `section`/`subPage`/
      `breadcrumb` (see `SHELL_NAV`); no console errors.
- [ ] All state branches from the matrices in §5 / §7 reachable and correct for
      **both roles** and **configured + not‑configured + disabled** servers.
- [ ] Cross‑page pre‑fill contracts (§6) intact — param names unchanged,
      destinations parse + pre‑fill + lock as specified.
- [ ] Mock simulations replaced with real API calls (§11); loading/error states
      preserved.
- [ ] Existing unit + e2e tests migrated (selectors/DOM updated, assertions
      kept); divergences in §12 reconciled; suite green.
- [ ] Responsive: rail behaves as drawer < 768px (`openRail` on row select);
      collapse‑to‑icon‑rail + hover‑resize inherited from `AppShell`.

---

### Quick file index (prototype → what to read)
- Data model & nav helpers → `mcp/mcp-catalog.jsx`
- Explore/My MCPs root + state → `mcp/bodhi-mcp-discover-app.jsx`
- Rows + sidebar filters → `mcp/mcp-discover-cards.jsx`
- **Detail rail (state matrix)** → `mcp/mcp-discover-detail.jsx`
- Server create/configure (admin) → `mcp/bodhi-mcp-server-app.jsx`
- New/Edit instance → `mcp/bodhi-mcp-new-instance-app.jsx`
- Playground → `mcp/bodhi-mcp-playground-app.jsx`
- Shell/nav (don't fork) → `shared/shell-*.jsx`, `shared/bodhi-app-shell.css`
