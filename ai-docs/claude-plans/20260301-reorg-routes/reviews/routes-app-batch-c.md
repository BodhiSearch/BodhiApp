# Review: routes_app Batch C -- Thin Modules, Shared Infrastructure, OpenAPI

Reviewed commits: `HEAD~2..HEAD` (Phase 2 AuthScope extractor + migrate tokens/ handlers, Phase 0 fix db import paths)

## Findings

| Priority | File | Location | Issue | Recommendation |
|----------|------|----------|-------|----------------|
| P1 | `crates/routes_app/src/settings/routes_settings.rs` | Lines 215-217 | **Duplicate test module declaration.** `test_settings` is declared both in `routes_settings.rs` (line 216-217) AND in `mod.rs` (line 6-7). Both use `#[path = "test_settings.rs"]`. This compiles because they are in different module scopes, but it means the test file is compiled twice under different module paths, inflating compile time and potentially causing confusion. Convention says Pattern A (from handler file) OR Pattern B (from mod.rs), not both. | Remove the `#[cfg(test)] #[path = "test_settings.rs"] mod test_settings;` block from `routes_settings.rs` (lines 215-217). Keep the declaration in `mod.rs` only, since `mod.rs` already declares it. |
| P1 | `crates/routes_app/src/setup/routes_setup.rs` | Lines 138-140 | **Duplicate test module declaration.** Same issue as settings: `test_setup` is declared in both `routes_setup.rs` (line 139-140) and `mod.rs` (line 6-7). The test file is compiled twice under different module paths. | Remove the `#[cfg(test)] #[path = "test_setup.rs"] mod test_setup;` block from `routes_setup.rs` (lines 138-140). Keep the declaration in `mod.rs` only. |
| P2 | `crates/routes_app/src/shared/openapi.rs` | Line 135 | **Stale comment referencing old module name.** Comment says `// MCP endpoint constants are defined in routes_mcp/mod.rs` but the module was renamed to `mcps/` in this reorganization. | Update comment to `// MCP endpoint constants are defined in mcps/mod.rs`. |
| P2 | `crates/routes_app/src/routes_dev.rs` | `dev_secrets_handler` (line 30) | **Legacy `Extension<AuthContext>` pattern not migrated.** The handler uses `Extension(auth_context): Extension<AuthContext>` plus `State(state): State<Arc<dyn RouterState>>` instead of the new `AuthScope` extractor. The diff touched this file (import path changes) but did not migrate the extractor pattern. This handler does not call `forward_request()`, so it does not qualify for the known exception. | Migrate `dev_secrets_handler` to use `AuthScope` extractor, accessing `auth_context` via `auth_scope.auth_context()`. Also migrate `envs_handler` and `dev_db_reset_handler` which use `State(state)` without `forward_request()`. Can be deferred since these are dev-only endpoints. |
| P3 | `crates/routes_app/src/settings/test_settings.rs` | `app()` fn, lines 32-33 | **Test router uses wrong path prefix.** The test helper registers routes at `/v1/bodhi/settings/{key}` instead of `/bodhi/v1/settings/{key}`. The CRUD tests are self-consistent (request URIs match), so they pass, but the paths differ from the real app routing. The auth tier tests (line 500+) correctly use `/bodhi/v1/settings/...`. | Update the `app()` helper to use `ENDPOINT_SETTINGS` with `/{key}` suffix (i.e., use `&format!("{ENDPOINT_SETTINGS}/{{key}}")`) for consistency with the real route registration in `routes.rs`. Update corresponding test request URIs. Pre-existing issue, low priority. |
| P3 | `crates/routes_app/src/ollama/ollama_api_schemas.rs` | Line 154, `ChatResponse` | **Cast `0 as f64` instead of `0.0f64`.** Line `total_duration: 0 as f64` uses an unnecessary integer-to-float cast. | Use `total_duration: 0.0` instead. Pre-existing, cosmetic. |

## Summary

The shared infrastructure migration is clean overall:

- **ApiError/OpenAIApiError/ErrorBody** are properly defined in `routes_app::shared` and cleanly removed from `services`. No stale `use services::ApiError` imports remain.
- **AuthScope extractor** (`auth_scope_extractor.rs`) correctly implements `FromRequestParts`, falls back to `AuthContext::Anonymous`, and uses `FromRef<S>` for state extraction. The `Deref` impl for ergonomic access is appropriate.
- **Blanket `From<T: AppError>` for `ApiError`** covers all service errors via the single impl in `api_error.rs`.
- **ValidatedJson** correctly chains `JsonRejection -> JsonRejectionError -> ApiError` and `ValidationErrors -> ObjValidationError -> ApiError`.
- **openapi.rs** has all path registrations updated to new handler names (e.g., `settings_index`, `setup_show`, `tokens_create`). Schema registrations are complete.
- **oai/ollama** handlers correctly retain `State<Arc<dyn RouterState>>` for `forward_request()` calls. No `Extension<AuthContext>` patterns in these modules.
- **Old module deletions** are clean: `routes_api_models/`, `routes_api_token/`, `routes_auth/`, `routes_models/`, `routes_settings/`, `routes_setup/`, `routes_toolsets/`, `routes_users/` are all deleted. No leftover references in source files.
- **settings/setup** modules follow the domain module structure correctly (error.rs, routes_*.rs, *_api_schemas.rs, mod.rs) with proper handler naming convention.
- **Error enums** (`SettingsRouteError`, `SetupRouteError`, `OAIRouteError`, `OllamaRouteError`, `ApiModelsRouteError`) all use `#[error_meta(trait_to_impl = AppError)]` correctly.
- **lib.rs** cleanly declares all new domain modules and re-exports them.
- **routes.rs** uses all new handler names consistently throughout the route tree.

The two P1 findings (duplicate test module declarations) should be fixed to maintain the crate's test organization convention and avoid unnecessary duplicate compilation.
