# PACKAGE.md - errmeta_derive

This document provides detailed technical information for the errmeta_derive crate, focusing on BodhiApp-specific procedural macro implementation patterns and error metadata generation.

See [crates/errmeta_derive/CLAUDE.md](crates/errmeta_derive/CLAUDE.md) for architectural overview and design decisions.

## Architecture Position

The errmeta_derive crate operates as BodhiApp's foundational macro infrastructure, providing compile-time code generation for structured error handling across all application layers. It integrates with the objs crate error system and enables consistent error metadata extraction throughout services, routes, and CLI components.

Key architectural decisions:
- Zero runtime overhead through compile-time code generation
- Integration with thiserror for seamless error handling patterns
- Support for transparent error wrapping to maintain error chains
- Flexible trait implementation for different error handling contexts

## Procedural Macro Implementation

### Core Derive Macro System
The main implementation generates three essential methods for error metadata extraction:

```rust
// Pattern structure (see crates/errmeta_derive/src/lib.rs for complete derive implementation)
#[proc_macro_derive(ErrorMeta, attributes(error_meta))]
pub fn derive_error_metadata(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);
  let output = impl_error_metadata(&input);
  output.into()
}

// Generated method signatures for all error types
impl ErrorType {
  pub fn error_type(&self) -> String { /* categorized error type */ }
  pub fn code(&self) -> String { /* localization key */ }
  pub fn args(&self) -> HashMap<String, String> { /* structured arguments */ }
}
```

**Implementation Details** (see crates/errmeta_derive/src/lib.rs):
- Handles both enum and struct error types with different generation strategies
- Enum processing via `generate_attribute_method()` for error_type and code methods
- Struct processing with required error_type validation and optional code defaults
- Union type rejection with clear compilation errors

### Attribute Parsing Architecture
Sophisticated attribute processing supports flexible error metadata customization:

```rust
// Attribute parsing patterns (see crates/errmeta_derive/src/lib.rs for complete implementations)
#[derive(Debug, PartialEq)]
struct EnumMetaAttrs {
  error_type: Option<syn::Expr>,
  code: Option<syn::Expr>, 
  args_delegate: Option<bool>,
}

// Expression evaluation examples
#[error_meta(error_type = "ValidationError")]           // String literal
#[error_meta(error_type = ErrorType::Validation)]       // Enum variant
#[error_meta(error_type = get_error_type())]            // Function call
#[error_meta(code = self.generate_code())]              // Method call
```

**Key Implementation Features** (crates/errmeta_derive/src/lib.rs):
- `syn::Expr` parsing enables arbitrary Rust expressions in attributes
- Separate parsing for enum-level, variant-level, and struct-level attributes
- Compile-time validation ensures required attributes are present
- Expression evaluation occurs at runtime while maintaining compile-time safety

### Code Generation Patterns
Pattern matching generation handles all Rust field types with appropriate destructuring:

```rust
// Field pattern generation (see crates/errmeta_derive/src/lib.rs for complete implementation)
match fields {
  Fields::Named(_) => quote! { { .. } },     // Named fields: { field1, field2 }
  Fields::Unnamed(_) => quote! { (..) },     // Tuple fields: (var_0, var_1)
  Fields::Unit => quote! {},                 // Unit variants: no fields
}

// Args method generation for different field types (see crates/errmeta_derive/src/lib.rs)
Fields::Named(named_fields) => {
  // Generates: args["field_name"] = field_name.to_string()
}
Fields::Unnamed(unnamed_fields) => {
  // Generates: args["var_0"] = var_0.to_string()
}
```

**Pattern Generation Details** (crates/errmeta_derive/src/lib.rs):
- Named fields use actual field names as HashMap keys
- Unnamed fields use "var_N" naming convention for tuple access
- Unit variants generate empty HashMaps
- All field values converted to strings via `format!("{}", value)`

## Transparent Error Integration

### Error Delegation System
Transparent error support enables clean error propagation while maintaining metadata:

```rust
// Transparent error patterns (see crates/errmeta_derive/src/lib.rs for detection logic)
fn is_transparent(variant: &Variant) -> bool {
  variant.attrs.iter().any(|attr| {
    // Detects #[error(transparent)] attributes
  })
}

// Delegation generation (see crates/errmeta_derive/src/lib.rs for complete implementation)
#[error(transparent)]
DatabaseError(DatabaseError),  // Delegates all methods to wrapped error

#[error(transparent)]
#[error_meta(args_delegate = false)]
IoError(std::io::Error),       // Custom args() with error string
```

**Delegation Behavior** (crates/errmeta_derive/src/lib.rs):
- `error_type()` and `code()` automatically delegate to wrapped error
- `args()` delegation configurable via `args_delegate` attribute
- `args_delegate = false` generates `{"error": err.to_string()}` instead
- Transparent variants can override error_type and code while maintaining args delegation

### Mixed Error Handling
Support for mixed transparent and non-transparent variants in the same enum:

```rust
// Mixed variant patterns (see crates/errmeta_derive/tests/test_error_metadata.rs for examples)
enum ServiceError {
  #[error_meta(error_type = "ValidationError", code = "invalid_input")]
  ValidationFailed { field: String, value: String },
  
  #[error(transparent)]
  DatabaseError(#[from] DatabaseError),
  
  #[error(transparent)]
  #[error_meta(args_delegate = false)]
  ExternalError(#[from] ExternalError),
}
```

## Testing Infrastructure

### Compile-Time Validation Testing
Comprehensive compile-time error validation using trybuild:

```rust
// Compile-time test pattern (see crates/errmeta_derive/tests/trybuild.rs for complete setup)
#[test]
fn compile_fail() {
  let t = trybuild::TestCases::new();
  t.compile_fail("tests/fails/*.rs");
}
```

**Validation Test Cases** (see crates/errmeta_derive/tests/fails/ directory):
- `missing_error_type.rs` - Ensures error_type required for enum variants
- `invalid_error_type.rs` - Validates expression syntax in attributes  
- `data_type_union.rs` - Confirms union type rejection with clear errors

### Runtime Behavior Testing
Parameterized testing with rstest for generated code validation:

```rust
// Runtime test patterns (see crates/errmeta_derive/tests/test_error_metadata.rs for complete examples)
#[rstest]
#[case::with_fields(
  TestError::WithFields { field1: "value1".to_string(), field2: 200 },
  ErrorMetas {
    code: "test_error_code".to_string(),
    error_type: "test_error_type".to_string(),
    args: HashMap::from([
      ("field1".to_string(), "value1".to_string()),
      ("field2".to_string(), "200".to_string())
    ]),
  }
)]
fn test_error_metadata(#[case] error: TestError, #[case] expected: ErrorMetas) {
  let error_metas = ErrorMetas::from(&error);
  assert_eq!(expected, error_metas);
}
```

**Test Coverage Areas** (crates/errmeta_derive/tests/test_error_metadata.rs):
- All field patterns (named, unnamed, unit) for enums and structs
- Transparent error delegation with and without args_delegate
- Integration with thiserror and strum for real-world patterns  
- Automatic snake_case code generation validation
- Expression evaluation in error_type and code attributes

### Integration Testing Patterns
Mock ErrorMetas struct mirrors objs crate interface for integration validation:

```rust
// Integration test structure (see crates/errmeta_derive/tests/objs.rs for complete definition)
#[derive(Debug, PartialEq)]
pub struct ErrorMetas {
  pub message: String,
  pub code: String,
  pub error_type: String,
  pub args: HashMap<String, String>,
}

// Conversion pattern for integration testing (crates/errmeta_derive/tests/test_error_metadata.rs)
impl From<&TestError> for ErrorMetas {
  fn from(error: &TestError) -> Self {
    Self {
      message: error.to_string(),
      code: error.code(),
      error_type: error.error_type(),
      args: error.args(),
    }
  }
}
```

## Cross-Crate Integration

### Integration with objs Crate
The errmeta_derive macro provides the foundation for objs crate error handling patterns used throughout BodhiApp services.

**Integration Features**:
- `trait_to_impl` parameter enables AppError trait implementation (crates/errmeta_derive/src/lib.rs:74-77)
- Generated methods provide structured data for HTTP response generation
- Error codes serve as localization keys for multi-language support
- Args extraction enables message templating with user data

### Service Layer Coordination
Service-specific error types derive ErrorMeta for consistent error reporting:

**Service Integration Benefits**:
- Consistent error metadata across all service boundaries
- Automatic error propagation with preserved context
- Integration with tracing and logging systems
- Localized error messages for user-facing components

## Unit Test Coverage

### Core Macro Functionality Tests
Comprehensive unit tests validate the macro's internal workings:

```rust
// Pattern testing (see crates/errmeta_derive/src/lib.rs for is_transparent tests)
#[rstest]
#[case(
  parse_quote!(#[error(transparent)] TransparentError),
  true
)]
fn test_is_transparent(#[case] variant: Variant, #[case] expected: bool) {
  assert_eq!(expected, is_transparent(&variant));
}

// Attribute parsing tests (crates/errmeta_derive/src/lib.rs)
#[case::all_provided(
  parse_quote!(#[error_meta(error_type = "TestError", code = "test_code")]),
  Some(EnumMetaAttrs { /* ... */ }),
)]
fn test_parse_error_meta(#[case] attr: Attribute, #[case] expected: Option<EnumMetaAttrs>) {
  let error_meta = parse_enum_meta_attrs(&[attr]);
  assert_eq!(expected, error_meta);
}
```

### Code Generation Validation
Tests ensure generated code matches expected patterns:

```rust
// Method generation tests (crates/errmeta_derive/src/lib.rs)
#[case("error_type", quote! {
  match self {
    TestEnum::Variant1 => <_ as AsRef<str>>::as_ref(&internal_server_error()).to_string(),
    TestEnum::Variant2 => <_ as AsRef<str>>::as_ref(&"Error2").to_string(),
    TestEnum::Variant3(err) => err.error_type(),
  }
})]
fn test_generate_attribute_method_for_enum(#[case] method: &str, #[case] expected: TokenStream2) {
  // Validates generated match statements for enum variants
}
```

## Extension Guidelines

### Adding New Attribute Support
When extending the macro with new attributes (following patterns in crates/errmeta_derive/src/lib.rs):

1. **Extend parsing structures** - Add new fields to EnumMetaAttrs/StructMetaAttrs
2. **Update Parse implementations** - Handle new attribute syntax in parsing logic
3. **Modify code generation** - Incorporate new attributes into generated code
4. **Add comprehensive tests** - Include both positive and negative test cases
5. **Update documentation** - Document new attribute behavior and constraints

### Macro Development Best Practices
For macro development and debugging:

```bash
# Debug generated code with cargo expand
cargo expand --bin your_binary

# Test macro compilation errors
cargo test --test trybuild -p errmeta_derive

# Validate integration with existing error libraries
cargo test --test test_error_metadata -p errmeta_derive
```

**Development Guidelines**:
- Use `cargo expand` to debug generated code structure
- Test edge cases with different field patterns and attribute combinations
- Validate integration with thiserror, strum, and other common error libraries
- Ensure generated code follows Rust naming conventions and best practices
- Test both compilation success and failure scenarios comprehensively

### Error Handling Pattern Extensions
When adding new error handling patterns:

1. **Define clear attribute syntax** - Ensure intuitive and consistent attribute design
2. **Implement compile-time validation** - Catch invalid usage during macro expansion
3. **Generate idiomatic Rust code** - Follow established Rust patterns and conventions
4. **Test with real-world scenarios** - Validate against actual service error patterns
5. **Document integration requirements** - Explain how new patterns work with existing systems

## Commands

**Testing**: `cargo test -p errmeta_derive` (includes unit, integration, and compile-time tests)
**Testing with trybuild**: `cargo test --test trybuild -p errmeta_derive` (compile-time error validation)
**Building**: `cargo build -p errmeta_derive`
**Expanding macros**: `cargo expand --test test_error_metadata -p errmeta_derive` (debug generated code)
**Integration testing**: `cargo test --test test_error_metadata -p errmeta_derive` (runtime behavior validation)