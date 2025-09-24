# PACKAGE.md - Commands Crate

*For architectural insights and design decisions, see [crates/commands/CLAUDE.md](crates/commands/CLAUDE.md)*

## Implementation Index

The `commands` crate implements CLI command orchestration with sophisticated service coordination and user experience patterns.

### Core Command Files

**src/lib.rs**
- Module structure and feature-gated test utilities
- L10n resources inclusion for error localization
- Public API exports for CreateCommand and PullCommand

**src/cmd_create.rs**
- CreateCommand implementation for alias creation with auto-download
- Multi-service coordination (DataService, HubService) for complete workflow
- Context parameter integration for llama.cpp configuration

**src/cmd_pull.rs**
- PullCommand enum with ByAlias and ByRepoFile variants
- Dual-mode operation supporting both alias creation and direct file download
- Progress tracking support for download operations

**src/objs_ext.rs**
- IntoRow trait implementations for pretty table CLI output
- Human-readable formatting for UserAlias, HubFile, and RemoteModel
- File size formatting (GB conversion) and snapshot hash truncation

### Command Examples

#### Create Command Usage

```rust
  let create_cmd = CreateCommand {
    alias: "tinyllama:instruct".to_string(),
    repo: Repo::tinyllama(),
    filename: Repo::TINYLLAMA_FILENAME.to_string(),
    snapshot: Some("main".to_string()),
    auto_download: true,
    update: false,
    oai_request_params: OAIRequestParamsBuilder::default()
      .frequency_penalty(1.0)
      .max_tokens(2048_u16)
      .build()?,
    context_params: vec![
      "--ctx-size 2048".to_string(),
      "--n-keep 2048".to_string(),
    ],
  };

  create_cmd.execute(service).await?;
```

#### Pull Command Usage

```rust
  // Pull by alias from remote model registry
  let pull = PullCommand::ByAlias {
    alias: "llama3:instruct".to_string(),
  };

  // Pull by repository and file
  let pull = PullCommand::ByRepoFile {
    repo: Repo::new("microsoft/DialoGPT-medium"),
    filename: "model.gguf".to_string(),
    snapshot: Some("main".to_string()),
  };

  pull.execute(service, progress).await?;
```

#### Pretty Table Output

```rust
  let alias = UserAlias::testalias();
  let row = alias.into_row();
  // Outputs: ["testalias:instruct", "MyFactory/testalias-gguf", "testalias.Q8_0.gguf", "5007652f"]

  let hub_file = HubFile::new(path, repo, filename, snapshot, Some(10_737_418_240));
  let row = hub_file.into_row();
  // Outputs: ["repo/name", "model.gguf", "1234abcd", "10.00 GB"]
```

## Service Coordination Patterns

### Multi-Service Workflows

Commands coordinate complex service interactions:

```rust
  // Create command service orchestration
  if service.data_service().find_user_alias(&alias).is_some() {
    if !update { return Err(AliasExistsError(alias).into()); }
  }

  let file_exists = service
    .hub_service()
    .local_file_exists(&repo, &filename, snapshot.clone())?;

  let local_model_file = match file_exists {
    true => service.hub_service().find_local_file(&repo, &filename, snapshot.clone())?,
    false => service.hub_service().download(&repo, &filename, snapshot, None).await?,
  };

  let alias = UserAliasBuilder::default()
    .alias(alias)
    .repo(repo)
    .filename(filename)
    .snapshot(local_model_file.snapshot)
    .request_params(oai_request_params)
    .context_params(context_params)
    .build()?;

  service.data_service().save_alias(&alias)?;
```

### Error Handling Coordination

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

## Testing Patterns

### Service Mock Coordination

```rust
  #[tokio::test]
  async fn test_cmd_create_downloads_model_saves_alias() {
    let create = CreateCommandBuilder::testalias()
      .snapshot(snapshot.clone())
      .build()?;

    test_hf_service
      .inner_mock
      .expect_download()
      .with(eq(create.repo.clone()), eq(create.filename.clone()), eq(snapshot.clone()), always())
      .return_once(|_, _, _, _| Ok(HubFile::testalias()));

    let service = Arc::new(
      AppServiceStubBuilder::default()
        .hub_service(Arc::new(test_hf_service))
        .with_data_service()
        .await
        .build()?,
    );

    create.execute(service.clone()).await?;

    let created = service
      .data_service()
      .find_alias("testalias:instruct")
      .await?;
    assert_eq!(Alias::User(UserAlias::testalias()), created);
  }
```

## Build Commands

```bash
# Test commands crate
cargo test -p commands

# Test with features
cargo test -p commands --features test-utils

# Format code
cargo fmt --package commands

# Run clippy
cargo clippy --package commands
```

## Domain Extensions

### CLI Table Formatting

The IntoRow trait provides consistent CLI table formatting:

- **UserAlias**: Displays alias, repo, filename, truncated snapshot (8 chars)
- **HubFile**: Shows repo, filename, snapshot hash, human-readable size
- **RemoteModel**: Lists alias, repo, filename for remote model registry

### Context Parameter Integration

Commands support llama.cpp context parameters:

```rust
  let context_params = vec![
    "--ctx-size 2048".to_string(),
    "--parallel 2".to_string(),
    "--seed 42".to_string(),
    "--threads 8".to_string(),
  ];
```

These parameters are passed through to the llama server process for execution.

## Recent Architecture Changes

### Chat Template Removal
- llama.cpp now handles chat templates internally
- Commands no longer download or manage tokenizer configurations
- Simplified command structure with template-related fields removed
- Testing scenarios updated to reflect template handling removal

### User Alias Specialization
- Commands now work with UserAlias type for type safety
- Find operations prioritize user aliases with `find_user_alias()`
- Builder patterns specialized for user-created aliases
- Enhanced validation and error handling for user alias workflows