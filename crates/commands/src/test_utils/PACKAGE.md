# PACKAGE.md - commands/test_utils

This document provides detailed technical information for the `commands/test_utils` module, focusing on BodhiApp's CLI command testing infrastructure and sophisticated service mock coordination patterns.

## CLI Command Testing Architecture

### Command Builder Pattern for Testing
Comprehensive command construction designed for CLI testing scenarios:

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
            // Chat template removed - llama.cpp now handles chat templates
            .oai_request_params(OAIRequestParams::default())
            .context_params(Vec::<String>::default())
            .to_owned()
    }
}
```

### CLI-Specific Testing Features
Command builders provide CLI-optimized testing capabilities:

**Pre-Configured Test Commands**:
- `CreateCommand::testalias()`: Ready-to-use command for alias creation testing with testalias repository
- Builder integration with objs domain factories (`Repo::testalias()`, `Repo::testalias_model_q8()`)
- Default parameter coordination with `OAIRequestParams::default()` and empty context params
- Service mock integration through AppService registry pattern

**CLI Testing Features**:
- Minimal builder pattern focused on essential command construction
- Integration with objs test utilities for consistent domain object usage
- Service mock coordination through AppServiceStubBuilder and TestHfService
- Simplified testing infrastructure for core command execution patterns

## Multi-Service Mock Coordination Architecture

### AppService Mock Composition for CLI Testing
Sophisticated service mocking patterns for command execution testing:

```rust
#[cfg(test)]
mod cli_command_tests {
    use mockall::predicate::*;
    use services::{MockAppService, MockDataService, MockHubService, MockAuthService};
    use objs::test_utils::setup_l10n;
    
    #[rstest]
    #[awt]
    async fn test_create_command_full_workflow(setup_l10n: ()) {
        let mut mock_app_service = MockAppService::new();
        let mut mock_data_service = MockDataService::new();
        let mut mock_hub_service = MockHubService::new();
        let mut mock_auth_service = MockAuthService::new();
        
        // Service mock orchestration for CLI workflow
        mock_data_service
            .expect_alias_exists()
            .with(eq("testalias:instruct"))
            .returning(|_| Ok(false))
            .times(1);
        
        mock_hub_service
            .expect_download_model()
            .with(eq(Repo::testalias()), eq("main"))
            .returning(|repo, snapshot| {
                Ok(vec![HubFileBuilder::testalias()
                    .repo(repo.clone())
                    .snapshot(snapshot.to_string())
                    .build()
                    .unwrap()])
            })
            .times(1);
        
        mock_data_service
            .expect_save_alias()
            .with(function(|alias: &Alias| {
                alias.alias == "testalias:instruct" 
                    && alias.repo == Repo::testalias()
            }))
            .returning(|_| Ok(()))
            .times(1);
        
        // Service registry mock coordination
        mock_app_service
            .expect_data_service()
            .return_const(Arc::new(mock_data_service) as Arc<dyn DataService>);
        mock_app_service
            .expect_hub_service()
            .return_const(Arc::new(mock_hub_service) as Arc<dyn HubService>);
        mock_app_service
            .expect_auth_service()
            .return_const(Arc::new(mock_auth_service) as Arc<dyn AuthService>);
        
        // Execute CLI command with full service coordination
        let command = CreateCommand::testalias();
        let result = command.execute(Arc::new(mock_app_service)).await;
        
        assert!(result.is_ok());
    }
}
```

### Cross-Service Error Testing Implementation
CLI command error testing with comprehensive service failure scenarios:

```rust
#[rstest]
#[awt] 
async fn test_create_command_hub_service_error(setup_l10n: ()) {
    let mut mock_app_service = MockAppService::new();
    let mut mock_data_service = MockDataService::new();
    let mut mock_hub_service = MockHubService::new();
    
    // Service error scenario orchestration
    mock_data_service
        .expect_alias_exists()
        .with(eq("testalias:instruct"))
        .returning(|_| Ok(false));
    
    mock_hub_service
        .expect_download_model()
        .with(eq(Repo::testalias()), any())
        .returning(|repo, snapshot| {
            Err(HubServiceError::HubApiError(HubApiError::new(
                "Repository not found".to_string(),
                404,
                repo.to_string(),
                HubApiErrorKind::NotFound
            )))
        });
    
    mock_app_service
        .expect_data_service()
        .return_const(Arc::new(mock_data_service) as Arc<dyn DataService>);
    mock_app_service
        .expect_hub_service()
        .return_const(Arc::new(mock_hub_service) as Arc<dyn HubService>);
    
    // Test CLI error handling and translation
    let command = CreateCommand::testalias();
    let result = command.execute(Arc::new(mock_app_service)).await;
    
    assert!(result.is_err());
    match result.unwrap_err() {
        CreateCommandError::HubServiceError(hub_err) => {
            // Validate CLI error translation maintains service context
            assert!(format!("{}", hub_err).contains("Repository not found"));
        }
        _ => panic!("Expected HubServiceError"),
    }
}
```

## CLI Output Testing Infrastructure

### Pretty Table Output Validation
CLI-specific output formatting testing for terminal display:

```rust
#[cfg(test)]
mod output_tests {
    use crate::objs_ext::IntoRow;
    use objs::{Alias, HubFile, RemoteModel};
    use prettytable::{Cell, Row, Table};
    use rstest::rstest;
    
    #[test]
    fn test_alias_pretty_table_formatting() {
        let alias = Alias::testalias();
        let row = alias.into_row();
        
        let expected = Row::from(vec![
            Cell::new("testalias:instruct"),
            Cell::new("MyFactory/testalias-gguf"),
            Cell::new("testalias.Q8_0.gguf"),
            Cell::new("5007652f"), // Truncated snapshot for CLI display
        ]);
        
        assert_eq!(expected, row);
    }
    
    #[test]
    fn test_hub_file_human_readable_formatting() {
        let hub_file = HubFile::new(
            PathBuf::from("."),
            Repo::llama3(),
            "model.gguf".to_string(),
            "1234567890abcdef".to_string(),
            Some(10 * 1024 * 1024 * 1024), // 10GB
        );
        
        let row = hub_file.into_row();
        let expected = Row::from(vec![
            Cell::new("QuantFactory/Meta-Llama-3-8B-Instruct-GGUF"),
            Cell::new("model.gguf"),
            Cell::new("12345678"), // 8-character snapshot truncation
            Cell::new("10.00 GB"), // Human-readable file size
        ]);
        
        assert_eq!(expected, row);
    }
    
    #[test]
    fn test_cli_table_consistency() {
        let aliases = vec![
            Alias::testalias(),
            AliasBuilder::default()
                .alias("another:model".to_string())
                .repo(Repo::llama3())
                .filename("model.gguf".to_string())
                .snapshot("abcdef123456".to_string())
                .source(AliasSource::Model)
                .build()
                .unwrap()
        ];
        
        let mut table = Table::new();
        table.set_titles(row!["Alias", "Repository", "Filename", "Snapshot"]);
        
        for alias in aliases {
            table.add_row(alias.into_row());
        }
        
        // Validate consistent table formatting
        let output = table.to_string();
        assert!(output.contains("testalias:instruct"));
        assert!(output.contains("another:model"));
        assert!(output.len() > 0);
    }
}
```

### CLI Error Message Testing Implementation
Comprehensive CLI error message validation with localization integration:

```rust
#[rstest]
#[awt]
async fn test_cli_error_message_quality(setup_l10n: ()) {
    let mut mock_app_service = MockAppService::new();
    let mut mock_data_service = MockDataService::new();
    
    // Create alias conflict scenario
    mock_data_service
        .expect_alias_exists()
        .with(eq("existing-alias"))
        .returning(|_| Ok(true));
    
    mock_app_service
        .expect_data_service()
        .return_const(Arc::new(mock_data_service) as Arc<dyn DataService>);
    
    let command = CreateCommandBuilder::default()
        .alias("existing-alias".to_string())
        .repo(Repo::testalias())
        .filename("model.gguf".to_string())
        .update(false) // Explicitly don't allow updates
        .build()
        .unwrap();
    
    let result = command.execute(Arc::new(mock_app_service)).await;
    
    assert!(result.is_err());
    match result.unwrap_err() {
        CreateCommandError::AliasExists(alias_err) => {
            let api_error = ApiError::from(alias_err);
            let message = api_error.message(&locale_en_us());
            
            // Validate CLI-specific error guidance
            assert!(message.contains("already exists"));
            assert!(message.contains("existing-alias"));
            // Error message should suggest --update flag
            assert!(message.contains("update") || message.contains("--update"));
        }
        _ => panic!("Expected AliasExists error"),
    }
}
```

## Integration Testing Patterns

### Cross-Crate Testing Coordination
CLI commands must coordinate testing across objs, services, and command layers:

```rust
#[rstest]
#[awt]
async fn test_full_cli_workflow_integration(
    setup_l10n: (),
    temp_bodhi_home: TempDir,
    temp_hf_home: TempDir
) {
    // Environment setup for integrated CLI testing
    std::env::set_var("BODHI_HOME", temp_bodhi_home.path());
    std::env::set_var("HF_HOME", temp_hf_home.path());
    
    // Real service coordination for integration testing
    let settings_service = LocalSettingsService::new(temp_bodhi_home.path().to_path_buf());
    let data_service = LocalDataService::new(settings_service.data_dir());
    let hub_service = OfflineHubService::default()
        .with_test_model("MyFactory/testalias-gguf", vec![
            HubFileBuilder::testalias()
                .hf_cache(temp_hf_home.path().join("huggingface"))
                .build()
                .unwrap()
        ]);
    
    let app_service = DefaultAppService::builder()
        .data_service(Arc::new(data_service))
        .hub_service(Arc::new(hub_service))
        .build();
    
    // Test complete CLI command workflow
    let command = CreateCommand::testalias();
    let result = command.execute(Arc::new(app_service)).await;
    
    assert!(result.is_ok());
    
    // Validate CLI command results with real services
    let data_service = app_service.data_service();
    assert!(data_service.alias_exists("testalias:instruct").unwrap());
    
    let alias = data_service.load_alias("testalias:instruct").unwrap();
    assert_eq!(alias.repo, Repo::testalias());
    assert_eq!(alias.filename, Repo::testalias_model_q8());
}
```

### CLI Command Pipeline Testing
Testing complex multi-command workflows with service state coordination:

```rust
#[rstest]
#[awt]
async fn test_create_then_pull_command_pipeline(
    setup_l10n: (),
    temp_bodhi_home: TempDir
) {
    let app_service = create_test_app_service_with_environment(temp_bodhi_home);
    
    // Execute create command
    let create_command = CreateCommand::testalias();
    let create_result = create_command.execute(app_service.clone()).await;
    assert!(create_result.is_ok());
    
    // Attempt pull command with conflict
    let pull_command = PullCommand::ByAlias {
        alias: "testalias:instruct".to_string(),
    };
    let pull_result = pull_command.execute(app_service.clone()).await;
    
    // Should fail due to alias conflict
    assert!(pull_result.is_err());
    match pull_result.unwrap_err() {
        PullCommandError::AliasExists(alias_err) => {
            assert_eq!(alias_err.alias, "testalias:instruct");
        }
        _ => panic!("Expected AliasExists error"),
    }
    
    // Validate CLI error provides actionable guidance
    let api_error = ApiError::from(pull_result.unwrap_err());
    let message = api_error.message(&locale_en_us());
    assert!(message.contains("already exists"));
}
```

## Extension Guidelines for CLI Command Testing

### Adding New Command Tests
When creating tests for new CLI commands:

1. **Service Mock Composition**: Design comprehensive service mock coordination for command workflows
2. **CLI Output Validation**: Test pretty table formatting and JSON output for all commands
3. **Error Message Quality**: Validate CLI-specific error messages provide actionable guidance
4. **Progress Feedback Testing**: Test multi-stage operation progress feedback for user experience
5. **Integration Testing**: Create end-to-end tests with real service coordination

### Cross-Command Testing Patterns
For testing interactions between multiple CLI commands:

1. **Service State Coordination**: Test command interactions with shared service state
2. **Error Recovery Testing**: Test command failure and recovery scenarios
3. **Output Format Consistency**: Ensure consistent formatting across different command types
4. **Parameter Integration**: Test OpenAI compatibility parameters across command boundaries
5. **Authentication Flow Testing**: Test gated repository access across command workflows

### CLI Testing Infrastructure Extensions
When extending CLI testing capabilities:

1. **Builder Pattern Extensions**: Create test builders for all new command types
2. **Mock Service Coordination**: Extend service mocking patterns for new command workflows
3. **Output Format Testing**: Add CLI-specific output validation for new command types
4. **Error Translation Testing**: Test CLI error message translation for new error types
5. **Integration Test Coordination**: Coordinate CLI testing with objs and services test infrastructure

## Commands for CLI Testing

**CLI Command Tests**: `cargo test -p commands` (includes CLI workflow testing)  
**Service Mock Tests**: `cargo test -p commands --features test-utils` (includes service mock coordination)  
**Integration Tests**: `cargo test -p commands --features integration-tests` (includes cross-crate CLI testing)