# Exchange (Upgrade) Access Token — Preselect Flow

> **Post-approval adjustments (in effect):**
> 1. The stored column/field is named **`source_access_request_id`** (was `previous_access_request_id`).
> 2. **SDK + E2E are out of scope** for this iteration — implement BodhiApp (Rust + React) with backend integration tests + Vitest/MSW frontend tests only. The `bodhi-js-sdk` change + black-box E2E are deferred (they require publishing ts-client/app-bindings, bumping, then wiring — tracked separately).

## Context

Today a 3rd-party app gets scoped access to a user's resources via the **app consent flow**: it POSTs `/bodhi/v1/apps/request-access` (unauthenticated) to create a *draft*, the owner reviews at `/ui/apps/access-requests/review/?id=…`, and on approve the backend registers a Keycloak consent scope (`scope_access_request:<id>`) + persists `ApprovedResources` JSON. When the app later calls Bodhi with its KC token, the RFC-8693 token-exchange machinery (`handle_external_client_token`) resolves grants from that access-request row.

An app that **already holds an approved grant** and wants **more** permissions currently has to start a fresh request from a blank, fail-closed consent form — the owner must re-pick everything by hand. We want an **upgrade / exchange** path: the app re-requests with an `exchange` flag and its **current token in the `Authorization` header**; the backend records which prior grant this upgrades; and the review page loads **pre-selected with the app's existing permissions** so the owner only edits the delta.

This is the deferred **Phase 8** of `docs/claude-plans/202606/20260628-token-api-app.md`, deliberately **scoped down**: this iteration does the *preselect plumbing only*. Approve still just mints a fresh grant as it does today — it does **not** supersede/invalidate the old token. Marking the source token `exchanged` + rejecting it is captured as a **nice-to-have** (see bottom; append to `nicetohave.md` as implementation Step 0).

## Locked decisions (from interview)

1. **Exchange target = external-app consent token** (the KC token the app already holds from a prior *approved* access request). We read its `access_request_id`, load that request's `ApprovedResources` + `approved_role` to preselect the review form. **Not** user-minted `bodhiapp_` API tokens.
2. **Scope = preselect plumbing only.** `exchange` flag on request-access → parse token from header → store prior-grant ref on the draft → embed prior grant in the review response → preselect the form → **on approve, mint a new grant (unchanged)**. No supersede column, no `Exchanged` status, no old-token invalidation this iteration.
3. **Include SDK + E2E.** Update `bodhi-js-sdk` (bodhi-browser repo) to send the current token via `Authorization` + an `exchange` flag, and grow a black-box Playwright E2E.
4. **`exchange: true` with no auth header ⇒ reject.**
5. **Move the route to optional-auth.** Move `.route(ENDPOINT_APPS_REQUEST_ACCESS, post(apps_create_access_request))` from `public_apis` to the `optional_auth` group in `routes.rs` so `AuthContext` is injected. (Both groups sit under the same `public_router` `.layer(permissive_cors())`, so cross-origin CORS is preserved — verified.)

## End-to-end flow

```
App (already approved, holds KC token)
  └─ SDK: requestAccess({..., exchange:true}) WITH Authorization: Bearer <KC token>
        → POST /bodhi/v1/apps/request-access   (now under optional_auth_middleware)
             → AuthContext::ExternalApp { app_client_id, access_request_id: <prior> }
             → handler: exchange==true ⇒ require ExternalApp + app_client_id match, else 401
             → create_draft stores previous_access_request_id = <prior>
        → returns { id, review_url }
  └─ redirect owner to /ui/apps/access-requests/review/?id=<id>&auth_url=…&error_url=…
        → GET /bodhi/v1/access-requests/{id}/review (owner session)
             → draft has previous_access_request_id ⇒ load prior (Approved) request
             → embed previous_grant { approved_role, approved: ApprovedResources }
        → review form initialises from previous_grant (preselected), owner edits delta
        → PUT …/approve  → mints fresh grant + KC scope (UNCHANGED)
```

## Backend changes (upstream → downstream)

### 1. Migration (`services`)
New ALTER migration `crates/services/src/db/sea_migrations/m20250101_000027_app_access_request_previous_id.rs` (next free number after `000026`; register in the migrator list exactly as `000026` is registered):
- `ALTER TABLE app_access_requests ADD COLUMN previous_access_request_id TEXT NULL`.
- No new RLS needed — existing table policies apply to the added column. Extend the dual `#[values("sqlite","postgres")]` migration test to assert the new column exists and defaults NULL.
- **Do not** add `Exchanged` status / `supersedes_id` — those belong to the nice-to-have.

The single explicit flag (`exchange:true`) is represented by `previous_access_request_id IS NOT NULL`; no separate boolean column.

### 2. Domain types (`crates/services/src/app_access_requests/access_request_objs.rs`)
- `CreateAccessRequest` (`:290`): add `#[serde(default)] exchange: bool`. Client-supplied; the previous id itself is **derived server-side from the token**, never taken from the body.
- `AppAccessRequest` row struct (`:7`): add `previous_access_request_id: Option<String>`.
- `AppAccessRequestEntity` (`app_access_request_entity.rs:8`): add the column mapping.

### 3. Service + repository (`crates/services/src/app_access_requests/`)
- `AccessRequestService::create_draft` (`access_request_service.rs:116`): add a `previous_access_request_id: Option<String>` param, persist it on the draft row. (Service stays not-auth-scoped; the handler resolves the id from `AuthContext` and passes it in.)
- Repository `create` writes the new column; `get_request`/`from_row` read it through.

### 4. Handler + routing (`crates/routes_app/src/apps/routes_apps.rs` + `routes.rs`)
- **Move the route** (`routes.rs:120-123`): cut the `ENDPOINT_APPS_REQUEST_ACCESS` route out of `public_apis` and add it to the `optional_auth` router (`:129`). Leave `ENDPOINT_APPS_ACCESS_REQUESTS_ID` (status poll) in `public_apis`.
- **Handler `apps_create_access_request` (`:52`)**: signature already takes `AuthScope` implicitly? Today it takes `AuthScope`(auth_scope) + `ValidatedJson<CreateAccessRequest>`. With optional-auth it now receives a populated `AuthContext`. Logic:
  - `let prev = if request.exchange { … } else { None }`.
  - When `exchange`: match `auth_scope.auth_context()`:
    - `AuthContext::ExternalApp { app_client_id, access_request_id: Some(prev_id), .. }` **and** `app_client_id == request.app_client_id` ⇒ `Some(prev_id)`.
    - anything else (Anonymous, mismatch, missing `access_request_id`) ⇒ return the new error (401).
  - Pass `prev` into `create_draft(app_client_id, requested, requested_role, prev)`.
  - Non-exchange path is byte-for-byte unchanged (prev = None).

### 5. Review response embedding (`crates/routes_app/src/apps/`)
- New schema `PreviousGrantInfo { approved_role: UserScope, approved: ApprovedResources }` in `apps_api_schemas.rs`.
- `AccessRequestReviewResponse` (`:44`): add `previous_grant: Option<PreviousGrantInfo>`.
- Handler `apps_get_access_request_review` (`routes_apps.rs:154`): if `draft.previous_access_request_id` is `Some(id)`, `get_request(id)`; when that prior request is `Approved` with an `approved` payload, deserialize (fail-closed) and set `previous_grant = Some({approved_role, approved})`. Prior not found / not approved / corrupt ⇒ `previous_grant = None` (UI falls back to fail-closed defaults; log at warn).

### 6. Error (`routes_app`)
Add a `BodhiErrorResponse`-mapped variant (e.g. `AppsError::ExchangeRequiresAuth` → **401**, code `apps_error-exchange_requires_auth`). Assert via `.code()` in tests, per conventions.

### 7. OpenAPI
Register `PreviousGrantInfo` schema + the changed `CreateAccessRequest`/`AccessRequestReviewResponse`; `cargo run --package xtask openapi` then `make build.ts-client`.

## Frontend changes (`crates/bodhi/src`)

The review page is `routes/apps/access-requests/review/index.tsx`; form state is plain `useState` in `ReviewContent` (`:89-103`), fail-closed by default. The pure `toApproveBody(req, mcpsInfo, state)` (`review/-shared/toApproveBody.ts`) already maps UI-state → `ApprovedResources`.

- **New pure helper** `review/-shared/previousGrantToState.ts`: `previousGrantToState(previous: PreviousGrantInfo, mcpsInfo): PreselectState` — the inverse mapping:
  - `listModels` ← `previous.approved.models_list`
  - `modelMode`/`models` ← `previous.approved.models_access` (`ModelGrant`: `{type:'all'}`→`'all'`; `{type:'specific',ids}`→`'specific'`+ids)
  - `listMcps` ← `previous.approved.mcps_list`
  - `mcpExtraMode`/`mcpsExtra` ← `previous.approved.mcps_access` (`McpGrant`)
  - `approvedMcps`/`selectedMcpInstances` ← `previous.approved.mcps` (per-url `McpApproval[]`: approved url → `true` + `instance.id`, keyed by url, aligned against `mcpsInfo`)
  - `approvedRole` ← `previous.approved_role`
- **Wire preselect**: in `ReviewContent`, when `reviewData.previous_grant` is present, seed the `useState` initialisers (and the existing on-load effects at `:155-169`) from `previousGrantToState(...)` instead of the fail-closed defaults. When absent, behaviour is unchanged. Reuse the existing `GrantBlock`/`AccessPicker`/`McpServerCard` stack — no new components. (The green/amber "previously granted vs new" pills are **out of scope** — noted at `access-picker.css:5` as intentionally omitted.)
- Approve continues to call `toApproveBody` → `PUT …/approve`; no change to the mutation.

## SDK changes (bodhi-browser repo: `bodhi-js-sdk/`)

- **Type**: `CreateAccessRequest` gains `exchange?: boolean` via the regenerated `@bodhiapp/ts-client` (link/local for E2E — verify per the "published vs installed" gotcha).
- **Builder** `core/src/access-request.ts`: add `.exchange(true)` → sets `exchange` in `build()`.
- **Transport** `core/src/direct-client-base.ts:665-675`: `requestAccess` currently calls `sendApiRequest('POST', endpoint, body, {}, /*authenticated*/ false)`. Change the last arg to `body.exchange === true` so `_getAccessTokenRaw()` attaches `Authorization: Bearer <current token>` only for exchange.
- **Flow** `web/src/direct-client.ts` (`login`/facade): expose an upgrade entry (builder sets `.exchange(true)`) that requires existing auth state; non-exchange `login` is unchanged.

## Tests (all layers)

- **services (`cargo test -p services --lib`)**: migration test asserts new column; `create_draft` persists `previous_access_request_id`; round-trip `CreateAccessRequest{exchange}` serde default; repository isolation includes the new column.
- **routes_app (`oneshot`)**: exchange + `ExternalApp` context ⇒ draft stores prev id; exchange + Anonymous ⇒ 401 (`.code()`); exchange + `app_client_id` mismatch ⇒ 401; non-exchange still creates draft anonymously (route-move regression); review returns `previous_grant` when draft points at an Approved request, `None` when prior missing/denied. Use the existing `test_utils` `ExternalApp` AuthContext helper.
- **frontend (Vitest + MSW)**: `previousGrantToState.test.ts` pure-maps All/Specific + per-url MCP + role; `review/index.test.tsx` exchange case — MSW review response with `previous_grant` ⇒ form renders preselected (list toggles, all/specific + ids, MCP rows, role) and approve posts the expected `ApprovedResources`.
- **E2E (Playwright, black-box, grow ONE spec with `test.step`s)**: locate the current app-consent spec under `crates/lib_bodhiserver/tests-js/specs/`. Scenario: app approves grant G1 (existing flow) → app initiates `exchange` requesting a superset → owner opens review, **asserts the form is preselected with G1** → approves the upgraded grant → new grant active. Only UI interactions (no `page.evaluate`); `throw` in `beforeAll` on missing env; no if-else branching; `reducedMotion:'reduce'` for the rail. Depends on the linked SDK + ts-client carrying `exchange`.

## Nice-to-have hardening (Implementation Step 0: append verbatim to `docs/claude-plans/nicetohave.md`)

> **Exchanged-token hardening (token exchange / upgrade flow).** Not required for the preselect feature; hardens invalidation of the superseded token.
> - In the middleware: when a 3rd-party OAuth token is parsed **for the first time**, check whether it is an *exchanged* token (its access request was superseded by an upgrade). If so, mark the **source** token/request as `exchanged`, and purge it from the exchange cache using the existing needle-based removal method (`cache_service.remove_entries_containing`, same as `revoke` via `access_request_cache_needle(id)`).
> - Add an `Exchanged` value to the `AppAccessRequestStatus` enum + column, and reject any token whose access request is `Exchanged`.
> - An `Exchanged` request's status must **not** be resettable via Revoke/Approve (guard those transitions).

## Verification

1. `make app.run.live`; drive the app-consent flow to approve an initial grant (one model + one MCP). Then trigger an exchange (SDK `.exchange(true)`, current token in `Authorization`). Confirm `/ui/apps/access-requests/review/?id=…` loads **preselected** with the prior grant (list toggles, all/specific + model ids, MCP rows + instances, role). Edit the delta, approve; confirm a fresh grant mints and the app works with it. (Old token still valid — invalidation is the nice-to-have.)
2. `curl` `POST /bodhi/v1/apps/request-access` with `exchange:true` and **no** `Authorization` ⇒ 401; with a valid external-app bearer ⇒ 201 + draft carries `previous_access_request_id`; without `exchange` ⇒ 201 anonymous (route-move regression).
3. Gates: `make test.backend`, `cd crates/bodhi && npm test`, and the grown E2E spec all green; `cargo run --package xtask openapi` + `make build.ts-client` clean.

## Critical files

- `crates/services/src/app_access_requests/{access_request_objs.rs, access_request_service.rs, access_request_repository.rs, app_access_request_entity.rs}`
- `crates/services/src/db/sea_migrations/m20250101_000027_app_access_request_previous_id.rs` (+ migrator registration)
- `crates/routes_app/src/apps/{routes_apps.rs, apps_api_schemas.rs}`, `crates/routes_app/src/routes.rs` (route move), apps error + `shared/openapi.rs`
- `crates/bodhi/src/routes/apps/access-requests/review/{index.tsx, -shared/previousGrantToState.ts}` (+ tests)
- `bodhi-js-sdk/core/src/{access-request.ts, direct-client-base.ts}`, `bodhi-js-sdk/web/src/direct-client.ts` (bodhi-browser repo)
- `crates/lib_bodhiserver/tests-js/specs/` app-consent E2E spec + page objects
- `docs/claude-plans/nicetohave.md` (Step 0 append)
</content>
</invoke>
