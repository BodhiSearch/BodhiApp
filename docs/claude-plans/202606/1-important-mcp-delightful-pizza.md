# Token-Grants Review — Remediation Plan

## Context

The API-Token + App-Token **grants** work (`4dea5ea9bb..HEAD`, 35 commits) was audited by a
6-analyst review (reports in `docs/claude-plans/202606/review/`). The architecture is sound
and well-unified (one `AccessPolicy` over a shared `ResourceGrants` seam), but the review +
user feedback surfaced **1 Critical, 16 Important, 13 Nice-to-have** issues spanning a latent
fail-open default, a hot-path inefficiency, test-fixture duplication, and several
coverage/altitude gaps. This plan sequences the fixes upstream→downstream so each layer's
gate passes before the next, and folds in the user's feedback overrides (recorded below).

This plan supersedes `docs/claude-plans/202606/review/fix-plan.md` where they differ — the
overrides here win. Finding IDs (C1, I1…I16, N1…N13) reference `review/index.md`.

---

## Decisions resolved (feedback + exploration)

1. **Fail-closed grant defaults everywhere (I1 + arch #6 override).** The migration's
   `DEFAULT_GRANTS` was all-access, which the user calls incorrect. The migration is
   **unreleased** (added 2026-06-29, after `v0.0.25`) and its column DEFAULT is never actually
   exercised (insert path always sets `grants` explicitly), but the **serde `#[serde(default)]`
   on `CreateTokenRequest.grants` and on `TokenGrantsV1.mcps`** *is* the real fail-open seam.
   → Make **all** default sources most-restrictive (deny). Blast radius: only **new tokens
   created without an explicit `grants` body** change (→ deny instead of all-access); no
   released/pre-existing tokens affected.

2. **I2 inefficiency — confirmed double full-body read on OpenAI/Anthropic inference**
   (middleware + handler `Json` extractor). Web search confirms axum/serde have **no clean
   lazy single-field extraction** (JSON must be scanned whole). Interceptor audit:
   `model_inference_grant_middleware` is the **only** body-reading interceptor (no
   interceptor-level duplication to merge); Gemini (model-from-path) and MCP proxy are already
   single-read. → Short-circuit `Unrestricted` before buffering + minimal-struct parse, and add
   the user-requested `// TODO: inefficient interceptor` comment for the residual handler
   re-parse. (Per user: "if not [lazy], add comment".)

3. **I6 black-box reframed (user override).** Calling APIs via **node-side** `fetch` /
   Playwright `request` fixture from test context is **acceptable** (that's how external apps
   use the API). Exploration confirms all three flagged specs are node-side, **not** browser
   context → **not violations.** Only keep the targeted improvement: `request-access` is a
   UI-possible flow, so `request-access-version-validation` should drive the **test OAuth app
   form** (`ConfigSection.configureOAuthForm` + `submitAccessRequest`) with a bad/missing
   version instead of a raw `request.post`.

4. **I5 (user override).** `/ui/chat` does not enforce API/app-token MCP/model grants, so test
   enforcement by calling the API **directly with the token** (node-side `fetchWithBearer` in
   `utils/api-model-helpers.mjs`), granted model → 200, non-granted → 403. Not via UI chat.

5. **UI picker defaults (user decision).** Both the New API Token form **and** the 3rd-party
   consent/approve screen default all grant pickers to **Specific/none** (least-privilege),
   matching the fail-closed backend (resolves U4/N13).

6. **token-refresh-integration.spec (N9, user override).** Known local-only limitation. Do
   **not** add a CI-tier test. Add a header comment documenting it is excluded from CI and is
   for manually debugging refresh issues locally. Keep `@scheduled`.

7. Everything else follows the review recommendations (`review/*.md`) as written.

The restrictive grants JSON (note: corrected to valid JSON — `[]`, not Rust `vec![]`):
```json
{"version":"1","models_list":false,"models":{"type":"specific","ids":[]},"mcps_list":false,"mcps":{"type":"specific","ids":[]}}
```

---

## Batch 1 — services (objs + domain + fixtures + fail-closed defaults)

**Fail-closed defaults (I1 + override)** — `crates/services/src/grants/grant_objs.rs`,
`crates/services/src/tokens/token_objs.rs`,
`crates/services/src/db/sea_migrations/m20250101_000025_token_grants.rs`:
- `impl Default for McpGrant` → `Specific { ids: vec![] }` (currently `All`); confirm
  `ModelGrant::default()` is already `Specific{[]}`.
- `TokenGrants::default()` / the `TokenGrantsV1` default → `models_list:false, models:Specific[],
  mcps_list:false, mcps:Specific[]` (was all-access). Update the doc-comment ("All-access
  (parity…)" → "fail-closed deny by default").
- `default_grants_json()` → the restrictive JSON string above.
- Migration `const DEFAULT_GRANTS` → the same restrictive JSON, edited **in place** (unreleased).
- Tests: flip `token_grants_defaults_missing_fields` to assert **deny** for `{"version":"1"}`
  and a payload omitting `mcps`; update the `default_grants_json()` pin test. Add an
  **enforcement** assertion that a parsed-but-partial grants payload omitting `mcps` denies MCP
  access (covers the I1 "Missing coverage" row).

**DRY versioned deserialize (I3)** — `tokens/token_objs.rs`,
`app_access_requests/access_request_objs.rs`: add one shared `deserialize_versioned`
helper/macro (single supported-version list + error message); route `TokenGrants`,
`RequestedResources`, `ApprovedResources` through it.

**Shared `with_api_token_grants` builder (I8)** — `crates/services/src/test_utils/auth_context.rs`:
add `with_api_token_grants(self, TokenGrants)` (mirror `with_external_app_grants`) +
`test_api_token_with_grants(user, scope, grants)`. (Consumers updated in Batch 2.) Switch
`shared/test_token_grants.rs::external_app` to `test_external_app(..).with_external_app_grants(..)`.

**Fixture/builder dedup**
- **I10** `app_access_requests/test_access_request_service.rs`: add
  `#[fixture] access_request_service(db)` (+ pre-stubbed-`MockAuthService` variant for approve);
  promote `make_request`/`approved_request` from the isolation test to a shared mod consumed by both.
- **I11** `tokens/test_token_service.rs` + `test_token_repository.rs` + `test_token_repository_isolation.rs`:
  one `token_entity(...)` builder + `#[fixture] token_service(db)`; consume in all three.

**Coverage**
- **I12** revoke→cache-eviction: test coupling `access_request_cache_needle(id)` to
  `serde_json::to_string(&CachedExchangeResult{ access_request_id: Some(id), .. })` (fail-open
  seam — revoke silently stops evicting if serialization drifts).
- **I13** `models/model_objs.rs::retain_listable_models`: unit-test the `Alias::Api`
  per-inner-model branch (Specific grant naming one model of a multi-model /
  `forward_all_with_prefix` alias keeps that model, drops the alias when empty).
- **I14** `app_access_requests/access_request_service.rs::approve_request`: test the
  Keycloak-409/UUID-collision branch (→ `Ok(Failed row)` + `update_failure`) and the generic
  error branch (→ `KcRegistrationFailed`) via `MockAuthService::expect_register_access_request_consent`.

**Nice-to-have**: **N1** delete dead `AuthContext::external_app_grants()` + `TokenGrants::version()`;
**N2** promote `model_listable`/`mcp_listable` to `ResourceGrants` default methods; **N4** replace
`use super::*` in `auth_context.rs` tests + collapse the `require_*` cluster into one `#[rstest]`;
**N3** (optional) drift-guard test asserting migration `DEFAULT_GRANTS` == `default_grants_json()`.

**Verify:** `cargo test -p objs -p services`

---

## Batch 2 — routes_app (enforcement layer)

**C1 🔒 MCP connect deny test** — `crates/routes_app/src/mcps/test_mcps.rs`: oneshot mounting
`mcp_proxy_handler` with `AuthContext::ApiToken` whose `mcps: Specific{["mcp-uuid-1"]}` →
`POST /apps/mcps/mcp-uuid-2/mcp` = 403 (`token_grant_error-mcp_forbidden`), granted id passes.
Parameterize both principals (`api_token` + `external_app`) in one `#[rstest]`, mirroring
`body_inference_paths_enforce_model_grant`. (Closes the only Critical: the sole MCP-invoke
authz gate currently has Rust coverage for listing only; deny is E2E-only because live proxy
tests run as `resource_admin` → `Unrestricted`.)

**I2 body-buffer ordering** — `crates/routes_app/src/middleware/model_grant.rs::model_inference_grant_middleware`:
- Resolve `AuthContext` + `AccessPolicy::of(&ctx)` **before** `to_bytes`. If `Unrestricted`
  (or `classify` says no enforcement) `return next.run(req)` **without buffering**. Only buffer
  for `Grants`/`Deny`.
- Replace the full `serde_json::Value` parse with a minimal `#[derive(Deserialize)] struct
  ModelField { model: Option<String> }` (cheaper than `Value`).
- Add the comment: `// TODO: inefficient interceptor — buffers complete body to read "model";
  handler re-parses it. Lazy single-field extraction not feasible with axum/serde (body must be
  scanned whole). Gemini/MCP paths avoid this (model from path / no grant mw).`
- Tests: add the `model: None` pass-through case and the oversized-body (`Err(to_bytes)`) arm
  (currently forwards an empty body) — both branchy, both untested.

**I9 rstest merge** — `crates/routes_app/src/oai/test_chat_completions.rs`: collapse
`test_chat_completions_scoped_token_forbidden` + `…external_app_model_forbidden` into one
`#[rstest]` with `#[case::api_token]`/`#[case::external_app]` (build router once). Uses the
Batch-1 `with_api_token_grants` builder; replace the other ~6 inline `AuthContext::ApiToken{…
grants:V1}` literals (I8 consumers) across `test_model_grant.rs`, `test_token_grants.rs`,
`oai/test_models.rs`, `mcps/test_mcps.rs`, `users/test_user_info.rs`.

**Nice-to-have**: **N5** per-format error-envelope body assertions (OpenAI vs Anthropic vs
Gemini) in `test_model_grant.rs`; **N6** scoped-token 404 existence-hiding test for `mcps_show`.

**Verify:** `cargo test -p objs -p services -p routes_app`

---

## Batch 3 — server_app (live HTTP)

**I4 🔒 grant-aware `ExternalTokenSimulator`** — `crates/server_app/tests/utils/external_token.rs`:
add `create_token_with_grants(role, azp, grants: Option<ApprovedResources>)` (or
`with_grants(self, …)`) seeding `CachedExchangeResult.grants`; keep `grants:None` as the default
for existing fail-closed assertions. Add a live test pair: approved app with `models_access:
Specific(["m"])` / `mcps_access: Specific(["mcp-1"])` → granted resource 200, ungranted 403 over
the real proxy. (Closes the gap where the entire approved-app enforcement path had no
live-HTTP coverage — the existing 200 test passes against an empty list.)

**Verify:** `cargo test -p server_app`, then full backend (capture output per the long-command
convention).

---

## Batch 4 — ts-client (only if any schema changed)

The fail-closed default changes the **value** of grants, not the schema, so regeneration is
likely a no-op. Regenerate only if a utoipa annotation / struct shape changed in Batches 1–3.

**Verify:** `cargo run --package xtask openapi && cd ts-client && npm run generate && make ci.ts-client-check`

---

## Batch 5 — UI

**I16 🔒 extract + test `toApproveBody`** — `crates/bodhi/src/routes/apps/access-requests/review/index.tsx`:
pull the consent grant mapping out of `handleApprove` into a pure exported
`toApproveBody(req, state)` (symmetric with `toCreateTokenRequest`). Unit-test the branch matrix:
`models_access` requested+all / requested+specific / not-requested→(per decision) **none**;
`mcps_access` requested+all / requested+specific(empty & non-empty) / not-requested→none; list
flags gated by `req.models_list`/`req.mcps_list`.

**UI picker defaults (decision #5)** — default both pickers to least-privilege:
- Consent screen `review/index.tsx`: `modelMode` initial `'specific'` (was `'all'`); MCP-extra
  already `'specific'`.
- New API Token form `crates/bodhi/src/routes/tokens/-components/TokenForm.tsx`: model picker
  default → `'specific'` (was `'all'`); verify `toCreateTokenRequest` maps the empty-specific
  state correctly and update its unit tests.

**I15 extract shared rail primitives** — `crates/bodhi/src/routes/tokens/index.tsx` +
`crates/bodhi/src/routes/tokens/apps/index.tsx`: factor a `useUrlMirroredSelection(routeTo)`
hook (owns `readSelectFromUrl` + `popstate` + replace-navigate), a shared `DetailRow` +
`fmtDate` rail kit, and one `grantSummary(access, noun)` consumed by both; use `GrantChips` in
both detail panels.

**Nice-to-have**: **N12** add an `AccessPicker.test.tsx` case for the Local/API type-filter
narrowing + group headers.

**Verify:** `cd crates/bodhi && npm test` (component unit tests).

---

## Batch 6 — E2E

**I5 🔒 API-token model-grant inference (override)** — `specs/tokens/api-tokens.spec.mjs`
scoped-grants step: capture the scoped `bodhiapp_` token and use node-side
`fetchWithBearer(serverUrl, token, '/v1/chat/completions', body)` (from
`utils/api-model-helpers.mjs`): granted model → 200, non-granted → 403. Keep it one journey via
`test.step`. Do **not** route through `/ui/chat` (it doesn't enforce token grants).

**I6 request-access via the form (override, scoped)** — `specs/request-access/request-access-version-validation.spec.mjs`:
replace the raw `request.post('/bodhi/v1/apps/request-access')` with the test OAuth app UI flow
— `ConfigSection.configureOAuthForm({ …, requested: JSON.stringify({ version: '99' }) })` then
`submitAccessRequest()`, asserting the rejection surfaces. Leave the node-side calls in
`oauth2-token-exchange.spec.mjs` and `multi-tenant-lifecycle.spec.mjs` **as-is** (acceptable
node-side API usage, not black-box violations).

**I7 kill timeouts** — `crates/lib_bodhiserver/tests-js/pages/TokensPage.mjs`: replace both
`waitForTimeout(100)` with UI-state waits — `copyTokenFromDialog` → `expect.poll` on the mocked
clipboard until `/^bodhiapp_/`; `toggleTokenStatus` → drop the sleep, rely on the existing
`expectTokenStatus` poll. (Prefer asserting an updated UI element over a blank wait — faster,
non-flaky.)

**N9 token-refresh doc (override)** — `specs/auth/token-refresh-integration.spec.mjs`: add a
header comment stating the `@scheduled` expiry test is **excluded from CI** and exists to debug
session-refresh issues locally by running it manually; document the `/dev/secrets` + long-wait
limitation. No new CI test.

**Other nice-to-have**: **N7** restore a 401/403 status assertion on the empty-Specific MCP deny
in `mcps-sdk-compat-everything.spec.mjs`; **N8** rename the stale "gets 401 → 404" test + header
in `mcps-auth-restrictions.spec.mjs`; **N10** delete dead `approveWithResources` in
`AccessRequestReviewPage.mjs`; **N11** collapse the dead `waitForToast` if/else branch in
`BasePage.mjs`.

**Verify:** `make build.dev-server` then a filtered run, e.g.
`cd crates/lib_bodhiserver && npm run test:playwright -- specs/tokens specs/mcps specs/request-access`.

---

## Verification (final gate — run before committing the series)

1. `make test.backend` — Rust/backend tests (Batches 1–3).
2. `make test.ui.unit` — UI component unit tests (Batch 5).
3. `make test.e2e` — full Playwright E2E (Batch 6).
4. `make format` before each commit; rebase onto `origin/main` (trunk-based, linear history).

Per-batch fast inner-loop checks are listed under each batch; the three `make` gates above are
the authoritative final verification.

---

## Suggested commit grouping (trunk-based, focused commits)

| Commit | Contents |
|--------|----------|
| `fix(grants): fail-closed grant defaults (migration + serde) + tests (I1)` | Batch 1 defaults |
| `refactor(grants): shared versioned-envelope deserialize (I3)` | Batch 1 I3 |
| `test(grants): shared auth-context/entity/service fixtures + dedup (I8,I10,I11,N4)` | Batch 1 fixtures |
| `test(grants): revoke-eviction, listable, approve-branch coverage (I12,I13,I14)` | Batch 1 coverage |
| `refactor(grants): drop dead accessors + trait-default listable (N1,N2)` | Batch 1 cleanup |
| `test(routes): MCP connect deny + body-order short-circuit + rstest merge (C1,I2,I9,N5,N6)` | Batch 2 |
| `test(server_app): grant-aware external-token simulator + live enforcement (I4)` | Batch 3 |
| `refactor(ui): least-privilege picker defaults + toApproveBody + shared rail kit (I15,I16,U4,N12)` | Batch 5 |
| `test(e2e): api-token model enforcement, form-driven request-access, kill timeouts (I5,I6,I7,N7-N11)` | Batch 6 |

---

## Open items deferred (not in this plan)

- **arch embeddings/responses 403 parity** (nice-to-have): an integration test that a
  non-granted token gets 403 specifically on `/v1/embeddings` + `/v1/responses` — fold into
  Batch 2 if cheap, else defer.
- A true single-parse of the inference body (middleware→handler via request extensions) is
  **not** in scope — the audit shows no interceptor-level duplication, only the inherent
  handler re-parse, which the TODO documents. Revisit only if profiling shows it matters.
