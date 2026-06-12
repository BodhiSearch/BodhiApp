# Review: Services Domain Error Types and Service Traits

**Scope**: `crates/services/src/{tokens,mcps,toolsets,models,users}/` changes from HEAD~2..HEAD
**Reviewer**: Claude Code
**Date**: 2026-03-02

## Summary

The changes introduce `TokenServiceError` as a new domain error enum, add `Auth(#[from] AuthContextError)` variants to `McpError` and `ToolsetError`, move `ApiKeyUpdate` from `db/objs.rs` to `models/model_objs.rs`, and migrate `TokenService` trait return types from `DbError` to `TokenServiceError`. Overall the changes are well-structured with correct `From` conversions and consistent transparent delegation patterns.

## Findings

| Priority | File | Location | Issue | Recommendation |
|----------|------|----------|-------|----------------|
| P2 | `crates/services/src/tokens/error.rs` | `TokenServiceError::Db` variant (line 11) | Variant named `Db` while all other domain error enums in the codebase use `DbError` as the variant name (see `McpError::DbError`, `ToolsetError::DbError`, `McpServerError::DbError`, `AccessRequestError::DbError`). This causes an inconsistent error code: `token_service_error-db` vs `mcp_error-db_error`, `toolset_error-db_error`, etc. | Rename to `DbError(#[from] DbError)` for consistency with existing error enums, or document the intentional departure. The shorter `Db` name is arguably better -- but then all enums should be updated together. |
| P2 | `crates/services/src/tokens/token_service.rs` | `DefaultTokenService` impl (lines 44-87) | `DefaultTokenService` methods now return `TokenServiceError` but the implementation only ever produces `DbError` conversions. The `Auth` and `Entity` variants of `TokenServiceError` are never constructed by `DefaultTokenService` -- they are only used by `AuthScopedTokenService`. This means `DefaultTokenService` pays for an error type richer than it needs. | Consider whether `DefaultTokenService` should remain on `DbError` return types (since it has no auth or entity concerns) and only `AuthScopedTokenService` uses `TokenServiceError`. Alternatively, accept this as intentional trait unification -- but document the rationale. |
| P3 | `crates/services/src/tokens/token_service.rs` | `DefaultTokenService` vs `AuthScopedTokenService` | `AuthScopedTokenService` bypasses `TokenService` trait entirely, calling `db_service()` directly for all operations. `DefaultTokenService` wraps the same `db_service()` calls with no additional logic. This makes `DefaultTokenService` effectively dead code in auth-scoped flows -- `AuthScopedTokenService.tokens()` returns `AuthScopedTokenService` which never delegates to `TokenService`. The passthrough accessor `AuthScopedAppService::token_service()` exists but is unused by `AuthScopedTokenService`. | This is a pre-existing design issue, not introduced by these commits, but the `TokenServiceError` migration amplifies it. Consider whether `AuthScopedTokenService` should delegate to `TokenService` trait methods (adding auth on top), or whether `DefaultTokenService` should be removed if truly unused in production flows. |
| P3 | `crates/services/src/models/model_objs.rs` | `ApiKeyUpdate` (lines 925-932) | `ApiKeyUpdate` was moved from `db/objs.rs` to `models/model_objs.rs`. The doc comment says "for API model aliases and toolsets" -- it is used by both `api_alias_repository.rs` and `toolsets/toolset_repository.rs`. Placement in `models/` is reasonable since it originated there, but it is a cross-domain type. | Acceptable placement given that `models/model_objs.rs` already serves as a home for shared model-adjacent types. If more domains start using it, consider moving to `shared_objs/`. No action needed now. |

## Verification Summary

The following were checked and found correct (no findings):

- **TokenServiceError design**: `Auth(#[from] AuthContextError)`, `Entity(#[from] EntityError)`, and `Db(#[from] DbError)` all have correct `From` conversions. `Auth` and `Entity` use transparent delegation (no explicit `error_meta`), which correctly delegates `error_type()`, `code()`, and `args()` to the inner error. `Db` correctly uses `args_delegate = false` since `DbError` has complex args that should not be forwarded.
- **McpError and ToolsetError Auth variants**: Both correctly add `Auth(#[from] AuthContextError)` with `#[error(transparent)]` and no explicit `error_meta`, matching the `TokenServiceError` pattern.
- **Error enum consistency**: All three auth-scoped domain errors (`TokenServiceError`, `McpError`, `ToolsetError`) plus `AuthScopedUserError` have `Auth(#[from] AuthContextError)`. `McpServerError` does not have one, which is correct since it is admin-only and not used in auth-scoped services.
- **No remaining ApiError references**: None of the reviewed service files reference `ApiError`. The only `ApiError` in `models/` is `hf_hub::api::tokio::ApiError` (HuggingFace SDK type), which is unrelated.
- **ApiKeyUpdate move**: Properly removed from `db/objs.rs` (file deleted), added to `models/model_objs.rs`, all import paths updated across `toolsets/`, `models/`, and `test_utils/`.
- **Re-exports**: `tokens/mod.rs` exports `error::*`, `models/mod.rs` exports `model_objs::*`, and `lib.rs` re-exports both modules via glob, making `TokenServiceError` and `ApiKeyUpdate` accessible as `services::TokenServiceError` and `services::ApiKeyUpdate`.
