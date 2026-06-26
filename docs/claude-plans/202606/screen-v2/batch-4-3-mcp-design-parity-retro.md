# Batch 4-3 — MCP design-parity final pass — Retrospective

Status: **complete, V2-only (no flag), live-verified + RTL + E2E (standalone + multi_tenant).** This is the
**final** MCP pass — every MCP page now matches the hi-fi prototypes. Follows 4-2
(`batch-4-2-mcp-screens-retro.md`).

## What landed

The 4-2 screens shipped functionally but diverged from the hi-fi prototypes (`design/mcp/*.jsx`,
`Bodhi MCP Discover v2.html` / `Bodhi MCP Server.html`). This pass closes the gaps:

**Rails (Explore + My MCPs).** Rebuilt to the prototype's `mcp-discover-detail.jsx` shape:
- **Removed the rail Status section** (the list STATUS column stays — matches the prototype).
- Renamed Connection → **Server** spec (URL / Transport `Streamable HTTP` / Publisher / **Supported-auth
  badges**). Supported-auth = the reference-API `auth_type` (Explore) or the unique kinds of the
  configured auth-configs + always-Public (My MCPs), rendered via a new shared `auth-badges` module.
- **Connect-with** rows gained an **icon tile** (lock/key/unlock by kind) and **Public is now FIRST**
  (flipped `buildAuthMechanisms` to mirror the prototype's `availableAuth`). Subtitle = `name · detail`.
- Unregistered catalog server: admin now gets a **"Connect Server" footer** (lotus button, `dp-btn-lotus`)
  + a "Not configured yet" note — replacing the old inline "Add this server" link. The deep-link is
  `/mcps/servers/new/?url=&name=&auth=<auth_type>`.

**New Server `?auth=` prefill.** The Connect-Server link forwards the catalog `auth_type`. The form maps
it: `oauth-dcr` → Auth section open, **OAuth + Dynamic Registration**, and the existing AuthConfigForm
auto-DCR effect discovers against the prefilled URL (graceful fallback to Pre-Registered on failure);
`oauth-pre-registered` → OAuth + Pre-Registered; `key` → Header. The reference-API `McpAuthType` enum now
ships these real values (`http | public | oauth-dcr | oauth-pre-registered | key`).

**Server view rebuilt → "Configure server" hub** (`bodhi-mcp-server-app.jsx` view mode, on shell tokens):
- One bordered card. **Basic information** = key/value read rows (Name/URL/Description/Status with a
  green ✓ Enabled pill) + an **Edit** button → **per-section inline edit** (URL **locked** as the server
  identity; Name/Description/Enabled) wired to `useUpdateMcpServer`, with a transient "Saved" check.
- **Auth mechanisms** = list with icon tiles; the **synthetic Public row is always shown** (Built-in,
  not deletable); explicit mechanisms show a mono config-name + detail + **delete** (delete-and-re-add,
  no edit, per the prototype). **"+ Add auth mechanism"** is an inline dashed button → AuthConfigForm in
  place. Footer: **"← Back to My MCPs"**.
- **`/mcps/servers/edit` now redirects to `/view`** — inline editing replaces the separate edit page
  (the prototype has no edit page). Route kept as a `beforeLoad` redirect for old deep-links.

## Decisions worth recording

- **Shell tokens, not a verbatim CSS port.** Per the user's choice, the prototype's `spec-*`/`acr-*`/
  `ns-*`/`bf-*` vocabulary was reproduced structurally on the existing `dp-*`/token system (new
  `auth-badges.css`, `server-config.css`, additions to `my-mcps.css`) — consistent with the shipped V2
  screens, no new design system.
- **`auth_type` is real now.** 4-1/4-2 treated `auth_type` as the always-`http` placeholder; the
  reference API was updated to emit `oauth-dcr` etc. The new `authKind()` collapses both oauth variants
  to the OAuth badge and keeps `http` as a neutral fallback, so old + new data both render.
- **Public-first ordering** is the one connect-with divergence from the New-Instance dropdown (which is
  Public-LAST); the rail's "Connect with" intentionally leads with Public per the prototype.

## Surprises / gotchas

- **Relative CSS import trap.** `./-components/server-config.css` from `servers/view/index.tsx` resolved
  to `view/-components/…` (404 in Vite). Use the `@/routes/mcps/servers/-components/…` alias for
  cross-directory CSS.
- **Auth-config kind label.** The view's auth rows label `header` configs as **"API Key"** (the user-
  facing kind), not "Header" — the RTL assertion was updated accordingly.
- **`auth_type` test fixture stays `http`** (the AUTH-column + facet tests assert it); the Connect-Server
  test overrides a catalog summary to `oauth-dcr` to exercise the realistic prefill.

## Verification

RTL: full UI suite **1202 pass / 5 skip / 0 fail** (explore-rail + server-view assertions migrated to the
new structure; New-Server `?auth=` covered). E2E: all **8 MCP specs + `chat/chat-mcps` + `explore-mcp`**
green on **standalone AND multi_tenant** — incl. the DCR "Add OAuth auth config via server view page"
test against the rebuilt hub. GATE B live (Claude-in-Chrome): Explore rail (registered + unregistered),
My MCPs rail, Connect-Server → New-Server prefill (`auth=oauth-dcr`, auto-discover), Configure-server hub
(read + inline Edit + inline Add-mechanism); console clean.

## Next

MCP is fully ported to screen-v2. Only **Batch 5 (Chat)** remains in the V2 migration.
