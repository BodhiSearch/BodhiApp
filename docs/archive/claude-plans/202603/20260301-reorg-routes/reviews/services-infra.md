# Services Crate Infrastructure Layer Review

**Scope**: `HEAD~2..HEAD` changes to `crates/services/src/{shared_objs,db,lib.rs,test_utils}`

## Findings

| Priority | File | Location | Issue | Recommendation |
|---|---|---|---|---|
| P2 | `crates/services/src/lib.rs` | Line 75 (comment) | Stale comment reads `// These are defined in shared_objs (axum/serde-dependent error types)` but `JsonRejectionError` (the axum-dependent type) has been removed. The remaining re-exports (`ObjValidationError`, `ReqwestError`, `SerdeJsonError`, `SerdeYamlError`) depend on `validator`, `reqwest`, `serde_json`, `serde_yaml` -- not axum. | Update comment to `// These are defined in shared_objs (serde/validator-dependent error types)` or similar. |
| P2 | `crates/services/CLAUDE.md` | Lines 20, 89, 115 | `JsonRejectionError` is still listed in three places as living in `shared_objs/error_wrappers.rs`. It was removed from services and moved to `routes_app::shared`. | Remove `JsonRejectionError` from these CLAUDE.md references. |
| P2 | `crates/services/PACKAGE.md` | Lines 17, 349 | `JsonRejectionError` still listed in `error_wrappers.rs` contents (line 17) and in the axum dependency justification (line 349: `axum - HTTP framework integration (for ApiError, JsonRejectionError)`). Both `ApiError` and `JsonRejectionError` have moved out. | Update PACKAGE.md: remove `JsonRejectionError` from error_wrappers listing; update axum dependency justification to reference `ai_apis` module (the actual remaining axum consumer). |
| P3 | `crates/services/src/db/error.rs` | `DbError::StrumParse` (line 40-42) | Changed from `#[error(transparent)]` to `#[error("{0}")]`. These produce different Display output: `transparent` delegates to inner error's Display, `{0}` also delegates but via format args. Functionally equivalent here but the change also removed the explicit `code="db_error-strum_parse"` and `args_delegate = false` attributes. The auto-generated code will be `db_error-strum_parse` anyway (matching the removed explicit code), so this is fine. No issue -- noting for completeness only. | No action needed. |
| P3 | `crates/services/src/test_utils/auth_context.rs` | `test_session`, `test_api_token`, `test_external_app` (all methods) | All factory methods hardcode `client_id: DEFAULT_CLIENT_ID` ("test-client-id"). There is no factory method for `AuthContext::Anonymous` with or without a client_id. If tests need Anonymous context with client_id, they must construct it manually. This is minor since Anonymous is simple to construct. | Consider adding `test_anonymous()` and `test_anonymous_with_client_id(client_id)` for completeness, but not blocking. |

## Summary

The `ApiError`/`OpenAIApiError`/`ErrorBody`/`JsonRejectionError` removal from services is clean at the code level -- no orphaned Rust references remain. The `db/objs.rs` deletion is also clean; `ApiKeyUpdate` was moved to `models/model_objs.rs` and all references updated. The `pub use db::*` re-export in `lib.rs` is correct. The `test_utils/auth_context.rs` factory methods properly include the new `client_id` field on all `AuthContext` variants.

The remaining issues are documentation staleness (P2) -- CLAUDE.md, PACKAGE.md, and a code comment still reference `JsonRejectionError` as living in services. The `axum` dependency itself is still needed by `ai_apis/ai_api_service.rs` and `test_utils/http.rs`, so it cannot be removed from `Cargo.toml` yet.
