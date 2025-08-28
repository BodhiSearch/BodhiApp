# CLAUDE.md

This file provides guidance to Claude Code when working with the `test_utils` module for CLI commands.

*For implementation details and extension patterns, see [crates/commands/src/test_utils/PACKAGE.md](crates/commands/src/test_utils/PACKAGE.md)*

## Purpose

The `test_utils` module provides specialized testing infrastructure for BodhiApp's CLI command orchestration layer, enabling comprehensive testing of multi-service workflows, CLI-specific user experience patterns, and command output validation.

## CLI Command Testing Architecture

### Command Builder Testing Foundation
Focused command construction testing for CLI workflows:
- **CreateCommand::testalias()**: Pre-configured command using testalias repository for consistent testing
- **CreateCommandBuilder::testalias()**: Builder pattern with CLI-optimized defaults (auto_download=true, update=false)
- **Parameter Integration**: Default OAIRequestParams and empty context_params for baseline testing
- **Service Mock Integration**: Command builders designed for service mocking with AppService registry
- **Minimal Test Infrastructure**: Focused on essential command construction patterns

### Multi-Service Mock Coordination
Service orchestration testing patterns for CLI commands:
- **AppService Mock Composition**: Central service registry mocking using AppServiceStubBuilder for command execution
- **HubService + DataService Coordination**: Multi-service workflow testing with TestHfService and data service stubs
- **Error Propagation Testing**: Service error coordination through CLI command error translation with transparent wrapping
- **Progress Feedback Integration**: Optional progress reporting testing with services::Progress parameter
- **Simplified Testing Infrastructure**: Focus on core command execution patterns with service mocking

### CLI User Experience Testing
Comprehensive user interface testing for command-line operations:
- **Output Format Validation**: Pretty table formatting and JSON output testing
- **Error Message Quality**: CLI-specific error message validation with actionable guidance
- **Progress Feedback Testing**: Multi-stage operation progress validation
- **Interactive Confirmation**: User prompt and confirmation testing for destructive operations
- **Terminal Output Optimization**: Column layout and human-readable value testing

## Cross-Crate Testing Integration

### Objs Domain Object Testing
Commands extensively test domain object integration:
- **Alias Builder Coordination**: Command testing uses objs alias builders for consistency
- **Repo and HubFile Integration**: Model management testing with domain object validation
- **Error System Integration**: Command errors coordinate with objs error system for localization
- **Parameter Object Testing**: OpenAI compatibility parameter validation across command and domain boundaries

### Services Layer Mock Coordination  
CLI commands require sophisticated service mocking:
- **Service Registry Mocking**: `AppService` trait mocking for comprehensive command testing
- **Cross-Service Transaction Testing**: Multi-service operation testing with rollback validation
- **Authentication Service Integration**: Credential management testing for CLI-specific flows
- **Database Service Coordination**: Alias persistence testing with transaction boundaries

### CLI-Specific Testing Patterns
Command testing focuses on CLI domain concerns:
- **Terminal Output Validation**: Pretty table formatting and human-readable value testing
- **Command Line Integration**: CLI argument parsing and command execution coordination
- **Batch Operation Testing**: Multiple command execution with consistent output formatting
- **Automation Support Testing**: JSON output and quiet mode validation for scripting scenarios

## Architecture Position

The `test_utils` module serves as:
- **CLI Testing Foundation**: Provides specialized infrastructure for command-line interface testing
- **Service Mock Orchestration**: Enables complex multi-service workflow testing for CLI operations
- **User Experience Validation**: Tests CLI-specific concerns like output formatting and error messaging
- **Integration Testing Hub**: Coordinates testing across domain objects, services, and CLI interfaces

## Testing Infrastructure Patterns

### Command Mock Composition
Testing patterns for multi-service command workflows:
- **Service Dependency Injection**: Mock services provided through `AppService` registry pattern
- **Error Scenario Orchestration**: Coordinate service failure scenarios for CLI error testing
- **Progress Feedback Validation**: Test multi-stage operation feedback with service coordination
- **Output Format Testing**: Validate CLI-specific formatting across different command types

### CLI Integration Testing Requirements
Command testing must validate CLI-specific functionality:
- **Terminal Output Standards**: Consistent formatting and layout across all command outputs  
- **Error Message Quality**: CLI error messages must be actionable with specific resolution guidance
- **User Experience Patterns**: Progress feedback and interactive confirmation testing
- **Automation Compatibility**: JSON output and quiet modes for scripting integration

### Cross-Command Testing Coordination
Testing patterns that span multiple CLI commands:
- **Consistent Builder Patterns**: All commands provide test builders with similar patterns
- **Service Mock Reuse**: Common service mocking patterns across different command tests
- **Error Handling Consistency**: Similar error translation and formatting across all commands
- **Output Format Standards**: Consistent table layouts and formatting across command types

## Important Constraints

### Service Mock Requirements
CLI command testing requires comprehensive service mocking:
- **AppService Registry**: All commands must be tested with mocked `AppService` registry
- **Multi-Service Coordination**: Tests must mock complex service interactions realistically
- **Error Propagation**: Service error mocking must test CLI error translation properly
- **Authentication Integration**: Credential management mocking for gated repository testing

### CLI Testing Standards
Command testing must validate CLI-specific concerns:
- **Output Format Validation**: All CLI output must be tested for formatting consistency
- **Error Message Quality**: CLI error messages must be tested for actionability and clarity
- **Progress Feedback**: Long-running operations must test progress feedback properly
- **Terminal Compatibility**: Output formatting must be tested for terminal display optimization

### Domain Object Integration Requirements
Command testing must coordinate with objs testing infrastructure:
- **Domain Builder Integration**: Command tests must use objs domain builders for consistency
- **Validation Testing**: Command validation must coordinate with domain object validation
- **Error System Integration**: Command errors must integrate properly with objs error system
- **Parameter Testing**: OpenAI compatibility parameters must be tested across command boundaries

### Testing Coordination Requirements
CLI command testing must coordinate across multiple testing systems:
- **Cross-Crate Test Infrastructure**: Commands must coordinate with objs and services test utilities
- **Service Mock Composition**: Command tests must compose multiple service mocks realistically
- **Integration Test Patterns**: Command tests must validate end-to-end CLI workflows properly
- **Output Validation Standards**: CLI output testing must maintain consistency across all command types