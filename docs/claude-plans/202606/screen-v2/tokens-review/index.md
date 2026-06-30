# Code Review — Tokens Screen-V2 + App Token Grants (Consolidated, Implementation-Ready)

> This is the **single source of truth** for the review. It supersedes the first pass: it is the
> output of an **ultracode multi-agent re-review** (12 blind multi-lens finders → triage/dedup
> against the prior reports → adversarial verify, refute-by-default → completeness critic), run on
> Sonnet. Every finding below **survived adversarial verification**. 6 candidate findings were
> refuted and are listed at the bottom so they are not re-raised. Per-finding evidence lives in the
> per-layer reports.

## Scope
- **Ref**: `4dea5ea9..HEAD` (25 commits) · **Date**: 2026-06-30
- **Plans**: `the-api-token-screen-mellow-porcupine.md` (API-token redesign + `McpGrant::None` removal) and `docs-claude-plans-202606-screen-v2-prom-lexical-wren.md` (App Token per-resource grants)
- **Changed**: 107 files / ~7,600 lines across services, routes_app, bodhi (UI), tests-js (E2E), ts-client + openapi (generated)
- **Focus (requested)**: security · API consistency (request/response field names) · architecture · test coverage
- **Pipeline stats**: 57 raw findings → 45 canonical (deduped, prior reports folded in) → **39 survived + 3 from the critic = 42 verified** · 6 refuted

## Summary
- **Total: 42** — **Critical: 1** · Important: 16 · Nice-to-have: 25
- by layer: routes_app 12 · ui 12 · services 7 · e2e 7 · cross-cutting 4
- **The re-review caught real defects the first pass missed** — most importantly a **confirmed CRITICAL grant bypass on `/v1/embeddings` (F1)** and a second confirmed security gap (**F5** — Anthropic/Gemini model-*listing* handlers skip the grant filter). Also new: a model-listing correctness bug (**F2**), a cross-client API footgun (**F13**), an HTTP-verb inconsistency (**F9**), fail-open grant display (**F26**), and a middleware-bypass regression test gap (**X1**).

---

## 🔴 Critical (blocks merge)
| # | Layer | File · Location | Issue | Fix | Verdict |
|---|-------|-----------------|-------|-----|---------|
| **F1** | routes_app | `oai/routes_oai_chat.rs` · `embeddings_handler` (~L255) | **Per-model grant bypass on `/v1/embeddings`.** The diff added `ensure_model_inference` to chat, anthropic, and gemini inference handlers but **not** to `embeddings_handler`. A token with `ModelGrant::Specific{ids:["allowed"]}` can POST `/v1/embeddings` with a non-granted model and get a **200**, while the same token is correctly 403'd on `/v1/chat/completions`. | Add `auth_scope.access_policy().ensure_model_inference(&model)?;` before `find_alias` in `embeddings_handler`. Add unit tests for ApiToken + ExternalApp restricted cases (mirror `test_chat_completions_scoped_token_forbidden`). | confirmed |

---

## 🟠 Important (should fix before ship)

### Security
| # | Layer | File · Location | Issue | Fix | Verdict |
|---|-------|-----------------|-------|-----|---------|
| **F5** | routes_app | `anthropic/routes_anthropic.rs` (list ~L184, get ~L229) · `gemini/routes_gemini.rs` (list ~L123, get ~L149) | **Model-listing handlers bypass the listable grant filter.** Inference got the guard; the 4 Anthropic/Gemini list/get handlers iterate all aliases with no `model_listable` check, so a restricted token sees every configured model (OAI `/v1/models` correctly filters). | In each list handler `retain` only entries where `policy.model_listable(prefixed_id)`; add a not-listable→404 guard to the get handlers (mirror `oai_model_handler`). | confirmed |
| **F7** | services + ui | `app_access_requests/access_request_objs.rs` `ApprovedResourcesV1::default` (~L281); consent `routes/apps/access-requests/review/index.tsx` (L270-274, 333) | **Apps self-select whether models can be restricted.** `ModelGrant::default()=All`; the consent screen only renders the model-restrict UI if the app requested `models_list‖models_access`, else forces `{type:'all'}`. Owner has no way to clamp an app that didn't ask → identical to the pre-effort "apps get all models". MCP correctly defaults to deny. | Always render the model-restrict `GrantBlock` at consent (default All, owner can lower) — OR document as accepted design in `security.md`. | confirmed |

### API consistency
| # | Layer | File · Location | Issue | Fix | Verdict |
|---|-------|-----------------|-------|-----|---------|
| **F13** | cross-cutting | `access_request_objs.rs` · `RequestedResourcesV1.{models,mcps}_access` (bool) vs `ApprovedResourcesV1.{models,mcps}_access` (`ModelGrant`/`McpGrant`) | **Same field name, different type across request vs approve.** A non-TS client that copies `requested.models_access` (`true`) into the approve body sends a bool where a `{type:...}` object is required → 400/422, silent approval failure. | Rename the request booleans to break the collision (e.g. `show_models_access` / `show_mcps_access`); regen openapi + ts-client + fixtures. | confirmed |
| **F9** | routes_app | `apps/routes_apps.rs` (utoipa ~L227) · `routes.rs` (~L354) | **HTTP verb mismatch**: approve is `PUT`, deny + revoke are `POST`. All three are one-shot non-idempotent transitions (2nd call → 409). Clients/SDKs inferring the verb from siblings hit 405. | Change approve to `POST` (utoipa + `post(apps_approve_access_request)`) so all three transitions match. | confirmed |

### Correctness
| # | Layer | File · Location | Issue | Fix | Verdict |
|---|-------|-----------------|-------|-----|---------|
| **F2** | services | `models/model_objs.rs` · `Alias::retain_listable_models` forward_all branch (~L1008) | **`/bodhi/v1/models` specific-grants broken for `forward_all_with_prefix` aliases.** It judges the whole alias by `keep(prefix)` (e.g. `""`), while the UI picker and OAI listing key on `prefix+model.id`. A `Specific{ids:["gemini-flash"]}` grant prunes the alias entirely from `/bodhi/v1/models` (OAI listing + inference still work). | Iterate `models` and `keep(&format!("{}{}",prefix,m.id()))`, returning true if any passes; add unit tests for this branch. | confirmed |
| **F16** | ui | `lib/grantItems.ts` · `grantableModelItems` (L12-22) | **ModelRouter aliases misclassified as `'local'`.** `'alias' in alias` also matches `ModelRouterResponse`, so routers land in the "Local Models" group with a `local` badge and are wrongly hidden by an "API only" filter. | Add a guard (`'targets' in alias && 'strategy' in alias`) before the `'alias'` branch; emit untyped/ungrouped (or use `source`). | confirmed |
| **F17** | ui | `routes/tokens/new/index.tsx` · `CardTitle` | **Wrong title "New App Token" on the API-Token create page** — contradicts its own breadcrumb and collides with the separate `/tokens/apps/` screen. | Revert to "New API Token". | confirmed |
| **F24** | e2e | `specs/tokens/app-tokens-grants.spec.mjs` | `reducedMotion:'reduce'` set too late (inside a step) / missing on the scoped-grants context → view-transition detach races. | Set `reducedMotion:'reduce'` at context creation for all rail/VT specs. | confirmed |
| **F25** | e2e | `pages/TokensPage.mjs` · `selectSpecificFromPanel` | Doesn't wait for the shadcn Sheet overlay (`fixed inset-0 z-50`) to detach after Done before the next click → flake. Sibling `AccessRequestReviewPage` already guards this. | Wait for overlay detach before proceeding. | confirmed |

### Test coverage
| # | Layer | File · Location | Missing | Verdict |
|---|-------|-----------------|---------|---------|
| **F12** | routes_app | anthropic/gemini/mcp_proxy handlers | Handler-level grant-enforcement tests: anthropic/gemini disallowed-model 403; mcp_proxy disallowed-connect 403 + owner-extra connect; `mcps_show` grant-hidden 404. Only OAI chat is covered. | confirmed |
| **X1** | routes_app | `middleware/access_requests/test_access_request_middleware.rs` | No test for the new `AuthContext::ApiToken → Session` bypass arm (L92). Removing it silently 403s API tokens on `/apps/mcps/*` with no failing test. | confirmed |
| **F3** | services | `app_access_requests/test_access_request_repository.rs` | `update_revocation` + `list_approved_for_user` have **zero** repo-level unit/cross-tenant tests (the app-token kill-switch). | confirmed |
| **F4** | services | `models/model_objs.rs` | `retain_listable_models` branch matrix untested (relevant to F2). | confirmed |
| **F18** | ui | `lib/grantItems.ts` | `grantableModelItems` / `grantableMcpItems` have no unit tests (relevant to F16). | confirmed |
| **F19** | ui | consent approval payload test | Does not assert the `mcps_list` field in the approve payload. | confirmed |
| **F21** | e2e | `pages/AccessRequestReviewPage.mjs` · `approveWithGrants()` | No owner-extra-MCP (`mcpsSpecific`) path → the owner-extra MCP grant flow is never E2E-exercised. | plausible* |

---

## 🟡 Nice-to-have (future)
| # | Layer | File · Location | Issue | Verdict |
|---|-------|-----------------|-------|---------|
| F6 | routes_app | `shared/token_grants.rs` `AccessPolicy::of` wildcard | Unbound `ExternalApp{grants:None}` → `Unrestricted` on model paths. Currently unreachable (role-None rejected by api_auth) but undocumented/fragile — make it fail-closed or document the minting invariant. | plausible |
| F26 | services | `tokens/token_objs.rs` `TokenDetail::from` (L192); `apps_api_schemas.rs` `AppAccessSummary::from_row` (L104) | **Grant display fails OPEN** (`unwrap_or_default()` = all-access) on corrupt grants JSON, while auth middleware fails closed → UI shows a healthy all-access token that 401s on every call. Make display fallback empty-deny. | confirmed |
| F29 | routes_app | `token_service.rs` `access_request_cache_needle` | Revoke eviction depends on undocumented compact-JSON substring; a serialization change silently no-ops eviction (token works until TTL). Add a unit test asserting the needle is in the serialized value. | confirmed |
| X3 | routes_app | `apps_api_schemas.rs` `AppAccessSummary::from_row` (L111) | `approved_role` parsed from DB without the exchange-time privilege ceiling → a tampered row shows an elevated role in list/revoke (token still can't exchange at that level). Cap at user's role or document. | confirmed |
| X2 | routes_app | `app_mcps()` | Union of by-url approved instances + owner-extra ids is untested. | confirmed |
| F8 | routes_app | `update_revocation` | `AccessRequestNotDraft` reused as the revoke status guard — misleading error code/name. | confirmed |
| F10 | routes_app | `users/routes_users_info.rs` | `/bodhi/v1/user` reflects effective grants at different JSON depths for ApiToken vs ExternalApp (consumer must special-case). | plausible |
| F32 | routes_app | `routes.rs` | Collection endpoint `/access-requests/apps` nested under the per-ID action group (path-shape inconsistency). | plausible |
| F33 | routes_app | utoipa | OperationId `approveAppsAccessRequest` uses plural "Apps", breaking the naming pattern. | plausible |
| F27 | services | `tokens/token_service.rs` | `delete_token` service-level 404 mapping untested. | confirmed |
| F28 | services | `utils/cache_service.rs` | New cache test uses `(actual, expected)` assertion order (convention is expected-first). | confirmed |
| F14 | cross-cutting | grant envelopes | `mcps` name collision: `McpGrant` in `TokenGrantsV1` vs `Vec<McpApproval>` in `ApprovedResourcesV1`. | plausible |
| F45 | cross-cutting | `tokens/token_objs.rs` | `TokenGrantsV1` verb-first names (`list_models`) vs app envelopes' noun-first (`models_list`) — 3 conventions for one concept. | plausible |
| F15 | ui | `hooks/apps/*` | Apps mutation hooks use inline error extraction instead of the shared `extractErrorMessage` helper. | confirmed |
| F34 | ui | `AccessPickerPanel` | Group labels "Local Models"/"API Models" hardcoded regardless of resource noun (reused for MCPs). | confirmed |
| F35 | ui | `ReviewContent` | Fetches models + MCPs unconditionally even when the app didn't request those controls. | plausible* |
| F36 | ui | `AppDetailPanel` | Hardcodes an "active" status chip instead of reading `app.status` (won't show Revoked). | confirmed |
| F37 | ui | `ListingToggle` | Keyboard activation (Space/Enter) not unit-tested. | confirmed |
| F38 | ui | `AccessPickerPanel` | Type filter (Local/API dropdown) not unit-tested. | confirmed |
| F39 | ui | `GrantBlock` | No dedicated unit test. | confirmed |
| F40 | ui | `TokenForm` | PowerUser scope card disabled-state for `resource_user` not tested. | confirmed |
| F20 | e2e | spec | Missing positive enforcement paths (granted model infers, granted MCP connects) + `/v1/models`,`/v1/mcps` reflection. | plausible |
| F41 | e2e | `api-tokens.spec.mjs` | Phase-4 "List-all with All access" second case not implemented. | confirmed |
| F42 | e2e | spec | Keyboard-nav assertion reaches into internal `.l-listrow.active` CSS instead of a page-object method. | confirmed |
| F43 | e2e | spec | `INTEG_TEST_USER_MANAGER` env not validated in `beforeAll` (silent failure vs loud throw). | confirmed |

\* **F21, F35** verdicts are `plausible` because their two verify agents hit a transient API connection drop mid-run; reasoning held but the final adversarial confirmation didn't complete. Treat as real pending a quick recheck.

---

## Missing Test Coverage (aggregated)
| Layer | What's missing | Tied to |
|-------|----------------|---------|
| routes_app | embeddings grant-enforcement; anthropic/gemini/mcp-proxy enforcement; ApiToken middleware-bypass arm; `app_mcps` union | F1, F5, F12, X1, X2 |
| services | `update_revocation`/`list_approved_for_user` repo+isolation; `retain_listable_models` branches; `delete_token` 404; cache needle | F3, F4, F27, F29 |
| ui | `grantItems` mappers; `GrantBlock`; `ListingToggle` keyboard; `AccessPickerPanel` filter; `mcps_list` payload assert; PowerUser card | F18, F19, F37, F38, F39, F40 |
| e2e | positive infer/connect + list reflection; owner-extra MCP path; List-all case | F20, F21, F41 |

## Fix Order (layered)
1. **services** — F26 (fail-closed display), F2 + F4, F3, F27, F28, F29-test, F8 → `cargo test -p objs -p services`
2. **routes_app** — **F1 (critical)**, F5, F12, X1, X2, F6/X3 (decide), F9, F32/F33 → `cargo test -p objs -p services -p routes_app`
3. Full backend → `make test.backend`
4. Decide F7 + F13 (cross-layer): default-clamp policy + request-bool rename. If F13 renames fields → `cargo run -p xtask openapi && make build.ts-client`
5. **frontend** — F16 + F18, F17, F19, F15, F34, F35, F36, F37-F40 → `cd crates/bodhi && npm test`
6. **e2e** — F24, F25, F21, F20, F41, F42, F43 → `make build.dev-server && make test.e2e`
7. **docs** — update `security.md` if F6/F7/X3 accepted as design; crate CLAUDE.md for the grants module

## Refuted (investigated, not actionable — do not re-raise)
| # | Layer | Why dismissed |
|---|-------|---------------|
| F11 | routes_app | `AccessPolicy::of` using `.v1()` does not defeat versioning — it's the intended accessor. |
| F31 | routes_app | 403-echo (inference) vs 404 (listing) reveal split is acceptable/by-design, not a leak. |
| F30 | routes_app | Cache-eviction side-effect *is* covered (the revoke→immediate-reject path is exercised). |
| F22 | e2e | `copyTokenFromDialog()` clipboard read is a legitimate observation, not a black-box violation. |
| F23 | e2e | `toggleTokenStatus()` `waitForTimeout(100)` is benign, not a flake source. |
| F44 | e2e | `findTokenByName` `includes()` is safe given the test's unique token names. |

## Reports
- `services-review.md` · `routes_app-review.md` · `ui-review.md` · `e2e-review.md` · `cross-cutting-review.md` (per-finding evidence + verify notes)
- `_salvage-verified-findings.json` (machine-readable: 42 verified + 6 refuted) · `_salvage-raw-findings.json` (49 pre-triage)
</content>
