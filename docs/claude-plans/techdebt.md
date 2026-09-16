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

# Remote Access

Gaps carried out of the Cloudflare named-tunnel work
([`202609/remote-access-plan.md`](202609/remote-access-plan.md)). Decisions referenced as D1–D10 and
risks as R1–R6 are defined in that plan.

## `/bodhi/v1/info` discovery: three accepted gaps
- **Source**: [`202609/remote-access-info-discovery.md`](202609/remote-access-info-discovery.md), shipped
  in `38058130`. All three were named in the plan and deliberately left out of scope.
- **RunPod reports itself as not public.** `on_runpod_enabled()` already forces `https` and a public
  `*.proxy.runpod.net` host, so those instances genuinely are reachable — but `url_public` is
  env-declared only (owner decision: absence means false, nothing infers it), so they answer `false`.
  Deriving it there would contradict that decision, so it needs an explicit call rather than a patch.
- **"Unavailable" and "available but unconfigured" are indistinguishable.** `remote_access` is omitted
  in both cases, so a third-party onboarding flow cannot tell "this build cannot do Remote Access"
  from "it can — tell the user to switch it on". The endpoint's primary question ("can I reach it
  now?") is unaffected, which is why this was accepted.
- **The anonymous build fingerprint is now internet-facing.** `/bodhi/v1/info` has always published
  `version` and `commit_sha` without auth. That was low-risk on a LAN-only instance; with a live
  tunnel it is an exact build identifier on the open internet, which helps someone match a known CVE
  to a specific build. Pre-existing, but its risk profile changed with this feature — worth a
  deliberate decision on whether to coarsen or gate it.

## No end-to-end coverage: the Playwright journey was skipped
- **Source**: Remote Access plan — Phase 6 exit gate, scoped out by the owner on 2026-09-18.
- **What**: Phase 6 called for one Playwright journey of many `test.step()`s against the fake
  connector. It was **not written**, and Phase 6 closed on docs and cleanup only. Concretely there is
  no tunnels page object in `crates/lib_bodhiserver/tests-js/pages/`, no tunnel spec, and
  `make test.e2e` exercises nothing of Remote Access.
- **Why skipped — the blocker**: the fake `cloudflared`
  (`services/src/test_utils/tunnels.rs`) can replace the binary through
  `BODHI_TUNNEL_CLOUDFLARED_PATH`, but the Cloudflare API base URL is only injectable via
  `DefaultTunnelService::with_cloudflare_api`, which is `#[cfg(any(test, feature = "test-utils"))]`.
  A real server process therefore still calls `api.cloudflare.com` for the zone and DNS lookups, so
  the states that motivated the plan — creating, dns-conflict, live — are unreachable in E2E without
  a real Cloudflare account. The owner chose the existing 24 `services` tests and 45 component tests
  over adding a production-visible setting purely for testing.
- **What is consequently uncovered end to end**: enable → live → disable → re-enable through the real
  HTTP stack; the DNS-conflict confirm path; the auth-sync states as served by the real backend; and
  any regression in the `/bodhi/v1/tunnel*` routes' wiring that unit tests mock past.
- **Fix**: add a Cloudflare API base URL setting gated to non-production builds, then write the
  journey per `docs/conventions/testing.md` (one `test()`, many `test.step()`s, shared server).

## Chrome verification is not driven by the fake connector
- **Source**: Remote Access plan — Phase 4 and Phase 5 automated gates, 2026-09-18.
- **What**: Both gates ask for Chrome at desktop and 430px *driven by the fake `cloudflared`*, so
  every state is reachable from a real backend. In practice the browser check covered the `off` and
  setup states against a live backend; the rest (creating, dns-conflict, kc-fail-net, kc-fail-auth,
  live) were verified by component tests only. A layout regression in those states would not be
  caught.
- **Why deferred**: blocked on the same API-base injection as the item above.
- **Fix**: same fix; then walk the states in Chrome at both widths.

## Remote Access is undefined under a clustered deployment (D10)
- **Source**: Remote Access plan — decision D10, 2026-09-18.
- **What**: The feature assumes one instance. The tunnel name derives from the instance's OAuth client
  id, so every replica of a clustered deployment resolves the *same* name, each starts its own
  connector against it, and Cloudflare load-balances one public hostname across unrelated instances.
  Sessions would land on arbitrary replicas.
- **Why deferred**: Remote Access is a native, single-instance feature; fixing it means electing one
  connector per deployment, which is a different feature.
- **Fix**: none planned. `BODHI_TUNNEL` must stay unset (its default) for non-native deployments. If
  this is ever wanted in a cluster, tunnel identity needs a deployment-level owner, not an
  instance-level one.

## Turning remote access off leaves the DNS record behind
- **Source**: Remote Access — owner decision, 2026-09-18.
- **What**: `disable()` stops the connector but leaves the CNAME pointing at the tunnel, so while
  remote access is off a visitor to the address gets Cloudflare's 1033 "tunnel not found" page rather
  than a clean NXDOMAIN. Deliberate: re-enabling is then instant and the address keeps its identity.
  Separate from the Keycloak registration, which D9 says is never cleared on disable because the PATCH
  is gateway-keyed and a later change overwrites it.
- **Why deferred**: chosen behaviour, not an oversight. Recorded so the 1033 page is not later
  mistaken for a bug.
- **Fix**: none planned; the FAQ explains it. If it becomes a support burden, delete the record on
  disable and accept the propagation delay on re-enable.

## `BODHI_TUNNEL_ORIGIN_CERT` vs the bare `TUNNEL_ORIGIN_CERT` env var
- **Source**: Remote Access planning — unresolved, 2026-09-18.
- **What**: `TUNNEL_ORIGIN_CERT` is what we *write* into the connector's environment;
  `BODHI_TUNNEL_ORIGIN_CERT` is the setting we *read*. Someone already exporting the bare
  `TUNNEL_ORIGIN_CERT` for their own `cloudflared` use will find the app ignores it and reports the
  certificate missing, which reads as a bug.
- **Why deferred**: never decided whether honouring a non-namespaced variable is desirable, since it
  breaks the `BODHI_*` convention every other setting follows.
- **Fix**: either honour `TUNNEL_ORIGIN_CERT` as a lowest-precedence read path, or say so explicitly
  in the certificate FAQ entry. Doing neither is the current state.

## `scrub_secrets` is an entropy heuristic, not a parser
- **Source**: Remote Access Phase 3 (R6), 2026-09-18.
- **What**: `DefaultTunnelService::scrub_secrets` redacts runs of ≥40 characters that mix case and
  digits (or any run ≥80). It is deliberately biased toward readability: lowercase runs survive so
  tunnel names, hostnames and paths stay diagnosable. A short secret, or one that happens to be all
  lowercase, would pass through; a long mixed-case identifier that is *not* secret would be redacted.
- **Why deferred**: cloudflared's output has no schema to parse, and the tokens actually at risk
  (connector token, Cloudflare API token) are long and base64-ish, which the heuristic catches. Tested
  against the real token shape in `scrubs_opaque_secrets_while_keeping_the_text_diagnosable`.
- **Fix**: if cloudflared gains structured log output, key off field names instead.

## Auth-sync configuration errors are reported as `rejected`
- **Source**: Remote Access Phase 3 (D9), 2026-09-18.
- **What**: `TunnelAuthSyncState` has two failure states, matching the two messages the design
  specifies. "No standalone authorization client is configured" and "Authorization synchronization is
  unavailable" are neither — they are internal configuration faults — but map to `rejected` because,
  like a refusal, retrying the network cannot help. The user is then told Keycloak refused, which is
  not literally what happened.
- **Why deferred**: both indicate a broken install rather than a state a normal user reaches, and a
  third failure state would add a UI branch the design does not have.
- **Fix**: if these prove reachable in practice, add a distinct state and copy.

## `Retry sync` is offered even when Keycloak refused
- **Source**: Remote Access Phase 3 / design `ra-app.jsx`, 2026-09-18.
- **What**: The design shows `Retry sync` on both sign-in failure states. On `kc-fail-net` retrying is
  the remedy; on `kc-fail-auth` the client's permissions must change first, so retrying unchanged
  fails again identically.
- **Why deferred**: followed the design rather than diverging mid-implementation.
- **Fix**: on `kc-fail-auth`, hide the button or relabel it so it does not read as the fix.

## Orphaned tunnels can only be cleaned up with the CLI
- **Source**: Remote Access — one-time cleanup performed 2026-09-18.
- **What**: The pre-plan code minted a new tunnel per subdomain, leaving four orphans on the owner's
  account; they were removed by hand with `cloudflared tunnel delete`. Nothing in the app lists or
  removes tunnels it no longer uses. D1 (client-id-derived name) prevents *new* orphans, so this is
  cleanup for existing accounts only.
- **Why deferred**: a one-off for one known account; a management UI is disproportionate.
- **Fix**: if it recurs, surface `cloudflared tunnel list` filtered to `bodhi-app-tunnel-*` with a
  delete affordance for names that do not match this instance.
</content>
