# CLAUDE.md

This file provides guidance to Claude Code when working with the `commands` crate.

*For detailed implementation examples and technical depth, see [crates/commands/PACKAGE.md](crates/commands/PACKAGE.md)*

## Purpose

The `commands` crate serves as BodhiApp's **CLI command orchestration layer**, coordinating multiple services to implement sophisticated user workflows with comprehensive error handling and CLI-optimized user experience.

## Key Domain Architecture

### CLI Command Orchestration System
Multi-service coordination for complex CLI operations:
- **CreateCommand**: Orchestrates alias creation with automatic model downloading and validation
- **PullCommand**: Coordinates model discovery and download with conflict resolution  
- **Service Registry Integration**: Uses `AppService` trait to coordinate `HubService`, `DataService`, and other services
- **Workflow Management**: Implements multi-step operations with rollback capabilities and progress feedback
- **Error Orchestration**: Translates service errors into actionable CLI messages with context

### Cross-Service Coordination Architecture
Complex service interactions orchestrated through CLI commands:
- **HubService ↔ DataService**: Model download coordination with alias creation and file system management
- **DataService ↔ SettingService**: Configuration validation and storage path coordination
- **AuthService Integration**: Credential management for gated repository access
- **Error Service Coordination**: Cross-service error propagation with CLI-specific error translation
- **Transaction Boundaries**: Multi-service operation coordination with rollback capabilities

### CLI-Specific Domain Extensions
Domain object extensions optimized for command-line interface:
- **IntoRow Trait**: Pretty table formatting for `Alias`, `HubFile`, and `RemoteModel` objects with human-readable values
- **Command Builder Patterns**: Complex command construction with validation and CLI-optimized defaults
- **Output Formatting**: Human-readable file sizes (GB conversion), truncated snapshot hashes (8 chars), and consistent table layouts
- **User Experience Optimization**: CLI-specific error messages with actionable guidance and progress feedback

### Model Management Command Pipeline
Sophisticated workflows combining multiple service operations:
- **Create Command Pipeline**: Alias conflict detection → local file existence check → auto-download coordination → alias creation with metadata
- **Pull Command Orchestration**: Dual-mode operation (ByAlias/ByRepoFile) → conflict resolution → download management → alias generation
- **Parameter Management**: OpenAI compatibility settings with CLI-specific parameter handling and context parameter coordination
- **Progress Coordination**: Multi-stage operation feedback with optional progress reporting and debug logging

## Architecture Position

The `commands` crate serves as BodhiApp's **CLI orchestration layer**:
- **Above objs and services**: Coordinates domain objects and business logic services for CLI workflows
- **Below CLI applications**: Provides high-level command implementations for CLI binaries
- **Parallel to routes**: Similar orchestration role but optimized for command-line interface instead of HTTP
- **Cross-cutting with auth_middleware**: Integrates authentication flows for CLI-specific credential management

## Cross-Crate Integration Patterns

### Service Orchestration Dependencies
Complex coordination across BodhiApp's service layer:
- **AppService Registry**: Central coordination point for accessing all business services
- **HubService Integration**: Model discovery, download, and repository authentication
- **DataService Coordination**: Alias management, file system operations, and configuration storage
- **AuthService Integration**: Credential management for gated repository access
- **Error Propagation**: Service errors translated to CLI-specific user messages with localization

### Domain Object Integration
Extensive use of objs crate for CLI operations:
- **Repo, Alias, HubFile**: Core entities for model management command workflows
- **OAIRequestParams**: OpenAI compatibility parameter handling for CLI configuration
- **Error System Integration**: Commands implement `AppError` trait for consistent error handling
- **Builder Patterns**: Commands use objs builder patterns with CLI-specific extensions

### CLI-Specific Extensions
Command layer adds CLI-optimized functionality to domain objects:
- **IntoRow Trait**: Pretty table formatting for terminal output of domain objects
- **Progress Feedback**: CLI-specific progress indicators during long-running service operations
- **User Experience**: Human-readable error messages with actionable guidance and context

## Command Orchestration Workflows

### Multi-Service Create Command Flow
Complex service coordination for alias creation:

1. **Alias Conflict Detection**: `DataService.find_alias()` checks existing alias conflicts with update mode handling
2. **Local File Discovery**: `HubService.local_file_exists()` checks HF cache for existing model files
3. **Auto-Download Coordination**: `HubService.download()` manages model download when files don't exist locally
4. **Alias Construction**: `AliasBuilder` creates alias with repo, filename, snapshot, and OpenAI parameters
5. **Alias Persistence**: `DataService.save_alias()` saves alias configuration to BODHI_HOME/aliases
6. **Debug Logging**: Comprehensive logging throughout the workflow for CLI user feedback

### Pull Command Service Orchestration
Two-mode operation with service coordination:

**ByAlias Mode**:
1. **Alias Conflict Prevention**: `DataService.find_alias()` prevents local alias conflicts
2. **Remote Model Lookup**: `DataService.find_remote_model()` retrieves model metadata from remote registry
3. **Model Download**: `HubService.download()` coordinates model file downloads with optional progress reporting
4. **Local Alias Creation**: `AliasBuilder` creates local alias from remote model metadata with AliasSource::User

**ByRepoFile Mode**:
1. **Local File Check**: `HubService.local_file_exists()` verifies if model already exists in HF cache
2. **Direct Download**: `HubService.download()` downloads model file directly without alias creation
3. **No Alias Creation**: Files downloaded to HF cache without local alias registration

### Cross-Service Error Coordination
Error handling across service boundaries:
- **Service Error Translation**: Hub and data service errors converted to CLI-specific messages
- **Context Preservation**: Original error context maintained through transparent wrapping
- **User Guidance**: CLI-specific error messages provide actionable resolution steps
- **Rollback Coordination**: Failed operations properly clean up partial state across services

## Important Constraints

### Service Coordination Requirements
- Commands must use `AppService` registry for all service access to maintain proper dependency injection
- Multi-service operations require proper error handling and rollback coordination
- Service errors must be transparently wrapped to preserve context while providing CLI-specific user messages
- Authentication state must be coordinated across services for gated repository access

### CLI User Experience Standards
- All long-running operations must provide progress feedback with cancellation support
- Error messages must be actionable with specific guidance for resolution
- Command output must be consistent with CLI conventions and support quiet modes for automation
- Domain object formatting must be optimized for terminal display with human-readable values

### Domain Object Integration Rules
- Commands must use objs domain builders with CLI-specific extensions
- All domain object validation must occur before service operations begin
- Domain objects must support CLI-specific serialization formats (pretty tables, JSON output)
- Builder patterns must provide sensible defaults optimized for CLI usage

## CLI Domain Extensions

### Pretty Table Integration
CLI-optimized display formatting for domain objects:
- **IntoRow Trait**: Converts `Alias`, `HubFile`, and `RemoteModel` to formatted table rows
- **Human-Readable Values**: File sizes in GB, snapshot hashes truncated to 8 characters
- **Consistent Column Layout**: Standardized table structure across all CLI output
- **Terminal Optimization**: Column sizing and formatting optimized for terminal display

### Command Builder Architecture
Sophisticated command construction with CLI-specific validation:
- **Builder Pattern Extensions**: CLI-specific defaults and validation rules
- **Parameter Coordination**: OpenAI compatibility parameters with CLI argument integration
- **Validation Pipeline**: Pre-execution validation with detailed error reporting
- **Test Integration**: Builder patterns designed for comprehensive CLI testing scenarios

### User Experience Orchestration
CLI-specific user interaction patterns:
- **Progress Feedback Systems**: Multi-stage operation progress with service coordination
- **Interactive Confirmation**: User prompts for potentially destructive operations
- **Error Recovery Guidance**: Context-aware suggestions for resolving command failures
- **Output Format Options**: Support for JSON, table, and quiet output modes

## Command Extension Patterns

### Adding New CLI Commands
When creating new commands that coordinate multiple services:

1. **Service Coordination Design**: Plan multi-service interactions with proper error boundaries
2. **CLI-Specific Error Types**: Create command errors that wrap service errors with CLI context
3. **Builder Pattern Implementation**: Provide CLI-optimized builders with sensible defaults
4. **Progress Feedback Integration**: Design user feedback for long-running service operations
5. **Testing Infrastructure**: Use service mocking for isolated command testing

### Cross-Command Patterns
Consistent patterns across all CLI commands:
- **AppService Dependency**: All commands take `Arc<dyn AppService>` for service coordination
- **Transparent Error Wrapping**: Service errors wrapped with CLI-specific context
- **Domain Object Extensions**: CLI-specific formatting and display extensions
- **Progress Coordination**: Consistent progress feedback patterns across all commands

### Testing Command Orchestration
CLI command testing with service mock coordination:
- **Service Mock Composition**: Coordinate multiple service mocks for complex command workflows
- **Error Scenario Testing**: Test error propagation and recovery across service boundaries
- **Output Validation**: Test CLI-specific formatting and user experience patterns
- **Integration Testing**: Validate complete command workflows with realistic service interactions