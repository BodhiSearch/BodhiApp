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
the backend composes the Keycloak authorize URL server-side. The Keycloak-side record is deleted;
forgery is prevented by BodhiApp re-checking `(tenant_id, user_id, app_client_id, uuid)` against its
own row, which it already does. Five hops become three, the pre-create call and polling disappear,
the open redirect is eliminated by construction, and one of Keycloak's three SPI registrations goes
away.

**Scope of this plan:** `keycloak-bodhi-ext` and `BodhiApp` only. SDK and third-party app updates are
planned separately.

**Design spec:** `docs/archive/auth-params-design.md`. One correction to it is recorded below and
must be applied to that document: `scope_user_*` are **not** Keycloak client scopes and must never be
forwarded to Keycloak.

---

## Scope vocabulary

Two distinct vocabularies. Conflating them is the mistake an earlier revision made.

### App-facing — sent by the app to `/ui/apps/auth/`, consumed by BodhiApp only

Never forwarded to Keycloak. These drive the consent UI and constrain the record that gets created.

| Token | Effect |
|---|---|
| `scope_user_user` | role ceiling User |
| `scope_user_power_user` | role ceiling PowerUser; consent screen offers a User/PowerUser selector so the user may downgrade |
| *(no `scope_user_*`)* | inject `scope_user_user` |
| `scope_apps:llms` / `scope_apps:llms:true` / *(absent)* | LLM access requested |
| `scope_apps:llms:false` | LLM access **not** requested — section not rendered, no model grant may be stored |
| `scope_apps:mcps` / `scope_apps:mcps:true` / *(absent)* | MCP access requested |
| `scope_apps:mcps:false` | MCP access **not** requested — section not rendered, no MCP grant may be stored |

Defaults are permissive-then-narrowable so a third-party app can send a minimal request and still get
a sensible screen.

**Behaviour change to record:** today `RequestedResourcesV1` defaults `models_access` to `true` but
`mcps_access` to **`false`** (`access_request_objs.rs:184-201`). Defaulting both to true for
consistency means an app that requests nothing specific now gets an MCP section it never asked for,
and a user could grant MCP access to an app with no use for it. Deliberate, and the app can always
opt out with `scope_apps:mcps:false` — but it is a widening of the current default, not a
like-for-like port.

**`scope_apps:llms:false scope_apps:mcps:false` is valid**, not an error. It means the app is asking
only for role-gated API access via `scope_user_*` — no inference, no tools. The resulting record has
`models_access` and `mcps_access` both empty `Specific` and both list flags false, so `AccessPolicy`
denies models and MCPs while the `ExternalApp` context still carries the approved role for
role-gated routes. There is therefore **no "empty access" error condition at all**.

The consent screen must render meaningfully in that case rather than showing an empty form: no
resource sections, just the app identity, what is being granted in words ("access the Bodhi APIs as
User — no model or tool access"), and the approve/deny actions.

Unknown tokens are rejected rather than ignored — a typo that silently yields a grant-nothing token
is a worse failure than an error on a human-facing screen.

### Keycloak-facing — composed by the BodhiApp backend

```
openid profile email roles scope_access_request:<resource-client-id>.<uuid>
```

**One dynamic scope, one mapper.** A two-scope variant (`scope_access_request:<uuid>` plus a separate
`scope_resource:<resource-client-id>`) was considered and rejected: it would mean registering a
second dynamic client scope, mirroring it into two `.ftl` test realm templates, running a realm
import at every environment, maintaining two protocol mappers, and first spiking whether Keycloak
26.6.4 even accepts two different dynamic scopes in one request. The dotted form needs none of that.

**The existing realm config already accepts it.** `scope_access_request` is declared with
`dynamic.scope.regexp: scope_access_request:(.*)` (`realm-import-files/common.json`), so a dotted
value matches unchanged — **no realm-config change and no `make import.*` step**.

**Parse by splitting on the LAST `.`,** not the first. The uuid segment is a ULID or UUID and can
never contain a dot, so a last-dot split is correct regardless of what the client id contains.
Resource client ids are generated as `bodhi-resource-<uuid>` / `test-resource-<uuid>`
(`Constants.java:11-12`, `ResourceService.java:129`) and are dot-free today, but a last-dot split
removes the assumption rather than depending on it.

**`scope_user_*` must NOT be forwarded.** An earlier revision said to; that was wrong and would have
broken the flow. Verified:

- The live realm config `realm-import-files/common.json` defines exactly two client scopes — `roles`
  and `scope_access_request`. Neither role scope appears in `defaultOptionalClientScopes`.
- `src/test/resources/import-files/bodhi-realm-v26.json`, which does contain them, is **completely
  unreferenced** by any Java, XML, Python, Makefile or `.ftl` file. Dead legacy fixture.
- Commit `6434d8d` removed them deliberately: *"Remove `scope_user_*` and `scope_token_*` client
  scopes (redundant with resource_access role claims)."*

Sending an unregistered scope to `/protocol/openid-connect/auth` returns `invalid_scope`. It is also
unnecessary: the effective role comes from the DB row — `token_service.rs:376-380` parses
`validated_record.approved_role`, and `:384-392` clamps it against `resource_access`.

---

## Sequencing and gates

The Playwright E2E suite authenticates against the live `main-id.getbodhi.app` — the Railway `main`
environment (`INTEG_TEST_MAIN_AUTH_URL` in `crates/lib_bodhiserver/tests-js/.env.test`, default at
`test-helpers.mjs:66`). Note this is `main`, **not** `dev`: `dev-id.getbodhi.app` also exists
(`railway.toml:76-87`) but nothing in the E2E suite points at it. So the two repos cannot be verified
independently, and the Keycloak image must reach `main` before BodhiApp can be tested.

```
Phase 1  keycloak-bodhi-ext          →  merge, tag, build image
Phase 2  GATE: deploy image to main  →  no realm import needed
Phase 3  BodhiApp backend
Phase 4  BodhiApp frontend + E2E     →  verified against main Keycloak
Phase 5  GATE: production cutover    →  image + BodhiApp
```

**No realm import, and no DDL.** The dotted value matches the existing
`scope_access_request:(.*)` regexp, and the `bodhi_access_request` table is retained rather than
dropped. Each gate is therefore a plain image deploy, and rollback at any point is a redeploy of the
previous image.

**Production cutover invalidates every existing app grant.** Existing tokens carry an undotted
`scope_access_request:<uuid>`; after the mapper change that yields no audience and the token is
rejected. With one production app this is acceptable, but it is a user-visible re-authorization.

---

## Phase 1 — keycloak-bodhi-ext

Repo: `/Users/amir36/Documents/workspace/src/github.com/BodhiSearch/keycloak-bodhi-ext`

### 1.1 Add `redirect_uris` to app-info (additive, independently deployable)

- `AppInfoResponse.java` — third field, `@JsonProperty("redirect_uris")`, matching the JSON name in
  `ClientRequest.java:19-21`. Always emit the key; never `null`.
- `ResourceService.java:362` — the single construction site. `ClientModel.getRedirectUris()` returns
  an unordered `Set<String>`; sort it or assertions flake.
- `BodhiResourceProvider.java:214-230` — `@ApiResponse` schema doc only, no route change.
- `AppInfoTest.java` — extend; assert `[]` for a client with none.

Emitting the key unconditionally lets the Rust side model it as `Option<Vec<String>>` where *absent*
means "older extension, skip validation" and `Some([])` means "registered with no URIs, every match
fails". Do not collapse those.

### 1.2 Make the mapper stateless

**No realm-config change.** `scope_access_request` keeps its existing declaration and regexp; a
dotted value already matches. Nothing to mirror into the `.ftl` test templates, no `make import.*`.

`AccessRequestScopeProtocolMapper.java` — replace the body of the scope loop (currently `:82-98`).
The single existing mapper keeps doing both jobs:

1. Split the scope value on the **last** `.` → `resourceClientId` and `uuid`. Reject if there is no
   dot, or if either side is empty.
2. `realm.getClientByClientId(resourceClientId)`; reject if absent, or if its `bodhi.client_type`
   attribute is not `resource` (constants at `ResourceService.java:60-62`). This is what stops an app
   naming an arbitrary realm client as its audience.
3. `token.addAudience(resourceClientId)` and set claim `access_request_id` = `uuid`.
4. Keep the existing "one resource audience per token" guard (`:100-105`).

Delete the `AccessRequestRepository.findById` lookup and the user/client checks — BodhiApp enforces
`azp` at `token_service.rs:290-300` and `sub` at `:313-323` before the exchange.

### 1.3 Delete the access-request record

- `ResourceService.requestAccess` (243-343); the `POST users/request-access` route
  (`BodhiResourceProvider.java:190-212`); `AppAccessRequest.java`; `AppAccessResponse.java`.
- `jpa/BodhiAccessRequestEntity.java`, `jpa/AccessRequestRepository.java`.
- `jpa/BodhiJpaEntityProvider.java` — `getEntities()` returns `List.of()`. **Keep the provider,
  factory and `META-INF/services/` registration**, and leave `getChangelogLocation()` pointing at the
  existing changelog.
- **No `dropTable`, no changeSet 6, no DDL at all.** The `bodhi_access_request` table stays in place
  with its rows. Deleting the `@Entity` class simply leaves the table unmapped, which is harmless —
  and it makes rollback clean: redeploying the previous image restores the entity class, which finds
  its table and data exactly as it left them. This removes what was otherwise the single riskiest
  action in the plan.
- Add a `TECHDEBT.md` entry in `keycloak-bodhi-ext`: the `bodhi_access_request` table is no longer
  read or written by any code, retained deliberately in case something was missed. Note that dropping
  it later is a new changeSet `6` following the `dropTable` precedent of changeSets 4 and 5, and that
  once run it is not reversible by an image rollback because changeSet 1 stays recorded in
  `DATABASECHANGELOG`.
- Remove orphaned `AnalyticsEvents.ACCESS_REQUESTED` / `AnalyticsProps`. CI runs PMD dead-code
  (`make ci.quality`), so removal must be complete in one pass.
- `httpyac-scripts/common.http:69` and `marketplace-flow.http:170-199` reference the dead endpoint.

### 1.4 Tests

| File | Action |
|---|---|
| `RequestAccessTest.java` | **Delete** — 384 lines, covers only the removed endpoint |
| `AccessRequestMapperSecurityTest.java` | **Rewrite** — every test calls `requestAudienceAccessResponse` in setup |
| `integration/TokenExchange{,Security}IntegrationTest.java` | Rewrite setup to compose scopes directly. **Keep** `testCrossResourceExchangeFails` and `testTokenExchangeFailsWithoutAccessRequestScope` — they are the proof that audience gates the exchange |
| `util/BodhiProviderClient.java`, `BaseTest.java` | Drop `requestAccess*` / `requestAudienceAccess*`; add a scope-composition helper |
| `AppInfoTest.java` | Extend for `redirect_uris` |

**Replacing the deleted JPA assertions.** `testMapperRejectsForDifferentUser`,
`testMapperRejectsForUnauthorizedClient`, `testResourceClientBCannotUseResourceClientAAccessRequest`
and `testMapperRejectsNonExistentAccessRequest` are no longer meaningful at the Keycloak layer —
that authority moved to BodhiApp, and Phase 3.4 adds the Rust tests that carry it. Replace with:

- value with no `.` separator, or an empty side → rejected
- resource-client segment names no client → rejected
- names a client whose `bodhi.client_type != resource` → rejected
- two `scope_access_request` values naming different resource clients → rejected
- well-formed value → `aud` is exactly the resource client, `access_request_id` is the uuid
- a resource client id containing a dot still resolves correctly (last-dot split)

The suite also gets materially faster: no pre-registration round trip per test.

### 1.5 Ship

`make test` → `make ci.quality` → `make openapi` (committed artifacts, nothing in CI diffs them) →
`make release-server` tags `release/vX.Y.Z` → Actions pushes to `ghcr.io/bodhisearch/bodhi-auth-server`.

**Gate:** Railway deploy to the `main` environment is a manual dashboard action (`SETUP.md:94`,
`railway.toml:102-113`). Confirm
the running image digest. **No realm import is needed** — no client-scope config changed.

---

## Phase 3 — BodhiApp backend

Upstream-first: `services` → `routes_app` → OpenAPI/ts-client.

### 3.1 services

**`ScopeClaims.aud` widening — land this first, as its own commit.**
`crates/services/src/shared_objs/token.rs:92` is `Option<String>` and `extract_claims` uses
`serde_json::from_slice`, so a token carrying an `aud` **array** fails deserialization and is
rejected whole. Widen to an untagged one-or-many enum. Latent availability bug independent of this
work, and it sits directly under the new audience mechanism.

**New `crates/services/src/app_access_requests/app_scopes.rs`** — pure, table-driven:

- parse the app-facing vocabulary into `{ role: UserScope, llms: bool, mcps: bool }` applying the
  defaults above (absent role → `User`; absent/valueless/`:true` → allowed; `:false` → denied)
- reject unknown tokens. **`llms:false mcps:false` is valid** — a role-only request
- compose the Keycloak-facing scope string
- `validate_grant_against_scope(parsed, &ApprovedResourcesV1) -> Result<()>` — **the backend must
  refuse a grant the scope did not ask for.** If `llms == false`, `models_access` must be the empty
  `Specific` and `models_list` false; same for MCPs. This is what stops a tampered POST widening a
  grant beyond what the app requested.
- exact-match `redirect_uri` against the app's registered URIs

**`AccessRequestService`** (`access_request_service.rs:28-81`):
- add `create_approved(...)` writing an Approved row with `tenant_id`/`user_id` set at creation
- delete `create_draft`, `approve_request`, `deny_request`, `build_review_url`
- keep `get_request`, `list_approved_for_user`, `revoke_request`, `build_authorize_endpoint`
- prior-grant lookup by `(tenant_id, app_client_id, user_id)` + `status = approved`, newest first

**`AccessRequestRepository`** — add `create_approved`; delete `update_approval`, `update_denial`,
`update_failure`, the bypass-RLS existence probe (`:142-149`) and the `with_tenant_txn("")`
empty-tenant read path — both existed only because the row was born anonymous.

**`AuthService`** — delete `register_access_request_consent` (`:120-126`, impl `:771-822`) and
`RegisterAccessRequestConsentResponse`. `AppClientInfo` gains `redirect_uris: Option<Vec<String>>`;
drop the stale `// TODO: KC endpoint not yet implemented` at `:833` — the endpoint exists at
`BodhiResourceProvider.java:214-230`. `get_app_client_info` has zero callers today and gains its
first; note it requires a *user* token from a resource client
(`ResourceService.checkForUserToken:368-403`), which the consent page's session satisfies.

**Migration `m20250101_000028`** — data-only:
- `UPDATE app_access_requests SET status='expired' WHERE status IN ('draft','failed')`
- `UPDATE app_access_requests SET status='revoked' WHERE status='approved' AND access_request_scope NOT LIKE '%.%'`
  — legacy rows that can never validate again; stops the Connected Apps screen advertising dead grants

Keep every `AppAccessRequestStatus` variant — removing `Draft`/`Failed` breaks
`sea_orm(value_type="String")` deserialization of historical rows for no benefit. Follow
`m20250101_000027`'s style: local `DeriveIden` enum, one `alter_table` per column. Register in
`mod.rs` at both sites.

**Scopes are deliberately not stored on the row.** The record is immutable once created, so the
grant envelope is the source of truth. Accepted consequence: there is no server-side record of
*requested* versus *granted* for audit, and `validate_grant_against_scope` trusts the scope echoed
back in `query_params` — acceptable because the POST is made by the authenticated resource owner in
their own session, so the only thing a tampered scope could do is grant the app *more* than it asked
for. Revisit if a requested-vs-granted audit trail is ever needed.

### 3.2 routes_app

Both new handlers in `user_session_apis` (`routes.rs:346-363`):

| Method | Path | Purpose |
|---|---|---|
| GET | `/bodhi/v1/apps/access-requests/consent?<the full query string>` | `{ app, prior_grant, sections }` or `{ error, error_description, error_redirect_url \| null }` |
| POST | `/bodhi/v1/apps/access-requests` | create-and-approve or deny; returns `{ id, redirect_url }` |

The GET takes the **whole query string**, so scope parsing and `redirect_uri` matching live in one
backend place. The page must know *before* render whether an error can be redirected or must be
shown in-app, and it needs app identity, `redirect_uris`, the prior grant and the section flags to
render at all. "One call" means one *mutating* call — the frontend still does exactly one POST and
one unconditional `window.location.href =`.

**Delete** `apps_create_access_request`, `apps_get_access_request_status`,
`apps_get_access_request_review`, `apps_approve_access_request`, `apps_deny_access_request`, their
route constants (`:19-23`) and registrations (`routes.rs:120-130`, `:346-357`). **Keep**
`apps_list_user_access` and `apps_revoke_access_request`.

**Move verbatim, do not rewrite:** the privilege-ceiling block (`:294-330`) and the MCP
ownership/enabled checks (`:336-380`).

**`error.rs`** — add `OauthInvalidRequest`, `OauthInvalidScope`, `OauthUnauthorizedClient`,
`OauthAccessDenied`, `RedirectUriMismatch` following the existing two-attribute pattern
(`PrivilegeEscalation:43-48` models struct variants). Delete `ExchangeRequiresAuth`.

`access_request_cache_needle` (`token_service.rs:44` → `routes_apps.rs:517`) keeps working; the
`access_request_id` claim still exists.

`test_access_request_auth.rs` — add both new endpoints to the 401 matrix.

### 3.3 OpenAPI and ts-client

`shared/openapi.rs` has four edit sites: DTO imports (`:1-6`), `__path_*` imports (`:12-15`),
`components(schemas(...))` (`:295-310`), `paths(...)` (`:486-493`). Then
`cargo run --package xtask openapi` → `make build.ts-client` → `make ci.ts-client-check`.

### 3.4 server_app — the coverage that replaces the Keycloak checks

`ExternalTokenSimulator` (`tests/utils/external_token.rs:82-95`) seeds the exchange cache with a
random `access_request_id` tied to no DB row, so it bypasses exactly the code now carrying the
security burden. Extend it to insert a real row and seed a matching token, then add: foreign uuid →
`AppClientMismatch`; wrong `sub` → `UserMismatch`; unknown scope → `ScopeNotFound`; non-approved row
→ `NotApproved`; and an assertion that `aud` is exactly one value.

Rewrite `test_oauth_external_token.rs:24` (asserts `review_url` reflects the Host header — that URL
no longer exists) to assert the returned `redirect_url` is the Keycloak authorize endpoint carrying
both composed scopes. Rewrite `crates/routes_app/tests/test_live_auth_middleware.rs:170-240`, which
calls `register_access_request_consent` directly.

---

## Phase 4 — BodhiApp frontend and E2E

### 4.1 The route

`crates/bodhi/src/routes/apps/auth/index.tsx`, modelled on the existing review page.

- `validateSearch` — zod for `client_id`, `redirect_uri`, `response_type`, `state`, `code_challenge`,
  `code_challenge_method`, `scope`, optional `source_access_request_id`
- `<AppInitializer allowedStatus="ready" authenticated={true}>` — `AppInitializer.tsx:79` stashes
  `window.location.href` (query string included) and `auth/callback/index.tsx:29-32` restores it via
  `handleSmartRedirect`. **Handle the guest branch** at `AppInitializer.tsx:86-89`: a
  `resource_guest`/no-role user — exactly a first-time third-party user — is bounced to
  `ROUTE_REQUEST_ACCESS`, abandoning the flow and leaving a stale `bodhi-return-url` to misfire on a
  later login.
- `resolveShellRoute.ts:16` — add `/apps/auth` to `BARE_PREFIXES`, remove
  `/apps/access-requests/review`; update `resolveShellRoute.test.ts:16`

**Sections are driven by the parsed scope, not by envelope booleans.** `scope_apps:llms:false` means
the model section does not render; `scope_apps:mcps:false` means the MCP section does not render.
`scope_user_power_user` renders the role selector; otherwise the role is fixed at User.

**Prior-grant prefill must be scope-compliant.** When `mcps:false`, a prior grant's MCP selections
are neither shown nor carried into the new record — the section does not exist. The three-group diff
(still-requested / newly-requested / being-relinquished) applies only within rendered sections.

**Reuse:** `GrantBlock` (`@/components/access-picker`), `grantableModelItems` / `grantableMcpItems`
(`@/lib/grantItems`), `computeRoleOptions` (lift it out of `review/index.tsx:61-81`),
`previousGrantToState` (`-shared/previousGrantToState.ts`), `safeNavigate` (`@/lib/safeNavigate`).

Per-URL MCP requests are dropped, so `-components/McpServerCard.tsx` goes and `toApproveBody` loses
its `mcps[]` mapping.

**Delete** the whole `routes/apps/access-requests/review/` directory including `-shared/authUrl.ts`
and its 1086-line test. Update `ROUTE_APP_REVIEW_ACCESS` (`lib/constants.ts:19`).

### 4.2 Hooks and mocks

Replace `useGetAppAccessRequestReview` / `useApproveAppAccessRequest` / `useDenyAppAccessRequest`
with `useGetConsentContext(search)` and `useSubmitConsent()`. **Keep** `useListAppAccess` and
`useRevokeAppAccess` — the App Tokens page is unaffected. Update
`test-utils/msw-v2/handlers/apps.ts` and `test-fixtures/apps.ts` per their existing patterns.

Vitest coverage: default-injection cases (absent role → User; absent `scope_apps:*` → both sections
render), `:false` suppression, double-`false` error text, in-app error on `redirect_uri` mismatch,
scope-compliant prefill, and the unconditional `window.location.href = redirect_url`.

### 4.3 test-oauth-app — prove a stock library works

Rewrite `crates/lib_bodhiserver/test-oauth-app/` on **`oauth4webapi`** — the most-downloaded
candidate, and `openid-client` (the runner-up) is built on it, so proving this proves both:

| package | weekly downloads |
|---|---|
| `oauth4webapi` | 12.86M |
| `openid-client` | 12.73M |
| `@auth/core` | 4.20M |
| `oidc-client-ts` | 2.50M |
| `arctic` | 0.87M |
| `simple-oauth2` | 0.70M |
| `@badgateway/oauth2-client` | 0.10M |

**Verified against the library source** (`oauth4webapi@3.8.7`), because this design puts the
authorize entry point on a different origin from the token endpoint:

- `AuthorizationServer` declares `issuer` and `authorization_endpoint` as independent fields
  (`build/index.d.ts:557,561`) — no same-origin constraint.
- `validateAuthResponse` compares the returned `iss` against `as.issuer` only
  (`build/index.js:2068`). Keycloak emits `iss`; we set `as.issuer` to Keycloak's, so it passes. The
  origin of `authorization_endpoint` is never checked.
- `processDiscoveryResponse` (`build/index.js:295`) is what *would* enforce issuer/URL agreement, and
  we do not call it — the server object is hand-built:

```ts
const as = {
  issuer: 'https://main-id.getbodhi.app/realms/bodhi',
  authorization_endpoint: `${bodhiUrl}/ui/apps/auth/`,
  token_endpoint: 'https://main-id.getbodhi.app/realms/bodhi/protocol/openid-connect/token',
};
```

Delete `src/lib/api.ts` (`requestAccess`) and `buildReviewRedirect` (`src/lib/oauth.ts:47-52`) — the
one function encoding the old contract. `ConfigForm.handleRequestAccess` (`:76-137`) collapses to
PKCE generation plus one navigation. Keep every `data-testid`; `ConfigSection.mjs` depends on them.

### 4.4 E2E

Keep the page object's method names and testids (rename `AccessRequestReviewPage.mjs` →
`AppsAuthPage.mjs`) and six specs need no change: `api-live-upstream`, `api-sdk-compat`,
`mcps-mcp-proxy-everything`, `mcps-sdk-compat-everything`, `mcps-auth-restrictions`,
`mcps-oauth-dcr` — all use the consent screen only as a token-minting setup step.

Rewrite the specs whose assertions are about the consent surface:

| Spec | Why |
|---|---|
| `specs/oauth/oauth2-token-exchange.spec.mjs` | 3 flow tests; the upgrade test at `:170` POSTs `{exchange:true}` to the deleted endpoint and moves to `source_access_request_id` |
| `specs/tokens/app-tokens-grants.spec.mjs` | both tests drive grant selection and assert enforcement |
| `specs/mcps/mcps-oauth-auth.spec.mjs` | denial at `:249` pins the `error_source=bodhi` contract; popup at `:318` |
| `specs/oauth/oauth-chat-streaming.spec.mjs` | call site only |

**Delete** `specs/request-access/request-access-version-validation.spec.mjs` — it tests the removed
endpoint's version rejection. **Leave alone** `multi-user-request-approval-flow.spec.mjs`:
`/ui/request-access/` is an unrelated tenant-join feature that merely shares the name.

### 4.5 Tech debt entry

Add to `TECHDEBT.md`: `scope_user_*` and `scope_apps:*` are consumed by the BodhiApp UI and never
reach Keycloak, so the token's `scope` claim and the DB grant record describe different things.
Investigate registering them as Keycloak client scopes with passthrough — noting the O(M×N)
client-scope assignment problem that caused the `6434d8d` revert, and the fact that adding
audience-bearing scopes would turn `aud` into an array. Goal: claims present in both scope and record,
and verifiably matching.

---

## Verification

**Before each commit:** `make format`, `make test.backend`, `cd crates/bodhi && npm test`, and from
Phase 4 `make test.e2e`.

**End-to-end against the main Keycloak**, after Phase 4:

1. `make build.dev-server && make app.run.live`
2. Drive the rewritten `test-oauth-app` through a full authorization in Chrome — navigate to
   `/ui/apps/auth/?...`, bounce through login, consent, land back at the app with a working token.
3. Confirm the token carries `aud` = the tenant's resource client id (single value) and
   `access_request_id` = the created row.
4. `/v1/chat/completions` with the token; enforcement matches what was approved.
5. **Scope defaults:** request with no `scope_apps:*` → both sections render. With
   `scope_apps:mcps:false` → MCP section absent, and a POST carrying MCP grants is rejected.
   With no `scope_user_*` → role fixed at User. With `scope_user_power_user` → selector offered.
6. **Role-only grant:** `scope_apps:llms:false scope_apps:mcps:false` → consent screen shows no
   resource sections and states plainly what is granted; the resulting token is accepted on
   role-gated routes but denied on `/v1/chat/completions` and MCP.
7. **Denial:** deny at consent → app receives `error=access_denied` with its original `state`.
8. **Tamper:** unregistered `redirect_uri` → in-app error, **no** redirect.
9. Revoke from App Tokens → token stops working.

**Multi-tenant:** repeat 2-4 to confirm the tenant resolves from the session and the right resource
client id lands in the scope's resource-client segment.

---

## Risk and rollback

**Decisions recorded.** A two-release split — a non-destructive R1 keeping the legacy JPA path, then
a later R2 cleanup — was considered and **declined** in favour of a single release. Separately, the
`dropTable` was **dropped from scope**: no DDL runs at all, so the two decisions largely cancel out
on risk. What remains accepted deliberately is the cutover window, not a data-loss risk.

**Rollback is clean, because no schema changes.** Redeploying the previous Keycloak image restores
the `@Entity` class, which finds `bodhi_access_request` and its rows untouched. On the BodhiApp side
the only migration is data-only. There is no irreversible step in this plan.

**Riskiest remaining step: the cutover window.** Between the Keycloak deploy and BodhiApp landing,
the `main` environment is knowingly broken — the new mappers emit no audience for legacy single-scope tokens, and the
old BodhiApp calls a `users/request-access` endpoint that no longer exists. In production the same
window invalidates every existing app grant until the user re-authorizes. Keep the two deploys close
together and verify against `main` fully first.

**The mapper's failure mode is silent** — a token issued with no audience fails later at BodhiApp's
front door, not at issuance. `testCrossResourceExchangeFails` and
`testTokenExchangeFailsWithoutAccessRequestScope` are what keep this honest; run them before tagging.

**The manual Railway deploy is outside CI.** Nothing asserts the running image matches the tag, and
nothing diffs the committed `openapi.json`/`openapi.yaml`. Both need a human check at the gate.

Phase 3's migration is data-only; the rows it marks `revoked` were already unusable.
