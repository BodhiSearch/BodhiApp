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
`NetworkService` is a tiny, stateless utility — one method `get_server_ip() -> Option<String>`
(`DefaultNetworkService` is a zero-field unit struct using a UDP-socket egress-interface trick).
Its only consumers are host/URL resolution: `routes_setup.rs` (OAuth redirect URIs) and
`DefaultAccessRequestService` (review URL). This forces callers to inject a **paired**
`SettingService` + `NetworkService` (see `DefaultAccessRequestService::new`,
`AppServiceBuilder::build_access_request_service`).

Proposal: `SettingService` owns the network lookup.
- `DefaultSettingService` holds an `Arc<dyn NetworkService>` that is **optionally injectable**
  (for tests / stubbing the detected IP) and otherwise defaults to `DefaultNetworkService`.
- `resolve_public_server_url(request_host)` drops its `server_ip` parameter and fetches the
  detected IP internally, so consumers pass only the request host.
- `DefaultAccessRequestService` then depends on `SettingService` alone (revert the
  `network_service` field/ctor arg added for the review-URL fix).
- Keep the `AppService::network_service()` accessor if `routes_setup.rs` still wants direct
  access, or migrate it to `setting_service.resolve_public_server_url(...)` too and remove the
  standalone accessor.

Net effect: one fewer cross-service dependency to thread through the builder and every test
construction site, and server-IP validation logic lives in one place (SettingService).
