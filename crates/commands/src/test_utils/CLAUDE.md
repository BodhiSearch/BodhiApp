# CLAUDE.md

This file provides guidance to Claude Code when working with the `test_utils` module for CLI commands.

*For implementation details and extension patterns, see [crates/commands/src/test_utils/PACKAGE.md](crates/commands/src/test_utils/PACKAGE.md)*

## Purpose

The `test_utils` module provides minimal testing utilities for BodhiApp's CLI commands crate, currently focused on command builder patterns for consistent test data creation. This module serves as a foundation for CLI command testing and provides extension points for more comprehensive testing infrastructure.

## Current Implementation Architecture

### Command Builder Testing Foundation
Minimal command construction utilities for testing CLI workflows:
- **CreateCommand::testalias()**: Pre-configured command instance using testalias repository for consistent testing
- **CreateCommandBuilder::testalias()**: Builder pattern with sensible defaults for test scenarios
- **Domain Object Integration**: Uses objs crate test utilities (`Repo::testalias()`, `Repo::testalias_model_q8()`)
- **Parameter Defaults**: Provides default OAIRequestParams and empty context_params for baseline testing
- **Service Integration Ready**: Designed to work with services crate test utilities for mocking

### Testing Infrastructure Design
Current minimal infrastructure with clear extension points:
- **Builder Pattern Focus**: Emphasizes consistent test command creation over complex mocking
- **Cross-Crate Coordination**: Leverages existing test utilities from objs and services crates
- **Domain Object Consistency**: Uses standardized test data objects across command testing
- **Service Mock Compatibility**: Compatible with AppServiceStubBuilder and TestHfService from services crate

## Cross-Crate Testing Integration

### Objs Domain Object Integration
Commands test utilities coordinate with domain objects:
- **Standardized Test Data**: Uses `Repo::testalias()` and related test factories from objs crate
- **Domain Builder Coordination**: Command builders use objs builders for consistency
- **Parameter Object Integration**: Default OAIRequestParams integration with command testing
- **Error System Compatibility**: Command errors designed for objs error system integration

### Services Layer Compatibility
CLI command test utilities work with services crate testing:
- **Service Mock Integration**: Compatible with AppServiceStubBuilder from services crate
- **Hub Service Testing**: Works with TestHfService for model download testing
- **Data Service Coordination**: Compatible with data service mocks for alias persistence testing
- **Multi-Service Workflows**: Test utilities support complex service interaction testing

## Architecture Position

The `test_utils` module currently serves as:
- **Minimal CLI Testing Foundation**: Provides basic command builder utilities for test data creation
- **Cross-Crate Integration Point**: Coordinates with objs and services test utilities
- **Extension Foundation**: Provides structure for future CLI testing infrastructure expansion
- **Domain Object Bridge**: Connects CLI command testing with domain object test factories

## Extension Architecture

### Current Extension Points
Areas designed for future testing infrastructure expansion:
- **Command Builder Expansion**: Additional command types can follow the CreateCommand::testalias() pattern
- **Service Mock Integration**: Ready for integration with more sophisticated service mocking
- **Output Testing Infrastructure**: Foundation for CLI-specific output validation
- **Error Testing Patterns**: Compatible with comprehensive CLI error message testing

### Future Testing Infrastructure Patterns
Architecture supports future expansion into:
- **Multi-Service Mock Coordination**: Service orchestration testing for complex CLI workflows
- **CLI User Experience Testing**: Output format validation, error message quality, progress feedback
- **Integration Testing Hub**: Comprehensive testing across domain objects, services, and CLI interfaces
- **Cross-Command Testing Coordination**: Consistent testing patterns across different command types

## Implementation Constraints

### Current Implementation Scope
The test_utils module has specific architectural constraints:
- **Minimal Implementation**: Currently provides only essential command builder utilities
- **Cross-Crate Dependencies**: Relies on objs and services crates for comprehensive testing infrastructure
- **Test Data Focus**: Emphasizes consistent test data creation over complex testing scenarios
- **Extension Ready**: Designed for future expansion without breaking existing patterns

### Domain Object Integration Requirements
Command test utilities must coordinate with objs testing infrastructure:
- **Standardized Test Data**: Must use objs test factories (`Repo::testalias()`, etc.) for consistency
- **Builder Pattern Compatibility**: Command builders must integrate with objs domain builders
- **Error System Coordination**: Command errors must work with objs error system via thiserror templates
- **Parameter Object Integration**: OAIRequestParams and other domain objects must be consistent

### Cross-Crate Testing Coordination
CLI command testing coordinates across multiple crates:
- **Services Integration**: Command test utilities work with services crate AppServiceStubBuilder and TestHfService
- **Test Infrastructure Reuse**: Leverages existing test utilities rather than duplicating functionality
- **Consistent Patterns**: Maintains consistency with testing patterns across the workspace
- **Future Expansion**: Ready for more sophisticated testing infrastructure as commands crate grows

### Testing Implementation Guidelines
When extending command test utilities:
- **Follow Builder Pattern**: New command test utilities should follow CreateCommand::testalias() pattern
- **Use Domain Factories**: Always use objs test factories for consistent test data
- **Coordinate with Services**: Integrate with services crate test utilities for service mocking
- **Maintain Simplicity**: Keep test utilities focused and avoid complex testing logic in builders

## Usage Examples

### Basic Command Testing
Using test_utils for consistent command testing:
```rust
#[cfg(test)]
mod tests {
    use crate::CreateCommand;
    use services::test_utils::AppServiceStubBuilder;
    
    #[tokio::test]
    async fn test_create_command_execution() {
        // Use test utility for consistent command creation
        let command = CreateCommand::testalias();
        
        // Integrate with services test utilities for mocking
        let service = Arc::new(
            AppServiceStubBuilder::default()
                .with_hub_service()
                .with_data_service()
                .await
                .build()?
        );
        
        let result = command.execute(service).await;
        assert!(result.is_ok());
    }
}
```

### Extension Pattern
Adding new command test utilities:
```rust
// Follow the established pattern for new commands
impl PullCommand {
    pub fn testalias() -> PullCommand {
        PullCommand::ByAlias {
            alias: "testalias:instruct".to_string(),
        }
    }
}
```

## Important Design Decisions

### Chat Template Evolution
The commands crate test utilities reflect the architectural evolution of chat template handling:
- **Legacy Removed**: Chat template parameters removed from command builders since llama.cpp now handles templates
- **Simplified Testing**: Test builders focus on core command functionality without deprecated template parameters
- **Future Compatibility**: Architecture supports future template-related testing if requirements change

### Testing Philosophy
Command test utilities follow specific testing philosophy:
- **Minimal Viable Testing**: Provide essential utilities without over-engineering
- **Cross-Crate Coordination**: Leverage existing test infrastructure rather than duplicating
- **Extension Ready**: Design for future growth while maintaining current simplicity
- **Domain Consistency**: Ensure test data consistency across the entire workspace

This minimal approach allows the commands crate to focus on core CLI functionality while providing solid testing foundations that can grow with the crate's complexity.