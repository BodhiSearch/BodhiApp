# Third-party app authorization — OAuth-aligned flow

## Context

Third-party apps currently gain access to a user's BodhiApp through a bespoke, five-hop flow: the app
POSTs `/bodhi/v1/apps/request-access` (anonymous) to pre-create a Draft row, opens the returned
`review_url` with its own hand-built Keycloak authorize URL passed as `?auth_url=` plus an
`?error_url=`, the consent page appends `scope_access_request:<uuid>` and redirects to Keycloak, and
the app separately polls for status. On approve, BodhiApp makes a synchronous HTTP call to Keycloak
to persist a `bodhi_access_request` row so the protocol mapper can verify the id and set the token
audience.

This deviates from standard OAuth in ways that raise the integration cost for every third-party
developer, and it carries real defects: `error_url` is redirected to with no validation (a live open
redirect), the status-poll endpoint has zero callers, the Keycloak-side record duplicates checks
BodhiApp already performs authoritatively, and the pre-create call forces a NULL-tenant window with
RLS escape clauses to match.

The redesign replaces the entry point with a single UI route, `/ui/apps/auth/`, that accepts a
standard OAuth authorization request. Consent happens once, creating an already-approved record, and
the backend composes the Keycloak authorize URL server-side. The Keycloak-side record is deleted
entirely; forgery is prevented by BodhiApp re-checking `(tenant_id, user_id, app_client_id, uuid)`
against its own row, which it already does. Outcome: five hops become three, the pre-create call and
polling disappear, the open redirect is eliminated by construction, and one of Keycloak's three SPI
registrations goes away.

**Scope of this plan:** `keycloak-bodhi-ext` and `BodhiApp` only. SDK and third-party app updates are
planned separately.

**Full design spec:** `scratchpad/auth-params-design.md` (query params, scope vocabulary, request and
response shapes).

---

## Sequencing and gates

The Playwright E2E suite authenticates against the live `dev-id.getbodhi.app`, so the two repos
cannot be verified independently. Order is mandatory:

```
Phase 1  keycloak-bodhi-ext          →  merge, tag, build image
Phase 2  GATE: deploy to dev-id      →  dev is BROKEN for the old flow from here
Phase 3  BodhiApp backend            →  services → routes_app → OpenAPI/ts-client
Phase 4  BodhiApp frontend + E2E     →  verified against dev Keycloak
Phase 5  GATE: production cutover    →  both repos, close together
```

**Phase 2 breaks dev deliberately.** Once the stateless mapper ships, a `scope_access_request:<uuid>`
value with no `<resource-client-id>.` prefix yields no audience, so the current BodhiApp's token
exchange fails. Dev stays broken until Phase 3 lands. Do not deploy Phase 1 to production until
Phase 4 is verified.

**Production cutover invalidates every existing app grant.** Existing tokens carry the old
undotted scope; after the mapper change they get no audience and are rejected. With one production
app this is acceptable — but it is a user-visible re-authorization, not a silent migration.

---

## Phase 1 — keycloak-bodhi-ext

Repo: `/Users/amir36/Documents/workspace/src/github.com/BodhiSearch/keycloak-bodhi-ext`

### 1.1 Add `redirect_uris` to app-info

- `src/main/java/com/bodhisearch/AppInfoResponse.java` — add a third field, `@JsonProperty("redirect_uris")`, matching the existing JSON name in `ClientRequest.java:19-21`.
- `ResourceService.java:362` — the single construction site. `ClientModel.getRedirectUris()` returns an unordered `Set<String>`; sort it for a stable response.
- `BodhiResourceProvider.java:214-230` — no route change; update the `@ApiResponse` schema doc only.
- `AppInfoTest.java:37-38` — assert the new field. The fixture app already registers `http://testapp.localhost/callback` at `:27-28`.

Needed because BodhiApp originates the OAuth error redirect and must exact-match `redirect_uri`
before redirecting, per RFC 6749 §4.1.2.1.

### 1.2 Make the protocol mapper stateless

`src/main/java/com/bodhisearch/AccessRequestScopeProtocolMapper.java` — replace the body of the scope
loop (currently `:82-98`):

1. Split the scope value on the **first** `.` → `prefix` (resource client id) and `uuid`.
2. `realm.getClientByClientId(prefix)`; reject if absent **or** if its `bodhi.client_type` attribute
   is not `resource` (constants at `ResourceService.java:60-62`). This is what stops an app naming an
   arbitrary realm client as its audience.
3. `token.addAudience(prefix)`; set claim `access_request_id` = `uuid`.
4. Keep the existing "one resource audience per token" guard (`:100-105`).

Delete the `AccessRequestRepository.findById` lookup and both duplicated checks — BodhiApp enforces
`azp` at `token_service.rs:290-300` and `sub` at `:313-323` before the exchange.

No realm-config change: the dynamic-scope regexp `scope_access_request:(.*)`
(`realm-import-files/common.json:20-40`) already accepts a dotted value. `make import.dev` is not
required.

### 1.3 Delete the access-request record

- `ResourceService.requestAccess` — lines 243-343.
- `BodhiResourceProvider.java:190-212` — the `POST users/request-access` route.
- `AppAccessRequest.java`, `AppAccessResponse.java`.
- `jpa/BodhiAccessRequestEntity.java`, `jpa/AccessRequestRepository.java`.
- `jpa/BodhiJpaEntityProvider.java` — `getEntities()` returns an empty list. **Keep the provider and
  its factory registered** in `META-INF/services/` so Liquibase still runs against each realm's
  datasource.
- `src/main/resources/META-INF/bodhi-changelog.xml` — new changeSet `6`,
  `<dropTable tableName="bodhi_access_request"/>`. changeSets 4 and 5 (lines 106-112) are the
  precedent: drop-follow-ups, never edits to the original createTable.
- Remove now-unused `AnalyticsEvents.ACCESS_REQUESTED` and any orphaned `AnalyticsProps`.
- `httpyac-scripts/common.http:69` and `marketplace-flow.http:170-199` reference the deleted endpoint.

CI runs PMD dead-code checks (`make ci.quality`), so removal must be complete in one pass.

### 1.4 Tests

| File | Action |
|---|---|
| `RequestAccessTest.java` | **Delete** — 384 lines covering only the removed endpoint |
| `AccessRequestMapperSecurityTest.java` | **Rewrite.** Every test calls `requestAudienceAccessResponse` in setup, which no longer exists |
| `integration/TokenExchangeIntegrationTest.java` | Rewrite setup — compose the scope string directly instead of calling `requestAccessResponse` (`:55`) |
| `integration/TokenExchangeSecurityIntegrationTest.java` | Same, at `:49` and `:132`. **Keep `testCrossResourceExchangeFails` and `testTokenExchangeFailsWithoutAccessRequestScope`** — they are the evidence that audience is load-bearing |
| `util/BodhiProviderClient.java`, `BaseTest.java` | Remove `requestAccess*` / `requestAudienceAccess*` helpers; add a scope-composition helper |
| `AppInfoTest.java` | Extend for `redirect_uris` |

**Coverage replacing the deleted JPA assertions.** `testMapperRejectsForDifferentUser`,
`testMapperRejectsForUnauthorizedClient` and `testMapperRejectsNonExistentAccessRequest` are no
longer meaningful at the Keycloak layer — that authority moved to BodhiApp. Replace with:

- prefix names no client in the realm → rejected
- prefix names a client whose `bodhi.client_type != resource` → rejected
- value with no `.` separator → rejected
- well-formed value → `aud` contains the prefix and `access_request_id` equals the uuid

The user/app binding those tests used to cover is now asserted in BodhiApp's
`routes_app` suite instead — call that out in the commit message so the coverage move is traceable.

### 1.5 Verify and ship

`make test` → `make ci.quality` → `make openapi` (`openapi.json` / `openapi.yaml` are committed and
nothing in CI diffs them). Then `make release-server` to tag `release/vX.Y.Z`; GitHub Actions pushes
the multi-arch image to `ghcr.io/bodhisearch/bodhi-auth-server`.

**Gate:** Railway deploy to the `dev` environment is a manual dashboard action (`SETUP.md:94`,
`railway.toml:75-99`). Confirm the running image before starting Phase 3.

---

## Phase 3 — BodhiApp backend

Repo: `/Users/amir36/Documents/workspace/src/github.com/BodhiSearch/BodhiApp`. Upstream-first:
`services` → `routes_app` → OpenAPI/ts-client.

### 3.1 services

**Scope vocabulary** — new `crates/services/src/app_access_requests/scope_vocab.rs`:

- parse the app-facing scope string into `(UserScope, requested_llms: bool, requested_mcps: bool)`
- exactly one role token required; unknown tokens rejected
- neither `scope_apps_llms` nor `scope_apps_mcps` → error carrying
  `empty access requested, at least one of scope_apps_llms, scope_apps_mcps is required`
- compose the Keycloak-facing scope:
  `openid profile email roles scope_access_request:<resource-client-id>.<uuid>`

**Do not forward `scope_user_user` / `scope_user_power_user` to Keycloak.** An earlier revision of
the spec said to; that was wrong and would break the flow. Verified:

- The live realm config `realm-import-files/common.json` defines exactly two client scopes — `roles`
  and `scope_access_request` — and neither role scope appears in `defaultOptionalClientScopes`.
- `src/test/resources/import-files/bodhi-realm-v26.json`, which does contain them, is **completely
  unreferenced** — no Java, XML, Python, Makefile or `.ftl` file mentions it. It is a dead legacy
  fixture.
- Commit `6434d8d` removed them deliberately: *"Remove `scope_user_*` and `scope_token_*` client
  scopes (redundant with resource_access role claims)."*

Sending an unregistered scope to `/protocol/openid-connect/auth` returns `invalid_scope` and kills
the authorization. It is also unnecessary: the effective role is read from the DB row, not the token
— `token_service.rs:376-380` parses `validated_record.approved_role`, and `:384-392` clamps it
against `resource_access`. The role scope stays a purely app-facing vocabulary token, exactly like
`scope_apps_*`.

**Correction needed in the design spec** (`scratchpad/auth-params-design.md` §2 and §7) — it
currently states the opposite.

**`AccessRequestService`** (`access_request_service.rs:28-81`):
- add `create_approved(...)` — one call, writing an Approved row with `tenant_id` and `user_id` set at
  creation
- delete `create_draft`, `approve_request`, `deny_request`, `build_review_url`
- keep `get_request`, `list_approved_for_user`, `revoke_request`, `build_authorize_endpoint`
- add prior-grant lookup by `(tenant_id, app_client_id, user_id)` + `status = approved`, newest first

**`AccessRequestRepository`** — add `create_approved`; delete `update_approval`, `update_denial`,
`update_failure` and the bypass-RLS existence probe (`access_request_repository.rs:142-149`) and the
`with_tenant_txn("")` empty-tenant read path, both of which existed only because the row was born
anonymous.

**`AuthService`** — delete `register_access_request_consent` (`:120-126`, impl `:771-822`) and
`RegisterAccessRequestConsentResponse`. Add `redirect_uris: Vec<String>` to `AppClientInfo` (`:182-186`)
and remove the stale `// TODO: KC endpoint not yet implemented` at `:833`. `get_app_client_info` has
zero callers today and gains its first.

**Widen `ScopeClaims.aud`** (`crates/services/src/shared_objs/token.rs:92`) from `Option<String>` to
accept one-or-many via an untagged enum. `extract_claims` uses `serde_json::from_slice`, so today a
token carrying an `aud` **array** fails deserialization and is rejected whole — and Keycloak
serializes `aud` as an array as soon as there are two audiences. This is a latent availability bug
independent of the redesign; land it as its own commit at the head of Phase 3, before the flow
changes, so it is independently verifiable.

**Migration `m20250101_000028`** — data-only, no schema change:
- `UPDATE app_access_requests SET status='expired' WHERE status IN ('draft','failed')`
- `UPDATE app_access_requests SET status='revoked' WHERE status='approved' AND access_request_scope NOT LIKE '%.%'`
  — these carry the old undotted scope and can never validate again after Phase 1; marking them
  revoked stops the Connected Apps screen showing dead grants as live

Keep every `AppAccessRequestStatus` variant. Removing `Draft`/`Failed` would break
`sea_orm(value_type="String")` deserialization of historical rows for no benefit. Follow the style of
`m20250101_000027_app_access_request_source_id.rs`; register in `mod.rs` at both sites (`:28` and
after `:64`).

### 3.2 routes_app

**New handlers** in `crates/routes_app/src/apps/routes_apps.rs`, both in `user_session_apis`
(`routes.rs:346-363`):

| Method | Path | Purpose |
|---|---|---|
| GET | `/bodhi/v1/apps/access-requests/consent?<the full query string>` | validates and returns either `{ app, prior_grant, diff }` or `{ error, error_description, error_redirect_url \| null }` |
| POST | `/bodhi/v1/apps/access-requests` | create-and-approve (or deny); returns `{ id, redirect_url }` |

The GET takes the **whole query string**, not just `app_client_id`, so scope parsing and
`redirect_uri` exact-matching happen once in one backend place. The page must know *before* render
whether an error can be redirected or has to be shown in-app, and it needs app name, description,
`redirect_uris`, the prior grant and the three-group diff to render at all. "One call" in the design
spec means one *mutating* call — the frontend still does exactly one POST and one unconditional
`window.location.href =`.

**Delete** `apps_create_access_request`, `apps_get_access_request_status`,
`apps_get_access_request_review`, `apps_approve_access_request`, `apps_deny_access_request` and their
route constants (`:19-23`), plus their registrations at `routes.rs:120-130` and `:346-357`. **Keep**
`apps_list_user_access` and `apps_revoke_access_request` untouched.

Move verbatim into the new handler, do not rewrite: the privilege-ceiling block (`:294-330`) and the
MCP ownership/enabled checks (`:336-380`). These are the security core.

**`error.rs`** — add OAuth-shaped variants following the existing two-attribute pattern
(`PrivilegeEscalation` at `:43-48` is the model for struct variants):
`OauthInvalidRequest`, `OauthInvalidScope`, `OauthUnauthorizedClient`, `OauthAccessDenied`,
`RedirectUriMismatch`. Delete `ExchangeRequiresAuth`.

**`access_request_cache_needle`** (`token_service.rs:44`, called from `routes_apps.rs:517`) keeps
working — the `access_request_id` claim still exists. No change.

**`test_access_request_auth.rs`** — add the two new session endpoints to the 401 matrix.

### 3.3 OpenAPI and ts-client

`crates/routes_app/src/shared/openapi.rs` has four edit sites: DTO imports (`:1-6`), `__path_*`
imports (`:12-15`), `components(schemas(...))` (`:295-310`), `paths(...)` (`:486-493`).

Then `cargo run --package xtask openapi` → `make build.ts-client` → `make ci.ts-client-check`.

---

## Phase 4 — BodhiApp frontend and E2E

### 4.1 The new route

Create `crates/bodhi/src/routes/apps/auth/index.tsx`. Model it on the existing review page
(`routes/apps/access-requests/review/index.tsx`), which is the closest analogue.

- `validateSearch` — zod schema for `client_id`, `redirect_uri`, `response_type`, `state`,
  `code_challenge`, `code_challenge_method`, `scope`, optional `source_access_request_id`
- wrap in `<AppInitializer allowedStatus="ready" authenticated={true}>` — this is what makes the
  login-and-return work; `AppInitializer.tsx:79` stashes `window.location.href` (query string
  included) and `auth/callback/index.tsx:29-32` restores it via `handleSmartRedirect`
- `resolveShellRoute.ts:16` — add `/apps/auth` to `BARE_PREFIXES`, remove
  `/apps/access-requests/review`; update `resolveShellRoute.test.ts:16`

**Reuse, do not rewrite:** `GrantBlock` (`@/components/access-picker`), `grantableModelItems` /
`grantableMcpItems` (`@/lib/grantItems`), `computeRoleOptions` (currently local to `review/index.tsx:61-81`
— lift it), `previousGrantToState` (`-shared/previousGrantToState.ts`) for the reauthorize prefill,
and `safeNavigate` (`@/lib/safeNavigate`) for the final redirect.

**Simplification:** per-URL MCP requests are dropped, so `-components/McpServerCard.tsx` is no longer
needed and `toApproveBody` loses its `mcps[]` mapping. The screen becomes: model grant block, MCP
grant block, role select.

**Reauthorize diff view** — when a prior grant is in play, render three groups: still-requested
(pre-checked), newly-requested (highlighted), and being-relinquished (shown as dropped, with an
option to keep).

**Delete** the whole `routes/apps/access-requests/review/` directory, including `-shared/authUrl.ts`
(`validateAuthUrl`, `appendScopeToAuthUrl`, `readState`, `buildErrorRedirect`) and its 1086-line test
file. Update `ROUTE_APP_REVIEW_ACCESS` in `lib/constants.ts:19`.

### 4.2 Hooks and mocks

`hooks/apps/` — replace `useGetAppAccessRequestReview` / `useApproveAppAccessRequest` /
`useDenyAppAccessRequest` with `useGetAppInfo(appClientId)` and `useSubmitConsent()`. **Keep**
`useListAppAccess` and `useRevokeAppAccess` — the App Tokens page (`routes/tokens/apps/index.tsx`) is
unaffected.

Update `test-utils/msw-v2/handlers/apps.ts` and `test-fixtures/apps.ts` following their existing
patterns.

### 4.3 test-oauth-app — prove standard OAuth works

Rewrite `crates/lib_bodhiserver/test-oauth-app/` to drive the flow with **`oauth4webapi`** rather
than hand-rolled fetch calls, so the suite demonstrates that a direct integrator using a stock
library can complete the flow.

**Why this library** — it is the most-downloaded of the candidates, and `openid-client` (the runner-up)
is built on top of it, so proving the flow here proves it for both:

| package | weekly downloads |
|---|---|
| `oauth4webapi` | 12.86M |
| `openid-client` | 12.73M |
| `@auth/core` | 4.20M |
| `oidc-client-ts` | 2.50M |
| `arctic` | 0.87M |
| `simple-oauth2` | 0.70M |
| `@badgateway/oauth2-client` | 0.10M |

**Integration verified against the library source** (`oauth4webapi@3.8.7`), not assumed. Two facts
matter, because this design puts the authorize entry point on a different origin from the token
endpoint:

- `AuthorizationServer` declares `issuer` and `authorization_endpoint` as independent fields
  (`build/index.d.ts:557,561`) — there is no same-origin constraint between them.
- `validateAuthResponse` compares the returned `iss` against `as.issuer` only
  (`build/index.js:2068`, `if (iss && iss !== as.issuer)`). Keycloak emits `iss` and we set
  `as.issuer` to Keycloak's, so it passes. The origin of `authorization_endpoint` is never checked.
- The function that *would* enforce issuer/URL agreement is `processDiscoveryResponse`
  (`build/index.js:295`), and we do not call it — the server object is hand-built:

```ts
const as = {
  issuer: 'https://dev-id.getbodhi.app/realms/bodhi',
  authorization_endpoint: `${bodhiUrl}/ui/apps/auth/`,
  token_endpoint: 'https://dev-id.getbodhi.app/realms/bodhi/protocol/openid-connect/token',
};
```

Delete `src/lib/api.ts` (`requestAccess`) and `buildReviewRedirect` (`src/lib/oauth.ts:47-52`) — the
one function that encodes the old contract. `ConfigForm.handleRequestAccess` (`:76-137`) collapses to
PKCE generation plus one navigation. Keep every `data-testid`; `ConfigSection.mjs` depends on them.

### 4.4 E2E

The page object is the choke point. **Keep the method names and testids** on
`pages/AccessRequestReviewPage.mjs` (rename the file to `AppsAuthPage.mjs`) and six specs need no
change at all: `api-live-upstream`, `api-sdk-compat`, `mcps-mcp-proxy-everything`,
`mcps-sdk-compat-everything`, `mcps-auth-restrictions`, `mcps-oauth-dcr` — all use the consent screen
only as a setup step to mint a token.

Rewrite the specs whose assertions are about the consent surface itself:

| Spec | Why |
|---|---|
| `specs/oauth/oauth2-token-exchange.spec.mjs` | 3 flow tests; the exchange/upgrade test at `:170` POSTs `{exchange:true}` to the deleted endpoint and must move to `source_access_request_id` |
| `specs/tokens/app-tokens-grants.spec.mjs` | both tests drive grant selection and assert enforcement |
| `specs/mcps/mcps-oauth-auth.spec.mjs` | the denied test at `:249` pins `buildErrorRedirect`'s `error_source=bodhi` contract; the popup test at `:318` |
| `specs/oauth/oauth-chat-streaming.spec.mjs` | single flow test, page-object call site only |

**Delete** `specs/request-access/request-access-version-validation.spec.mjs` — it tests
`POST /bodhi/v1/apps/request-access` version rejection, which no longer exists. **Leave alone**
`specs/request-access/multi-user-request-approval-flow.spec.mjs`: `/ui/request-access/` is an
unrelated tenant-join feature that merely shares the name.

### 4.5 server_app

`crates/server_app/tests/test_oauth_external_token.rs:24` asserts the `review_url` reflects the Host
header — rewrite for the new route. `ExternalTokenSimulator`
(`tests/utils/external_token.rs:82-95`) seeds the exchange cache with a random `access_request_id`
not tied to any DB row; extend it to seed a real row so the new lookup path is exercised.

`crates/routes_app/tests/test_live_auth_middleware.rs:170-240` calls
`register_access_request_consent` directly — rewrite to seed an approved row and compose the scope.

---

## Verification

**Per phase, before each commit:** `make format`, `make test.backend`,
`cd crates/bodhi && npm test`, and from Phase 4 `make test.e2e`.

**End-to-end in dev**, after Phase 4:

1. `make build.dev-server && make app.run.live`
2. Drive the rewritten `test-oauth-app` through a full authorization in Chrome: it should navigate to
   `/ui/apps/auth/?...`, bounce through login, show the consent screen, and land back at the app with
   a working token.
3. Confirm the token's `aud` is the tenant's resource client id and its `access_request_id` claim
   matches the created row.
4. Call `/v1/chat/completions` with the token; confirm grant enforcement matches what was approved.
5. Denial path: deny at consent, confirm the app receives `error=access_denied` with its original
   `state` at its registered `redirect_uri`.
6. Tamper path: request with a `redirect_uri` not registered for the client; confirm an in-app error
   and **no** redirect.
7. Revoke from the App Tokens screen; confirm the token stops working.

**Multi-tenant**: repeat steps 2-4 against a multi-tenant deployment to confirm the tenant is resolved
from the session and the right resource client id lands in the scope.

---

## Risk and rollback

**Riskiest step: Phase 1.2, the stateless mapper.** It changes token minting for every app, and its
failure mode is silent — a token issued with no audience fails later, at BodhiApp's front door, not
at issuance. Mitigate by landing 1.1 (additive) and 1.2/1.3 (breaking) as separate commits, and by
running `TokenExchangeSecurityIntegrationTest` before tagging: `testCrossResourceExchangeFails` and
`testTokenExchangeFailsWithoutAccessRequestScope` are the assertions that prove audience still gates
the exchange.

**Rollback:** Phase 1 rolls back by redeploying the previous GHCR image — but the Liquibase
`dropTable` in changeSet 6 is not reversed, so a rollback after deploy leaves the old code without
its table. Treat the dev deploy as the point where forward-only begins, and verify thoroughly before
the production tag.

Phase 3's migration is data-only and reversible in principle, but the rows it marks `revoked` were
already unusable.
