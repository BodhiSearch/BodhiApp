# PACKAGE.md - errmeta Crate Implementation Index

*For architectural documentation and design rationale, see [crates/errmeta/CLAUDE.md](crates/errmeta/CLAUDE.md)*

## Module Structure

- `src/lib.rs` - Crate root: module declarations, re-exports, `impl_error_from!` macro definition, re-exports `errmeta_derive::ErrorMeta`
- `src/app_error.rs` - `AppError` trait definition with `error_type()`, `status()`, `code()`, `args()` methods; blanket `From<T: AppError>` for `Box<dyn AppError>`
- `src/error_type.rs` - `ErrorType` enum with 10 variants mapping error categories to HTTP status codes; `strum` derives for string serialization
- `src/entity_error.rs` - `EntityError` enum with `NotFound(String)` variant; `ErrorType::NotFound` (404)
- `src/io_error.rs` - `IoError` enum with 6 variants for filesystem operations; convenience constructors `with_path()`, `dir_create()`, `file_read()`, `file_write()`, `file_delete()`
- `src/rwlock_error.rs` - `RwLockReadError` struct with manual `AppError` impl; `ErrorType::InternalServer` (500)

## Test Files

- `src/test_error_type.rs` - ErrorType serialization, status code mapping, default behavior
- `src/test_entity_error.rs` - EntityError AppError trait implementation
- `src/test_io_error.rs` - IoError variant construction, source preservation, path context
- `src/test_rwlock_error.rs` - RwLockReadError AppError trait implementation

## Key Implementation Patterns

### AppError Trait
```rust
pub trait AppError: std::error::Error + Send + Sync + 'static {
  fn error_type(&self) -> String;
  fn status(&self) -> u16 { /* derives from error_type() via ErrorType */ }
  fn code(&self) -> String;
  fn args(&self) -> HashMap<String, String>;
}
```

### impl_error_from! Macro
```rust
// Bridges orphan rule: ExternalError -> IntermediateWrapper -> ServiceError
impl_error_from!(
  std::io::Error,            // source error type
  DataServiceError::Io,      // target enum::variant
  errmeta::IoError            // intermediate wrapper type
);
```

### IoError Convenience Constructors
```rust
// Each constructor captures both the std::io::Error source and the path
let err = IoError::file_read(io_err, "/path/to/file");
let err = IoError::dir_create(io_err, parent.display().to_string());
let err = IoError::file_write(io_err, filename.clone());
let err = IoError::file_delete(io_err, path.to_string());
```

### ErrorType Usage in Error Enums
```rust
#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum MyError {
  #[error("Resource not found: {0}.")]
  #[error_meta(error_type = ErrorType::NotFound)]
  NotFound(String),
}
```

## Crate Commands

### Building
```bash
cargo build -p errmeta
cargo check -p errmeta
```

### Testing
```bash
cargo test -p errmeta
cargo test -p errmeta -- --nocapture
```

### Documentation
```bash
cargo doc -p errmeta --open
```

## Dependencies

- `errmeta_derive` -- proc macro for `#[derive(ErrorMeta)]`
- `strum` -- enum string serialization (`Display`, `AsRefStr`, `EnumString`)
- `thiserror` -- `#[derive(Error)]` for error type definitions

### Dev Dependencies
- `rstest` -- test parameterization
- `pretty_assertions` -- readable assertion diffs

## Feature Flags

None. This crate is intentionally minimal with no optional features.
