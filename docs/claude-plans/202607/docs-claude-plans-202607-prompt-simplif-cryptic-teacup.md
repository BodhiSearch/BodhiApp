# BodhiApp — standard OAuth flow for third-party apps (BodhiApp half)

## Context

Third-party apps currently authorize via a bespoke five-hop flow: anonymous `POST /bodhi/v1/apps/request-access` pre-creates a Draft row, the app hand-builds a Keycloak authorize URL and passes it as `?auth_url=` to the review page, the page appends the dynamic scope and redirects, and BodhiApp synchronously registers the access request in Keycloak so its protocol mapper can verify it. The Keycloak half of the redesign (**keycloak-bodhi-ext commit `6dd708e`, released `v0.0.27`**) is done and **verified live on `main-id.getbodhi.app`** (deleted `request-access` route probes 404; app-info probes 401). The mapper is now stateless: it parses `scope_access_request:<resource-client-id>.<uuid>` by last-dot split, validates the named client is a `bodhi.client_type=resource` client, and stamps `aud` + `access_request_id`. Any malformed value → **HTTP 500 at the token endpoint**, so BodhiApp must never compose a malformed scope.

Consequence right now: the old BodhiApp flow against `main` is **already broken** (`register_access_request_consent` calls a 404 route), so consent-driving E2E specs are red until this whole plan lands. That is accepted; E2E gates only the final commit.

This plan is BodhiApp-only (joint plan Phases 3–5), verified against live code by five parallel exploration agents on 2026-08-27. Where the joint plan (`prepare-plan-for-implementation-drifting-conway.md`) is wrong, this plan says so and follows the code.

### Decisions taken with the user (2026-08-27)

1. **Guest at consent** (resource_guest / no role): render the consent page in a **blocked state** — app identity + "you don't have access to this Bodhi instance yet" + a single "Return to <app>" action that POSTs a deny and redirects `error=access_denied` to the validated `redirect_uri`. No approve button. No bounce to the tenant-join page.
2. **Scope handling — passthrough model** (supersedes the joint plan's "unknown tokens are rejected"):
   - `scope_user_*` → consumed by BodhiApp (role ceiling), never forwarded. Absent → inject `scope_user_user`. Both present → higher ceiling wins.
   - `scope_apps:llms[:true|:false]`, `scope_apps:mcps[:true|:false]` → consumed by BodhiApp (section show/hide + grant validation), never forwarded. Absent/valueless/`:true` → requested; `:false` → not requested. Malformed `scope_apps:*` or conflicting duplicates → `invalid_scope`.
   - `scope_access_request:*` from the app → **rejected** (`invalid_scope`). Reserved for backend composition; passing it through would let an app inject a second dynamic scope (mapper 500 / audience confusion).
   - **Everything else passes through to Keycloak verbatim** (deduped). Backend ensures `openid profile email roles` are present. If Keycloak rejects a passthrough scope, its own `invalid_scope` redirect back to the app's registered redirect_uri is the standard OAuth outcome — acceptable.
   - `scope_apps:llms:false scope_apps:mcps:false` is **valid** (role-only request), not an error.
3. **Commits**: 4 commits, E2E gates last (see Commit plan).
4. **Consent UI**: **refactor/relocate the existing review page** into the new route — not a from-scratch build. `git mv` the page, its components and its 1083-line test, then rework the data flow. Only genuinely dead code is deleted (`authUrl.ts`, `McpServerCard.tsx` per-URL flow).

### Corrections to the joint plan (verified against live code)

- `token_service.rs` with the exchange path is **`crates/routes_app/src/middleware/token_service/token_service.rs`** (723 lines), not in `crates/services`. All exchange-path line refs belong there: azp check `:283-293`, sub check `:302-312`, approved_role parse `:373-380`, clamp `:382-408`, needle `:43-45`.
- Services access-request files live under **`crates/services/src/app_access_requests/`** (not `src/db/`): `access_request_service.rs` (trait `:25-78`, impl `:126-312`), `access_request_repository.rs` (trait `:9-52`), `access_request_objs.rs` (RequestedResourcesV1 `:188-204`, ApprovedResourcesV1 `:227-241`, status enum `:64-72`), `app_access_request_entity.rs`.
- **There is no prior-grant lookup by `(tenant_id, app_client_id, user_id)`** — it must be added; nothing filters on `AppClientId` in services today.
- **`Claims.aud` (token.rs:124) has the identical `Option<String>` bug** as `ScopeClaims.aud` (token.rs:92). Widen both. The only production `.aud` reader is token_service.rs:239-247 (tenant resolution).
- The "1086-line authUrl test" does not exist — the 1083-line file is **`review/index.test.tsx`**; `authUrl.ts` has no dedicated test. `resolveShellRoute.ts` is at `crates/bodhi/src/components/shell/`, hooks are all in one file `src/hooks/apps/useAppAccessRequests.ts`.
- routes_app: handlers at `crates/routes_app/src/apps/routes_apps.rs`; the privilege-ceiling "move verbatim" block is `:303-331` (entangled with the record fetch `:312-318`); MCP checks `:334-379`. Route constants live in routes_apps.rs `:19-25` (routes.rs only imports them). Registration tiers: status GET is **public** (routes.rs:120-123), create POST is **optional-auth** (:127-130), review/approve/deny are in `user_session_apis` (:345-357) gated `api_auth_middleware(ResourceRole::User)` at :402-405.
- `ExchangeRequiresAuth` has exactly one usage (routes_apps.rs:70, inside the deleted create handler) → delete the variant as planned.
- test-oauth-app: **`RestClientSection.tsx` is a second call path** encoding the old contract (`buildReviewLink` :37-48, fires on any REST response containing `review_url`, anchor `link-rest-review` used by RESTPage.clickReviewLink + oauth2-token-exchange.spec.mjs:241). It must be removed along with `api.ts`/`buildReviewRedirect`.
- The upgrade E2E test's `{exchange:true}` POST is at oauth2-token-exchange.spec.mjs:225-235 (test decl at :170).
- `apps_deny_access_request` today has **no role guard** (unlike approve) — the new design keeps that: deny allowed for any authenticated session (guests decline), approve requires User+.
- `server_app` has **zero integration tests driving an external token through `access_request_auth_middleware`** (the mcps *list* route isn't behind it; only `/bodhi/v1/apps/mcps/{id}` and `/{id}/mcp` are). `ExternalTokenSimulator` builders needed for a real row (`make_request`/`approved_request` in `test_access_request_builders.rs`) are `pub(crate)` — promote via services test_utils.

---

## Target design

### Flow

```
app → top-level navigate → /ui/apps/auth/?client_id&redirect_uri&response_type&state&
                              code_challenge&code_challenge_method&scope[&source_access_request_id]
   (not logged in → existing bodhi-return-url stash → login → return with query intact)
page → GET /bodhi/v1/apps/access-requests/consent?<same query string>   (session auth, no role floor)
page → renders consent from parsed scope; user approves/denies
page → POST /bodhi/v1/apps/access-requests { query, decision, approved_role?, approved? }
     ← { id | null, redirect_url }
page → window.location.href = redirect_url          (unconditional, no inspection)
   approve: redirect_url = Keycloak authorize URL with composed scope
            openid profile email roles [passthrough…] scope_access_request:<resource-client-id>.<row-id>
            (<resource-client-id> = the session tenant's Keycloak client_id, e.g. bodhi-resource-<uuid>,
             resolved via tenant service — NOT the tenant row id)
   deny/param-error: redirect_url = app redirect_uri + error/error_description/error_source=bodhi/state
            (RFC 6749 §4.1.2.1; error_source=bodhi is an existing contract pinned by mcps-oauth-auth.spec.mjs:311-315)
   redirect_uri mismatch/unknown client: NO redirect — rendered in-app
Keycloak → app redirect_uri with code+state → app exchanges at Keycloak token endpoint
```

### Backend API

**`GET /bodhi/v1/apps/access-requests/consent?<full query string>`** — session auth with **Guest floor**: register in the existing `guest_endpoints` group (`routes.rs:173-180`, `api_auth_middleware(ResourceRole::Guest, None, None)`, merged into `session_protected` at :556) — session-only by construction (ApiToken/ExternalApp rejected), reachable by `resource_guest`. The handler calls Keycloak's app-info via the session user token from `auth_scope.auth_context().token()` (same mechanism today's approve handler uses at routes_apps.rs:283-286); a guest session carries the same resource-client-azp token, so this works for the blocked-guest state too. Always 200 with a discriminated body; only infrastructure failures are non-200:

```jsonc
// success
{ "result": "ok",
  "app": { "client_id", "name", "description", "redirect_uri" },
  "scope": { "role": "user"|"power_user", "llms": bool, "mcps": bool, "passthrough": ["..."] },
  "prior_grant": { "id", "approved_role", "approved": ApprovedResources, "source": "explicit"|"latest" } | null,
  "can_approve": bool }          // false for guest/no-role sessions → blocked state
// failure
{ "result": "error", "error": "invalid_request|invalid_scope|unauthorized_client|server_error",
  "error_description": "...", "redirect_url": "https://app/cb?error=...&state=..." | null }
```

`redirect_url` is non-null only when `redirect_uri` exact-matched the registered list (fetched via `get_app_client_info`; `redirect_uris` absent → older extension → skip validation and allow redirect). Null → the page renders the error in place and navigates nowhere.

**`POST /bodhi/v1/apps/access-requests`** — same `guest_endpoints` tier; in-handler: `decision=approve` requires ResourceRole::User+ (today's guard moved from routes_apps.rs:289-302; `Guest.has_access_to(User) == false` makes the floor work), `decision=deny` allowed for any authenticated session. Body:

```jsonc
{ "query": "<the raw query string as received by the page>",
  "decision": "approve" | "deny",
  "approved_role": "scope_user_user" | "scope_user_power_user",   // approve only
  "approved": ApprovedResourcesV1 }                                // approve only
```

The backend re-parses `query` (never trusts a client-side reading), re-validates everything the GET validated, enforces `approved_role ≤ requested ceiling ≤ approver's max_user_scope`, and **validates the grant envelope against the parsed scope**: `llms == false` → `models_access` must be empty `Specific` and `models_list` false; same for MCPs. Response `{ id, redirect_url }` (deny: `{ id: null, redirect_url }`). Deny creates **no row**.

- Approve: `create_approved(...)` writes one row — tenant_id + user_id from session, status `approved`, `approved_role`, `approved` JSON, `source_access_request_id` when reauthorizing, `expires_at = created_at` (column is NOT NULL; auto-expiry only applies to Draft rows, so the value is semantically unused for Approved), and `access_request_scope` = the full dotted value `scope_access_request:<resource-client-id>.<row-id>` — verified that `handle_external_client_token` matches the **full** scope string via `get_by_access_request_scope` (token_service.rs:249-260), so storing the full dotted value is exactly right. Row id generation unchanged (`new_ulid()` at access_request_service.rs:134 — dot-free, required by the last-dot split).
- Reauthorize: `source_access_request_id` query param → validated against `(tenant_id, app_client_id, user_id, status=approved)`; non-matching → ignored, fresh flow. Absent → newest prior approved grant for `(tenant, app_client_id, user)` offered as **unselected** "restore previous selections". Prior grants stay live.

### Consent screen (refactored review page at `/ui/apps/auth/`)

- Sections driven by parsed scope: `llms:false` → model section absent; `mcps:false` → MCP section absent; `power_user` → role selector (downgrade allowed), else fixed User.
- Role-only (`llms:false mcps:false`): no resource sections; app identity + plain-words statement ("access the Bodhi APIs as User — no model or tool access") + approve/deny.
- Prior grant in play → three-group diff **within rendered sections only**: still-requested (pre-checked) / newly-requested (highlighted) / being-relinquished (shown, option to keep). Scope-suppressed sections show nothing and carry nothing forward.
- `can_approve=false` → blocked state: identity + notice + "Return to <app>" (deny POST).
- Errors with `redirect_url=null` → in-app error, no navigation.

---

## Commit plan

Working directly on `main` (trunk-based). Gates run before each commit. `make test.backend` output tee'd to a file the first time.

### Commit 1 — widen `aud` to one-or-many (independent, all gates green)

**`crates/services/src/shared_objs/token.rs`**
- New `#[derive(..., Serialize, Deserialize)] #[serde(untagged)] pub enum Audience { One(String), Many(Vec<String>) }` with helpers (`iter()`, `contains()`); apply to **both** `ScopeClaims.aud` (:92) and `Claims.aud` (:124) as `Option<Audience>`.
- Unit tests: string aud, array aud, missing aud, single-element array — through `extract_claims`.

**`crates/routes_app/src/middleware/token_service/token_service.rs:239-247`** — the only `.aud` reader: iterate candidates, first `tenant_service.get_tenant_by_client_id` hit wins; none resolve → `InvalidAudience`. Add a routes_app unit test with an array-aud token (test_token_service.rs patterns).

Gates: `make format`, `make test.backend`, `cd crates/bodhi && npm test` (unaffected but cheap). Commit.

### Commit 2 — backend cutover: services → routes_app → OpenAPI/ts-client

Backend gates green; **frontend unit tests are expected red between commits 2 and 3** (ts-client types change) — accepted, stated in the commit message.

**services — new `crates/services/src/app_access_requests/app_scopes.rs`** (pure, table-driven):
- `parse_app_scope(&str) -> Result<ParsedAppScope, AppScopeError>` where `ParsedAppScope { role: UserScope, llms: bool, mcps: bool, passthrough: Vec<String> }`, implementing decision 2 above (incl. `scope_access_request:*` rejection, conflicting-duplicate rejection, higher-role-wins). Introduce a shared `SCOPE_ACCESS_REQUEST_PREFIX` const (today an inline literal at token_service.rs:252 and apps_api_schemas.rs:28).
- `compose_keycloak_scope(parsed, resource_client_id, record_id) -> String` — `openid profile email roles` ∪ passthrough (deduped, stable order) + the dotted dynamic scope. Must be structurally incapable of emitting a dotless/empty-segment value (KC 500 otherwise).
- `validate_grant_against_scope(&ParsedAppScope, &ApprovedResourcesV1) -> Result<()>` — refuses grants the scope didn't request (tampered-POST guard).
- `match_redirect_uri(requested, registered: Option<&[String]>) -> RedirectDecision` — exact match; `None` (older extension) → allow-unvalidated; `Some([])` → every match fails. Do not collapse those.
- Exhaustive unit tests (rstest tables) incl. `llms:false mcps:false` valid, passthrough preservation, injection of defaults, both-role-tokens, `scope_apps:garbage` rejection.

**services — `AccessRequestService` / repository / entity** (`crates/services/src/app_access_requests/`):
- Trait: add `create_approved(...) -> AppAccessRequest` (row born Approved with tenant/user set) and `latest_approved_for_app_user(tenant_id, app_client_id, user_id)`; delete `create_draft`, `approve_request`, `deny_request`, `build_review_url`; keep `get_request`, `list_approved_for_user`, `revoke_request`, `build_authorize_endpoint`.
- Repository: add `create` usage for approved rows + the new `(tenant, app_client, user, approved, newest-first)` query; delete `update_approval`, `update_denial`, `update_failure`, all three bypass-RLS probes (`:141-149`, `:206-214`, `:270-278`) and the `with_tenant_txn("")` empty-tenant read path in `get` (:97-99) — rows are never tenant-less anymore. `get_request` signature gains a real tenant_id (drop the `get("", id)` call at access_request_service.rs:168); update the ~4 call sites.
- Keep all six `AppAccessRequestStatus` variants (historical rows deserialize).
- Delete `CreateAccessRequest` (with its `exchange` flag); adapt/replace with the new consent-POST service inputs.
- **`AuthService`**: delete `register_access_request_consent` (trait :120-126, impl :766-817) + `RegisterAccessRequestConsentResponse` (:171-175). `AppClientInfo` (:177-181) gains `redirect_uris: Option<Vec<String>>`; drop the stale TODO at :828. `get_app_client_info` (impl :823-853) gains its first caller — it needs a *user* token from a resource client, which the consent session provides (same token source `register_access_request_consent` used).
- Follow `test-services` skill patterns for new/changed tests; promote `make_request`/`approved_request` (test_access_request_builders.rs) from `pub(crate)` to services `test-utils` export (server_app tests need them in commit 2's server_app work).

**services — migration `m20250101_000028_expire_stale_app_access_requests`** (data-only, style of m20250101_000023 backfill; register at both mod.rs sites, `Box::new` appended last):
```sql
UPDATE app_access_requests SET status='expired' WHERE status IN ('draft','failed');
UPDATE app_access_requests SET status='revoked'
 WHERE status='approved' AND (access_request_scope IS NULL OR access_request_scope NOT LIKE '%.%');
```
(NULL-scope guard added — `NOT LIKE` alone skips NULLs.) `down()` = no-op with comment. Migration-governance rules honored: new migration, immutable priors, no DDL.

**routes_app** (`crates/routes_app/src/apps/`):
- New handlers in routes_apps.rs: `apps_get_consent_context` (GET `/bodhi/v1/apps/access-requests/consent`, takes the raw query string via `RawQuery`), `apps_submit_consent` (POST `/bodhi/v1/apps/access-requests`). Register both in **`guest_endpoints`** (routes.rs:173-180); in-handler floors: approve→User+ via the moved guard :303-331 incl. its record fetch, deny→any session. Move the MCP ownership/enabled checks (:334-379) verbatim into the approve path. Keep `caller_max_user_scope` (:468-480, used by list+revoke).
- Delete: `apps_create_access_request`, `apps_get_access_request_status`, `apps_get_access_request_review`, `apps_approve_access_request`, `apps_deny_access_request`; their 5 ENDPOINT consts (keep `ENDPOINT_ACCESS_REQUESTS_APPS`, `ENDPOINT_ACCESS_REQUESTS_REVOKE`); `resolve_previous_grant` (:235-250, superseded by the consent-context prior-grant logic); dead DTOs in apps_api_schemas.rs (`CreateAccessRequestResponse`, `AccessRequestStatusResponse`, `AccessRequestReviewResponse`, `PreviousGrantInfo`→superseded, `McpServerReviewInfo`, `AccessRequestActionResponse`); registrations in routes.rs (:120-123 public status, :127-130 optional-auth create, :345-357 review/approve/deny) and imports (:17-19, :30-33).
- New DTOs: `ConsentContextResponse` (ok/error union per Target design), `SubmitConsentRequest`, `SubmitConsentResponse { id: Option<String>, redirect_url: String }`, `ConsentPriorGrant`.
- `error.rs`: add `OauthInvalidRequest`, `OauthInvalidScope`, `OauthUnauthorizedClient`, `OauthAccessDenied`, `RedirectUriMismatch` (two-attribute pattern; struct variants like `PrivilegeEscalation` :43-48); delete `ExchangeRequiresAuth` (:39-41).
- `routes.rs`: note `POST /bodhi/v1/apps/access-requests` shares no path with the manager-tier `/bodhi/v1/access-requests` (users domain) — distinct prefixes, no collision.
- `test_access_request_auth.rs` (src/apps/): repoint the 401 matrix at the two new endpoints + list + revoke (currently review/approve/deny only → would go empty). Copy the richer 3-tier matrix template from `src/users/test_access_request_auth.rs` where it fits.
- `test_access_request.rs` (1441 lines): keep/adapt the 3 list+revoke tests; rewrite approve/deny/privilege/MCP-validation tests against `apps_submit_consent` (the scenarios survive: privilege escalation, valid downgrade, MCP not-owned, tampered-role clamp, unknown version); delete create/status/review-specific tests; add: grant-vs-scope violations (`mcps:false` + MCP grant → rejected), role-only approve, guest deny allowed / guest approve 403, redirect_uri mismatch → in-app error shape, `scope_access_request:` injection → invalid_scope, passthrough preserved into redirect_url. Follow `test-routes-app` skill patterns.
- `tests/test_live_auth_middleware.rs:147-275`: rewrite — compose the dotted scope string directly (`scope_access_request:<resource-client>.<uuid>`), seed the Approved row with tenant/user set at creation, keep the live-KC exchange assertion.
- OpenAPI (`src/shared/openapi.rs`): swap the apps entries at the four verified sites — imports :2-4, `__path_*` :12-15 (alphabetical, rustfmt reflows), schemas :295-312, paths :487-493. Then `cargo run --package xtask openapi` → `make build.ts-client` → `make ci.ts-client-check`.

**server_app** (`crates/server_app/tests/`):
- `utils/external_token.rs`: extend `ExternalTokenSimulator` to insert a real `app_access_requests` row (via `app_service.db_service().create(&row)`, builders from services test-utils; tenant_id = TEST_TENANT_ID to match existing seeding convention; status Approved; app_client_id/user_id matching the minted token) and keep the cache-seed path.
- `test_oauth_external_token.rs`: delete the review_url/Host test (:24, endpoint gone). Add integration tests through the **guarded** routes (`/bodhi/v1/apps/mcps/{id}`, `/{id}/mcp` — the list route is NOT behind access_request_auth_middleware): valid row → 200; revoked row → 401; app_client_id mismatch → 401; user mismatch → 401; and revoke-then-immediate-401 proving the cache-needle eviction (routes_apps.rs:515-518) over real HTTP. Assert `aud` handling single-value. Add a session-driven test of the new consent POST: approve creates the row and returns a Keycloak authorize `redirect_url` carrying the composed scope (assert exact scope string, both vocabularies absent, dotted dynamic scope present).

Gates: `make format`, `make test.backend` (tee'd), `make ci.ts-client-check`. Commit (message notes UI red until next commit).

### Commit 3 — frontend consent page (UI gates green)

**Relocate, don't rebuild** — `git mv crates/bodhi/src/routes/apps/access-requests/review → crates/bodhi/src/routes/apps/auth`, then rework:
- `index.tsx`: `validateSearch` = zod passthrough object (client_id, redirect_uri, response_type, state, code_challenge, code_challenge_method, scope, source_access_request_id — all optional strings; the backend is the validator; template: `routes/auth/callback/index.tsx:14` passthrough + `mcps/oauth/callback` shapes). Replace the preflight state machine (:206-234) with: raw query string → `useGetConsentContext(queryString)` → ok / error-with-redirect / error-in-app / blocked-guest. Keep the `ReviewContent` component structure, GrantBlock wiring, `useListModels`/`useListMcps` data hooks, testids (renamed `review-*` → `consent-*` consistently with the page object update in commit 4).
- Sections/diff/role-only/blocked states per Target design. `computeRoleOptions` (:61-81) stays with the moved file; adapt to scope-provided ceiling. `previousGrantToState.ts` adapted for the three-group diff + unselected-restore affordance; `toApproveBody.ts` loses the per-URL `mcps[]` mapping; **delete** `-shared/authUrl.ts` and `-components/McpServerCard.tsx`.
- Submit: `useSubmitConsent()` → single unconditional `window.location.href = redirect_url` (keep `safeNavigate`).
- `AppInitializer.tsx`: new prop (e.g. `skipRoleGate`) set by this page only — a no-role user renders the page (blocked state) instead of bouncing to `ROUTE_REQUEST_ACCESS` (:85-89). Unauthenticated stash (:77-83) already preserves the full query string; `auth/callback` restore (:29-32) unchanged.
- `components/shell/resolveShellRoute.ts:16`: BARE_PREFIXES `- '/apps/access-requests/review'` `+ '/apps/auth'`; update `resolveShellRoute.test.ts`.
- `lib/constants.ts:19`: `ROUTE_APP_REVIEW_ACCESS` → `ROUTE_APPS_AUTH = '/apps/auth/'`.
- `hooks/apps/useAppAccessRequests.ts` + `constants.ts`: replace review/approve/deny hooks with `useGetConsentContext` + `useSubmitConsent`. `useGetConsentContext` appends the raw, already-encoded `window.location.search` to the endpoint string (do NOT pass it via axios `params` — it would re-encode); **keep** `useListAppAccess`/`useRevokeAppAccess` (tokens/apps page untouched). Update `test-utils/msw-v2/handlers/apps.ts` + `src/test-fixtures/apps.ts` to the new DTOs.
- `index.test.tsx` (1083 lines): refactor in place — keep the harness (router mocks, `setupWindowLocation`, msw lifecycle); coverage: default injection (no role token → User; no `scope_apps:*` → both sections), `:false` suppression, role-only rendering, three-group diff, scope-compliant prefill (mcps:false prior MCP selections neither shown nor carried), redirect_uri-mismatch in-app error, blocked-guest state + decline, unconditional `window.location.href = redirect_url`.

Gates: `make format`, `cd crates/bodhi && npm run lint && npm test`, `make test.backend` still green. Browser sanity per GATE-B rule: `make build.dev-server && make app.run.live`, drive the page manually in Chrome with hand-built query strings. Commit.

### Commit 4 — test-oauth-app rewrite + E2E (full `make test.e2e` green)

**test-oauth-app** (`crates/lib_bodhiserver/test-oauth-app/`) on **oauth4webapi**:
- Hand-built AS object (no discovery): `issuer` = Keycloak realm, `authorization_endpoint` = `${bodhiUrl}/ui/apps/auth/`, `token_endpoint` = Keycloak. Keycloak still issues the code at the app's redirect_uri, so `validateAuthResponse`'s iss check holds (verified against oauth4webapi@3.8.7 source in the joint plan).
- Replace `src/lib/oauth.ts` hand-rolled PKCE/state/URL/exchange with library calls; **delete** `src/lib/api.ts`, `buildReviewRedirect`, and `RestClientSection.tsx`'s `buildReviewLink` + `link-rest-review` anchor (:37-48, :122-125, :236-237). `ConfigForm.handleRequestAccess` (:77-140) collapses to PKCE generation + one navigation to `/ui/apps/auth/` (scope field now carries the app-facing vocabulary; default `scope_user_user`). **PKCE becomes unconditional**: today `buildAuthUrl` appends it only when `!isConfidential` (oauth.ts:39-42), but the new backend requires `code_challenge`+`S256` always — keep the `toggle-confidential` testid, send PKCE in both modes.
- Add a **Reauthorize** affordance (TokenPage or ConfigForm): decode the current access token's `access_request_id` claim → start a new authorize navigation with `source_access_request_id` + editable scope. This replaces the `{exchange:true}` REST path for the upgrade E2E test.
- **Keep**: every ConfigForm/RestClientSection data-testid (:145-301), sessionStorage contract (`oauthConfig`/`accessToken`), OAuthConfig shape, popup postMessage protocol (OAuthCallbackPage :8-46 ↔ ConfigForm :42-69) incl. `error_source` plumbing, `/callback` + `/rest` routes, vite `envDir ../tests-js` + `INTEG_TEST_` prefix.

**tests-js**:
- Rename `pages/AccessRequestReviewPage.mjs` → `AppsAuthPage.mjs`; keep method names (`approve`, `approveWithGrants`, `approveWithMcps`, `approveWithRole`, `clickDeny`, toggles) mapped to the new `consent-*` testids; preserve the fail-closed empty-Specific picker semantics (app-tokens-grants depends on it). Update the 10 import sites / 20 construction sites (incl. the popup construction at mcps-oauth-auth:375). `ConfigSection.mjs` unchanged except `submitAccessRequest` semantics (button now navigates instead of POSTing); `OAuthSection.waitForAccessRequestRedirect` (bodhi-origin wait) still holds — the app navigates to `/ui/apps/auth/` on the bodhi origin.
- Rewrite: `specs/oauth/oauth2-token-exchange.spec.mjs` (upgrade test → Reauthorize affordance + `source_access_request_id`; delete the `link-rest-review` step and `RESTPage.clickReviewLink`), `specs/tokens/app-tokens-grants.spec.mjs`, `specs/mcps/mcps-oauth-auth.spec.mjs` (denial keeps `error_source=bodhi` contract via the deny redirect_url; popup flow), `specs/oauth/oauth-chat-streaming.spec.mjs` (call sites only).
- No-change six (call sites only, verify green): `api-live-upstream`, `api-sdk-compat`, `mcps-mcp-proxy-everything`, `mcps-sdk-compat-everything`, `mcps-auth-restrictions`, `mcps-oauth-dcr`.
- **Delete** `specs/request-access/request-access-version-validation.spec.mjs` (tests the removed endpoint; the playwright.config `**/request-access/**` multi-tenant ignore stays — it still covers the unrelated tenant-join spec). **Leave** `multi-user-request-approval-flow.spec.mjs` and `multi-tenant-lifecycle.spec.mjs` (tenant-join feature, name collision only).
- New consent-surface coverage inside the rewritten specs (E2E black-box, no page.evaluate): scope-default rendering, `mcps:false` section absence, role-only grant journey (token works on role-gated route, 403 on inference), denial with original `state`, unregistered `redirect_uri` → in-app error and **no** redirect.
- Precondition satisfied: new Keycloak verified live on `main-id.getbodhi.app` (probed 2026-08-27). E2E OAuth flakiness rules apply (commit-waits, retries:2, verify failing specs in isolation).

Gates: `make format`, `make test.backend`, `cd crates/bodhi && npm test`, `make build.dev-server`, **`make test.e2e`** (from repo root; suite lives in lib_bodhiserver/tests-js). Commit.

### Post-commit docs pass (folded into commit 4 or a small commit 5)

- Update `crates/services/CLAUDE.md`/`PACKAGE.md`, `crates/routes_app/CLAUDE.md`, `crates/bodhi/src/CLAUDE.md`, `crates/lib_bodhiserver/tests-js/CLAUDE.md` for the new flow.
- `TECHDEBT.md`: `scope_user_*`/`scope_apps:*` are consumed by BodhiApp and never reach Keycloak — token's `scope` claim and the DB grant describe different things; revisit KC passthrough registration (O(M×N) problem from `6434d8d`, array-aud consequence).
- `docs/claude-plans/202607/index.md`: entry for this plan file (date 2026-08-27).

---

## Verification (end-to-end, after commit 4, against main Keycloak)

1. `make build.dev-server && make app.run.live`; drive the rewritten test-oauth-app through full authorization in Chrome: `/ui/apps/auth/?...` → login bounce → consent → land back with a working token.
2. Token carries `aud` = the tenant's resource client id (single value) and `access_request_id` = the created row id.
3. `/v1/chat/completions` enforcement matches the approved grants.
4. Scope defaults: no `scope_apps:*` → both sections; `scope_apps:mcps:false` → MCP section absent AND a tampered POST with MCP grants rejected; no `scope_user_*` → role fixed User; `scope_user_power_user` → selector.
5. Role-only: no resource sections, plain-words statement; token accepted on role-gated routes, denied on inference/MCP.
6. Deny → app receives `error=access_denied` with original `state`. Guest login → blocked state → decline round-trips.
7. Unregistered `redirect_uri` → in-app error, no redirect. `scope_access_request:evil.x` in the app's scope → invalid_scope.
8. Passthrough scope (e.g. `offline_access`) survives into the Keycloak authorize URL. (Note: passthrough scopes are deliberately NOT carried into the later app→resource token exchange — token_service.rs:319-324 forwards only the access-request scope + openid/email/profile/roles; that is unchanged.)
9. Revoke from App Tokens → token stops working immediately (cache needle).
10. Multi-tenant: repeat 1-3; tenant resolves from session, correct resource-client segment in the dotted scope.
