# Access-request review wireframe

*Archive spec for the Access-Request screen added in v31 (2026-04-20). Reviewer decides which of their local resources a 3rd-party app may use, and resolves missing MCP prerequisites without leaving the page. Read alongside `mcp.md` (entity model + `McpServerForm` reuse) and `models.md` (row-shape duality + capability metadata).*

Wireframe: `design/models-page/project/screens/access-request.jsx`.

---

## 1. What this screen is — and what the production page lacked

A 3rd-party app (identified by a client-id like `bodhi-app-f181a4d1-…`) requests access to the user's Bodhi resources. The user is sent to `/apps/access-requests/review?id=…` to approve or deny.

Production today (screenshots `download (25).png` + `download (26).png`) has two states:

- **MCP configured** → dropdown to pick an instance, Approve All enabled.
- **MCP not configured** → red dead-end: *"No MCP instances connected to this server. Create an instance first."*

The dead-end forces the reviewer to leave the page, go to MCP admin, register the server, create an instance, complete OAuth in the same tab (losing the review context), and hope everything reloads cleanly. The redesign eliminates this.

### What the v31 wireframe adds

1. **Per-model allow-list** — replace "Approve All" with multi-select checkboxes over the user's configured models.
2. **Capability envelope from the app** — app declares `tool-use`, `vision`, `min-ctx`, `max-cost`, plus an optional suggested-model list. The UI pre-checks matching models and disables below-envelope rows.
3. **Inline prerequisite resolution for MCP** — a 6-state row model with role-adaptive CTAs; missing server → admin one-click inline mini-overlay, non-admin "Request admin to add" filing into the v30 Admin Inbox; missing instance → OAuth in a popup window with on-focus refetch.
4. **Role-adaptive UI** — default user view; if the viewer has admin/manager, admin CTAs appear inline. Toggle in the wireframe card header.

---

## 2. Entity model

Extends the v30 MCP entity set. Two new entities specific to this flow:

| # | `kind` | Source of truth | Role here |
|---|---|---|---|
| A | `access-request` | `getbodhi-auth` callback payload | The object being reviewed. Carries client-id, display name, capability envelope, requested MCPs, suggested models. |
| B | `allowed-model-grant` | Bodhi DB (created on Approve) | Output of this flow — the per-model decisions. |

Re-uses:
- `mcp-catalog-entry` for pre-fill on `One-click Add`
- `mcp-server-registry` to detect `needs-server` state
- `mcp-instance` to detect `has-instance` / `needs-reauth`
- User's models (aliases + api-models + provider-models) from Bodhi DB

---

## 3. Per-MCP row state matrix (6 states × role)

| Row state | Condition | User CTA | Admin CTA |
|---|---|---|---|
| `needs-server` | no registry entry for this slug | `✉ Request admin to add` → files into Admin Inbox | `⚡ One-click Add MCP Server` → inline mini-overlay of `McpServerForm` pre-filled from catalog |
| `pending-admin` | user has filed a request, admin not acted | `⏳ Admin notified · Nm ago` (disabled) | (admin sees the row under `needs-server` + a pending-request chip) |
| `needs-instance` | server exists, user has no instance | `+ Connect instance (OAuth ↗)` → popup | same |
| `oauth-in-progress` | Connect clicked, popup open | `⏳ Waiting for OAuth confirmation in popup…` (pulsing indigo dot) | same |
| `has-instance` | instance connected | instance dropdown (current UX) | same |
| `needs-reauth` | instance exists, `authState='needs_reauth'` | `⚠ Reconnect via OAuth ↗` | same |

State is **derived, not stored** — joining `access-request.mcps[].slug` with `mcp-server-registry` and `mcp-instance`. The join happens at render time; the UI re-evaluates on every `refetchOnWindowFocus` tick.

### On-focus refetch contract

TanStack Query is configured with `refetchOnWindowFocus: true` for the two queries this page depends on: `GET /mcp/server-registry` and `GET /mcp/instances`. Clicking `+ Connect instance (OAuth ↗)` opens a popup window; when the user completes OAuth in the popup and focus returns to the review tab, both queries revalidate. The freshly-created instance surfaces automatically, state recomputes, and the row transitions `oauth-in-progress` → `has-instance`.

This makes the popup flow feel seamless to non-technical users. Do not change the refetch cadence without a plan for the worst-case flow (OAuth failure / popup blocked / tab never refocused).

---

## 4. Per-model row state matrix (5 states)

| Row state | Condition | Default checked? | Visual |
|---|---|---|---|
| `app-suggested` | slug appears in `request.suggestedModels[]` | ✓ | leaf-soft background · `★ app-suggested` chip |
| `matches-envelope` | model satisfies capability envelope | ✓ | `matches` chip |
| `user-config` | other configured models | ☐ | default paper |
| `below-envelope` | fails one or more envelope constraints | ☐ (disabled) | 58% opacity · `below envelope` warn chip |
| `unavailable` | model requires resource user lacks (future) | ☐ (disabled) | 45% opacity · `unavailable` chip |

Codified in `accessModelState(model, caps, suggested)` in `primitives.jsx`. Single source of truth for the pre-check decisions.

### Envelope fields

```
caps: {
  required:     ['tool-use', 'text-to-text'],
  preferred:    ['vision'],              // soft signal; doesn't pre-check but highlights in drawer
  minContext:   128_000,
  maxCostUsdPerMTok: 2.5,                // applies to output tokens in the current check
}
```

A future pass may extend with latency / throughput envelopes.

---

## 5. Section-by-section anatomy

### Section 0 — Header (`AccessRequestHeader`)

Left → right: logo · display-name + `✓ verified` chip + `3rd-party app` chip · "is requesting access to your resources." sub · client-id monospace chip · description paragraph · role toggle (User | Admin) pinned top-right.

### Section 1 — Required capabilities (`AccessCapsEnvelope`)

Indigo-soft pill row: required caps as `on` indigo chips, preferred caps as neutral chips, `min-ctx · Nk` chip, `max-cost · $N/MTok` chip. Helper line below: "we'll pre-check models that match".

### Section 2 — Model access (`AccessModelGroup` × 3)

Three grouped lists: **Aliases (local) · API models · Provider models**. Each group has a head row with `select all` affordance. Each `AccessModelRow` shows:

- checkbox
- model title (hand font, bold)
- up to 3 capability chips
- reason chip (`★ app-suggested` / `matches` / `below envelope`)
- meta (right-aligned): context size + cost-or-origin

Footer hint reports suggested count and below-envelope count.

### Section 3 — MCP access (`AccessMcpRow` × N)

Per-MCP row driven by the 6-state matrix in §3. Each row has:

- checkbox · server name · URL chip · right-aligned state label
- body (indented): state-appropriate CTA + tool-preview hint (`Will use: notion-search · notion-fetch · notion-create-pages…`)
- when `needs-server` on admin role and the `One-click Add` button is clicked, the inline mini-overlay expands below the row (`AccessMcpInlineAddServer`) containing the full `McpServerForm` in `mode='prefilled'` + Cancel/Save footer

Footer hint reminds the reviewer that OAuth happens in a popup and the page auto-refreshes.

### Section 4 — Approved role (`AccessRoleSelect`)

Simple dropdown: `User / PowerUser / Admin`. Mirrors production. Hint: "drives what this app can see in your resources".

### Sticky action bar (`AccessActionBar`)

Bottom of the card (sticky on Desktop/Medium). Left: `Deny`. Right: `Approve N of M resources` primary. Above the primary: optional warn-tone hint when the button is disabled (e.g. `2 MCPs need setup before approving`).

---

## 6. Approve / Deny rules

### Approve button label

`Approve {checkedModels + checkedMcps} of {totalModels + totalMcps} resources`. Updates live as checkboxes change.

### Approve disabled when

1. Any **checked** MCP row is not in `has-instance` state (blockers include `needs-server`, `needs-instance`, `needs-reauth`, `oauth-in-progress`, `pending-admin`). Hint: `N MCPs need setup before approving`.
2. Zero resources (models + MCPs) are checked. Hint: `select at least one resource`.

### Deny

Always enabled. Exits the flow; records a denial event.

### On-focus refetch contract

See §3.

---

## 7. Inline prerequisite resolution

### Admin — `needs-server` → inline mini-overlay

When the viewer has admin role and the row state is `needs-server`:

1. Row shows **`⚡ One-click Add MCP Server`** primary CTA + hint: `pre-filled from Bodhi catalog · review and save without leaving this page`.
2. Click → the `AccessMcpInlineAddServer` accordion expands below the row head:
   - Heading: `★ One-click Add · pre-filled from catalog: <slug>` + `✗ Cancel` button.
   - Body: `McpServerForm` in `compact` mode, `mode='prefilled'`, with all fields pre-loaded from the matching `mcp-catalog-entry`.
   - Footer: `Cancel` + primary `Save & continue`.
3. On Save → backend creates the `mcp-server-registry` row; the server-registry query invalidates; this row transitions `needs-server` → `needs-instance` (because admin added but user has no instance yet). Accordion collapses.
4. The admin / user then sees the `+ Connect instance (OAuth ↗)` CTA in the same row and proceeds.

### Non-admin user — `needs-server` → request ticket

When the viewer has user role and the row is `needs-server`:

1. Row shows **`✉ Request admin to add`** secondary CTA + hint: `your admin gets a notification · reviewer can approve from Admin Inbox`.
2. Click → files a request into the v30 Admin Inbox (same queue as the MCP Admin screen renders).
3. Row transitions `needs-server` → `pending-admin`. CTA becomes `⏳ Admin notified · Nm ago` (disabled).
4. When admin approves the ticket (in MCP Admin), a push/poll on this page transitions the row to `needs-instance`.

### OAuth popup — `needs-instance` → `has-instance`

1. Row shows **`+ Connect instance (OAuth ↗)`** primary CTA.
2. Click → app opens OAuth in a popup window (see v30 decisions: same-window redirect was the production default; this flow uses a popup specifically so the review page stays mounted).
3. Row transitions `needs-instance` → `oauth-in-progress` with the `AccessOAuthHint` pulsing indigo dot.
4. When the popup redirects to `/ui/auth/mcp/callback` and closes itself, the review tab regains focus. TanStack Query `refetchOnWindowFocus: true` re-runs the mcp-instances query. The new instance appears. Row transitions `oauth-in-progress` → `has-instance`.

### Header/Query (non-OAuth) MCPs — deferred

Today's wireframe assumes OAuth for the Connect step. For header-auth MCPs (e.g. Exa, Firecrawl) the inline mini-overlay should render `McpInstanceForm` inline instead of opening a popup. This is deferred (see §11).

---

## 8. Responsive deltas

### Desktop variant (`AccessRequestDesktop`)

Single centered card (max-width 760px). Sticky action bar at the bottom of the card. Inline mini-overlay expands in-card. Role toggle pinned top-right of the header.

### Medium variant (`AccessRequestMedium`)

Same card at 95% width. Tighter paddings. Action bar rendered as non-sticky row at card bottom. Demonstrated in user role (non-admin) so the role-adaptive CTA difference is visible at a glance.

### Mobile variant (`AccessRequestMobile`)

Three PhoneFrames, progressive disclosure:

1. **Stage 1** — header + capability envelope → `Continue`.
2. **Stage 2** — model selection (all groups) → `Continue`.
3. **Stage 3** — MCP access + Approved role + Approve action → `Approve`.

Deny button is available on every stage (top-right of stage header). Back button on stages 2 and 3.

---

## 9. Decisions archive

1. **4 sections, one card.** Rejected: multi-page wizard. Rationale: desktop reviewers want to see everything at once; mobile gets the wizard variant specifically.
2. **Envelope pre-check, not hard-lock.** App's capability envelope pre-checks matching rows but doesn't forbid the user from granting others. User sovereignty over their resources > app's preferences.
3. **Below-envelope rows disabled.** A model that can't satisfy `tool-use` when the app needs it cannot be granted. Disabled-with-reason is clearer than just unchecking-by-default.
4. **Role-adaptive CTAs, not role-hidden.** Even the user view shows a `Request admin to add` affordance on `needs-server` rows. Hiding it would leave users stuck; showing it creates a clear recovery path.
5. **Inline mini-overlay, not real overlay.** Admin's inline `Add MCP Server` expands inside the card (accordion), not as a layered modal. Rationale: we want the reviewer to see their in-progress approvals + the new server config together; layering would occlude context.
6. **OAuth in popup, not same-window.** Overrides the v30 MCP decision of same-window redirect. Specifically for this flow, same-window would destroy the review context. Popup + `refetchOnWindowFocus` gives the seamless UX.
7. **5-state model contract + 6-state MCP contract.** Centralised in `accessModelState()` and the `AccessMcpRow` switch. Adding a new state requires touching only those two places + the matrices in §3/§4 of this doc.
8. **Approve button reflects reality.** "Approve 3 of 5 resources" not "Approve All". Makes the decision granular and the outcome auditable.
9. **Footer hint over silent disable.** Disabled primary button always explains why. Disabled buttons without explanation are hostile to non-tech users.
10. **Role toggle in the card header.** For the wireframe only — a demo chip so reviewers can see both admin and user CTAs side by side. Production has the role from auth context.

---

## 10. Verification checklist

After reload of `http://localhost:8000/`:

1. **Top tabs (8 total):** includes `Access Request` as the 8th tab. ✓
2. **Desktop variant:**
   - Header shows `Research Copilot` + `✓ verified` + `3rd-party app` chip + client-id `bodhi-app-f181a4d1…`.
   - Capability envelope shows 4 pills: tool-use · text-to-text · preferred vision · min-ctx · max-cost.
   - Model section shows 3 groups with 8 rows total; at least 1 disabled `below-envelope` row; suggested rows are leaf-soft.
   - MCP section shows 4 rows: Exa (has-instance) · Notion (needs-reauth) · Linear (needs-instance) · Gmail (needs-server).
   - Gmail row shows the **inline mini-overlay open by default** with `McpServerForm` pre-filled from catalog (demo state).
   - Role toggle top-right; clicking `User` flips Gmail CTA to `Request admin to add`.
   - Sticky action bar shows `Approve N of M resources` with disabled-reason hint.
3. **Medium variant:** User role by default; Gmail row shows `Request admin to add`.
4. **Mobile variant:** 3 PhoneFrames with Continue/Approve buttons per stage; Deny persistent.
5. **No console errors.** Tweaks panel still works.
6. **All previous 7 tabs render unchanged.**

---

## 11. Out of scope / deferred

- **Per-tool MCP scoping.** Today the grant is whole-server. Per-tool grants (like OAuth scopes) are a future pass; the catalog already carries the tool list.
- **Temporal grants / expiry.** Approvals are permanent until revoked. Time-boxed grants (30 days / 90 days) deferred.
- **Rate-limit overrides per app.** No per-app rpm / tpm throttles in this screen.
- **Cost ceiling enforcement per app.** Hard budget caps deferred.
- **Header-auth MCP inline form.** Today the `needs-instance` CTA assumes OAuth popup. For header-auth servers the inline mini-overlay should render `McpInstanceForm` inline instead — deferred.
- **Request audit log / history.** User has no "My approved apps" view; separate screen.
- **Batch review across multiple requests.** Request-at-a-time for now.
- **Popup-blocked / OAuth-failed recovery UX.** Wireframe shows the in-progress state; the failure/retry overlay is deferred.
- **App icon / logo upload.** Fixture uses a letter glyph; production would render an app-registered URL.
- **3rd-party app directory.** Far future.

---

## 12. AI-agent hand-off notes

- **Adding a new MCP row state.** Touch `AccessMcpRow` switch + the §3 matrix here + the `.state-*` CSS classes. Keep the six-state contract coherent.
- **Adding a new model row state.** Touch `accessModelState()` + the `AccessModelRow` reason-chip switch + the §4 matrix. The below-envelope default (disabled + 58% opacity) is the current contract; don't create multiple disabled variants without a reason.
- **Changing envelope fields.** `caps.required` / `caps.preferred` are array literals the UI iterates; `minContext` / `maxCostUsdPerMTok` are scalar. Adding a new envelope field (e.g. `minThroughputTps`) means updating `accessModelState()` + `AccessCapsEnvelope` + the fixture.
- **OAuth popup UX.** The wireframe shows `oauth-in-progress`; production implementation needs to handle: popup blocked (show inline error + retry button), popup closed without OAuth (stale state fallback after N seconds), OAuth error return.
- **Card width.** `.access-card` defaults to 760px max-width; Medium overrides with `.fill`. Changing the default affects visual hierarchy — review the whole section layout if you do.
- **Role toggle.** Wireframe-only. Production uses auth context; remove the toggle chip when the wireframe lands in the product.
- **Cache-buster.** v31 in `index.html`. Bump to `?v=32` on next structural change.
