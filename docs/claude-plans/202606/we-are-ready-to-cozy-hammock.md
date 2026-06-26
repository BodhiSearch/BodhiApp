# Batch 4 (cont.) — MCP Screens V2 Migration — Plan

> Working folder: `docs/claude-plans/202606/screen-v2/`. Process: `process.md`. Prior MCP batch:
> `batch-4-mcp-explore-retro.md` (Explore·MCP catalog, shipped 4-1). Tech debt: `mcp-techdebt.md`.
> Design brief: `screen-v2/mcp-design-prompt.md` + `design/prompts/mcp-screen-v2.md`. Prototypes:
> `Bodhi MCP Discover v2.html`, `Bodhi MCP My MCPs.html` (served on :8000).

## Context

The MCP nav section is the last non-Chat section to migrate to the V2 shell. Today it is a mix:

- **Explore · MCP Servers** (`/ui/mcps/explore/`) — already shipped V2 (Batch 4-1), a reference-API
  catalog browser (CatalogTable + rail). Its rail only shows Status/Description/Connection + a
  prefill-less "Add to My MCPs" deep-link. **Keep + upgrade**, don't rebuild.
- **My MCPs** (`/ui/mcps/`) — V1: a flat list of the user's *instances* (DataTable + `McpManagementTabs`).
- **My MCP Servers** (`/ui/mcps/servers/`) — V1: the admin's registered-server allowlist (second tab).
- **Forms** — all V1 shadcn-Card pages: New/Edit Instance (`/mcps/new/`), New Server
  (`/mcps/servers/new/`), View/Configure + Edit Server (`/mcps/servers/view|edit/`), Playground
  (`/mcps/playground/`).

The new design unifies the two list pages into **one server-centric list+detail-rail component** with
two modes. Per the design brief and confirmed decisions, the rail surfaces all per-server actions
(My Instances, Connect-with, admin Configure) as **deep-links to the existing forms** — no inline form
in the rail. This is the same list+rail+keyboard-nav pattern already shipped for My Models / Explore
Models, so reuse is high.

**Decisions locked (from the user, this session):**
1. **Page→data mapping:** *Explore MCPs* = the reference catalog (`/api/v1/mcp-servers`, ~198 public
   servers). *My MCPs* = the **registered servers** (`/bodhi/v1/mcps/servers`) joined with the user's
   instances (`/bodhi/v1/mcps`). Both render through one `mode`-parameterized list+rail component.
2. **Rail actions = deep-links, NO inline rail form.** Admin "Configure server" → server form (url
   pre-filled); "Connect with <mechanism>" → New Instance form (server+auth pre-selected); My-Instances
   rows → playground / edit / delete (delete is the one inline mutation, with confirm).
3. **Request-server-approval flow and `verified` flag are DEFERRED** (no backend exists; design brief
   calls request-flow a *future* want, admin-approval out of scope). Non-admin on an unregistered
   catalog server → informational note only (no Request button). `verified` badge only if the reference
   API serves it (it already does on `McpServerSummary.verified`).
4. **Phased, V2-only, NO feature flags, commit per phase** (matches batches 3-2/3-3/3-4/4-1). The
   already-registered `mcp-discover`/`new-mcp`/`mcp-playground` flags were never built against — they
   will simply be removed from `lib/uiV2Flags.ts` at the end (no code gates on them).

**Backend:** no changes required. All needed surface exists — server/instance/auth-config CRUD,
OAuth login/token/discovery, DCR, enabled/disabled (server + instance), proxy/playground.

## Validation corrections (verified against code — do NOT assume otherwise)

- **There is no `bf-*` form CSS kit.** The migrated-form V2 pattern (batch 3-2 `ApiModelForm`) is just
  `useShellChrome({ breadcrumb })` + a `container mx-auto max-w-3xl px-4 py-6` wrapper around the
  existing react-hook-form/shadcn-Card body. P1 = **chrome swap only**; keep every form body as-is.
- **`chat/chat-mcps.spec.mjs` is a hidden P2 dependency** — it imports `McpsPage` and drives the V1
  list↔form flow. It must be migrated when P2 rewrites `McpsPage.mjs`.
- **OAuth survival linchpin:** `mcpFormStore.saveToSession()` snapshots the live URL into `return_url`;
  `oauth/callback` returns there. A deep-linked `/mcps/new/?server=&auth=` round-trips for free **only
  if the form does not strip its prefill params on mount**. Confirm restored session
  (`mcp_server_id`/`selected_auth_config_id`) wins over URL so a returning user lands connected.
- **Standardize the instance-edit param on `id`** (existing). Do NOT introduce `edit`/`instance`
  aliases — `useGetMcp(editId)` and `mcpFormStore.mcp_id` already key on `search.id`.
- **Catalog id ≠ registered `mcp_server_id`.** Explore rows are catalog servers; the registered id only
  exists when the catalog `endpoint_url` matches a registered server (via `instance-join.ts`). Connect
  deep-links need the registered id.

---

## Phase 1 — Migrate the 5 forms to the V2 shell (chrome only)

Goal: every MCP form renders inside `<AppShell>` with a breadcrumb, bodies unchanged. Define the
deep-link prefill contracts that P2/P3 rails consume.

**Modify:**
- `crates/bodhi/src/routes/mcps/new/index.tsx` (New/Edit Instance) — swap the `max-w-2xl` Card-header
  chrome for `useShellChrome({ breadcrumb })` + `container mx-auto max-w-3xl …` wrapper. **Extend
  `validateSearch`** `{ id }` → `{ id, server, auth }`; on create-mode mount, preselect server from
  `?server=` and auth-config from `?auth=` (`public` → the `__public__` select value). Keep the whole
  body: `mcpFormStore`, `McpServerSelector`, `HeaderCredentialsFields`, `OAuthConnect*`, the
  auto-select-first-auth effect (now "select the `auth` from URL if present").
- `crates/bodhi/src/routes/mcps/servers/new/index.tsx` (New Server) — `useShellChrome` breadcrumb;
  **add `validateSearch: { url, name }`** and seed the `url`/`name` `useState` from search on first
  render. Keep `AuthConfigForm` + DCR body.
- `crates/bodhi/src/routes/mcps/servers/view/index.tsx` + `servers/edit/index.tsx` (Configure/Edit
  Server) — `useShellChrome` breadcrumb; `{ id }` already validated. Keep bodies (view already does the
  v2 "add + delete auth-config, no edit" model).
- `crates/bodhi/src/routes/mcps/playground/index.tsx` — replace the ad-hoc `border-b` header bar with
  `useShellChrome` breadcrumb; reconcile its `h-[calc(100vh-4rem)]` layout with the shell content
  region. Keep `useMcpClient` + `ToolSidebar`/`ExecutionArea`/`FormInput`/`ResultSection`.

**Reuse:** `useShellChrome` (`@/components/shell`); all `@/hooks/mcps/*` unchanged; `useMcpFormStore`
unchanged; all `-components/` sub-components unchanged.

**Prefill contracts (canonical, parsed by destinations):**
- New Server: `/mcps/servers/new/?url=<endpoint_url>&name=<name>`
- New Instance: `/mcps/new/?server=<mcp_server_id>&auth=<auth_config_id|public>`; edit: `/mcps/new/?id=<instance_id>`

**Tests:** update unit tests `routes/mcps/new`, `servers/new`, `servers/view`, `playground`
`index.test.tsx` (shell-chrome render + new prefill cases). E2E unchanged this phase — form testids
preserved (`mcp-server-url-input`, `mcp-name-input`, `auth-config-select`, …) so `McpsPage.mjs` keeps
working. **Gate B (live) + commit.**

---

## Phase 2 — My MCPs V2 (server-centric list+rail); delete V1 list/servers/tabs

Goal: `/ui/mcps/` becomes the V2 server-centric list+detail-rail; the two old list pages + tabs are
deleted.

**Create / rewrite:**
- `crates/bodhi/src/routes/mcps/index.tsx` — rewrite to the V2 screen, modeled on the structural twin
  `routes/mcps/explore/-components/ExploreMcpScreen.tsx`: `useShellChrome({ breadcrumb, sidebar, rail,
  railHeader, railDefaultOpen:false })`, `useListKeyNav`, `CatalogTable`, `ShellSearch`,
  `ShellPagination`, `useViewTransition`, `select`-in-URL via `validateSearch`.
- `crates/bodhi/src/routes/mcps/-components/MyMcpsScreen.tsx` + `MyMcpsRail.tsx` (+ `MyMcpsSidebar.tsx`
  with the **Scope** facet: Configured / Connected per the prototype). Ship as a **sibling** to Explore
  in P2; the `mode`-parameterized unification happens in P3 (when Explore's rail converges) to avoid two
  big rewrites landing together.

**My MCPs data model:**
- Rows: `useListMcpServers()` (registered servers) — server-centric.
- Instances: `useListMcps()`, grouped by **`mcp_server_id`** (FK on the instance — more robust than the
  URL match `instance-join.ts` uses for the catalog). Row chip = instance count / install state.
- Per-selected-server (lazy in rail): `useListAuthConfigs(serverId)`.
- Sidebar Scope facet: `all` ("Configured") = all registered servers; `mine` ("Connected") = servers
  with ≥1 instance.
- **Rail sections** (selected server):
  - *My Instances* — grouped instances → playground (`/mcps/playground/?id=`), edit (`/mcps/new/?id=`),
    delete (inline `useDeleteMcp` + confirm dialog, mirroring the V1 delete).
  - *Connect with* — mechanisms = server auth-configs (header/oauth) **+ synthetic `public`**; each a
    `LinkRow` → `/mcps/new/?server=<id>&auth=<configId|public>`.
  - *Admin only* (`useGetUser()` + `isAdminRole`) — "Configure server" → `/mcps/servers/view/?id=<id>`.

**Delete:**
- `crates/bodhi/src/routes/mcps/servers/index.tsx` (V1 server list)
- `crates/bodhi/src/components/McpManagementTabs.tsx`
- old V1 body of `routes/mcps/index.tsx`; `routes/mcps/index.test.tsx` (replace with `index.v2.test.tsx`)

**Modify:**
- `crates/bodhi/src/components/shell/shell-nav-config.tsx` (MCP group): relabel `my-mcps` → "My MCPs";
  no "MCP Servers" sub-page (servers live inside the My MCPs rail). Keep `explore-mcp` + `new-mcp`.
- Keep `servers/new|view|edit` routes (deep-link targets) + `ROUTE_MCP_SERVERS`; only the *list* route
  dies.

**Shared helper (small, data-only reuse):**
- `crates/bodhi/src/routes/mcps/-shared/auth-mechanisms.ts` —
  `buildAuthMechanisms(authConfigs) → {id|'public', label, type}[]`, consumed by both the rail's
  "Connect with" and the New Instance form's `authConfigOptions`.

**Tests:**
- Unit: `routes/mcps/index.v2.test.tsx` (model on `explore/index.v2.test.tsx`).
- Page object: rewrite `crates/lib_bodhiserver/tests-js/pages/McpsPage.mjs` (~613 lines — the bottleneck):
  shell-nav instead of `McpManagementTabs`; list→rail interactions instead of DataTable row buttons.
- E2E (migrate this phase): `specs/mcps/mcps-crud`, `mcps-header-auth`, `mcps-oauth-auth`,
  `mcps-oauth-dcr`, `mcps-auth-restrictions`, `mcps-mcp-proxy-everything`, `mcps-sdk-compat-everything`,
  **and `specs/chat/chat-mcps`**. Black-box only; settle-wait after facet/select before asserting.

**Gate B (live, **rebuild binary not needed** — no backend change) + commit.**

---

## Phase 3 — Upgrade Explore rail + unify the shared component

**Modify:**
- `crates/bodhi/src/routes/mcps/explore/-components/ExploreMcpRail.tsx` — add the *Connect with* /
  *Configure server (admin)* / *My Instances* sections. Replace the bare "Add to My MCPs" link with the
  contract deep-links. Resolve the **registered `mcp_server_id`** via the instance-join; unregistered
  catalog server → non-admin info note ("Not in this workspace — ask an admin"), admin → "Configure
  server" → `/mcps/servers/new/?url=<endpoint_url>&name=<name>`.
- Extract the `mode`-parameterized list+rail (fold MyMcps + Explore into one component now that both
  rails converge), per the generic/evolvable-design preference.

**Tests:** update `explore/index.v2.test.tsx`, `pages/McpExplorePage.mjs`, `specs/mcps/explore-mcp`.
**Gate B + commit.** Then update `tracker.md` (MCP section ✅; remove the 3 dead flags) + write
`batch-4-mcp-screens-retro.md` + (if Chat next) kick-off.

---

## Critical files
- `crates/bodhi/src/routes/mcps/index.tsx` — P2 rewrite (V2 list+rail; delete V1 body)
- `crates/bodhi/src/routes/mcps/new/index.tsx` — P1 chrome + `?server=&auth=&id=` prefill (OAuth `return_url` linchpin)
- `crates/bodhi/src/routes/mcps/explore/-components/{ExploreMcpScreen,ExploreMcpRail}.tsx` — twin source + P3 rail upgrade
- `crates/bodhi/src/components/shell/shell-nav-config.tsx` — nav after tab deletion
- `crates/bodhi/src/stores/mcpFormStore.ts` — sessionStorage OAuth round-trip (read-only reference)
- `crates/lib_bodhiserver/tests-js/pages/McpsPage.mjs` — P2 e2e page-object rewrite (the bottleneck)

## Risks / gotchas
- **OAuth sessionStorage survival** (highest): verify restored session beats URL prefill so a returning
  user lands connected, not reset.
- **`McpManagementTabs` deletion ripples** to every MCP page object + all 8 MCP specs + `chat-mcps`.
- **Catalog id ≠ registered id** in Explore deep-links — resolve via instance-join.
- **Role gating reads session** (`useGetUser` + `isAdminRole`), never a UI toggle — drop the
  prototype's "Role: User/Admin" switch entirely.
- **Playground full-height layout** vs the shell content grid — reconcile in P1.
- Run the **full E2E matrix** at each phase (shell/nav are app-wide). Known pre-existing failures
  (chat-resize, api-live-upstream, browser-extension) are not regressions — see `techdebt.md`.

## Verification (end-to-end, per phase)
1. `cd crates/bodhi && npm test` (RTL) after each phase.
2. `make test.e2e` from `crates/lib_bodhiserver/tests-js` (dev-server + Vite HMR; no ui-rebuild).
   P2/P3 run the migrated MCP + chat-mcps specs in standalone **and** multi_tenant.
3. **Gate B live** (`make app.run.live`, log in): for each migrated screen — interactions
   (select→rail, search, facets, deep-link prefill, OAuth connect round-trip, playground tool exec);
   **light + dark**; **responsive** (rail→drawer <768px); `read_console_messages` 0 errors. Walk both
   **admin and a non-admin** session for the rail role branches.
