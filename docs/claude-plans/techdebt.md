# Tech Debt Backlog

Deferred work items intentionally scoped out of their originating effort, to be addressed later.

## Deferred tokens-review nice-to-haves (Batch 4)
- **Source**: Tokens screen-v2 / App Token grants review — Batch 4 (2026-06-30). Deferred for session
  length; all low-value.
- **F35** — `ReviewContent` (`routes/apps/access-requests/review/index.tsx`) fetches models + MCPs
  unconditionally. Make them conditional on the requested flags (`reviewData.requested.models_access`
  / `mcps_access`). Needs `useListModels` (`hooks/models/useModels.ts`) to accept an `enabled` option
  (it currently takes only a `filter`), so it's a shared-hook signature change. Perf-only.
- **F8** — rename `DbError::AccessRequestNotDraft` (`services/src/db/error.rs`): it now guards the
  revoke transition (must-be-Approved), so the "NotDraft" name misleads. Rename to a status-neutral
  variant (e.g. `AccessRequestStatusConflict`) + update the 4 call-sites and the error-code assertions.
- **Component tests** — F37 (ListingToggle Space/Enter activation), F39 (GrantBlock), F40 (TokenForm
  PowerUser card disabled for `resource_user`). The components ship and are exercised indirectly;
  these add focused unit coverage.

## Relocate access-request handlers to a dedicated `access_requests` module + normalize endpoint paths
- **Source**: Tokens screen-v2 / App Token grants review — finding F32 (`docs/claude-plans/202606/screen-v2/tokens-review/`).
- **Date logged**: 2026-06-30
- **What**: `access-requests` is a domain model, but its handlers currently live under
  `routes_app/src/apps/`. Move them to a dedicated `routes_app/src/access_requests/` module and
  normalize the endpoint path set to the domain-first shape:
  ```
  ENDPOINT_ACCESS_REQUESTS_REVIEW   = "/bodhi/v1/access-requests/{id}/review"
  ENDPOINT_ACCESS_REQUESTS_APPROVE  = "/bodhi/v1/access-requests/{id}/approve"
  ENDPOINT_ACCESS_REQUESTS_DENY     = "/bodhi/v1/access-requests/{id}/deny"
  ENDPOINT_ACCESS_REQUESTS_APPS     = "/bodhi/v1/access-requests/apps"
  ENDPOINT_ACCESS_REQUESTS_REVOKE   = "/bodhi/v1/access-requests/{id}/revoke"
  ```
- **Why deferred**: expands scope beyond the grants review; touches routing, OpenAPI, ts-client,
  and frontend wiring.
- **Blocks these review fixes** (discovered during Batch 1 impl, 2026-06-30): the *app* access-request
  endpoints currently share the `/bodhi/v1/access-requests/{id}/...` namespace with the *user*
  (Manager/Admin) access-request domain (`users/routes_users_access_request.rs`), separated only by
  HTTP method (app approve = `PUT`, user approve = `POST`, both at `…/{id}/approve`; user list =
  `GET /bodhi/v1/access-requests`). Until the app endpoints are relocated to their own namespace
  (e.g. `/bodhi/v1/app-access-requests/*`), these three review fixes are **infeasible** and are
  deferred here:
  - **F9** — app approve `PUT`→`POST` (collides with user-domain `POST …/approve` at the same path).
  - **F32** — drop `/apps` from the list endpoint (`/bodhi/v1/access-requests` is taken by the user
    domain's `listAllAccessRequests`).
  - **F33** — de-pluralize operationId `approveAppsAccessRequest` (a bare `approveAccessRequest`
    duplicates the user domain's operationId). Do this rename as part of the relocation.
- **Note / reconcile on landing**: the review's in-scope decision drops the `/apps` qualifier on
  the *user's* list endpoint (use `/bodhi/v1/access-requests/` to list a user's access-requests,
  since user access-requests now live under `/users/*` and the `/apps` disambiguation is
  redundant). When this debt item is picked up, reconcile `ENDPOINT_ACCESS_REQUESTS_APPS` with that
  decision. `/bodhi/v1/apps/*` remains a reserved prefix for endpoints accessed directly by 3rd-party
  apps — that placement is acceptable and is **not** a reason to move on its own.
- **Reference**: see `decision.md` (F32) in the tokens-review folder for the full rationale.

## Inference grant middleware double-parses the request body
- **Source**: Grants-review remediation — finding I2 (`docs/claude-plans/202606/review/architecture-review.md`), 2026-07-01.
- **What**: For OpenAI / Anthropic inference, `model_inference_grant_middleware`
  (`routes_app/src/middleware/model_grant.rs`) buffers the full body and parses it (into a minimal
  `ModelField`) to read `model`, then reconstructs the request; the handler's `Json<…>` extractor
  parses the same in-memory body a **second** time. There is a `// TODO: inefficient interceptor`
  comment at the read site. (Batch 2 already removed the wasted read for the dominant `Unrestricted`
  session principal by short-circuiting before buffering, and Gemini/MCP paths are single-read.)
- **Why deferred**: a true single-parse requires the middleware to parse the full typed payload and
  hand it to the handler via request extensions (touching 4 handlers) — bigger than the review's
  scope, and the residual cost only applies to grant/deny (token/app) principals, not sessions.
- **Fix**: parse once in the middleware and stash the parsed value in `req.extensions()`; have the
  OpenAI/Anthropic handlers read from extensions instead of re-extracting. Revisit only if profiling
  flags it.

## Missing embeddings/responses grant-enforcement parity test
- **Source**: Grants-review remediation — architecture-review "Missing Test Coverage", 2026-07-01.
- **What**: The unified inference middleware was created specifically to close the `/v1/embeddings`
  and `/v1/responses` gap, but the routes_app tests only assert a non-granted token gets 403 on
  `/v1/chat/completions`. There is no integration test pinning 403 on `/v1/embeddings` and
  `/v1/responses` specifically.
- **Why deferred**: nice-to-have; the middleware `classify()` covers all three via one code path, so
  the risk is a future `classify()` edit silently dropping an endpoint.
- **Fix**: add a `routes_app` (or `server_app`) test asserting a scoped/deny token → 403 on
  `/v1/embeddings` and `/v1/responses`, mirroring the chat-completions forbidden test.

## E2E `mockClipboard` does not survive full-reload navigation
- **Source**: Grants-review remediation — discovered fixing E2E fallout from fail-closed defaults, 2026-07-01.
- **What**: `TokenFixtures.mockClipboard` (`tests-js/fixtures/tokenFixtures.mjs`) installs the
  `navigator.clipboard` / `window.clipboardData` mock via a one-shot `page.evaluate`, so any
  `page.goto` (full reload — e.g. `navigateToTokens`/`navigateToChat`) wipes it. `copyTokenFromDialog`
  now re-installs the mock defensively before reading, but the underlying fixture is still
  navigation-fragile for any other consumer.
- **Why deferred**: the localized re-install in `copyTokenFromDialog` fixed the observed failures;
  hardening the fixture is a broader cleanup.
- **Fix**: install the clipboard mock via `page.addInitScript` so it re-applies on every document load,
  then drop the defensive re-install in `copyTokenFromDialog`.

## `BasePage.waitForToastOptional` still has the dead if/else branch
- **Source**: Grants-review remediation — N11 (`tests-js/pages/BasePage.mjs`), 2026-07-01.
- **What**: N11 collapsed the identical `if (message instanceof RegExp) … else …` branches in
  `waitForToast` (both arms call `toContainText`, which already accepts a string or RegExp), but the
  same dead branch remains in the sibling `waitForToastOptional`.
- **Why deferred**: intentionally kept N11 scoped to the finding; trivial.
- **Fix**: collapse the branch in `waitForToastOptional` the same way.
</content>
