# errmeta_derive -- CLAUDE.md
**Companion docs** (load as needed):
- `PACKAGE.md` -- Implementation details, file index, attribute reference

## Purpose

Proc macro crate providing `#[derive(ErrorMeta)]` that generates `error_type()`, `code()`,
and `args()` methods for error enums and structs. Used by every error type in BodhiApp to
implement the `AppError` trait from `errmeta`.

## Architecture Position

- **No runtime dependencies** (proc-macro crate: `syn`, `quote`, `proc_macro2`, `convert_case`)
- **Consumed by**: `errmeta` (re-exports `ErrorMeta`), and transitively by all crates

## Non-Obvious Rules

### Error code auto-generation
When `code` is not specified in `#[error_meta(...)]`, it is auto-generated as
`{enum_name_snake_case}-{variant_name_snake_case}`.
Example: `MyServiceError::ConnectionTimeout` -> `"my_service_error-connection_timeout"`.
**Renaming an enum or variant changes the error code** -- update test assertions.

### args() calls format!("{}", field) on all named fields
All fields in the variant are passed through `format!("{}", field)`, so every field must
implement `Display`. This means `Option<String>` fields will NOT compile -- use a manual
`AppError` impl instead. See `src/generate.rs:180-188` for the code generation.

### Transparent variant delegation
Variants with `#[error(transparent)]` automatically delegate `error_type()`, `code()`, and
`args()` to the wrapped error. This requires the wrapped type to have these methods.

Override behavior:
- `#[error_meta(error_type = ..., code = ...)]` on transparent variants overrides those methods
  while still delegating `args()`
- `#[error_meta(args_delegate = false)]` generates `{"error": err.to_string()}` instead of
  delegating `args()` -- use this for external error types that lack an `args()` method

### trait_to_impl attribute
- Enums: `#[error_meta(trait_to_impl = AppError)]` at enum level generates `impl AppError for X`
- Structs: `#[error_meta(trait_to_impl = ..., error_type = ...)]` at struct level
- Without `trait_to_impl`: generates inherent `pub fn` methods instead of trait impl
- Structs require `error_type` attribute (no default); `code` defaults to struct name in snake_case

### Compile-time validation
- Non-transparent enum variants MUST have `error_type` specified (compile error otherwise)
- Unions are rejected with a panic
- Invalid attribute syntax causes compile-time errors via syn parsing

## Testing

- **Unit tests**: inline in `src/lib.rs` (tests for parsing, code generation, transparent detection)
- **Integration tests**: `tests/test_error_metadata.rs` (rstest parameterized, uses `tests/objs.rs` helper struct)
- **Compile-fail tests**: `tests/trybuild.rs` loads `tests/fails/*.rs`:
  - `missing_error_type.rs`, `invalid_error_type.rs`, `data_type_union.rs`, `missing_struct_error_meta.rs`
- **Expression tests**: `tests/test_args_as_expr.rs`, `tests/test_args_delegate.rs`

Run: `cargo test -p errmeta_derive`
Debug generated code: `cargo expand --test test_error_metadata -p errmeta_derive`
