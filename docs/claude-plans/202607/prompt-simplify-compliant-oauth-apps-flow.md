# Kickoff prompt — BodhiApp session

Paste the block below into a fresh Claude Code session rooted in `BodhiApp`.

**First run:** `/add-dir /Users/amir36/Documents/workspace/src/github.com/BodhiSearch/keycloak-bodhi-ext`
— the session needs to read the landed Keycloak commit to code against its actual contract.

---

```
We are re-engineering how third-party apps authorize against BodhiApp, replacing a bespoke flow with
a standard OAuth authorization-code flow. The work spans two repos and is deliberately split across
two sessions so neither carries the whole context.

**The keycloak-bodhi-ext half is DONE and committed.** This session is the BodhiApp half.

## Read these first

- The joint multi-repo plan (BodhiApp phases are 3, 4 and 5):
  docs/claude-plans/202607/prepare-plan-for-implementation-drifting-conway.md
- The design spec for the request/response shapes:
  docs/archive/auth-params-design.md
- What the Keycloak side actually shipped — read the commit, not just the plan:
  git -C /Users/amir36/Documents/workspace/src/github.com/BodhiSearch/keycloak-bodhi-ext show 6dd708e
- That session's own plan and handoff notes:
  /Users/amir36/Documents/workspace/src/github.com/BodhiSearch/keycloak-bodhi-ext/ai-docs/claude-plans/ai-docs-claude-plans-keycloak-bodhi-ext-steady-book.md

## The contract Keycloak now guarantees (verified against the landed code)

Scope format, composed by BodhiApp server-side and sent to Keycloak's authorize endpoint:

    openid profile email roles scope_access_request:<resource-client-id>.<uuid>

`AccessRequestScopeProtocolMapper` (now stateless, no JPA):
- splits the value on the **LAST** dot — the uuid segment never contains one, the client id might
- requires the named client to exist AND carry attribute `bodhi.client_type=resource`
- sets `aud` via `token.addAudience(resourceClientId)` and claim `access_request_id` = the uuid part
- an empty value after the prefix is skipped silently
- malformed value (no dot, leading dot, trailing dot), unknown client, non-resource client, or two
  different resource audiences in one request all **throw → HTTP 500 at the token endpoint**

That last point matters: there is no clean OAuth error for a bad scope. **BodhiApp must never compose
a malformed scope**, and must treat a 500 from the token endpoint as a composition bug of its own.

`GET /realms/{realm}/bodhi/users/apps/{client_id}/info` now returns `redirect_uris` — a `List<String>`,
always present, sorted, `[]` when none registered. `AppClientInfo` in `auth_service.rs` needs the new
field. Note the endpoint requires a *user* token whose `azp` is a resource client, which the consent
page's session satisfies.

`POST /realms/{realm}/bodhi/users/request-access` is **deleted**. `AuthService::register_access_request_consent`
therefore calls a route that no longer exists and must be removed, not repaired.

The `bodhi_access_request` table is retained but unmapped — no DDL was run. Nothing in BodhiApp
should reference it.

## Decisions already made — do not re-litigate

- Entry point is a UI route, `/ui/apps/auth/`, reached by a top-level browser navigation. It is NOT
  an API endpoint; `/bodhi/v1/oauth/*` and `/bodhi/v1/auth/*` are BodhiApp's own user login and must
  not be repurposed.
- App-facing scope vocabulary, consumed by BodhiApp only and NEVER forwarded to Keycloak:
  `scope_user_user` / `scope_user_power_user` (absent → inject `scope_user_user`; power_user renders
  a downgrade selector), and `scope_apps:llms[:true|:false]` / `scope_apps:mcps[:true|:false]`
  (absent or valueless or `:true` → requested; `:false` → not requested).
- `scope_apps:llms:false scope_apps:mcps:false` is **valid**, not an error — a role-only request for
  API access with no inference and no tools. There is no "empty access" error condition.
- `scope_user_*` are NOT Keycloak client scopes — commit 6434d8d removed them and the only file
  still containing them is an unreferenced legacy fixture. Forwarding them returns `invalid_scope`.
- Create-and-approve collapse into ONE mutating call. The user is authenticated at consent, so there
  is no Draft status, no 10-minute TTL, and no NULL-tenant window.
- The backend composes the full Keycloak authorize URL and returns it as `redirect_url`; the frontend
  does one unconditional `window.location.href =` with no inspection and no approve/deny branching.
  This is what keeps `validateAuthUrl`, `appendScopeToAuthUrl` and `buildErrorRedirect` deleted
  rather than reincarnated in a new shape.
- Errors and denial follow RFC 6749 §4.1.2.1: redirect to the app's `redirect_uri` with `error`,
  `error_description` and the original `state` — but only after exact-matching `redirect_uri` against
  the registered list. On mismatch, render in-app and redirect nowhere.
- Reauthorize is triggered by a `source_access_request_id` query param, with an UNSELECTED "restore
  previous selections" affordance as the fallback. Prior grants stay live.
- Per-URL MCP requests (`mcp_servers: [{url}]`) are dropped.
- `ApprovedResourcesV1` is unchanged — the enforcement path, `ResourceGrants`, and the
  least-privilege defaults need no modification.
- No backwards compatibility except the database. Committed migrations are immutable; production
  rows are real user data.

## Known traps, all verified

- `ScopeClaims.aud` is `Option<String>` (crates/services/src/shared_objs/token.rs). A token carrying
  an `aud` ARRAY fails deserialization and is rejected whole. Widen it to one-or-many, as its own
  commit, before the flow changes.
- `AppInitializer.tsx` bounces a `resource_guest`/no-role user to `ROUTE_REQUEST_ACCESS` — exactly a
  first-time third-party user. That abandons the flow and leaves a stale `bodhi-return-url` to
  misfire on a later login. Handle it inline.
- `/ui/request-access/` is an UNRELATED tenant-join feature that merely shares the name. Do not touch
  it or `multi-user-request-approval-flow.spec.mjs`.
- `ExternalTokenSimulator` seeds the exchange cache with a random `access_request_id` tied to no DB
  row, so it bypasses precisely the code that now carries the security burden. Extend it.
- `access_request_cache_needle` (token_service.rs → routes_apps.rs) is what makes revoke take effect
  before the 300s cache TTL. It keeps working because the `access_request_id` claim still exists —
  verify that stays true.
- Existing production grants carry an undotted scope, get no audience under the new mapper, and are
  rejected. The data-only migration should mark them `revoked` so the Connected Apps screen stops
  advertising dead grants.

## What is still open and is yours to work out

- The consent screen itself: section rendering driven by the parsed scope, the three-group
  reauthorize diff (still-requested / newly-requested / being-relinquished), and what the role-only
  case (`llms:false mcps:false`) renders instead of an empty form.
- Backend validation that the POSTed grant envelope complies with the requested scope — if
  `mcps:false`, no MCP grant may be stored.
- The test-oauth-app rewrite on `oauth4webapi`, and which E2E specs need rewriting versus only a
  page-object rename. The joint plan has a first pass at both; verify it.
- Whether the E2E suite can run at all before the Keycloak image is deployed to
  **main-id.getbodhi.app** — the Railway `main` environment, per `INTEG_TEST_MAIN_AUTH_URL` in
  crates/lib_bodhiserver/tests-js/.env.test and the default at test-helpers.mjs:66. This is `main`,
  NOT `dev`: dev-id.getbodhi.app also exists but nothing in the E2E suite points at it. Confirm the
  deploy landed on `main` before relying on E2E.

## What I want from you

Explore the codebase yourself. Verify the joint plan's file and line references rather than trusting
them — they were written from a different session's reading and have drifted at least once already.
Where you find the plan is wrong, say so explicitly rather than working around it silently.

Ask me questions with AskUserQuestion wherever a decision is genuinely mine to make.

Then present a plan scoped to BodhiApp only, following the layered methodology (services →
routes_app → OpenAPI/ts-client regen → frontend → E2E), with commit boundaries and the gate checks
that run before each.
```

---

## Why this exists

The joint plan at `prepare-plan-for-implementation-drifting-conway.md` covers both repos. The
`keycloak-bodhi-ext` half landed as commit `6dd708e` — stateless mapper, dotted single dynamic scope,
`redirect_uris` on app-info, the Keycloak-side record deleted with no DDL. This prompt hands the
remaining BodhiApp work to a fresh session with that contract pinned down, so it can start from
verified facts rather than re-deriving them.

The Keycloak image must be deployed to the Railway `main` environment (`main-id.getbodhi.app`) before
the BodhiApp session can verify anything end-to-end, because that is the instance the Playwright
suite authenticates against.
