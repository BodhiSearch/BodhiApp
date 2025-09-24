# PACKAGE.md - commands

This document provides detailed technical information for the `commands` crate, focusing on BodhiApp's CLI command orchestration architecture and sophisticated service coordination patterns.

## File Structure

```
crates/commands/
├── Cargo.toml                     # Dependencies: objs, services, derive-new, prettytable
├── src/
│   ├── lib.rs:1-18               # Module exports and localization resources
│   ├── cmd_create.rs:1-235       # CreateCommand orchestration and testing
│   ├── cmd_pull.rs:1-230         # PullCommand dual-mode operation
│   ├── objs_ext.rs:1-112         # IntoRow trait for CLI formatting
│   ├── test_utils/
│   │   ├── mod.rs:1-2            # Test utility module
│   │   └── create.rs:1-23        # CreateCommand test builders
│   └── resources/
│       └── en-US/messages.ftl    # Localization resources (placeholder)
└── [CLAUDE.md, PACKAGE.md]       # Documentation files
```

## CLI Command Orchestration Architecture

The `commands` crate serves as BodhiApp's **CLI command orchestration layer**, implementing complex multi-service workflows with comprehensive error handling and CLI-optimized user experience.

### Service Coordination Architecture
Commands orchestrate multiple services through the `AppService` registry pattern:

```rust
impl CreateCommand {
    pub async fn execute(self, service: Arc<dyn AppService>) -> Result<()> {
        // Alias conflict detection with update mode support
        if service.data_service().find_user_alias(&self.alias).is_some() {
            if !self.update {
                return Err(AliasExistsError(self.alias.clone()).into());
            }
            debug!("Updating existing alias: '{}'", self.alias);
        } else {
            debug!("Creating new alias: '{}'", self.alias);
        }
        
        // Local file existence check and auto-download coordination
        let file_exists = service.hub_service()
            .local_file_exists(&self.repo, &self.filename, self.snapshot.clone())?;
        
        let local_model_file = match file_exists {
            true => {
                debug!("repo: '{}', filename: '{}', already exists in $HF_HOME", 
                    &self.repo, &self.filename);
                service.hub_service()
                    .find_local_file(&self.repo, &self.filename, self.snapshot.clone())?
            }
            false => {
                if self.auto_download {
                    service.hub_service()
                        .download(&self.repo, &self.filename, self.snapshot, None).await?
                } else {
                    return Err(CreateCommandError::HubServiceError(
                        HubFileNotFoundError::new(self.filename.clone(), self.repo.to_string(),
                            self.snapshot.clone().unwrap_or_else(|| SNAPSHOT_MAIN.to_string())).into()
                    ));
                }
            }
        };
        
        // User alias creation with metadata coordination
        let alias = UserAliasBuilder::default()
            .alias(self.alias)
            .repo(self.repo)
            .filename(self.filename)
            .snapshot(local_model_file.snapshot)
            .request_params(self.oai_request_params)
            .context_params(self.context_params)
            .build()?;
        
        service.data_service().save_alias(&alias)?;
        debug!("model alias: '{}' saved to $BODHI_HOME/aliases", alias.alias);
        Ok(())
    }
}
```

### Cross-Service Error Coordination
Sophisticated error handling across service boundaries with CLI-specific translation:

```rust
#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum CreateCommandError {
    #[error(transparent)]
    Builder(#[from] BuilderError),
    #[error(transparent)]
    AliasExists(#[from] AliasExistsError),
    #[error(transparent)]
    ObjValidationError(#[from] ObjValidationError),
    #[error(transparent)]
    HubServiceError(#[from] HubServiceError),
    #[error(transparent)]
    DataServiceError(#[from] DataServiceError),
}
```

**Key Error Coordination Features**:
- Transparent error wrapping preserves original service error context
- CLI-specific error messages provide actionable guidance for users
- Error propagation maintains full error chain for debugging
- Localized error messages via `errmeta_derive` integration with objs error system

## CLI-Specific Domain Extensions

### Pretty Table Integration Architecture
Sophisticated domain object formatting for CLI output:

```rust
pub trait IntoRow {
    fn into_row(self) -> Row;
}

impl IntoRow for UserAlias {
    fn into_row(self) -> Row {
        Row::from(vec![
            Cell::new(&self.alias),
            Cell::new(&self.repo.to_string()),
            Cell::new(&self.filename),
            Cell::new(&self.snapshot[..8]), // Truncated hash for readability
            // Chat template column removed - llama.cpp now handles chat templates
        ])
    }
}

impl IntoRow for HubFile {
    fn into_row(self) -> Row {
        let human_size = self.size
            .map(|size| format!("{:.2} GB", size as f64 / 2_f64.powf(30.0)))
            .unwrap_or_else(|| String::from("Unknown"));
        Row::from(vec![
            Cell::new(&self.repo.to_string()),
            Cell::new(&self.filename),
            Cell::new(&self.snapshot[..8]),
            Cell::new(&human_size), // Human-readable file sizes
        ])
    }
}
```

**CLI Display Optimization Features**:
- Human-readable file sizes with GB conversion for terminal display
- Snapshot hash truncation (8 characters) for compact table formatting
- Consistent column layouts across all CLI commands
- Terminal-optimized formatting for readability and professional appearance

### Command Builder Pattern Implementation
Advanced builder patterns with CLI-specific validation and defaults:

```rust
#[derive(Debug, Clone, PartialEq, derive_new::new, derive_builder::Builder)]
#[allow(clippy::too_many_arguments)]
pub struct CreateCommand {
    #[new(into)]
    pub alias: String,
    pub repo: Repo,
    #[new(into)]
    pub filename: String,
    pub snapshot: Option<String>,
    #[builder(default = "true")]
    pub auto_download: bool,
    #[builder(default = "false")]
    pub update: bool,
    pub oai_request_params: OAIRequestParams,
    pub context_params: Vec<String>,
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

**Builder Pattern CLI Features**:
- CLI-optimized default values (`auto_download = true` for user convenience)
- Integration with objs domain builders for consistency
- Test-specific builders for comprehensive CLI testing scenarios
- Validation coordination with domain object validation rules

## Pull Command Service Orchestration

### Dual-Mode Operation Architecture
Sophisticated command architecture supporting multiple execution modes:

```rust
#[derive(Debug, PartialEq)]
pub enum PullCommand {
    ByAlias {
        alias: String,
    },
    ByRepoFile {
        repo: Repo,
        filename: String,
        snapshot: Option<String>,
    },
}

impl PullCommand {
    pub async fn execute(
        self, 
        service: Arc<dyn AppService>, 
        progress: Option<services::Progress>
    ) -> Result<()> {
        match self {
            PullCommand::ByAlias { alias } => {
                // Prevent alias conflicts
                if service.data_service().find_user_alias(&alias).is_some() {
                    return Err(AliasExistsError(alias.clone()).into());
                }
                
                // Remote model lookup from data service
                let Some(model) = service.data_service().find_remote_model(&alias)? else {
                    return Err(RemoteModelNotFoundError::new(alias.clone()))?;
                };
                
                // Download with optional progress reporting
                let local_model_file = service.hub_service()
                    .download(&model.repo, &model.filename, None, progress).await?;
                
                // Create local alias from remote metadata
                let alias = UserAliasBuilder::default()
                    .alias(model.alias)
                    .repo(model.repo)
                    .filename(model.filename)
                    .snapshot(local_model_file.snapshot)
                    .request_params(model.request_params)
                    .context_params(model.context_params)
                    .build()?;
                
                service.data_service().save_alias(&alias)?;
                debug!("model alias: '{}' saved to $BODHI_HOME/aliases", alias.alias);
                Ok(())
            }
            PullCommand::ByRepoFile { repo, filename, snapshot } => {
                // Check if model already exists locally
                let model_file_exists = service.hub_service()
                    .local_file_exists(&repo, &filename, snapshot.clone())?;
                
                if model_file_exists {
                    debug!("repo: '{repo}', filename: '{filename}' already exists in $HF_HOME");
                    return Ok(());
                } else {
                    service.hub_service()
                        .download(&repo, &filename, snapshot.clone(), progress).await?;
                    debug!("repo: '{repo}', filename: '{filename}' downloaded into $HF_HOME");
                }
                Ok(())
            }
        }
    }
}
```

### Service Error Translation Architecture
CLI-specific error handling with comprehensive service error integration:

```rust
#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum PullCommandError {
    #[error(transparent)]
    Builder(#[from] BuilderError),
    #[error(transparent)]
    HubServiceError(#[from] HubServiceError),
    #[error(transparent)]
    AliasExists(#[from] AliasExistsError),
    #[error(transparent)]
    RemoteModelNotFound(#[from] RemoteModelNotFoundError),
    #[error(transparent)]
    DataServiceError(#[from] DataServiceError),
    #[error(transparent)]
    ObjValidationError(#[from] ObjValidationError),
}
```

**Error Translation Features**:
- Service errors transparently wrapped with full context preservation
- CLI-specific error messages generated via `errmeta_derive` integration
- Localized error messages coordinated with objs error system
- Actionable error guidance for CLI users with specific resolution steps

## Cross-Crate Integration Patterns

### Domain Object Coordination
Commands extensively coordinate with objs crate for domain consistency:

```rust
// Alias creation with parameter coordination
let alias = UserAliasBuilder::default()
    .alias(command.alias)
    .repo(command.repo.clone())
    .filename(command.filename)
    .snapshot(hub_file.snapshot.clone())
    .request_params(command.oai_request_params)
    .context_params(command.context_params)
    .build()?;

// Domain validation before service operations
if let Err(validation_error) = alias.validate() {
    return Err(CreateCommandError::ObjValidationError(validation_error));
}
```

### Service Registry Integration
All commands coordinate through the `AppService` trait for consistent service access:

```rust
pub trait CommandExecutor {
    async fn execute(self, service: Arc<dyn AppService>) -> Result<()>;
}

// Consistent service access pattern across all commands
impl CreateCommand {
    pub async fn execute(self, service: Arc<dyn AppService>) -> Result<()> {
        let data_service = service.data_service();
        let hub_service = service.hub_service();
        let auth_service = service.auth_service(); // For gated repositories
        
        // Multi-service orchestration with proper error boundaries
    }
}
```

## CLI User Experience Architecture

### Progress Feedback Integration
Commands provide comprehensive progress feedback for long-running operations:

```rust
impl CreateCommand {
    async fn execute_with_progress(self, service: Arc<dyn AppService>) -> Result<()> {
        // File discovery feedback
        debug!("Checking for existing alias: {}", self.alias);
        
        // Download progress coordination
        if self.auto_download {
            debug!("Auto-downloading model files for: {}", self.repo);
            let hub_files = hub_service.download_model(&self.repo, &snapshot).await?;
            debug!("Downloaded {} files", hub_files.len());
        }
        
        // Alias creation confirmation
        debug!("Creating alias: {} -> {}/{}", self.alias, self.repo, self.filename);
        data_service.save_alias(&alias)?;
        debug!("Alias created successfully: {}", self.alias);
        
        Ok(())
    }
}
```

### Output Format Coordination
CLI commands support multiple output formats for different use cases:

```rust
// Pretty table output for interactive use
pub fn format_aliases_table(aliases: Vec<UserAlias>) -> Table {
    let mut table = Table::new();
    table.set_titles(row!["Alias", "Repository", "Filename", "Snapshot"]);
    
    for alias in aliases {
        table.add_row(alias.into_row());
    }
    
    table
}

// JSON output for automation and scripting
pub fn format_aliases_json(aliases: Vec<UserAlias>) -> serde_json::Value {
    serde_json::to_value(aliases).unwrap_or_default()
}
```

## Extension Guidelines

### Adding New CLI Commands
When creating commands that coordinate multiple services:

1. **Service Dependency Design**: Use `Arc<dyn AppService>` for service registry access
2. **Error Handling Architecture**: Create command-specific errors with transparent service error wrapping
3. **Builder Pattern Implementation**: Provide CLI-optimized builders with sensible defaults
4. **Domain Integration**: Coordinate with objs domain builders for consistency
5. **Testing Infrastructure**: Design for comprehensive service mocking and CLI testing

### Cross-Service Operation Patterns
For commands requiring multiple service coordination:

1. **Transaction Boundaries**: Design operations with proper rollback capabilities
2. **Error Recovery**: Implement error recovery with partial state cleanup
3. **Progress Feedback**: Provide multi-stage progress updates for user experience
4. **Authentication Coordination**: Handle gated repository access across services
5. **Validation Pipeline**: Validate domain objects before service operations

## Commands Testing Architecture

### Service Mock Coordination
CLI commands require sophisticated service mocking for isolated testing:

```rust
#[cfg(test)]
mod tests {
    use mockall::predicate::*;
    use services::{MockAppService, MockDataService, MockHubService};
    
    #[tokio::test]
    async fn test_create_command_success() {
        let mut mock_service = MockAppService::new();
        let mut mock_data_service = MockDataService::new();
        let mut mock_hub_service = MockHubService::new();
        
        // Coordinate service mock expectations
        mock_data_service
            .expect_alias_exists()
            .with(eq("test-alias"))
            .returning(|_| Ok(false));
            
        mock_hub_service
            .expect_download_model()
            .with(eq(Repo::testalias()), eq("main"))
            .returning(|_, _| Ok(vec![HubFile::testalias()]));
            
        mock_data_service
            .expect_save_alias()
            .with(function(|alias: &UserAlias| alias.alias == "test-alias"))
            .returning(|_| Ok(()));
        
        mock_service
            .expect_data_service()
            .return_const(Arc::new(mock_data_service) as Arc<dyn DataService>);
        mock_service
            .expect_hub_service()
            .return_const(Arc::new(mock_hub_service) as Arc<dyn HubService>);
        
        // Execute command with mocked services
        let command = CreateCommandBuilder::default()
            .alias("test-alias".to_string())
            .repo(Repo::testalias())
            .filename("model.gguf".to_string())
            .build()
            .unwrap();
            
        let result = command.execute(Arc::new(mock_service)).await;
        assert!(result.is_ok());
    }
}
```

**Testing Integration Features**:
- Service mock coordination for complex multi-service workflows
- Error scenario testing with service failure simulation
- CLI output validation with format testing
- Integration testing with realistic service interaction patterns

## Commands

**Testing**: `cargo test -p commands` (includes CLI command integration tests)  
**CLI Testing**: `cargo test -p commands --features test-utils` (includes test fixtures)