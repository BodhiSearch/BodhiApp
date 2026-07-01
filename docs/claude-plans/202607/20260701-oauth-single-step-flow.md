# Single-Step 3rd-Party App OAuth Access-Request Flow

## Context

**Problem.** Today a 3rd-party app integrating with Bodhi App goes through a **two-step OAuth hop**:

1. App `POST /bodhi/v1/apps/request-access` → gets `{ id, review_url }`.
2. App redirects the user to `review_url` (Bodhi review screen); user approves/denies.
3. **Control returns to the 3rd-party app** (redirect flow: `redirect_url?id=…`; popup flow: poll + `window.close()`).
4. The app then **polls** `GET /bodhi/v1/apps/access-requests/{id}` to obtain the dynamic scope `scope_access_request:<id>`, **builds its own Keycloak authorize URL** (PKCE, state, base scopes + dynamic scope), and redirects the user to Keycloak.
5. Keycloak logs the user in, redirects back to the app with a code; the app exchanges it for a token carrying the `access_request_id` claim; Bodhi resolves DB grants from that claim.

That round-trip back into the app (step 3–4) is an extra callback every integrator must implement, and it doesn't fit the many standard OAuth flows we want to plug into.

**Change.** Collapse steps 3–4. The app generates PKCE + state up front, builds the **full Keycloak authorize URL** itself, and hands it (plus an `error_url`) to the Bodhi **review page as URL query params**. On approve, the review page appends the freshly-issued dynamic scope to that `auth_url`'s `scope` param and redirects the browser **straight to Keycloak** — no return trip to the app. On deny or validation failure, the review page redirects to `error_url` with OAuth-style error params plus an `error_source=bodhi` marker so the app knows which system raised it. Keycloak then redirects to the app's `redirect_uri` (registered) with the code, exactly as a normal OAuth login would.

**Key insight that keeps this simple:** `auth_url`/`error_url` live **only** in the review page URL (client-side). The backend never receives, validates, or stores them. The backend's only new responsibility is returning the freshly-minted `access_request_scope` from `approve`; the scope-append + redirect happen entirely in the review page. The backend contract *shrinks*.

**Security model (unchanged real boundaries).** Frontend `auth_url` validation is UX + defense-in-depth, **not** the security boundary. A caller who skips the Bodhi UI gains nothing: the dynamic scope is issued only by an **authenticated `approve`** (needs the victim's Bodhi session), and **Keycloak independently validates** the `redirect_uri` (must be pre-registered for the client) and PKCE. The `token_service` scope-validation + `access_request_id` claim path is **untouched**.

**No backwards compatibility / data migration** — no production 3rd-party app or Bodhi App is deployed. Existing migration and structs are edited in place.

**Scope: BodhiApp repo only.** Backend + review UI + the in-repo `test-oauth-app` + E2E. The published `bodhi-js` SDK (sibling repo `BodhiSearch/bodhi-browser`) gets the matching change in a follow-up; the in-repo `test-oauth-app` fully exercises both flows end-to-end and is the E2E vehicle.

---

## Final Contract

### `POST /bodhi/v1/apps/request-access` — request
```jsonc
{
  "app_client_id": "my-app-client",
  "requested": { "mcp_servers": [ { "url": "https://mcp.example.com/mcp" } ], "version": "1" },
  "requested_role": "scope_user_user"
}
```
Removed: `flow_type`, `redirect_url`. No `auth_url` in the body.

### `POST /bodhi/v1/apps/request-access` — response *(unchanged)*
```jsonc
{ "id": "550e…", "review_url": "http://localhost:1135/ui/apps/access-requests/review?id=550e…", "status": "draft" }
```
The app then navigates the user (top-tab **or** popup) to:
`review_url + "&auth_url=" + enc(authUrl) + "&error_url=" + enc(errorUrl)`

### Review page (frontend route only) — `/ui/apps/access-requests/review?id=<id>&auth_url=<enc>&error_url=<enc>`
On load, validate `auth_url`:
- origin + path **==** canonical Keycloak authorize endpoint (`auth_endpoint`, from the review response);
- required OAuth params present: `response_type=code`, `client_id`, `redirect_uri`, `scope`, `code_challenge`, `code_challenge_method=S256`, `state`;
- `client_id` **==** the request's `app_client_id`.
- **No URL-scheme check** — http and https both accepted for `auth_url` and `error_url`.

On failure → `window.location = error_url` with `error=invalid_request&error_source=bodhi&error_description=…`.

### `GET /bodhi/v1/access-requests/{id}/review` — response *(one field added)*
```jsonc
{ /* …existing (id, app_client_id, app_name, requested, requested_role, mcps_info, status)… */,
  "auth_endpoint": "https://kc.example.com/realms/bodhi/protocol/openid-connect/auth" }
```

### `PUT /bodhi/v1/access-requests/{id}/approve` — response
```jsonc
{ "status": "approved", "access_request_scope": "scope_access_request:550e…" }
```
Removed: `flow_type`, `redirect_url`. Frontend appends `access_request_scope` to `auth_url`'s `scope` param and `window.location = auth_url` (identical for popup and redirect).

### `POST /bodhi/v1/access-requests/{id}/deny` — response
```jsonc
{ "status": "denied" }
```
Frontend → `window.location = error_url` with `error=access_denied&error_source=bodhi&error_description=…&state=<state from auth_url>`.

### `GET /bodhi/v1/apps/access-requests/{id}?app_client_id=…` — *(unchanged)*
Kept as-is (status/observability), including its `access_request_scope` field. No longer on the critical path.

### Popup completion
The single popup navigates review → Keycloak → the app's `redirect_uri` (its OAuth callback, in the popup) with the code. **The Bodhi server never sees the code** (Keycloak → app directly), so the old poll-for-status mechanism cannot deliver it. The app callback in the popup must `postMessage` the result to `window.opener` (extensions: `chrome.runtime` messaging) and close. In this plan the in-repo `test-oauth-app` callback implements `postMessage`.

---

## Implementation (upstream → downstream)

### Phase 1 — `services` crate
- **`crates/services/src/app_access_requests/access_request_objs.rs`** — In `CreateAccessRequest` (~L313) remove `flow_type` and `redirect_url`. Remove the `FlowType` enum and all its references (grep `FlowType` across the workspace first — see `[[feedback_plan_verification]]`). Keep `RequestedResources`, `ApproveAccessRequest`, `UserScope` untouched.
- **`crates/services/src/app_access_requests/app_access_request_entity.rs`** — Drop `flow_type` and `redirect_uri` columns from the SeaORM `Model`.
- **`crates/services/src/db/sea_migrations/m20250101_000009_app_access_requests.rs`** — Edit in place: remove the `flow_type` and `redirect_uri` column definitions (no new migration; no prod data).
- **`crates/services/src/app_access_requests/access_request_service.rs`** — `create_draft` (~L116): drop the `redirect_uri` append (`?id={id}`) and `flow_type` handling. `approve_request` already gets `access_request_scope` back from `auth_service.register_access_request_consent` — ensure the service returns it to the caller. Add a small helper to expose the **authorize endpoint** for the review response (reuse the `{auth_url}/realms/{realm}/…` construction that `auth_service` already does at `register_access_request_consent`, ~L776 in `auth/auth_service.rs`); prefer a new `AuthService::authorize_endpoint()` (or setting-derived helper) returning `{auth_url}/realms/{realm}/protocol/openid-connect/auth`.
- **`crates/services/src/app_access_requests/access_request_repository.rs`** — Remove `flow_type`/`redirect_uri` from `create` / `update_*` row mapping.
- **Tests** — `test_access_request_service.rs`: delete `test_create_draft_redirect_*` / `missing_uri` cases (no more `redirect_url`); update `create_draft` assertions (no `flow_type`/`redirect_uri`); assert `approve` returns `scope_access_request:<id>`. `test_access_request_repository.rs`: update CRUD roundtrips for dropped columns (both SQLite + Postgres value sets). Follow `[[test-services]]` patterns.
- Gate: `cargo test -p services`.

### Phase 2 — `routes_app` crate + regen
- **`crates/routes_app/src/apps/apps_api_schemas.rs`** — `CreateAccessRequestResponse` unchanged. `AccessRequestActionResponse` → `{ status, access_request_scope: Option<String> }` (scope on approve, absent on deny) — or split into two response types; single type with optional scope is simpler. Add `auth_endpoint: String` to `AccessRequestReviewResponse`. Update `utoipa` annotations/examples.
- **`crates/routes_app/src/apps/routes_apps.rs`** — `apps_create_access_request` (~L52): drop redirect-URL scheme validation (XSS-VULN-01) since there's no `redirect_url` param anymore. `apps_approve_access_request` (~L250): return `{ status, access_request_scope }`. `apps_deny_access_request` (~L394): return `{ status }`. `apps_get_access_request_review` (~L171): populate `auth_endpoint` from the services helper.
- **Tests** — `crates/routes_app/src/apps/test_access_request.rs`: update approve success (assert `access_request_scope`, no `redirect_url`/`flow_type`), deny success (only `status`), review (assert `auth_endpoint`); drop popup-vs-redirect branching in create/approve/deny tests. Follow `[[test-routes-app]]`.
- Regen: `cargo run --package xtask openapi && cd ts-client && npm run generate`, then `make build.ts-client`.
- Gate: `cargo test -p routes_app`, then `make test.backend`. Capture long runs to a file per `[[feedback_capture_long_commands]]`.

**Not touched (verify no regressions):** `crates/routes_app/src/middleware/token_service/token_service.rs` and `.../access_requests/access_request_middleware.rs` — the `scope_access_request:*` extraction, `access_request_id` claim resolution, and DB-grant lookup are unchanged.

### Phase 3 — frontend review screen
- **`crates/bodhi/src/routes/apps/access-requests/review/index.tsx`** —
  - Add `auth_url` + `error_url` (both **required**) to the route's `validateSearch` alongside `id`.
  - On load (after review data resolves, so `auth_endpoint` + `app_client_id` are known): validate `auth_url` per the contract. On failure → `safeNavigate(errorUrlWith('invalid_request', …))`.
  - Remove `NonDraftStatus` `window.close()` + all `flow_type` branching (~L37–86, L151–164).
  - On **approve** success: read `access_request_scope`, append to the `scope` param of `auth_url`, `safeNavigate(authUrl)`.
  - On **deny**: `safeNavigate(errorUrlWith('access_denied', state))` where `state` is parsed from `auth_url`.
  - Reuse the existing `safeNavigate` helper — keep its `javascript:`/`data:` block (an XSS guard) but it already permits both http and https, which is what we want.
- **`crates/bodhi/src/hooks/apps/useAppAccessRequests.ts`** — Update `useApproveAppAccessRequest` / `useDenyAppAccessRequest` return types to the regenerated `@bodhiapp/ts-client` shapes. `useGetAppAccessRequestReview` now surfaces `auth_endpoint`.
- **`toApproveBody.ts`** — unchanged.
- **Unit tests** — extend the review route's test (MSW-mocked) to cover: valid `auth_url` → approve → navigation to `auth_url` with appended scope; invalid `auth_url` → redirect to `error_url`; deny → redirect to `error_url` with `state`. Assert via a mocked navigation spy.
- Gate: `cd crates/bodhi && npm run test`.

### Phase 4 — in-repo `test-oauth-app` (E2E 3rd-party app)
Location: `crates/lib_bodhiserver/test-oauth-app/` (self-contained; has `lib/oauth.ts` `buildAuthUrl` + PKCE, `lib/api.ts`, `components/ConfigForm.tsx`, `pages/AccessCallbackPage.tsx`).
- **`components/ConfigForm.tsx`** — Request body drops `flow_type`/`redirect_url`. Generate PKCE (`code_verifier` + `code_challenge`) + `state` (existing helpers), build `auth_url` via `buildAuthUrl`, build `error_url` (app's own error page), persist `code_verifier`+`state`, then navigate to `review_url + &auth_url=… + &error_url=…` — top-tab for redirect flow, `window.open(...)` for popup flow.
- **OAuth callback page** (repurpose `AccessCallbackPage.tsx` or add `OAuthCallbackPage`) — This is the app's `redirect_uri`. Read `code`+`state` (success) or `error`+`error_source`+`error_description` (failure). Validate `state`, exchange `code` for token with `code_verifier`. **Redirect flow:** render result in the same tab. **Popup flow:** `postMessage({ code|error }, targetOrigin)` to `window.opener`, then `window.close()`. Remove all status-polling.
- **Error page** — `error_url` target; renders the OAuth-style error (incl. `error_source=bodhi`) with a stable terminal `data-test-state` for E2E.
- Opener side (popup): listen for the `postMessage`, then complete/exchange.

### Phase 5 — E2E (Playwright)
Location: `crates/lib_bodhiserver/tests-js/`. Grow **one** spec with many `test.step`s per `[[feedback_phased_vertical_slice_dev]]`; strictly black-box (`[[feedback_blackbox_e2e]]`), no conditional branching (`[[feedback_no_ifelse_in_e2e]]`), no `test.skip` for env (`[[feedback_no_skip_for_missing_env]]`).
- **Update page objects** — `pages/AccessRequestReviewPage.mjs` (approve/deny unchanged); rework `pages/sections/OAuthSection.mjs` + `AccessCallbackSection.mjs` for the new single-hop path (review → Keycloak → app callback), removing the app-side status-poll waits.
- **Redirect flow, approve** — app creates request → user lands on review with `auth_url`+`error_url` → (login to Bodhi if needed; ensure the login round-trip preserves the query params) → approve → **direct** Keycloak redirect (SSO-silent, since the user just authenticated) → app `redirect_uri` with code → token exchange succeeds → app can call a protected Bodhi endpoint.
- **Redirect flow, deny** — deny → browser lands on `error_url` with `error=access_denied&error_source=bodhi` → app error page shows terminal error state.
- **Popup flow, approve** — popup navigates review → Keycloak → app callback (in popup) → `postMessage` to opener → opener completes; popup closes.
- **Popup flow, deny** — popup → `error_url` → `postMessage` error to opener.
- Set `reducedMotion: 'reduce'` if the review screen participates in view transitions (`[[feedback_e2e_reduced_motion_for_view_transitions]]`).
- Update the version-validation spec (`request-access-version-validation.spec.mjs`) for the trimmed create body.
- Commands per `[[feedback_e2e_test_commands]]`: `make build.dev-server` then `make test.e2e` from `crates/lib_bodhiserver/tests-js`.

### Phase 6 — docs
- Update `crates/services/CLAUDE.md`, `crates/routes_app/CLAUDE.md`, and any access-request flow doc under `docs/` to describe the single-step flow and the new contract. Note the `bodhi-js` SDK follow-up.

---

## Verification

1. **Per-crate unit gates:** `cargo test -p services` (Phase 1), `cargo test -p routes_app` (Phase 2), then `make test.backend` after all Rust changes (Docker required for Postgres).
2. **Type sync:** `cargo run --package xtask openapi && cd ts-client && npm run generate && make build.ts-client`; `make ci.ts-client-check` to confirm no drift.
3. **Frontend unit:** `cd crates/bodhi && npm run test`.
4. **Manual Chrome walk-through** (`[[feedback_functional_testing]]`, `[[feedback_make_app_run]]`): `make app.run.live`, run the `test-oauth-app`, and drive both flows in the browser — approve reaches Keycloak with the appended `scope_access_request:<id>` and returns to the app with a working token; deny lands on `error_url` with `error_source=bodhi`. Confirm via network panel that there is **no** return hop into the app between review and Keycloak.
5. **E2E:** `make build.dev-server` then `make test.e2e` — redirect + popup, approve + deny, all green.
6. **Gate before each phase commit:** `make format`, backend + UI tests, and E2E when touched (`[[feedback_run_all_gate_checks]]`). Feature rollout → commit per phase (`[[feedback_layered_refactors]]`); rebase onto `origin/main` (trunk-based, linear history).

## Out of scope / follow-up
- `bodhi-js` SDK (`BodhiSearch/bodhi-browser`): `AccessRequestBuilder`, `DirectClient`, `ExtClient`, `review-manager` must adopt the URL-passthrough + `postMessage` model. Tracked separately.
