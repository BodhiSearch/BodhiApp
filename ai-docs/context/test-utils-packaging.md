# Test Utils Packaging Pattern

This document describes BodhiApp's unique test utilities packaging pattern that enables shared test fixtures across multiple crates while maintaining clean dependency boundaries.

## Architecture Pattern

### Dual Availability Mechanism

Test utilities are made available through two complementary mechanisms:

1. **Internal Testing** (`#[cfg(test)]`): Available during unit tests within the same crate
2. **Cross-Crate Testing** (`feature = "test-utils"`): Available to downstream crates via feature flag

### Implementation in `src/lib.rs`

```rust
#[cfg(feature = "test-utils")]
pub mod test_utils;
#[cfg(all(not(feature = "test-utils"), test))]
pub mod test_utils;
```

This conditional compilation ensures:
- Test utilities are always available during `#[cfg(test)]` compilation
- External crates can access utilities by enabling the `test-utils` feature
- No test code leaks into production builds

### Cargo.toml Configuration

**Provider crate** (`objs/Cargo.toml`):
```toml
[features]
test-utils = [
  "dircpy",
  "dirs", 
  "fs_extra",
  "http-body-util",
  "rstest",
  "tempfile",
  "tracing-subscriber",
]
```

**Consumer crate** (`other-crate/Cargo.toml`):
```toml
[dev-dependencies]
objs = { path = "../objs", features = ["test-utils"] }
```

## Benefits

### Cross-Crate Consistency
- Shared test fixtures ensure consistent testing patterns
- Domain object builders available across the workspace
- Centralized test data generation and management

### Clean Dependency Boundaries
- Test utilities don't pollute production dependencies
- Feature flag provides explicit opt-in for test infrastructure
- No circular dependencies between crates

### Centralized Test Infrastructure
- Single source of truth for complex test setups
- Shared localization testing across multiple crates
- Unified approach to temporary environments and fixtures

## Implementation Guidelines

### When to Use This Pattern
- Complex test fixtures needed across multiple crates
- Domain-specific test data generators
- Cross-crate integration testing requirements
- Specialized test environment setups

### Adding New Test Utilities
1. Add utilities to the `test_utils` module
2. Export via `pub use` in `test_utils/mod.rs`
3. Update feature dependencies if new external crates needed
4. Document usage patterns in crate-specific PACKAGE.md

### Best Practices
- Keep test utilities focused on domain-specific needs
- Use rstest fixtures for parameterized testing
- Provide builder patterns for complex test objects
- Include Python integration for generated test data where beneficial

## Example Usage Pattern

**In provider crate tests**:
```rust
#[cfg(test)]
mod tests {
    use crate::test_utils::*;
    
    #[rstest]
    fn test_with_fixture(temp_bodhi_home: TempDir) {
        // Test implementation
    }
}
```

**In consumer crate tests**:
```rust
#[cfg(test)]
mod tests {
    use objs::test_utils::*;
    
    #[rstest] 
    fn test_cross_crate_fixture(setup_l10n: ()) {
        // Cross-crate localization testing
    }
}
```

This pattern enables sophisticated testing infrastructure while maintaining clean architectural boundaries and explicit dependency management.