# MCP wireframes — Discover / My MCPs / Admin / Playground

*Archive spec for the four MCP screens added in v30 (2026-04-19). Audience: Claude / AI coding agents first, developers second. Read alongside `shared-primitives.md` §4 (primitives) and §6 (conversation context) for the cross-cutting entity model.*

Wireframes live in `design/models-page/project/screens/`:
- `mcp-discover.jsx` · `my-mcps.jsx` · `mcp-admin.jsx` · `mcp-playground.jsx`

---

## 1. What these screens are

Four **separate** top-level tabs in the wireframe deck. BodhiApp's existing MCP production UX is three pages (`/ui/mcps/`, `/ui/mcps/servers/new/`, `/ui/mcps/new/`, `/ui/mcps/playground/`); this redesign keeps them separate but adds a **Discover** store layer in front so non-technical users can find and add an MCP server in one or two clicks.

The IA choice was user-picked: not a single MCP mega-page with a mode toggle (like we did for Models), but four dedicated tabs so each screen can evolve independently and match the role split (user vs admin vs everyone).

### Screen map

| Tab | Audience | Purpose | Variants |
|---|---|---|---|
| **MCP Discover** | Everyone | Browse Bodhi-curated catalog; one-click Add or Submit-for-Approval | 5 (Desktop · 2 overlay-form demos · Medium · Mobile) |
| **My MCPs** | User | Instances the user has connected | 3 (Desktop · Medium · Mobile) |
| **MCP Admin** | Admin / Manager | Registered servers + approval inbox | 3 (Desktop · Medium · Mobile) |
| **MCP Playground** | User | Exercise tools on a connected instance | 3 (Desktop · Medium · Mobile) |

**Why overlays only on Discover.** The user-picked scope: the only reason we need overlay chrome is to pop the server form or the instance form over the Discover grid (users find a server → click CTA → overlay opens). Admin and My MCPs reach the same forms via full-page routes. Playground never needs an overlay.

---

## 2. Entity model

Four entities — three in Bodhi's DB, one catalog-side.

| # | `kind` | Source of truth | Owned by | Visible in |
|---|---|---|---|---|
| 1 | `mcp-catalog-entry` | `api.getbodhi.app` curated directory | Bodhi (hosted) | Discover (always) |
| 2 | `mcp-server-registry` | Bodhi server DB | Admin | Admin (list); Discover (status derive) |
| 3 | `mcp-instance` | Bodhi server DB | User | My MCPs; Discover (inline on card) |
| 4 | `mcp-tool` | Fetched live from a connected instance | N/A (live) | Playground (sidebar); Discover drawer (Capabilities tab) |

### Derived card state

Every Discover card resolves to one of **5 states** computed from joining 1+2+3:

| State | Condition | User CTA | Admin CTA |
|---|---|---|---|
| `catalog-only` | No registry entry for this slug | `Submit for Approval` | `One-click Add to app` |
| `pending-approval` | User has submitted, admin not yet acted | `Request pending…` (disabled) | `Review request` |
| `approved` | Registry entry exists, user has no instance | `+ Add MCP Server` | `+ Add MCP Server` |
| `connected` | User has an instance | `View instance ↗` + inline mini-summary | `View instance ↗` |
| `disabled` | Registry entry exists but `enabled=false` | `Unavailable` (disabled) | `Re-enable` |

The CTA selection is codified in `mcpCardCta({state, role})` — the single source of truth. Do not scatter state-to-CTA logic across components.

### Card visual encoding

- `.state-connected` → leaf-soft background, solid leaf border — reads as "you own this one".
- `.state-pending-approval` → dashed border, warn-soft background — reads as "in-flight".
- `.state-disabled` → 58% opacity, paper background — reads as "currently unavailable".
- `.state-approved` / `.state-catalog-only` → default paper-2 + solid ink — neutral.

---

## 3. Card & detail metadata (Bodhi-curated schema)

Synthesised from research across 7 reference registries (browsed via Claude-in-Chrome during design): registry.modelcontextprotocol.io, smithery.ai, mcp.so, mcpmarket.com, hub.docker.com/mcp, composio.dev, gumloop.com/mcp.

### `mcp-catalog-entry` shape

```
{
  slug: 'notion',                          // stable key, user-editable default for instance slug
  name: 'Notion',                          // display name
  logo: 'N',                               // icon ref (wireframe uses letter; prod serves URL)
  publisher: 'Notion Labs',
  verified: true,                          // known-publisher badge
  category: 'productivity',                // one of MCP_CATEGORIES
  tags: ['oauth','featured','official'],
  short: '40-80 char one-liner',
  description: 'marketing paragraph',
  defaultBaseUrl: 'https://mcp.notion.com/mcp',
  transport: 'streamable-http',            // HTTP-streamable only for MVP
  authType: 'oauth2' | 'header' | 'query' | 'none',
  authConfig: {                            // shape varies by authType
    // oauth2:
    registrationType: 'dynamic' | 'metadata' | 'preregistered',
    authorizationEndpoint, tokenEndpoint, registrationEndpoint, scopes[]
    // header/query:
    keyDefinitions: [{placement, name, hint}]
  },
  tools: [{name, description, parameters[]}],   // live from MCP_TOOLS_FIXTURE in wireframe
  stats: {installCount, weeklyCalls, uptimePct, latencyP50Ms},
  links: {homepage, repository, license, docs},
  screenshots: []                          // deferred for wireframe
}
```

### Category taxonomy

9 curated categories (+ `all`): **Productivity · Search & Web · Browser · Dev Tools · Data · AI & Content · Memory · Comms · Finance**. Narrower than Docker Hub (10) or mcpmarket (16); wider than Smithery (7). Trade-off favoured recognisability for non-tech users over completeness.

---

## 4. Discover — field-by-field + state × CTA matrix

### Top filter chrome (Desktop)

Three rows:

1. **Hero search** — single text input: `🔍 Search MCP servers by name, publisher, or tag…`. Full-width minus the status-filter pill group on the right.
2. **Status filter** — pill group: `All · Approved · Connected · Not connected · Pending`. Single-select. Default `All`.
3. **Category chips** — `McpCategoryChipRow` — 10 chips (All + 9 categories). Single-select. Default `All`.

### Card anatomy (top to bottom)

```
┌───────────────────────────────────────┐
│ [N] Notion · Notion Labs ✓ · Prodctvty│  ← head
│ Search, read and write pages & DBs   │  ← short description
│ across your Notion workspace.         │
│ ⚒ 7 tools · ↓ 7.4k · 🔐 OAuth         │  ← metrics strip
│ ┌─────────────────────────────────┐   │  ← inline mini-instance
│ │ ◉ notion · connected   yesterday│   │     (only when state='connected')
│ └─────────────────────────────────┘   │
│ instance · yesterday   View instance ↗│  ← cta row
└───────────────────────────────────────┘
```

For non-connected states the inline strip is omitted; the CTA row hint text adapts:
- `'not yet in this app'` for `catalog-only`
- `'submitted 1d ago'` for `pending-approval`
- `'✓ admin-approved'` for `approved`
- `'Admin disabled'` for `disabled`

### Drawer (right pane, Desktop only)

Tabs: **About · Capabilities · Connection · Metadata · Performance**. Capabilities lists the first 4 tools inline with `{name, description}`, then `+ N more…`. Connection shows `defaultBaseUrl · transport · authType`. Metadata shows license + repo.

### Overlay demos

Two variants demonstrate the forms:

- **Overlay · Server form (admin pre-fill).** `OverlayShell` wraps `McpServerForm` with `mode='prefilled'`. Every field filled from the catalog entry. Admin reviews and hits **Save MCP Server**. Matches production `download (24).png`.
- **Overlay · Instance form (user add).** `OverlayShell` wraps `McpInstanceForm` pre-selecting the server. For OAuth servers, shows the Connected state with Client ID + Disconnect button. Matches production `download (25).png`.

Both overlays include a `Open full page ↗` secondary button so the user can escape into the non-overlay route if they prefer.

### Responsive deltas

| Breakpoint | Grid | Drawer |
|---|---|---|
| Desktop | auto-fill `minmax(215px, 1fr)` | sticky right pane |
| Medium | 2 columns | hidden (card tap → full-screen sheet) |
| Mobile | 1 column | hidden (card tap → full-screen sheet) |

---

## 5. My MCPs — list shape + Pending banner

Production-parity: `Name · URL · Status · Actions` table (download (9).png). Adds two signals on top:

1. **`McpInstancePendingBanner`** — yellow dashed banner above the table when the user has one or more pending approval requests. Lists server names; primary CTA `View` jumps to Admin Inbox (read-only for user role).
2. **`needs_reauth` chip** — amber `⚠ reauth` chip on affected instance rows (replaces the active green chip). Clicking it — out of scope for this pass — should kick off OAuth re-auth.

### Actions

- **`▷ Play`** — jumps to MCP Playground with the instance pre-selected. No overlay; tab-switch is fine here because Playground is a full-bleed workspace.
- **`✎ Edit`** — navigates to full-page instance-edit (same `McpInstanceForm` body, without `OverlayShell` wrapper).
- **`🗑 Delete`** — destructive; wireframe omits a confirmation dialog but production should add one.

### Responsive deltas

- Desktop: 5-column grid row (`24px · 1fr · 2fr · 90px · 110px`).
- Medium: same grid, smaller type (11px).
- Mobile: each row becomes a stacked card with inline action chips.

---

## 6. MCP Admin — registry + inbox

Admin-only tab. Two sections:

### Registered servers section

Table: `Name · URL · Auth · Status · Uses · Actions`. Columns:

- **Name** — slug + approved-date subtitle + auth-type.
- **URL** — monospace code.
- **Status** — enabled/disabled chip.
- **Uses** — instance count (`N inst`).
- **Actions** — `✎ Edit` / `⏸ Disable` (or `▶ Re-enable`) / `🗑 Delete`.

### Pending approvals section

Inbox-style list in `warn-soft` background. Each `McpApprovalRow`:
- Requester email + submitted date.
- Free-text reason.
- Actions: `✗ Reject` / `✓ Approve`. Approve creates the `mcp-server-registry` entry for the whole app instance.

### Top CTAs

- **`+ Register MCP Server`** (primary) — opens a blank `McpServerForm` in full-page mode (`mode='blank'`).
- **`+ From catalog ▾`** dropdown — routes admin into the Discover flow with an admin-prefill handler. This is the "one-click Add to app" path.

### Admin tab badge

`<Chip>N</Chip>` in the tab label, where `N` = unresolved approval count. Matches the research pattern from mcp.so (sponsor badges) and Composio (action counts).

### Responsive deltas

- Desktop: `McpRail` sticky left with 2 anchors (Registered / Approvals).
- Medium: `McpMediumAnchors` strip at top instead of rail.
- Mobile: pill-tabs at top switch between Servers and Approvals; each section stacks cards.

---

## 7. MCP Playground — sidebar + executor

Matches production `download (11).png`. Not redesigned; the existing UX is already good.

### Desktop layout

```
┌─────────────┬───────────────────────────────────┐
│ [notion ▾]  │ notion-get-users                  │
│ ●connected  │ Retrieves a list of users…        │
│             │                                   │
│ 🔍 search   │ [Form] [JSON]                     │
│             │                                   │
│ ■ notion-   │ query        [ _______________ ]  │
│   search    │ start_cursor [ _______________ ]  │
│ ■ notion-   │ page_size    [ _______________ ]  │
│   fetch     │ user_id      [ _______________ ]  │
│ □ notion-   │                                   │
│   get-users │ [ Execute ]                       │
│ ▶ active    │                                   │
│ ■ notion-   │ [●Success][Response][Raw][Request]│
│   create…   │ ┌───────────────────────────────┐ │
│             │ │ [ { "type": "text", … } ]     │ │
│             │ │                               │ │
│             │ └───────────────────────────────┘ │
└─────────────┴───────────────────────────────────┘
```

### Sidebar — `McpToolSidebar`

- Instance pill at top (`● connected · notion`).
- Search input.
- Scrollable tool list. Each item: tool name (hand font) + description preview (60 chars).
- Active selection → `indigo-soft` background.

### Main pane — `McpToolExecutor`

- Tool header: name + full description.
- **Form/JSON toggle** (same Chip primitives as API-create form).
- Parameter fields, one per tool parameter. Each field: `{name} ({type})` label + hint + text input.
- **Execute** primary button.
- **Response tabs**: Success · Response · Raw JSON · Request. Active = underlined.
- **Response body**: monospace `var(--ink)` background, `cde` foreground, scrollable, JSON preview.

### Responsive deltas

- Medium: sidebar becomes a `◧ Tools` drawer button; main pane uses full width.
- Mobile: 3 frames as progressive disclosure — (1) tool picker, (2) params + Execute, (3) response tabs.

---

## 8. Overlay reusability — why this matters

Both `McpServerForm` and `McpInstanceForm` are designed as plain JSX bodies (no chrome of their own). They're used in **three** places each without code duplication:

| Form | Context 1 | Context 2 | Context 3 |
|---|---|---|---|
| `McpServerForm` | Admin full-page Register | Admin full-page Edit | Discover overlay (admin pre-fill) |
| `McpInstanceForm` | My MCPs full-page Edit | My MCPs full-page Create | Discover overlay (user add) |

The `OverlayShell` chrome is added by the caller; the form body doesn't know whether it's in an overlay. This makes the same form reachable via two UX modes (quick-add from Discover / detailed-edit from full-page) without divergence.

---

## 9. Decisions archive

1. **Four separate tabs over a single mega-page.** User-picked. Rationale: MCP's role split (admin registry vs user instance vs everyone in Discover) maps cleanly onto distinct pages; a mode toggle would add a layer of indirection without payoff.
2. **Bodhi-curated catalog over external registry read-through.** User-picked. Rationale: non-tech UX needs logos, categories, screenshots — external registries (official registry, mcp.so) carry raw reverse-DNS names and minimal metadata. External read-through is deferred as a second tier.
3. **Admin approval gate kept; user adds "Submit for Approval" path.** User-specified in the plan interview. Rationale: preserves current production security model while unblocking non-admin discovery.
4. **One-click pre-fill for admins.** User-specified. Admin clicks CTA on a catalog card → `McpServerForm` opens with every field filled from the entry. Saves a lot of typing and reduces config errors.
5. **Inline instance on connected card.** User-specified ("if user already has an instance, is also shown created instance, which he can view/edit from the Discovery page itself"). Rationale: Discover becomes a single place where users can see both "what's available" and "what I've connected".
6. **Overlays only for MCP forms.** User-specified ("only for the mcp form, we need overlays"). Rationale: overlay chrome has a real cost (focus traps, keyboard handling, backdrop) — only two forms need it, so scope is tight.
7. **Five card-states derived, not stored.** Central contract: state is computed by joining entities. No sixth state without updating `mcpCardCta()` and the matrix in §2 + §4. Keeps card logic deterministic.
8. **Response body on dark background in Playground.** Mirrors production. Debug data looks right in terminal colours; the form input above stays light.
9. **Playground not redesigned.** The existing production Playground is already good — no rework. We capture the spec here to prevent future agents from "improving" it without cause.
10. **Category taxonomy at 9.** Narrower than Docker Hub (10) or mcpmarket (16). Chose recognisability over completeness; non-tech users get 9 familiar buckets instead of 16 niches.

---

## 10. Verification checklist

Checked end-to-end on `http://localhost:8000/` after v30 load:

1. **Top tabs (7 total):** `Models | Create local alias | Create API model | MCP Discover | My MCPs | MCP Admin | MCP Playground`. ✓
2. **MCP Discover — 5 variants rendered:** Desktop / Overlay Server / Overlay Instance / Medium / Mobile. ✓
3. **Discover Desktop cards:** 12 catalog entries, 1 connected (Notion, leaf border), 1 pending (Slack, dashed warn), 1 disabled (Context7, faded), 2 approved (Linear, GitHub), rest catalog-only. ✓
4. **Discover Overlay · Server form (admin pre-fill):** Linear entry filled; Auth=OAuth; all 4 endpoints visible; `★ Pre-filled from catalog` callout shown. ✓
5. **Discover Overlay · Instance form (user add):** Server=linear pre-selected; Connected badge with Client ID + Disconnect; matches production `download (25).png`. ✓
6. **Discover Drawer (Desktop):** Notion opened; tabs About / Capabilities / Connection / Metadata / Performance; Capabilities lists 4 tools + "+3 more…". ✓
7. **My MCPs — 3 variants:** Desktop / Medium / Mobile. Desktop shows 4 instance rows including `gmail-a` with `⚠ reauth` chip. Pending banner at top. ✓
8. **MCP Admin — 3 variants:** Desktop (rail + 2 sections) / Medium (anchor strip) / Mobile (pill tabs). 6 registered servers (1 disabled), 2 pending approval rows. Inbox badge = 2. ✓
9. **MCP Playground — 3 variants:** Desktop (sidebar + executor + 4-tab response) / Medium (sidebar collapsed) / Mobile (3-frame wizard). ✓
10. **No console errors.** `window.__wfErrors = []`. ✓

---

## 11. Out of scope / deferred

See `shared-primitives.md` §7 → "MCP deferred (added v30)" for the full list. Summary:

- External registry read-through (official / Smithery / mcp.so)
- Community-submission flow
- Per-tool live analytics charts
- Resources & Prompts (tools-only MVP)
- stdio transport (HTTP-streamable only)
- Needs-reauth refresh overlay
- Audit log / activity history
- Playground history / saved invocations
- Role-switcher UI (wireframe uses a demo chip)
- Bulk actions
- Deep-link to Playground with encoded tool+params

---

## 12. AI-agent hand-off notes

- **Adding a new card state.** Touch three places in lockstep: `mcpCardCta({state, role})` in `primitives.jsx`; the state×CTA matrix in §2 of this doc; the `.mcp-card.state-*` styles in `wireframes.css`. Do not skip any.
- **Adding a new catalog entry to the fixture.** Give it a unique slug; pick one of the 5 states (spread across states so demo stays colourful); populate every field — empty values break the drawer. If you add a new category, extend `MCP_CATEGORIES` first.
- **Extending the form.** Both forms are auth-type-aware via a local `React.useState` inside `McpServerForm`. Adding a new auth type (e.g. `mtls`) requires: extending `MCP_CATALOG_FIXTURE.authType` union, adding a new `McpAuth*Config` component, and wiring it into `McpServerForm`'s conditional render.
- **Overlay launching.** Discover cards don't wire click-to-overlay interactively in the wireframe — the two overlay variants are demonstrations. Production code should drive this via React state local to the Discover page.
- **Tabs persistence.** `localStorage.bodhi-wf-tab` stores the current tab key. The four new keys are `mcp-discover` / `my-mcps` / `mcp-admin` / `mcp-playground`. No migration needed — they're new.
- **Version bump for next primitive change.** Cache-buster `?v=30` is currently in `index.html`. Bump to `?v=31` (and fully) on every subsequent structural change to `primitives.jsx` or any screen file.
