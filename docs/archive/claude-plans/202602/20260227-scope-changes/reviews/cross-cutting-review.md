# Cross-Cutting Review: Scope Removal / Access Request Flow Changes

**Reviewer**: Claude Sonnet 4.6
**Commit**: HEAD (8f0af5bee — `feat(kc-scope-removal): layer 2 — update services crate`)
**Scope**: Cross-cutting files only (openapi.json, ts-client types, CLAUDE.md, authentication.md, lib_bodhiserver, lib_bodhiserver_napi, server_app tests)

---

## Summary

The diff removes the scope-based access model from external app authorization and replaces it with a role-based model (`requested_role` / `approved_role`). The `CreateAccessRequestResponse` is simplified from a `oneOf` enum to a single struct (always draft). `resource_scope` is fully removed. Token scopes are trimmed to two-tier (`User`/`PowerUser` only). The changes are consistent across all reviewed files with one low-priority documentation issue.

---

## Findings

### P0 — Blockers

None identified.

---

### P1 — High Priority

None identified.

---

### P2 — Medium Priority

None identified.

---

### P3 — Low Priority / Minor Issues

#### 1. `.env.test.example` still documents `INTEG_TEST_RESOURCE_CLIENT_SCOPE`

**File**: `crates/server_app/tests/resources/.env.test.example`
**Line**: 12

The example file still documents `INTEG_TEST_RESOURCE_CLIENT_SCOPE=<resource-client-scope>`. The corresponding live server setup in `live_server_utils.rs` removed the `std::env::var("INTEG_TEST_RESOURCE_CLIENT_SCOPE")` read in this commit. The actual `.env.test` file does not contain the variable (correct), but the example template was not updated. A developer following the example to configure a new test environment would set a variable that is no longer used.

**Recommendation**: Remove line 12 from `.env.test.example`.

---

#### 2. Typo in CLAUDE.md `routes_app` keyword entry

**File**: `CLAUDE.md`
**Line**: 126

The `routes_app` keyword column has `{toolsets:[--]}` — the `--` appears to be a copy-paste error. Looking at the pre-change diff, the original had `{toolsets:[...]}` using `...` as a placeholder. The new text reads:

```
List response shapes: non-paginated use resource-plural field names ({mcps:[...]}, {toolsets:[--]}),
```

The `[--]` in `toolsets` is inconsistent with `[...]` in `mcps` and is not meaningful as documentation. This was introduced in this commit.

**Recommendation**: Change `{toolsets:[--]}` to `{toolsets:[...]}` to match the `mcps` example.

---

### Informational / Confirmed Correct

The following items from the review checklist were verified as correct:

**openapi.json**

- `CreateAccessRequestBody` schema has `requested_role` as a required property. Confirmed.
- `ApproveAccessRequestBody` schema has `approved_role` as a required property. Confirmed.
- `CreateAccessRequestResponse` is now a flat object (not `oneOf`), with required fields `id`, `status`, `review_url`. The status description says `"Status (always \"draft\")"`. Confirmed — the `oneOf` with auto-approve path is fully removed.
- `AccessRequestStatusResponse` now has `requested_role` (required) and `approved_role` (nullable). The old `resource_scope` field is gone. Confirmed.
- `AccessRequestReviewResponse` (list response) now includes `requested_role` as a required field. Confirmed.
- `TokenScope` enum trimmed to `scope_token_user | scope_token_power_user`. The `scope_token_manager` and `scope_token_admin` variants are removed. Confirmed.
- No remaining `resource_scope` references in `openapi.json`. Confirmed (grep count = 0).

**ts-client generated types**

- `openapi-schema.ts` and `types.gen.ts` are in sync with `openapi.json`. All field additions, removals, and type changes are reflected accurately. `CreateAccessRequestResponse` union type is replaced by a single interface. `TokenScope` and `UserScope` both trimmed to two-tier. Confirmed.

**lib_bodhiserver `app_service_builder.rs`**

- `build_access_request_service` no longer accepts `app_instance_service`. The `DefaultAccessRequestService::new` call signature matches the updated services crate (no `app_instance_service` parameter). Confirmed.
- `update_with_option` calls `create_instance` without the old `scope` parameter. The `AppInstanceService::create_instance` trait signature takes `(client_id, client_secret, status)` — exactly three arguments. Confirmed.
- `app_instance_service` is still constructed and passed to `DefaultAppService::new` as a separate registry member (line 147). The removal was only from `AccessRequestService` construction, not from the overall app service. This is correct.

**lib_bodhiserver_napi `config.rs`**

- The `AppInstance` struct literal construction no longer includes a `scope` field (removed in this commit at line 122–127). The `AppInstance` struct in the services crate has no `scope` field. Confirmed consistent.
- Two new constants exported (`BODHI_SESSION_DB_URL`, `BODHI_DEPLOYMENT`) — not directly related to scope removal, but correctly added to `index.d.ts` and `index.js`.

**server_app integration tests**

- `ExternalTokenSimulator::create_token_with_role` replaces `create_token_with_scope`. The new method:
  - Takes `role: Option<&str>` (not a scope string).
  - Sets JWT `scope` claim to `"openid"` only (not `scope_user_*`).
  - Seeds `CachedExchangeResult` with `role: Option<String>` and `access_request_id: Option<String>`.
  - `access_request_id` is set to `Some(Uuid::new_v4())` when a role is provided, and `None` when no role.
- `CachedExchangeResult` struct in `auth_middleware` has fields `token`, `app_client_id`, `role: Option<String>`, `access_request_id: Option<String>`. The simulator correctly populates both new fields. Confirmed.
- All three tests updated: `test_oauth_token_with_role_can_list_toolsets`, `test_oauth_token_without_role_is_rejected`, `test_oauth_token_rejected_on_session_only_get`. All use `create_token_with_role`. No remaining `create_token_with_scope` calls anywhere. Confirmed.
- The `live_server_utils.rs` `setup_minimal_app_service` no longer reads `INTEG_TEST_RESOURCE_CLIENT_SCOPE` env var, and `create_instance` is called without the scope argument at both call sites (lines ~171 and ~518). Confirmed.
- `DefaultAccessRequestService::new` calls in `live_server_utils.rs` (both `setup_minimal_app_service` and `setup_test_app_service`) no longer pass `app_instance_service`. Confirmed.

**CLAUDE.md documentation**

- `objs` keywords updated: `ResourceRole hierarchy (Admin>Manager>PowerUser>User)`, `UserScope/TokenScope two-tier (User, PowerUser only)`, `AppRole union type`. Accurate.
- `services` keywords updated: `AccessRequestService (draft-first with requested_role/approved_role)`, `AppInstanceService (no scope field)`. Accurate.
- `auth_middleware` keywords updated: `api_auth_middleware role checks`, `ExternalApp.role from DB approved_role (not JWT scopes)`, `CachedExchangeResult with role/access_request_id`, `test_external_app_no_role` factory. Accurate.
- `routes_app` keywords updated: `app access request workflow (draft-first, requested_role/approved_role)`. Accurate.
- `Access Request Workflow` section in architecture overview updated to reflect draft-first model. Accurate.

**authentication.md**

- Role hierarchy section split into `ResourceRole` (session, 4-tier) vs `UserScope/TokenScope` (external app/API token, 2-tier). This is a significant clarification and is accurate per the actual enum definitions in `token_scope.rs`.
- New `External App Access Request Workflow` section with accurate flow diagram. Matches implementation.
- Key design decisions documented accurately: no auto-approve, role from DB not JWT, two role columns.
- `AppRegInfo` code snippet retained from the old token section — this struct was replaced by `AppInstance` in a prior commit. This is pre-existing issue, not introduced by this commit.

**Dead code / removed symbol cleanup**

- `resource_scope`: zero remaining references anywhere in source, tests, or generated files (outside replaced generated files that were updated in this commit). Clean.
- `register_resource_access`: zero remaining references. Clean.
- `ResourceScope`: zero remaining references. Clean.
- `scope_user_manager`, `scope_user_admin`, `scope_token_manager`, `scope_token_admin`: zero remaining references in production code. The only appearances are as negative test cases in `token_scope.rs` `test_token_scope_parse_invalid` and `resource_role.rs` `test_role_parse_invalid`, which correctly assert these strings are rejected. These are correct, not dead code.
- `create_token_with_scope`: zero remaining references anywhere. Clean.
- `INTEG_TEST_RESOURCE_CLIENT_SCOPE`: absent from actual `.env.test` and absent from `live_server_utils.rs` code. Only remains in `.env.test.example` (see P3 finding #1).

---

## Checklist Verdict

| Area | Status | Notes |
|------|--------|-------|
| `openapi.json` — `CreateAccessRequestBody.requested_role` | PASS | Present as required field |
| `openapi.json` — `ApproveAccessRequestBody.approved_role` | PASS | Present as required field |
| `openapi.json` — `CreateAccessRequestResponse` is flat struct | PASS | `oneOf` removed, always draft |
| `openapi.json` — `AccessRequestStatusResponse` has `requested_role`+`approved_role` | PASS | `resource_scope` removed |
| `openapi.json` — no `resource_scope` remaining | PASS | Count = 0 |
| ts-client types match openapi.json | PASS | Both generated files consistent |
| `lib_bodhiserver` — scope removed from `create_instance` | PASS | 3-arg signature used |
| `lib_bodhiserver` — `app_instance_service` removed from `AccessRequestService` | PASS | Removed from builder method |
| `server_app` — `create_token_with_scope` replaced with `create_token_with_role` | PASS | No remaining uses |
| `server_app` — `ExternalApp` pattern uses role not scope | PASS | `CachedExchangeResult.role` populated |
| CLAUDE.md updated for scope changes | PASS | Minor typo in routes_app entry (P3) |
| `authentication.md` reflects new role-based model | PASS | Accurate and improved |
| No dead code: `ResourceScope`, `register_resource_access`, `resource_scope` | PASS | Zero remaining references |
| `.env.test.example` updated | FAIL (P3) | Still documents obsolete `INTEG_TEST_RESOURCE_CLIENT_SCOPE` |
