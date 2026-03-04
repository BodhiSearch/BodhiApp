# errmeta_derive -- PACKAGE.md

## Module Structure

- `src/lib.rs` -- Entry point: `derive_error_metadata` proc macro, `impl_error_metadata` dispatch (enum/struct/union), inline unit tests
- `src/parse.rs` -- Attribute parsing: `EnumMetaHeader`, `EnumMetaAttrs`, `StructMetaAttrs` structs with `syn::Parse` impls; `is_transparent()` detection
- `src/generate.rs` -- Code generation: `generate_impl`, `generate_attribute_method`, `generate_args_method`, `empty_enum`; pattern matching for Named/Unnamed/Unit fields

## Attribute Reference

### Enum-level
- `#[error_meta(trait_to_impl = AppError)]` -- generate trait impl instead of inherent methods

### Variant-level
- `#[error_meta(error_type = EXPR)]` -- required for non-transparent variants; any Rust expression
- `#[error_meta(code = EXPR)]` -- optional; defaults to `{enum_snake}-{variant_snake}`
- `#[error_meta(args_delegate = false)]` -- only on transparent variants; generates `{"error": err.to_string()}`

### Struct-level
- `#[error_meta(error_type = EXPR)]` -- required
- `#[error_meta(code = EXPR)]` -- optional; defaults to struct name in snake_case
- `#[error_meta(trait_to_impl = PATH)]` -- optional

## Test Files

- `src/lib.rs` (inline `#[cfg(test)]` module) -- Unit tests for parsing, code generation, transparent detection
- `tests/test_error_metadata.rs` -- rstest parameterized integration tests for all field patterns
- `tests/test_args_as_expr.rs` -- Expression evaluation in error_type/code attributes
- `tests/test_args_delegate.rs` -- args_delegate=false behavior
- `tests/trybuild.rs` -- Compile-fail test runner
- `tests/fails/` -- 4 compile-fail cases: missing_error_type, invalid_error_type, data_type_union, missing_struct_error_meta
- `tests/objs.rs` -- `ErrorMetas` helper struct for integration test assertions

## Generated Code Patterns

### Visibility
- With `trait_to_impl`: methods have no visibility modifier (trait impl)
- Without: methods are `pub fn`

### args() generation by field type
- Named fields: `map.insert(stringify!(field).to_string(), format!("{}", field))`
- Unnamed fields: `map.insert(stringify!(var_N).to_string(), format!("{}", var_N))`
- Unit: empty `HashMap::new()`
- Struct named: `format!("{}", self.field_name)`
- Struct unnamed: `format!("{}", self.N)`

### Transparent variant handling
- Default: delegates `error_type()`, `code()`, `args()` to wrapped `err`
- With `error_meta` overrides: uses specified expressions for error_type/code, still delegates args
- With `args_delegate = false`: generates `{"error": err.to_string()}` map
