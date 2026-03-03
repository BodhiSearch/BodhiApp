# Technical Debt — Multi-Tenant

Deferred items from the multi-tenant review (commit `3ba6997f0` and subsequent fixes).
Full review status: `ai-docs/claude-plans/20260303-multi-tenant/reviews/summary.md`

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
