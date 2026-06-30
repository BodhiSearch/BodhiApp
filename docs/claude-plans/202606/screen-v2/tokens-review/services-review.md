# Services Review

Ultracode re-review (Sonnet workflow) of diff range `4dea5ea9..HEAD` — "tokens screen-v2 migration + App Token grants" effort. Findings below survived adversarial verification (refute-by-default); each carries a verdict (`confirmed` = defect traced in committed code; `plausible` = likely real, severity/reachability not fully confirmed). Review only — no source modified.

## Summary
- Findings in this layer: 7 (Critical: 0, Important: 3, Nice-to-have: 4)

## Findings

### F2: retain_listable_models uses prefix string for forward_all_with_prefix aliases instead of per-model IDs
- **Priority**: important  ·  **Verdict**: confirmed  ·  **Category**: correctness
- **File**: `crates/services/src/models/model_objs.rs`
- **Location**: Alias::retain_listable_models, forward_all_with_prefix branch, ~lines 1008-1010
- **Issue**: For API aliases where forward_all_with_prefix = true, retain_listable_models calls keep(a.prefix.as_deref().unwrap_or("")) — judging the entire alias by the prefix string alone. The UI picker (grantItems.ts) generates grant item IDs as prefix + model.id (e.g. "gemini-flash" when prefix is ""), which is what OAI listing and inference enforcement also use. The Bodhi /bodhi/v1/models listing therefore uses a different ID namespace than every other enforcement site, making specific-model grants on forward_all aliases non-functional for that endpoint.
- **Failure scenario**: User creates a Gemini API alias with forward_all_with_prefix=true and empty prefix. Creates an API token with ModelGrant::Specific { ids: ["gemini-flash"] } (the ID the picker shows). GET /bodhi/v1/models → retain_listable_models calls keep("") → model_listable("") = false → alias pruned entirely. GET /v1/models (OAI) → per-entry check model_listable("gemini-flash") → true → alias shown. POST /v1/chat/completions → ensure_model_inference("gemini-flash") → true → succeeds. Inference and OAI discovery work but the Bodhi models list is broken for this token.
- **Recommendation**: For forward_all_with_prefix aliases, iterate the models array and call keep(&format!("{}{}", prefix, m.id())) for each entry (returning true if any passes), matching the per-model-ID logic used by OAI listing and matchable_models(). Update or add unit tests to cover the corrected branch.
- **Rationale**: grantableModelItems (new in this diff) and retain_listable_models (new in this diff) were added together but use different ID forms for this alias type, guaranteeing a broken grant for any user who uses a forward_all Gemini or OpenAI alias with specific-model grants from the UI.
- **Evidence**: Both `retain_listable_models` (services/src/models/model_objs.rs) and `grantableModelItems` (bodhi/src/lib/grantItems.ts) were introduced in this diff. The bug is at lines 1008–1009 of model_objs.rs:

```rust
if a.forward_all_with_prefix {
  return keep(a.prefix.as_deref().unwrap_or(""));   // BUG: uses prefix, not per-model IDs
}
```

The three sites use different IDs for the same entity:
- **UI picker** (`grantItems.ts` lines 13–16): iterates `alias.models`, generates `"${prefix}${model.id}"` (e.g., `"gemini-flash"` for prefix="" and model.id="gemini-flash"). This is what ends up in `ModelGrant::Specific { ids: [...] }`.
- **OAI `/v1/models`** (`routes_oai_models.rs` line 75): calls `api_alias.matchable_models()`, which regardless of `forward_all_with_prefix` iterates `self.models` and emits `prefix + model.id` strings, then filters via `policy.model_listable(&m.id)` using those full IDs.
- **Bodhi `/bodhi/v1/models`** (`routes_models.rs` line 147): calls `alias.retain_listable_models(|id| policy.model_listable(id))`. For `forward_all_with_prefix = true`, the closure receives only the raw prefix string (`""` for empty prefix), not per-model IDs. `model_listable("")` checks `ids.contains("")` → `false`.

Consequently, with `ModelGrant::Specific { ids: ["gemini-flash"] }` on a `forward_all_with_prefix = true` alias with empty prefix: `keep("")` → `false` → entire alias is prune …(truncated)
- **Verify notes**: The doc comment on `retain_listable_models` explicitly states "a `forward_all_with_prefix` API alias matches an unbounded set and is judged by `keep(prefix)` as a whole" — this was the *intended* design, but it is internally inconsistent with how the UI and the OAI listing expand the same aliases. Since both conflicting functions (retain_listable_models and grantableModelItems) were introduced together in this diff, neither was pre-existing; the finding correctly identifies a genuine correctness bug. Priority "important" is appropriate: the bug breaks the Bodhi models list endpoint for a specific combination (forward_all_with_prefix alias + specific model grant), while inference (OAI chat) a …
- **Sources**: bug:routes_app

### F3: update_revocation and list_approved_for_user have no repository-level unit tests or cross-tenant isolation tests
- **Priority**: important  ·  **Verdict**: confirmed  ·  **Category**: test-coverage
- **File**: `crates/services/src/app_access_requests/test_access_request_repository.rs`
- **Location**: entire file — update_revocation and list_approved_for_user absent
- **Issue**: Both new DefaultDbService methods introduced in this diff — update_revocation and list_approved_for_user — have zero coverage in test_access_request_repository.rs. Repository tests cover create, get, update_approval, update_denial, update_failure, get_by_access_request_scope, but not these two. Additionally, test_access_request_repository_isolation.rs (72 lines) was not modified; the cross-tenant isolation tests required by the codebase convention for all new tenant-scoped DB mutations are absent. Critical paths untested: (a) double-revoke of already-Revoked record; (b) list_approved_for_user excludes Revoked/Draft; (c) tenant isolation in update_revocation (the explicit TenantId filter defending SQLite); (d) wrong-user check returning ItemNotFound; (e) non-Approved status guard.
- **Failure scenario**: A regression removes the TenantId filter from update_revocation. User A in tenant A calls POST /access-requests/<id>/revoke where <id> belongs to user A in tenant B. The request succeeds, revoking a cross-tenant grant. No repository-level test exists to catch this.
- **Recommendation**: Add rstest/sqlx cases to test_access_request_repository.rs: (1) list_approved_for_user returns only Approved rows, excludes Revoked/Draft; (2) update_revocation happy path Approved→Revoked; (3) update_revocation wrong user → ItemNotFound; (4) update_revocation on non-Approved → AccessRequestNotDraft (or the renamed variant from F7); (5) update_revocation on a different tenant's record → ItemNotFound. Add test_cross_tenant_revoke_blocked and test_cross_tenant_list_approved_isolation to the isolation file. Use #[values("sqlite", "postgres")] on all cases.
- **Rationale**: The revocation path is security-critical: it is the kill-switch for app tokens. The crates/CLAUDE.md convention requires isolation tests for all new tenant-scoped DB operations. The two new methods are invoked by security-critical routes.
- **Evidence**: Confirmed by reading the actual files and running git diff:

1. `/crates/services/src/app_access_requests/test_access_request_repository.rs` (363 lines) contains zero test functions referencing `update_revocation` or `list_approved_for_user`. Grep across all test_*.rs files returns no hits for these method names in services.

2. `git diff 4dea5ea9..HEAD -- test_access_request_repository.rs test_access_request_repository_isolation.rs` produced empty output — neither file was modified in this diff. The isolation file (73 lines) only contains `test_cross_tenant_app_access_request_isolation` which tests the `get` path only.

3. Route-level tests in `crates/routes_app/src/apps/test_access_request.rs` DO exist and use a real `DefaultDbService` (not mocked via `AppServiceStubBuilder.with_db_service()`). They cover: happy-path Approved→Revoked transition, wrong-user rejection, and non-approved (Draft) rejection. But `build_test_harness` calls `builder.with_tenant(services::Tenant::test_default())` seeding only TEST_TENANT_ID — no multi-tenant setup, so cross-tenant isolation for revocation is untested at any layer.

4. Both `update_revocation` and `list_approved_for_user` are confirmed new in this diff (appear in `access_request_repository.rs` diff under `+` lines at lines 39-82 of the diff hunk).
- **Verify notes**: The finding accurately identifies the gap. The route-level tests (routes_app) do exercise the real DB and cover wrong-user and non-approved-status rejection paths, which partially mitigates the "zero coverage" claim — so this is not as dire as the finding implies for those specific scenarios. However, the cross-tenant isolation scenario remains genuinely uncovered at every layer: no repository test, no isolation test, and the route tests only seed a single tenant. The TenantId filter in `update_revocation` (line 388 of the diff: `.filter(app_access_request_entity::Column::TenantId.eq(&tenant_id_owned))`) is the SQLite defense-in-depth guard, and its removal would go undetected. Priority stay …
- **Sources**: test:backend (update_revocation), test:backend (list_approved_for_user), sec:lifecycle, bug:services, prior:services-review F1

### F4: retain_listable_models has no unit tests despite non-trivial branching
- **Priority**: important  ·  **Verdict**: confirmed  ·  **Category**: test-coverage
- **File**: `crates/services/src/models/model_objs.rs`
- **Location**: Alias::retain_listable_models, ~line 1002
- **Issue**: New method with four branches (User, Model, ModelRouter — single-model keep — and Api with forward_all_with_prefix sub-cases including in-place models.retain and empty-list drop signal) has zero services-level unit tests. The route handler test test_oai_models_scoped_token_filters_listing uses only User/Model aliases from the data fixture; the Api branch with prefix pruning is never exercised.
- **Failure scenario**: A scoped token with ModelGrant::Specific { ids: ["openai/gpt-4o"] } lists models when an ApiAlias has prefix="openai/" and models=[gpt-4o, gpt-3.5-turbo]. retain_listable_models should keep gpt-4o and drop gpt-3.5-turbo. If the retain call were accidentally written as a.models.retain(|m| !keep(...)) (negated), both models are dropped and the alias is removed. No test catches this.
- **Recommendation**: Add inline unit tests in model_objs.rs covering: (1) Api with forward_all_with_prefix=false, some models kept → returns true with pruned list; (2) all models filtered → returns false; (3) forward_all_with_prefix=true judged by prefix; (4) User/Model/ModelRouter pass-through. A route-handler test with a real ApiAlias + scoped token would also strengthen coverage.
- **Rationale**: The only non-trivial branch mutates state (in-place models.retain). The drop-on-empty contract is consumed by retain_mut at routes_app/src/models/routes_models.rs:147. Correctness cannot be inferred from other branches.
- **Evidence**: 1. `retain_listable_models` is introduced in this diff at `crates/services/src/models/model_objs.rs:1002`. The `Alias::Api` branch does in-place `a.models.retain(|m| keep(&format!("{}{}", prefix, m.id())))` and returns `!a.models.is_empty()` to signal the drop-on-empty contract. 2. `test_model_objs.rs` was not modified in this diff and has no occurrence of `retain_listable_models` — confirmed by grep returning empty output. 3. The route-level test `test_oai_models_scoped_token_filters_listing` (added in this diff in `crates/routes_app/src/oai/test_models.rs`) uses the `app()` fixture, which calls `with_data_service()` → `seed_test_user_aliases()`. That function seeds only three `UserAlias` rows (llama3, testalias_exists, tinyllama) — no `ApiAlias` entries. So only the `Alias::User(a) => keep(&a.alias)` branch of `retain_listable_models` is exercised; the `Alias::Api` branch (mutating retain + empty-list drop) is never reached by any test.
- **Verify notes**: The method implementation looks correct as written — the retain condition is not negated, and the prefix concatenation is sound. The gap is a test-coverage issue: a future refactor (e.g. negating the closure, changing prefix formatting, or altering the empty-list contract) would go undetected. The drop-on-empty return value is consumed by `retain_mut` at `routes_app/src/models/routes_models.rs:147`, making this contract load-bearing. Priority stays at important rather than critical because the current code is not defective — the risk is undetected future regression in the Api+prefix branch.
- **Sources**: test:backend (retain_listable_models), prior:services-review F2

### F26: Grant display paths fail-open (all-access) when grants JSON is unparseable, inconsistent with auth middleware's fail-closed behavior
- **Priority**: nice-to-have  ·  **Verdict**: confirmed  ·  **Category**: correctness
- **File**: `crates/services/src/tokens/token_objs.rs`
- **Location**: From<TokenEntity> for TokenDetail, line 192; also crates/routes_app/src/apps/apps_api_schemas.rs AppAccessSummary::from_row line 104
- **Issue**: Two display-path read functions use fail-open fallbacks when grants JSON is corrupt. (1) TokenDetail::from: serde_json::from_str(&t.grants).unwrap_or_default() falls back to TokenGrants::default() (list_models=true, models=All, list_mcps=true, mcps=All) when the stored grants column cannot be parsed. The auth middleware (token_service.rs) introduced in this diff does the opposite: it fails closed with TokenError::InvalidToken on a corrupt grants column. (2) AppAccessSummary::from_row: uses .and_then(|json| serde_json::from_str::<ApprovedResources>(json).ok()) with no warning log, swallowing deserialization errors silently. This was also the failure mode for tokens created with McpGrant::None (serialized as {type:"none"}) which was removed in this diff with no DB migration: the management UI displays such rows as all-access while every auth attempt is rejected 401.
- **Failure scenario**: For (1): A token's grants column is corrupted. GET /tokens/{id} returns grants:{list_models:true, models:{type:'all'}, ...} — all-access. The user sees no indication of a problem. Every API call using that token returns 401 (auth middleware fails closed). User sees a token that looks fine but never works. For (2): An invalid approved JSON blob appears in the DB. AppAccessSummary shows the grant as Approved with empty model+MCP access — indistinguishable from a legitimately empty-grant record. No log entry alerts on-call.
- **Recommendation**: For TokenDetail::from: change the fallback to TokenGrants::V1(TokenGrantsV1 { list_models: false, models: ModelGrant::Specific { ids: [] }, list_mcps: false, mcps: McpGrant::Specific { ids: [] } }) (empty-deny) so the display is conservative and hints at a problem. For AppAccessSummary::from_row: add a tracing::warn! inside the and_then closure when deserialization fails, logging the row id and raw JSON length, before calling .ok(). For the McpGrant::None migration: add a SeaORM migration converting grants column values containing "type":"none" to "type":"specific","ids":[].
- **Rationale**: Display and authentication paths give opposite answers for the same corrupt row. While the auth path correctly refuses access, the display path can mislead users into thinking the token is fully functional, delaying diagnosis of a corruption issue.
- **Evidence**: 1) crates/services/src/tokens/token_objs.rs lines 190-192: comment literally says "fall back to all-access defensively rather than panicking on an unexpected payload" and code is `serde_json::from_str(&t.grants).unwrap_or_default()`. TokenGrants::default() (lines 108-117) is V1 { list_models: true, models: All, list_mcps: true, mcps: All } — full all-access. 2) crates/routes_app/src/middleware/token_service/token_service.rs lines 135-138: comment says "Fail closed: a corrupt grants payload rejects the token rather than silently granting all-access" and code is `serde_json::from_str::<TokenGrants>(&api_token.grants).map_err(|e| TokenError::InvalidToken(format!("Invalid grants: {}", e)))?` — opposite behavior. 3) McpGrant::None introduced at 4dea5ea9 (diff start), removed at f1265209 without a DB migration; any token row with grants containing {"type":"none"} fails both parsers: auth returns 401 (fail-closed), display returns all-access (fail-open). 4) AppAccessSummary::from_row in crates/routes_app/src/apps/apps_api_schemas.rs lines 97-118 (new code in diff): parse failure arm returns Specific { list: false, ids: [] } (empty-deny, not all-access), with no tracing::warn! call.
- **Verify notes**: The asymmetry is real and ironically self-documented: the two code paths have opposing explanatory comments. However, this is a diagnostic/UX defect, not a security bypass — the auth middleware correctly rejects corrupt-grants tokens, so no unauthorized access occurs. The McpGrant::None migration concern is bounded to the development phase (introduced and removed within this same feature branch/range before any production deployment); existing production rows cannot have that variant. AppAccessSummary's failure path is correctly deny-empty (not all-access), so part (2) of the finding is slightly overstated in impact. Priority nice-to-have is appropriate: the recommendation to change unwrap_o …
- **Sources**: sec:enforcement F5 (McpGrant::None removal), bug:services (TokenDetail), sec:lifecycle F7 (AppAccessSummary)

### F27: delete_token service-level 404 mapping is untested
- **Priority**: nice-to-have  ·  **Verdict**: confirmed  ·  **Category**: test-coverage
- **File**: `crates/services/src/tokens/token_service.rs`
- **Location**: DefaultTokenService::delete_token
- **Issue**: The new method's whole reason to exist is the existence pre-check that maps a missing row to EntityError::NotFound("Token") (404) rather than letting the repository's DbError::ItemNotFound surface as a 500. Repository-level delete is tested (test_delete_api_token, test_cross_tenant_token_delete_blocked) but the service-level mapping branch is not — there is no test asserting delete_token on a missing id yields the NotFound domain error via .code().
- **Failure scenario**: A regression changes the error mapping in delete_token to propagate ItemNotFound as-is. GET /tokens/{id} DELETE on a non-existent token returns 500 instead of 404. No service-level test would detect this.
- **Recommendation**: Add a test_token_service.rs case: delete_token of an unknown id returns the not-found TokenServiceError/EntityError (assert via .code()); happy path deletes and returns Ok(()).
- **Rationale**: The mapping is the only logic the method adds; without a test the 404-vs-500 contract can regress unnoticed.
- **Evidence**: The `delete_token` method was introduced in this diff at `crates/services/src/tokens/token_service.rs` lines 222-244. Its only logic beyond delegating to the repository is the existence pre-check: `.ok_or_else(|| crate::EntityError::NotFound("Token".to_string()))`. The diff for `test_token_service.rs` adds only `grants: Default::default()` to an existing test — no `delete_token` test was added. Reading the full `test_token_service.rs` (286 lines) confirms there is no test that exercises `delete_token` at any path. The repository-level test at `test_token_repository.rs:338` verifies `DbError::ItemNotFound` for missing rows, but that is the layer below the service-level 404 mapping, which remains uncovered.
- **Verify notes**: The gap is real: if the `ok_or_else` branch were removed or changed to re-propagate `DbError::ItemNotFound` as-is, no service-level test would catch the 404→500 regression. Priority stays nice-to-have (not critical) because the method is simple, the mapping is a single expression, and Phase-9 E2E tests likely exercise the 404 path indirectly at the HTTP level.
- **Sources**: prior:services-review F3

### F28: New cache test uses wrong (actual, expected) assertion order
- **Priority**: nice-to-have  ·  **Verdict**: confirmed  ·  **Category**: convention
- **File**: `crates/services/src/utils/cache_service.rs`
- **Location**: tests::test_remove_entries_containing
- **Issue**: assert_eq!(cache.get("exchanged_token:aaa"), None) puts actual first, expected second, contrary to the crate convention assert_eq!(expected, actual). The pre-existing test above it has the same shape (consistency drift rather than a newly introduced violation in isolation), but the new test added in this diff follows the wrong order.
- **Recommendation**: Flip to assert_eq!(None, cache.get(...)). The remove_entries_containing logic and matrix coverage are otherwise correct.
- **Rationale**: Minor but the convention exists so pretty_assertions diffs read correctly.
- **Evidence**: The diff adds two assert_eq! calls in `test_remove_entries_containing` (crates/services/src/utils/cache_service.rs lines 101-102): `assert_eq!(cache.get("exchanged_token:aaa"), None)` and `assert_eq!(cache.get("exchanged_token:bbb"), None)` — actual-first, expected-second. The dominant convention across the services crate is expected-first: grep -rn "assert_eq!(None," shows 15+ usages in auth_context.rs, test_api_alias_repository.rs, test_settings_repository.rs, test_session_service.rs, etc. The pre-existing test at line 78 also uses actual-first (pre-existing drift), but the two newly-introduced lines in this diff consistently follow the wrong order.
- **Verify notes**: Confirmed as a real convention violation introduced in this diff. The logic and coverage of the test are correct; only the argument order to assert_eq! is reversed relative to the `assert_eq!(expected, actual)` crate convention. On failure, pretty_assertions will show "left"/"right" swapped, making the failure message misleading. Priority remains nice-to-have — no functional defect, no test correctness impact.
- **Sources**: prior:services-review F6

### X3: approved_role is validated against user's Keycloak resource role at exchange time but not at list time — a DB-tampered approved_role survives in AppAccessSummary
- **Priority**: nice-to-have  ·  **Verdict**: confirmed  ·  **Category**: security
- **File**: `crates/routes_app/src/apps/apps_api_schemas.rs`
- **Location**: AppAccessSummary::from_row(), line 111: `row.approved_role.and_then(|r| r.parse().ok())`
- **Issue**: token_service.rs lines 376-401 validate that `approved_role` in the DB does not exceed the user's current Keycloak resource role at token-exchange time. AppAccessSummary::from_row() (used by both GET /access-requests/apps list and POST /access-requests/{id}/revoke) parses `approved_role` directly from the DB without the same privilege-escalation check. If a DB row is tampered to hold an `approved_role` of `scope_user_power_user` for a user whose Keycloak role is `resource_user`, the list and revoke response will reflect the elevated role even though token exchange would reject it. This is a display-layer inconsistency, not an enforcement bypass, but it could mislead an owner reviewing their own app grants.
- **Failure scenario**: Tamper with an AppAccessRequest row to set approved_role = 'scope_user_power_user' for a user whose KC role is User. GET /access-requests/apps returns AppAccessSummary with approved_role: power_user. The actual token cannot be exchanged at power_user level (token_service rejects it), but the UI shows the elevated grant, confusing the owner.
- **Recommendation**: Either (a) apply the same privilege-ceiling check in from_row — cap approved_role at the user's current resource role — or (b) document in the API spec that approved_role in the list endpoint reflects the stored value (which may have been capped at exchange time), so clients should not rely on it for security decisions.
- **Rationale**: The token-exchange validation and the list-display path diverge on privilege capping. While not an enforcement gap, the inconsistency is a latent confusion vector when investigating a revoked or tampered grant.
- **Evidence**: Two concrete code sites confirm the divergence:

1. `crates/routes_app/src/apps/apps_api_schemas.rs` line 124 (new code in this diff):
   `approved_role: row.approved_role.and_then(|r| r.parse().ok())`
   — parses `approved_role` from the DB string as-is, no privilege ceiling applied.

2. `crates/routes_app/src/middleware/token_service/token_service.rs` lines 375–401 (pre-existing, not in this diff's additions):
   ```rust
   // Verify approved_role doesn't exceed user's resource role (prevent privilege escalation via DB tampering)
   if let Some(approved_role) = role {
     let user_resource_role = scope_claims.resource_access.get(&instance.client_id)
       .and_then(|rc| ResourceRole::from_resource_role(&rc.roles).ok());
     if let Some(resource_role) = user_resource_role {
       let max_scope = resource_role.max_user_scope();
       if !max_scope.has_access_to(&approved_role) { return Err(...PrivilegeEscalation{...}); }
     }
   }
   ```
   — caps the displayed/used role against the user's live Keycloak resource role.

`AppAccessSummary::from_row()` is called by both `list_app_access` (GET /access-requests/apps, line 434) and `revoke_app_access` (POST /access-requests/{id}/revoke, line 476). Neither path applies the ceiling. The diff shows `apps_api_schemas.rs` is entirely new in this range; `token_service.rs` diff only adds `grants` field handling — the ceiling check at …(truncated)
- **Verify notes**: The divergence is real and confirmed. However, the priority "nice-to-have" is correct and should not be raised. Exploitation requires DB-level write access (an attacker who can modify rows can do far worse than confuse a display); enforcement at token-exchange time is not affected — a tampered `approved_role` would be rejected at exchange with `PrivilegeEscalation`. The only consequence is display confusion in the list/revoke response body shown to an app owner. No security bypass exists.
- **Sources**: critic
