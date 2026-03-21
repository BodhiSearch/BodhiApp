# errmeta -- PACKAGE.md

## Module Structure

- `src/lib.rs` -- Module declarations, re-exports, `impl_error_from!` macro, re-exports `errmeta_derive::ErrorMeta`
- `src/app_error.rs` -- `AppError` trait definition; blanket `From<T: AppError>` for `Box<dyn AppError>`; `Error` impl for `Box<dyn AppError>`
- `src/error_type.rs` -- `ErrorType` enum (10 variants) with strum derives, `status()` method mapping to HTTP codes
- `src/entity_error.rs` -- `EntityError::NotFound(String)`, maps to 404
- `src/io_error.rs` -- `IoError` enum (6 variants: Io, WithPath, DirCreate, FileRead, FileWrite, FileDelete), convenience constructors
- `src/rwlock_error.rs` -- `RwLockReadError` struct with manual `AppError` impl, maps to 500

## Test Files

- `src/test_error_type.rs` -- ErrorType serialization, status code mapping, default behavior
- `src/test_entity_error.rs` -- EntityError AppError trait impl
- `src/test_io_error.rs` -- IoError construction, source preservation, path context
- `src/test_rwlock_error.rs` -- RwLockReadError AppError impl
- `src/test_impl_error_from.rs` -- impl_error_from! macro end-to-end test

## ErrorType Variant Table

| Variant | Serialized Name | HTTP Status |
|---|---|---|
| `BadRequest` | `invalid_request_error` | 400 |
| `Authentication` | `authentication_error` | 401 |
| `Forbidden` | `forbidden_error` | 403 |
| `NotFound` | `not_found_error` | 404 |
| `Conflict` | `conflict_error` | 409 |
| `UnprocessableEntity` | `unprocessable_entity_error` | 422 |
| `InternalServer` | `internal_server_error` | 500 |
| `InvalidAppState` | `invalid_app_state` | 500 |
| `Unknown` (default) | `unknown_error` | 500 |
| `ServiceUnavailable` | `service_unavailable` | 503 |

