# Technical Debt — Multi-Tenant

Deferred items from the multi-tenant review (commit `3ba6997f0` and subsequent fixes).
Full review status: `docs/archive/claude-plans/202603/20260303-multi-tenant/SUMMARY.md`

## P1-2: Missing AuthScopedAccessRequestService wrapper
`access_request_service()` is a bare passthrough — no AuthScoped wrapper. Not a data isolation bug but an architectural gap.

## P1-4: Access request scope index migration
`m20250101_000015_fix_access_request_scope_index.rs` not created. Need tenant-scoped unique index on `app_access_requests(tenant_id, access_request_scope)`.

## P1-6: Silent tenant fallback in token creation
`token_service.rs` — `unwrap_or_else(|| tenant_id.to_string())` when tenant not found. Should return error.

## P1-7: `has_api_key` hardcoded to `true` in list view
`api_model_service.rs` — list endpoint shows all models as having API keys. TODO comment added.

## P1-10: Non-atomic approve + role assignment
Three sequential operations without rollback. Accepted trade-off (non-financial, repo-level txn).

## P1-12: `tenant_id_or_empty()` in read methods
`auth_scoped_user_access_requests.rs` — mutations use `require_tenant_id()` but reads still use `tenant_id_or_empty()`.

## P1-15: `std::sync::RwLock` in StandaloneInferenceService
Standalone-only, locks not held across `.await`. Low risk but should migrate to `tokio::sync::RwLock`.

## PostgreSQL RLS Integration Tests
Only `api_tokens` table covered. Missing: all other tenant-scoped tables, cross-tenant mutation prevention, concurrent request isolation.

## Fold NetworkService into SettingService (drop the paired dependency)
Mostly resolved by the OAuth consent-flow cutover: `DefaultAccessRequestService` no longer takes
`SettingService`/`NetworkService` (the review URL is gone) and `resolve_public_server_url` was
removed with its last caller. Remaining: `routes_setup.rs` still injects `NetworkService`
directly for OAuth redirect-URI composition — fold that lookup into `SettingService` if a second
consumer ever appears; otherwise leave as-is.

## Scope vocabulary is not visible in the issued token
`scope_user_*` and `scope_apps:*` are consumed by the BodhiApp consent flow and never reach
Keycloak, so an app token's `scope` claim and the DB grant record describe different things
(the claim carries only `openid profile email roles` + passthrough + the dynamic
`scope_access_request:` value; the actual grant lives in `app_access_requests.approved`).
Revisit registering them as Keycloak client scopes with passthrough — noting the O(M×N)
client-scope assignment problem that caused the `6434d8d` revert in keycloak-bodhi-ext, and that
audience-bearing scopes would turn `aud` into an array (BodhiApp now parses one-or-many `aud`,
so that part is ready). Goal: claims present in both scope and record, verifiably matching.

## Orphaned approved grants when Keycloak rejects the authorize request
The consent flow validates only what BodhiApp itself depends on (client_id, redirect_uri
exact-match, duplicate params, scope vocabulary); `response_type`/`state`/PKCE are forwarded
verbatim for Keycloak to enforce. Consequence: an app sending params Keycloak later rejects
(e.g. missing `response_type`, or no PKCE against a PKCE-required client) still produces an
approved `app_access_requests` row before the browser hits Keycloak's error — an inert,
never-connected grant visible in Connected Apps. Accepted trade-off (rare, revocable, no token
can ever be exchanged against it). Revisit only if misbehaving apps make the list noisy — e.g.
a pending-until-first-exchange status.
