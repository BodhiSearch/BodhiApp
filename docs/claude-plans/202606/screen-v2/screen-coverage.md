# UI V2 Migration — Screen Coverage

> **SCOPE (locked):** This migration covers the **13 shell app screens** (Chat, Models ×4, MCP ×3,
> API Keys ×3, Settings ×2 — the "shell" layout family) **plus the OAuth-consent Access Request
> review** at `/apps/access-requests/review/`, which Batch 1 promoted into scope as the first **bare**
> (standalone, no-sidebar) screen — so 14 screens total. The remaining **access-request standalone**
> screens (`/request-access/`, pending), the **setup** wizard, and the **Keycloak/auth** screens
> stay **OUT OF SCOPE for now** (designs exist in `design/` — `bodhi-standalone.css`, `setup-flow.css`,
> `bodhi-auth.css` families — but are deferred). The layout model is built **shell vs bare** (Batch 1
> ships the reusable `BareLayout`) so those families can be added later without rework.


Which app screens **have** a V2 design, which **consolidate** into a shared screen, and which are
**design-pending**. This is a **live checklist** — update it as designs are provided or as screens
consolidate further. It complements [architecture.md](@architecture.md) (where screens live) and
@context.md (the task).

> Conventions baked into this map (from the design team + user):
> - **`/new/` and `/edit/` share one design** — the create and edit variants of a screen derive
>   from a single design, not two. So an "edit" route never needs its own design.
> - **Several app routes consolidate into one V2 screen** rather than staying as dedicated screens
>   (see "Consolidations" below).
> - The 14 design prototypes live in `design/` (see [design-reference.md](@design-reference.md));
>   non-screens there (Layout System, Migration Roadmap) are excluded.

## A. Covered — app screen ↔ V2 design (create/edit share one design)

| App route(s) | V2 design (`design/`) | Section / subPage |
|---|---|---|
| `/chat/` | Bodhi Chat | chat |
| `/models/` | Bodhi Models (All Models) | models / all-models |
| `/models/alias/new/` **+** `/models/alias/edit/` | Create New Local Model | models / new-local-model |
| `/models/api/new/` **+** `/models/api/edit/` | Create API Model | models / new-api-model |
| `/models/router/new/` **+** `/models/router/edit/` | Create Fallback Model (router = fallback alias) | models / new-fallback-model |
| `/mcps/` (+ Discover) | Bodhi MCP Discover v2 (All MCPs) | mcp / discover |
| `/mcps/new/` (+ edit) | Bodhi MCP New Instance | mcp / new-mcp |
| `/mcps/playground/` | Bodhi MCP Playground | mcp / (playground) |
| `/tokens/` | App Tokens | api-keys / app-tokens |
| `/tokens/new/` (create; **was a dialog**, design promotes to full page) | New App Token | api-keys / new-token |
| `/settings/` | Bodhi App Settings | settings / app-settings |
| `/users/` | Manage Users | settings / manage-users |
| `/users/access-requests/` (**consolidated**, see B) | Access Requests | api-keys / access-requests |
| `/apps/access-requests/review/?id=` | Bodhi Access Request (review) | **bare / standalone** (no AppShell; app-initiated OAuth consent) |

## B. Consolidations — multiple app routes → one V2 screen

The V2 design intentionally folds several dedicated app screens into shared screens:

- **`/models/files/pull/` → folded into the local-model alias page.** No standalone "pull/download"
  screen. On the New Local Model page the user enters any `<org>/<repo>` pattern; creating the alias
  downloads the referenced model. → `/models/files/pull/` is **retired** (no separate design).
- **`/models/files/` (local files browser)** — folded into the Models experience (the alias/local-
  model flow). No dedicated V2 "files browser" screen. *(Treat as retired/absorbed; confirm exact
  handling when the Models batch is planned.)*
- **`/mcps/servers/*` → folded into the single MCP screen.** One screen browses **configured**
  instances **and** discovers **new** MCP servers (Discover). The separate servers
  list/new/view/edit routes (`/mcps/servers/`, `/mcps/servers/new/`, `/mcps/servers/view/`,
  `/mcps/servers/edit/`) collapse into the unified MCP Discover + New Instance screens. → the
  `/mcps/servers/*` cluster is **retired/absorbed** (no separate designs).
- **`/users/pending/` → consolidated into `/users/access-requests/`.** The single "Access Requests"
  design covers pending + history (filter tabs). → `/users/pending/` is **retired** (no separate
  design).

## C. OUT OF SCOPE for this task — designs exist, deferred

Designs for these now exist in `design/` (three standalone layout families), but the user has
**explicitly deferred them** — this task + phased plan cover only the 13 shell app screens. They
render **bare** (outside AppShell) today and keep their current production layouts. When a future
task picks them up, they map to the design families noted below.

| Family | Design source (`design/`) | App route(s) | Notes |
|---|---|---|---|
| **Access-request standalone** (2 still deferred) | `Bodhi Request Access.html`, `Bodhi Access Request Pending.html` (`bodhi-standalone.css`) | `/request-access/`, pending | `.std-*` layout: slim topbar + centered card, no sidebar. **Deferred.** (`Bodhi Access Request.html` = the OAuth-consent **review** at `/apps/access-requests/review/` is NOW in scope §A as a **bare** screen — Batch 1; it is the first standalone screen we build, establishing the reusable `BareLayout`.) |
| **Setup / onboarding** (6) | `setup-1-get-started.html` … `setup-6-complete.html` (`setup-flow.css`, `setup-shell.jsx`) | `/setup/*` | Centered column + top stepper. Linear flow, bare. |
| **Keycloak / auth** (7) | `login.html`, `register.html`, `forgot-password.html`, `update-password.html`, `verify-email.html`, `consent.html`, `error.html` (`bodhi-auth.css`, `KEYCLOAK-MAPPING.md`) | IdP-hosted (`/login`, `/register`, …) | Keycloak FreeMarker themes; served by the auth server, not the app. |

Plus app routes that render bare and aren't redesigned in this task: `/login/`, `/auth/callback/`,
`/auth/dashboard/callback/`, `/mcps/oauth/callback/`.

## D. No design needed (redirect-only, no real UI)

`/` and `/home/` (root redirectors). *(Auth/oauth callbacks are listed in §C because the user wants
them created/designed; if any are pure redirects with no UI, they move here.)*

## E. Open items to resolve when batches are planned

- `/models/files/` — confirm it's fully absorbed into the Models flow vs. needs a (small) surface.
- Exact in-screen treatment of the MCP `servers/*` admin actions now that they live inside the one
  MCP screen (auth-config management, server registration) — verify the Discover/New Instance
  designs cover every action the old `servers/*` routes did.

---

**Maintenance:** when the user provides a design for a §C row, move it to §A (with its design name +
section/subPage) and flip the checkbox. When a consolidation is confirmed in code, note it in §B.
Keep this doc current — it's the authoritative "what's covered" reference for batch planning.
