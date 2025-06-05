# commands - CLI Command Interface

## Overview

The `commands` crate provides a comprehensive command-line interface (CLI) for BodhiApp, enabling users to interact with the application through terminal commands. It implements various operations for model management, alias configuration, and system administration.

## Purpose

- **CLI Interface**: Provides command-line access to BodhiApp functionality
- **Model Management**: Commands for downloading, listing, and managing models
- **Alias Management**: Create and manage model aliases
- **System Operations**: Environment setup and configuration
- **Automation**: Scriptable operations for CI/CD and automation

## Key Commands

### Model Management Commands

#### Pull Command (`cmd_pull.rs`)
- Download models from HuggingFace Hub
- Support for specific model versions and files
- Progress tracking and resumable downloads
- Validation of downloaded files

#### List Command (`cmd_list.rs`)
- List available models and aliases
- Display model metadata and information
- Filter and search capabilities
- Formatted output options

### Alias Management Commands

#### Alias Command (`cmd_alias.rs`)
- Create, update, and delete model aliases
- List existing aliases
- Alias validation and conflict resolution
- Bulk alias operations

### System Commands

#### Create Command (`cmd_create.rs`)
- Initialize new BodhiApp instances
- Setup configuration files
- Database initialization
- Default settings configuration

#### Environment Command (`cmd_envs.rs`)
- Display environment information
- Validate configuration
- Environment-specific operations
- Debug information output

### Core CLI Infrastructure

#### CLI Parser (`cmd_cli.rs`)
- Command-line argument parsing using clap
- Subcommand routing and validation
- Help text generation
- Error handling and user feedback

#### Output Writer (`out_writer.rs`)
- Formatted output handling
- Progress indicators
- Table formatting
- JSON/YAML output options

## Directory Structure

```
src/
├── lib.rs                    # Main module exports
├── cmd_alias.rs              # Alias management commands
├── cmd_cli.rs                # CLI parser and routing
├── cmd_create.rs             # Creation and initialization commands
├── cmd_envs.rs               # Environment information commands
├── cmd_list.rs               # Listing and display commands
├── cmd_pull.rs               # Model download commands
├── objs_ext.rs               # Object extensions for CLI
├── out_writer.rs             # Output formatting utilities
├── resources/                # Localization resources
│   └── en-US/
├── test_utils/               # Testing utilities
│   ├── mod.rs
│   └── create.rs
└── your_file.rs              # Additional command implementations
```

## Command Structure

### CLI Hierarchy
```
bodhi
├── alias                     # Alias management
│   ├── create <name> <model>
│   ├── list
│   ├── update <name> <model>
│   └── delete <name>
├── pull <model>              # Download models
│   ├── --version <version>
│   ├── --file <filename>
│   └── --force
├── list                      # List models/aliases
│   ├── --models
│   ├── --aliases
│   └── --format <json|table>
├── create                    # Initialize new instance
│   ├── --config <path>
│   └── --database <path>
└── envs                      # Environment information
    ├── --validate
    └── --debug
```

## Key Features

### User-Friendly Interface
- Clear command structure and help text
- Progress indicators for long-running operations
- Colored output for better readability
- Error messages with actionable suggestions

### Flexible Output
- Multiple output formats (table, JSON, YAML)
- Customizable verbosity levels
- Machine-readable output for automation
- Human-readable formatting for interactive use

### Error Handling
- Comprehensive error reporting
- Localized error messages
- Recovery suggestions
- Exit codes for scripting

### Progress Tracking
- Real-time progress indicators
- Download progress with speed and ETA
- Operation status updates
- Cancellation support

## Dependencies

### Core Dependencies
- **objs**: Domain objects and error types
- **services**: Business logic services
- **clap**: Command-line argument parsing
- **tokio**: Async runtime support

### Output and Formatting
- **prettytable**: Table formatting
- **indicatif**: Progress bars and spinners
- **serde**: Serialization for JSON/YAML output

### Localization
- **fluent**: Multi-language support
- **include_dir**: Embedded resource files

## Usage Patterns

### Service Integration
Commands use the services layer for all business operations:

```rust
async fn execute_command(
    services: &ServiceContainer,
    args: CommandArgs,
) -> Result<(), ApiError> {
    // Use services to perform operations
}
```

### Output Formatting
The `out_writer` module provides consistent output formatting:

```rust
let writer = OutWriter::new(format, verbosity);
writer.write_table(data)?;
writer.write_progress(progress)?;
```

### Error Handling
All commands use the centralized error handling from the `objs` crate:

```rust
match operation().await {
    Ok(result) => writer.write_success(result),
    Err(error) => writer.write_error(error),
}
```

## Command Examples

### Model Download
```bash
# Download a specific model
bodhi pull microsoft/DialoGPT-medium

# Download with specific version
bodhi pull microsoft/DialoGPT-medium --version v1.0

# Force re-download
bodhi pull microsoft/DialoGPT-medium --force
```

### Alias Management
```bash
# Create an alias
bodhi alias create chat-model microsoft/DialoGPT-medium

# List all aliases
bodhi alias list

# Update an alias
bodhi alias update chat-model microsoft/DialoGPT-large
```

### System Information
```bash
# Show environment information
bodhi envs

# Validate configuration
bodhi envs --validate

# Debug information
bodhi envs --debug
```

## Testing Support

The commands crate includes comprehensive testing utilities:
- Mock service implementations
- Command execution testing
- Output validation
- Integration test helpers

## Integration Points

- **Services Layer**: All business logic is delegated to services
- **Configuration**: Commands respect application configuration
- **Localization**: All user-facing text is localized
- **Error Handling**: Consistent error reporting across all commands

## Future Extensions

The command structure is designed to be easily extensible:
- New commands can be added by implementing the command trait
- Subcommands can be nested for complex operations
- Output formats can be extended
- New service integrations can be added seamlessly
