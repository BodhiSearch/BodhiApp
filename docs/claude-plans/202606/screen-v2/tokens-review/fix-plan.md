# App Token Grants — Review-Fix Implementation Plan (phased, batched)

> Plan file (auto-named); the working home for these reports is
> `docs/claude-plans/202606/screen-v2/tokens-review/`. Rename/move this file there when convenient.

## Context
The "tokens screen-v2 + App Token grants" effort (`4dea5ea9..HEAD`) was re-reviewed with an ultracode
multi-agent pass that surfaced **42 verified findings** (1 critical, 16 important, 25 nice-to-have) —
see `screen-v2/tokens-review/index.md` and the per-layer reports. The user recorded final
dispositions in `screen-v2/tokens-review/decision.md`. This plan implements **every non-rejected
finding**, organised the way the user asked: **phase-wise, in batches of related fixes**, each batch a
vertical slice (backend upstream→downstream + Rust tests + frontend + component tests + a focused
E2E spec), **gated and committed before moving to the next batch**.

Why now: the review found a **confirmed CRITICAL grant bypass** (`/v1/embeddings` skips
`ensure_model_inference`), a second confirmed enforcement gap (Anthropic/Gemini model-*listing*
handlers skip the listable filter), permissive-by-default models, and an inconsistent grant-field
naming/verb/path surface. Outcome: App Token grants become **deny-by-default and uniformly
enforced**, the API contract is internally consistent, and every load-bearing path is tested.

## Decisions in force (from `decision.md`, authoritative over the original finding text)
- **F7**: `ModelGrant::default()` → `Specific{ids:vec![]}` (empty-deny, symmetric with MCP); request
  flag `RequestedResourcesV1.models_access: bool` defaults **true** (consent shows the model selector
  unless the app sends `false`).
- **F6**: corrected from "no-op" — **fix it**: unbound `ExternalApp{grants:None}` resolves to
  **default-deny** (`ApprovedResourcesV1::default()`), not `Unrestricted`; reflect deny in
  `/bodhi/v1/user` and the UI. *(Action: update the F6 row in `decision.md` from "No-op" to "Fix —
  fail-closed" during Batch 2.)*
- **F45 + TokenGrantsV1**: rename `list_models→models_list`, `list_mcps→mcps_list` (keep `models`,
  `mcps`).
- **F9**: approve endpoint `PUT`→`POST`. **F32 (in-scope)**: list endpoint
  `/bodhi/v1/access-requests/apps` → `/bodhi/v1/access-requests/` (drop `/apps`).
- **Accepted as-is / no code**: F13 (bool→grant same name is deliberate), F14, F29 (won't fix).
- **Deferred to `docs/claude-plans/techdebt.md`**: F32 module relocation (already logged).

## Verified during planning
- **F1** real: `embeddings_handler` (`oai/routes_oai_chat.rs:250-301`) has no policy check; chat
  (`:134`), anthropic (`:148`), gemini (`:213`) do.
- **F5** real: `anthropic_models_list_handler`/`_get_handler` (`:184`/`:229`) and `gemini_models_list`/
  `_get` (`:123`/`:149`) never call `model_listable`; OAI equivalents do (`oai_models_handler:90`,
  `oai_model_handler:140-141`) — mirror that pattern.
- **F27 already covered** by `test_delete_token_handler` (`tokens/test_tokens_crud.rs:752-762`,
  asserts second delete → 404). **Dropped** (optionally add a thin service-unit test, low priority).
- **F2** real despite the "deliberate" doc-comment: the `forward_all_with_prefix` branch judges by
  `prefix` alone while the picker (`grantItems.ts`) and OAI listing key on `prefix+model.id`, so a
  `Specific` grant wrongly prunes the whole alias from `/bodhi/v1/models`. Fix the branch **and**
  the misleading comment (`model_objs.rs:1002-1016`).

## Critical ripple — `ModelGrant::default()` (handle in Batch 2)
Flipping `ModelGrant::default()` to empty-deny changes runtime behavior of every `#[serde(default)]`
`ModelGrant` field and `#[derive(Default)]` struct containing one — but **not** any field name (no
wire change, so no regen from this alone). `TokenGrants::default()` / `TokenGrantsV1::default()`
(manual all-access at `token_objs.rs:108-118`) is **separate** and must stay all-access for the
API-token-create parity path. F26 (fail-closed *display*: `TokenDetail::from:192`'s
`unwrap_or_default()` and `AppAccessSummary::from_row`) is therefore a **distinct** fix. Before
re-baselining tests: grep every `ModelGrant::default()` call-site + `#[serde(default)]` grant field +
`#[derive(Default)]` over a `ModelGrant`, and confirm the create path sets explicit grants.

## Regen & ordering (read first)
1. Wire-contract changes (F45 rename, F9 verb, F32 path, F33 operationId, F10 shape) land **first**
   so frontend builds on correct generated types. After any backend change to a utoipa annotation or
   serialized type: `cargo run --package xtask openapi` → `make build.ts-client` (regenerates
   `ts-client` types + `openapi-schema.ts` used by MSW). Hand-maintained MSW handlers
   (`crates/bodhi/src/test-utils/msw-v2/handlers/*`) and fixtures (`test-fixtures/*`) are updated by
   hand in the same batch.
2. The F7 default ripple is JSON-shape-neutral → re-baseline `cargo test`, regen only if F7/F10 alter
   a serialized shape.
3. Commit the regenerated `openapi.json` + ts-client artifacts as part of the batch that caused them.

---

## Batch 1 — API contract: renames, verb, path, operationId
**Findings:** F45 (+TokenGrantsV1 rename), F9, F32 (in-scope), F33, F27 (verify-only).
**Backend:** `services/src/tokens/token_objs.rs:34-43` field rename; `routes_app/src/apps/routes_apps.rs`
(approve utoipa `put`→`post`, list path drop `/apps`, operationId de-pluralize `Apps`); `routes.rs`
(~349-364) router method + path.
**Rust tests:** update `routes_app/src/apps/test_*` + `tokens/test_tokens_crud.rs` for new
names/verb/path (same-file CRUD tests as template); F27 verify present, add only if missing.
**Frontend + component tests:** `TokenForm.tsx` `toCreateTokenRequest` + `hooks/tokens/useTokens`
(new field names); `hooks/apps/useApproveAppAccessRequest` PUT→POST; `useListAppAccess` path; MSW
`handlers/{tokens,apps}.ts` (verb/path/keys); `test-fixtures/{tokens,apps}.ts`; adjust the three
hook unit tests.
**E2E:** `specs/tokens/app-tokens-grants.spec.mjs` + `pages/{AppTokensPage,AccessRequestReviewPage}.mjs`
approve-verb/list-path usages.
**Gate:** `cargo test -p services -p routes_app` → `cargo run --package xtask openapi` →
`make build.ts-client` → `cd crates/bodhi && npm test` → filtered E2E (token specs). **Commit.**

## Batch 2 — Fail-closed defaults, reflection depth & display
**Findings:** F7, F6, F10, F26, X3, X1, F28, F19. *(Also flip the `decision.md` F6 row here.)*
**Backend (upstream→downstream):** `grants/grant_objs.rs` `ModelGrant::default()`→empty-deny (+ run
the ripple audit above); `app_access_requests/access_request_objs.rs` `RequestedResourcesV1.models_access`
default `false`→`true`; `shared/token_grants.rs:27-36` `ExternalApp{grants:None}`→
`Grants(&ApprovedResourcesV1::default())`; reflection `users/routes_users_info.rs` +
`users/users_api_schemas.rs` (F6 deny + F10 consistent depth across ApiToken/ExternalApp); display
`token_objs.rs:192` `TokenDetail::from` (stop masking deny) + `apps_api_schemas.rs`
`AppAccessSummary::from_row` (F26 fail-closed + X3 cap `approved_role` at the user's resource role).
**Rust tests:** `shared/test_token_grants.rs` (grants:None→deny; template = existing
ensure_model_inference cases); `middleware/access_requests/test_access_request_middleware.rs` (X1
ApiToken-bypass arm via `test_api_token`; F28 assert-order fix); services default tests; users-info
reflection-shape test (F10). Factories: `services/src/test_utils/auth_context.rs`.
**Frontend + component tests:** verify consent selector now shows by default (`review/index.tsx`,
`access-picker/*`); tokens/apps display shows deny not all-access; F19 consent-approve test asserts
`mcps_list` in payload.
**E2E:** `app-tokens-grants.spec.mjs` deny-by-default consent + reflected grants after approve.
**Gate:** `cargo test -p services -p routes_app` (full re-baseline) → regen only if a serialized shape
changed → `cd crates/bodhi && npm test` → filtered E2E. **Commit.**

## Batch 3 — Enforcement parity & data-layer coverage (backend-only)
**Findings:** F1, F5, F2, F12, F4, X2, F3.
**Backend:** `oai/routes_oai_chat.rs:250` add `ensure_model_inference(&model)?` to
`embeddings_handler` (mirror `:134`); `anthropic/routes_anthropic.rs:184/229` +
`gemini/routes_gemini.rs:123/149` add `model_listable` filter on list (mirror `oai_models_handler:90`)
and 404 on get (mirror `oai_model_handler:140-141`); `models/model_objs.rs:1002-1016` fix the
`forward_all_with_prefix` branch to test `prefix+model.id` per model + correct the doc-comment.
**Rust tests (copy templates):** enforcement from `oai/test_chat_completions.rs`
(`_scoped_token_forbidden`/`_external_app_model_forbidden`) → new `test_embeddings.rs`,
`anthropic/test_anthropic_messages.rs` + `test_anthropic_models.rs`, `gemini/test_gemini_routes.rs`,
`mcps/test_mcps.rs`; listing from `oai/test_models.rs`
(`test_oai_model_handler_scoped_token_hides_ungranted`) → anthropic/gemini; F4 retain-branch tests
(`model_objs`); X2 `app_mcps` union test; F3 `update_revocation`/`list_approved_for_user` repo +
cross-tenant isolation (template: `tokens/test_token_repository_isolation.rs` +
`app_access_requests/test_access_request_repository_isolation.rs`).
**Frontend:** none (intentionally backend-only).
**E2E:** `app-tokens-grants.spec.mjs` positive infer + connect after grant; ungranted model absent
from list.
**Gate:** `cargo test -p services -p routes_app` → `make test.backend` (first full backend gate) →
filtered E2E. **Commit.**

## Batch 4 — Frontend correctness & shared utilities (frontend-only)
**Findings:** F16, F17, F36, F34, F35, F15, F8; component tests F18, F37, F38, F39, F40.
**Frontend:** `lib/grantItems.ts` distinguish `ModelRouterResponse` via `targets`/`strategy` (F16);
`routes/tokens/new/index.tsx:40` CardTitle → "New API Token" (F17); App-tokens `AppDetailPanel` chip
from `app.status` (F36); `access-picker/AccessPickerPanel` derive group labels vs hardcoded
"Local/API Models" (F34); `review/ReviewContent` fetch models/mcps conditionally (F35);
`hooks/apps/*` use shared `extractErrorMessage` from `lib/errorUtils.ts` (F15); rename the
`AccessRequestNotDraft` revoke-guard misnomer (F8, services + any FE error mapping).
**Component tests:** F18 grantItems (local/api/router), F37 ListingToggle keyboard, F38
AccessPickerPanel filter, F39 GrantBlock, F40 TokenForm PowerUser card disabled.
**Gate:** `cd crates/bodhi && npm test`. **Commit.**

## Batch 5 — E2E hardening & page objects (test-only) — ✅ DONE (2026-06-30)
**Findings:** F20 (finalize positive infer/connect + list reflection), F21 (owner-extra MCP in
`approveWithGrants`), F41 (List-all case), F42 (keyboard nav → page-object method), F43 (env
validation in `beforeAll`).
**E2E:** extend `specs/tokens/app-tokens-grants.spec.mjs`; add `approveWithGrants` extra-MCP path +
keyboard-nav method to `pages/{AccessRequestReviewPage,AppTokensPage,TokensPage}.mjs`; `beforeAll`
throws on missing `INTEG_TEST_*` env (no `test.skip`). Black-box only (UI interactions);
`reducedMotion:'reduce'` at context level.
**Gate:** `make build.dev-server` then full `make test.e2e`. **Commit.**

### Batch 5 — outcome
First fixed the four red specs (all asserting pre-change behavior, no code regressions):
- `mcps-auth-restrictions` Phase 5: restricted MCP get is now **404** (`entity_error-not_found`,
  F12/F31 hide-not-reveal), not 403 + `entity_not_approved` (that error path was deleted).
- `mcps-crud`: playground header renders the tool's friendly **title** ("Echo Tool").
- `mcps-header-auth`: `getPlaygroundResultContent` → `getPlaygroundResultRaw` (Playground-V2 rename).
- `mcps-sdk-compat-everything`: the obsolete "API token rejected (OAuth-only route)" step was
  **rewritten** (per operator) into API-token MCP grant coverage (All connects / empty-Specific
  denied / Specific-this connects) — API tokens are now first-class + grant-enforced on `/apps/mcps/*`.

Coverage added: **F20** (app-token specific model + owner-extra MCP → infer 200/non-granted 403,
granted MCP 200/restricted 404/hidden-in-list, `/bodhi/v1/user` reflection), **F21**
(`grantSpecificModels`/`grantSpecificMcps` — consent MCP picker defaults to Specific→open via Add;
model picker defaults to All→Specific auto-opens; select by explicit id), **F41** (list-all + All
case in api-tokens), **F42** (`TokensPage.selectRowByKeyboard`/`activeRowTestId` via `aria-selected`,
not the `.l-listrow.active` CSS class). **F43** already present.

Determinism work (no if-else / try-catch / white-box on UI state): `TokenForm` and the review screen
expose `data-test-state="ready"` once their grant lists settle; page objects wait for it before
interacting (clicking a picker mid-load dropped the event). Verified green on **standalone** (all 26
tokens+mcps specs) and **multi_tenant** (app-tokens-grants ×2, api-tokens lifecycle).
Commits: `e6078a9f` (phase-1 spec fixes), `3e99601d` (rust `grants:None` test-compile fix),
`691aab15` (F-2a), `9d5398b3` (F20/F21), `fb1dec25` (F41/F42).

---

## Coverage check (nothing dropped)
- **Security/enforcement**: F1,F5,F6,F7 → B2/B3.
- **API consistency**: F45,F9,F32,F33,F10,X3 → B1/B2.
- **Correctness**: F2,F16,F17,F26,F36,F34,F35,F15,F8 → B2/B3/B4.
- **Test coverage**: F12,X1,F3,F4,X2,F18,F19,F37,F38,F39,F40,F20,F21,F41,F42,F43,F28 → B1–B5.
- **Excluded** (decisions): F13, F14, F29, F27 (already covered), F32 module move (techdebt).

## Verification (end-to-end)
1. **Backend**: `make test.backend` green after Batch 3; OpenAPI + ts-client regenerated and committed
   (no stale diff: `cargo run --package xtask openapi && make build.ts-client` → clean `git status`).
2. **Frontend**: `cd crates/bodhi && npm test` after Batch 4.
3. **Chrome smoke** (`make app.run.live`, rebuilt dev-server per the GATE-B note): create an API token
   with Specific models + MCPs (new field names); drive an external-app request → consent shows the
   model selector by default, grant a specific model + MCP → granted model infers on
   chat/embeddings/anthropic/gemini, non-granted returns 403; `/v1/models`, `/anthropic/v1/models`,
   `/v1beta/models`, `/bodhi/v1/models` all hide non-granted; revoke → token stops; an unbound
   external app (no approved request) is denied (F6).
4. **E2E**: `make test.e2e` green incl. the new positive-enforcement, owner-extra-MCP, and
   list-reflection steps.
5. **Per-batch gates** before each commit (`make format`, scoped tests, regen when types change,
   filtered E2E when touched).

## Critical files
- Backend: `services/src/grants/grant_objs.rs`, `services/src/tokens/token_objs.rs`,
  `services/src/app_access_requests/access_request_objs.rs`, `services/src/models/model_objs.rs`,
  `routes_app/src/shared/token_grants.rs`, `routes_app/src/oai/routes_oai_chat.rs`,
  `routes_app/src/anthropic/routes_anthropic.rs`, `routes_app/src/gemini/routes_gemini.rs`,
  `routes_app/src/apps/routes_apps.rs`, `routes_app/src/apps/apps_api_schemas.rs`,
  `routes_app/src/users/{routes_users_info,users_api_schemas}.rs`, `routes_app/src/routes.rs`,
  `routes_app/src/middleware/access_requests/access_request_middleware.rs`.
- Frontend: `bodhi/src/lib/grantItems.ts`, `routes/tokens/-components/TokenForm.tsx`,
  `routes/tokens/new/index.tsx`, `routes/apps/access-requests/review/index.tsx`,
  `routes/tokens/apps/index.tsx`, `hooks/{tokens,apps}/*`, `components/access-picker/*`,
  `test-utils/msw-v2/handlers/{tokens,apps}.ts`, `test-fixtures/{tokens,apps}.ts`.
- E2E: `tests-js/specs/tokens/{api-tokens,app-tokens-grants}.spec.mjs`,
  `tests-js/pages/{TokensPage,AppTokensPage,AccessRequestReviewPage}.mjs`.
</content>
