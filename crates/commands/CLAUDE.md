# CLAUDE.md - commands

This file provides guidance to Claude Code when working with the `commands` crate, which implements CLI command logic for BodhiApp.

## Purpose

The `commands` crate provides high-level command implementations for the BodhiApp CLI:

- **Model Management Commands**: Create and pull model aliases with automatic download
- **Command Orchestration**: Coordinate multiple services to execute complex operations
- **User Interface**: Provide clear feedback and error messages during command execution
- **Parameter Validation**: Validate and process command-line arguments before execution
- **Business Logic**: Implement command-specific workflows and decision logic

## Key Components

### Create Command (`src/cmd_create.rs`)
- `CreateCommand` - Creates model aliases with download and configuration
- Supports manual and automatic model download from Hugging Face Hub
- Handles alias updates and conflict resolution
- Configures OpenAI request parameters and GPU context settings
- Validates file existence and downloads missing files when needed

### Pull Command (`src/cmd_pull.rs`)
- `PullCommand` - Downloads models by alias or repository specification
- Two execution modes:
  - `ByAlias` - Downloads pre-configured models from remote registry
  - `ByRepoFile` - Direct download from Hugging Face repository
- Prevents duplicate downloads and alias conflicts
- Provides clear status feedback during download operations

### Object Extensions (`src/objs_ext.rs`)
- Extension traits and utilities for domain objects
- Helper methods for command-specific object manipulation
- Enhanced functionality for working with aliases and models

## Dependencies

### Core Infrastructure
- `objs` - Domain objects, validation, and error handling
- `services` - Business logic services for data and hub operations
- `errmeta_derive` - Error metadata generation for localized messages

### Command Interface
- `derive_builder` - Builder pattern for command construction
- `prettytable` - Formatted table output for command results
- `thiserror` - Error type derivation for command-specific errors

### Development
- `rstest` - Parameterized testing for command validation
- `mockall` - Service mocking for isolated command testing

## Architecture Position

The `commands` crate sits at the application logic layer:
- **Above**: Services layer providing business logic and external integrations
- **Below**: CLI interface and application entry points
- **Coordinates**: Multiple services to implement complete user workflows
- **Translates**: User intentions into service operations with proper error handling

## Usage Patterns

### Create Command Execution
```rust
use commands::{CreateCommand, CreateCommandBuilder};
use objs::{Repo, GptContextParams, OAIRequestParams};

let command = CreateCommandBuilder::default()
    .alias("my-model".to_string())
    .repo(Repo::from("microsoft/DialoGPT-medium"))
    .filename("pytorch_model.bin".to_string())
    .snapshot(Some("main".to_string()))
    .auto_download(true)
    .update(false)
    .oai_request_params(OAIRequestParams::default())
    .context_params(GptContextParams::default())
    .build()?;

command.execute(app_service).await?;
```

### Pull Command Execution
```rust
use commands::PullCommand;

// Pull by alias from remote registry
let pull_alias = PullCommand::ByAlias {
    alias: "gpt-3.5-turbo".to_string(),
};
pull_alias.execute(app_service).await?;

// Pull by direct repository specification
let pull_repo = PullCommand::ByRepoFile {
    repo: Repo::from("microsoft/DialoGPT-medium"),
    filename: "config.json".to_string(),
    snapshot: Some("v1.0".to_string()),
};
pull_repo.execute(app_service).await?;
```

### Error Handling
```rust
use commands::{CreateCommandError, PullCommandError};

match create_command.execute(app_service).await {
    Err(CreateCommandError::AliasExists(alias_name)) => {
        println!("Alias '{}' already exists. Use --update to modify.", alias_name);
    }
    Err(CreateCommandError::HubServiceError(hub_err)) => {
        println!("Failed to access model repository: {}", hub_err);
    }
    Ok(()) => println!("Model alias created successfully"),
}
```

## Integration Points

### With Services Layer
- `AppService` - Central service registry for accessing business logic
- `DataService` - Model alias management and local storage
- `HubService` - Hugging Face Hub integration for model discovery and download
- Error propagation from service layer to command layer

### With CLI Layer
- Commands receive validated parameters from CLI argument parsing
- Command results and errors are formatted for terminal display
- Progress feedback provided during long-running operations

### With Domain Objects
- Uses `Alias`, `Repo`, `HubFile` for model representation
- Leverages `GptContextParams` and `OAIRequestParams` for configuration
- Builder patterns for constructing complex command objects

## Command Logic

### Create Command Workflow
1. **Validation**: Check if alias already exists and handle update scenarios
2. **File Discovery**: Determine if model files exist locally in HF_HOME
3. **Download Decision**: Automatically download missing files if enabled
4. **Alias Construction**: Build alias object with all configuration parameters
5. **Persistence**: Save alias to local storage for future use
6. **Feedback**: Provide clear status messages throughout the process

### Pull Command Workflow

#### By Alias Mode
1. **Conflict Check**: Ensure alias doesn't already exist locally
2. **Registry Lookup**: Find model in remote registry by alias
3. **Download**: Pull model files from Hugging Face Hub
4. **Alias Creation**: Generate local alias from remote model metadata
5. **Storage**: Save new alias to local alias registry

#### By Repository Mode
1. **Existence Check**: Verify if files already exist locally
2. **Direct Download**: Pull specified files from repository
3. **Status Feedback**: Report download completion without creating alias

## Error Handling

### Command-Specific Errors
- `CreateCommandError` - Alias conflicts, file not found, validation failures
- `PullCommandError` - Remote model not found, download failures, alias conflicts

### Error Propagation
- Service errors bubble up through transparent error wrapping
- Command errors include context about the failed operation
- Localized error messages provided via `errmeta_derive`

### User-Friendly Messages
- Clear descriptions of what went wrong and potential solutions
- Contextual information about affected resources (aliases, repositories, files)
- Guidance on how to resolve common issues (use --update, check network, etc.)

## Configuration Management

### Model Parameters
- OpenAI API compatibility parameters (temperature, max_tokens, etc.)
- GPU context configuration (context size, parallel processing)
- Model-specific settings preserved in alias definitions

### Download Behavior
- `auto_download` flag controls automatic file acquisition
- Update mode allows overwriting existing aliases
- Snapshot specification for version control

## Development Guidelines

### Adding New Commands
1. Create command struct with builder pattern
2. Implement error enum with `AppError` trait
3. Add async execute method taking `Arc<dyn AppService>`
4. Include comprehensive error handling and user feedback
5. Write unit tests with service mocking

### Error Handling Best Practices
- Use transparent error wrapping for service errors
- Provide specific error types for command-level failures
- Include contextual information in error messages
- Map errors to appropriate user actions

### Testing Strategy
- Unit tests with mocked services for isolation
- Test both success and failure scenarios
- Validate error message quality and actionability
- Test builder pattern validation and parameter handling

## User Experience

### Progress Feedback
- Status messages during file discovery and download
- Clear indication of update vs. create operations
- File size and progress information for large downloads

### Error Messages
- Specific, actionable error descriptions
- Context about what the user was trying to accomplish
- Suggestions for resolving common issues

### Command Output
- Consistent formatting across all commands
- Machine-readable output options where appropriate
- Quiet modes for scripting and automation

## Performance Considerations

### Async Operations
- All network operations are asynchronous
- Concurrent downloads when appropriate
- Non-blocking file system operations

### Resource Management
- Efficient memory usage during large file operations
- Proper cleanup of temporary resources
- Connection pooling for HTTP operations inherited from services

## Future Extensions

The commands crate is designed to be extensible for additional CLI functionality:
- Model list and search commands
- Alias management operations (delete, rename, copy)
- Configuration and settings management
- System status and health check commands