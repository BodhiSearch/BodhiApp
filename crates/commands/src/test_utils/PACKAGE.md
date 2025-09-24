# PACKAGE.md - commands/test_utils

This document provides implementation details for the `commands/test_utils` module, focusing on the actual minimal testing infrastructure for BodhiApp's CLI commands.

*See [crates/commands/src/test_utils/CLAUDE.md](crates/commands/src/test_utils/CLAUDE.md) for architectural guidance and extension patterns.*

## Current Implementation Structure

### Module Organization
The test_utils module currently contains minimal infrastructure:

**Files:**
- `crates/commands/src/test_utils/mod.rs:1` - Module declaration for create submodule
- `crates/commands/src/test_utils/create.rs:1-23` - CreateCommand test builder implementations

### Command Builder Implementation

**CreateCommand Test Utilities** (`crates/commands/src/test_utils/create.rs:4-23`):
```rust
impl CreateCommand {
  pub fn testalias() -> CreateCommand {
    CreateCommandBuilder::testalias().build().unwrap()
  }
}

impl CreateCommandBuilder {
  pub fn testalias() -> CreateCommandBuilder {
    CreateCommandBuilder::default()
      .alias("testalias:instruct".to_string())
      .repo(Repo::testalias())
      .filename(Repo::testalias_model_q8())
      .snapshot(None)
      .oai_request_params(OAIRequestParams::default())
      .context_params(Vec::<String>::default())
      .to_owned()
  }
}
```

## Implementation Features

### Minimal Test Infrastructure
Current implementation provides essential command testing utilities:

**Command Factories:**
- `CreateCommand::testalias()` - Pre-configured command using testalias repository
- Uses objs crate test factories (`Repo::testalias()`, `Repo::testalias_model_q8()`)
- Default OAIRequestParams and empty context_params for baseline testing
- Compatible with services crate AppServiceStubBuilder for mocking

**Domain Object Integration:**
- Leverages `Repo::testalias()` from objs crate for consistent test data
- Uses `Repo::testalias_model_q8()` for model filename consistency
- Default parameter objects for reproducible testing scenarios

## Usage in Tests

### Integration with Services Testing
Test utilities work with services crate infrastructure:

**Example from** `crates/commands/src/cmd_create.rs:201-204`:
```rust
let create = CreateCommandBuilder::testalias()
  .snapshot(snapshot.clone())
  .build()
  .unwrap();
```

**Service Mock Coordination** (`crates/commands/src/cmd_create.rs:122`):
```rust
use services::{
  test_utils::{test_hf_service, AppServiceStubBuilder, TestHfService},
};
```

### Command Execution Testing Pattern
Standard pattern used across command tests:

```rust
// From cmd_create.rs tests - typical usage pattern
let command = CreateCommand::testalias();
let service = Arc::new(
  AppServiceStubBuilder::default()
    .with_hub_service()
    .with_data_service()
    .await
    .build()?
);
let result = command.execute(service).await;
```

## Cross-Crate Dependencies

### Objs Crate Integration
Test utilities depend on objs crate test factories:

- `Repo::testalias()` - Provides consistent repository object for testing
- `Repo::testalias_model_q8()` - Provides consistent model filename
- `OAIRequestParams::default()` - Default API parameters for testing

### Services Crate Compatibility
Designed to work with services crate test infrastructure:

- Compatible with `AppServiceStubBuilder` for service mocking
- Works with `TestHfService` for hub service testing
- Integrates with data service mocks for persistence testing

## Extension Points

### Adding New Command Test Utilities
To add test utilities for new commands, follow the established pattern:

```rust
// Example extension for PullCommand
impl PullCommand {
  pub fn testalias() -> PullCommand {
    PullCommand::ByAlias {
      alias: "testalias:instruct".to_string(),
    }
  }
}
```

### Builder Pattern Extensions
New command builders should follow the CreateCommandBuilder::testalias() pattern:

1. Use objs test factories for consistent test data
2. Provide sensible defaults for all required parameters
3. Integrate with existing domain object test utilities
4. Maintain compatibility with services mock infrastructure

## Testing Infrastructure Context

### Current Scope
The test_utils module is intentionally minimal:

- **Single Command Support**: Only CreateCommand utilities currently implemented
- **Essential Functionality**: Focuses on basic command construction for testing
- **Cross-Crate Integration**: Leverages existing test utilities from objs and services
- **Extension Ready**: Architecture supports future command types

### Services Integration Pattern
Command test utilities integrate with broader testing ecosystem:

- **Service Mocking**: Uses services crate AppServiceStubBuilder
- **Hub Service Testing**: Compatible with TestHfService for download testing
- **Data Persistence**: Works with data service mocks for alias testing
- **Error Testing**: Compatible with service error mocking patterns

## Commands for Testing

**Command Tests**: `cargo test -p commands` - Run all command tests including test utilities  
**With Test Utils Feature**: `cargo test -p commands --features test-utils` - Enable test utilities feature  
**Individual Test**: `cargo test -p commands test_cmd_create_downloads_model_saves_alias` - Run specific test using utilities

## Architecture Evolution

### Chat Template Removal
Test utilities reflect the evolution of chat template handling:

- Previous versions included chat template parameters
- Current implementation removes these since llama.cpp handles templates
- Comments in code indicate this architectural change
- Future extensions can add template testing if requirements change

### Minimal Implementation Philosophy
The current implementation follows specific design principles:

- **Essential Only**: Provides only necessary testing utilities
- **Reuse Infrastructure**: Leverages existing test utilities from other crates  
- **Extension Ready**: Designed for growth without breaking changes
- **Domain Consistency**: Maintains test data consistency across workspace

This minimal approach ensures the commands crate can focus on core CLI functionality while providing solid testing foundations.