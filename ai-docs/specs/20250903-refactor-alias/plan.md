# Alias Structure Refactoring Plan

## Executive Summary

This document outlines the refactoring plan to restructure the alias system in BodhiApp. The goal is to create a proper enum-based type system that clearly distinguishes between three types of models:

- User-created local models with full configuration
- Auto-discovered local models with minimal configuration
- Remote API models

## Current State (After Initial Refactoring)

Based on recent commits, the following changes have already been completed:

1. ✅ Renamed `Alias` to `UserAlias` (commit c054a1b5)
2. ✅ Renamed `ApiModelAlias` to `ApiAlias` (commit dadfaa6b)
3. ✅ Renamed `alias.rs` to `user_alias.rs` (commit 71b7557d)
4. ✅ Renamed `model_alias.rs` to `alias.rs` (commit 08010334)
5. ✅ Created `Alias` enum with three variants (User, Model, Api)

Current file structure:

- `crates/objs/src/user_alias.rs` - Contains `UserAlias` struct and `AliasSource` enum
- `crates/objs/src/alias.rs` - Contains `Alias` enum with three variants
- `crates/objs/src/api_model_alias.rs` - Contains `ApiAlias` struct

## Problem Statement

Currently, the `Model` variant of the `Alias` enum uses `UserAlias` struct which includes unnecessary fields like `context_params` and `request_params` that auto-discovered models don't need. We need a simpler `ModelAlias` struct for auto-discovered models.

Additionally, `UnifiedModelResponse` in routes_app is redundant now that we have a proper `Alias` enum structure.

## Proposed Solution

### 1. Create Dedicated ModelAlias Struct

Create a new lightweight struct specifically for auto-discovered models that only contains the essential fields.

### 2. Update Alias Enum

Modify the `Alias` enum to use the new `ModelAlias` struct for the Model variant.

### 3. Remove UnifiedModelResponse

Replace all uses of `UnifiedModelResponse` with the `Alias` enum directly.

## Detailed Refactoring Plan

### Phase 1: Create ModelAlias Struct in objs Crate

**File: `crates/objs/src/model_alias.rs`** (new file)

```rust
use crate::{AliasSource, Repo};
use serde::{Deserialize, Serialize};
use derive_new::new;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, derive_builder::Builder, new)]
#[builder(
  setter(into, strip_option),
  build_fn(error = crate::BuilderError))]
#[cfg_attr(any(test, feature = "test-utils"), derive(Default))]
pub struct ModelAlias {
  pub alias: String,
  pub repo: Repo,
  pub filename: String,
  pub snapshot: String,
  #[serde(default)]
  #[builder(default = "AliasSource::Model")]
  pub source: AliasSource, // Will always be AliasSource::Model
}

impl ModelAlias {
  pub fn config_filename(&self) -> String {
    let filename = self.alias.replace(':', "--");
    crate::to_safe_filename(&filename)
  }
}

impl std::fmt::Display for ModelAlias {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "ModelAlias {{ alias: {}, repo: {}, filename: {}, snapshot: {} }}",
      self.alias, self.repo, self.filename, self.snapshot
    )
  }
}
```

**Actions:**

1. Create new file `crates/objs/src/model_alias.rs`
2. Add `mod model_alias;` to `lib.rs` (after existing alias modules)
3. Add `pub use model_alias::*;` to `lib.rs` exports
4. Run `cargo check -p objs`
5. Run `cargo test -p objs`

### Phase 2: Update Alias Enum

**File: `crates/objs/src/alias.rs`**

Update the enum to use the new `ModelAlias`:

```rust
use crate::{UserAlias, ModelAlias, ApiAlias};
use serde::{Deserialize, Serialize};

/// Flat enum representing all types of model aliases
/// Each variant contains its own source field, maintaining single source of truth
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Alias {
  /// User-defined local model (source: AliasSource::User)
  User(UserAlias),
  /// Auto-discovered local model (source: AliasSource::Model)
  Model(ModelAlias), // Changed from UserAlias to ModelAlias
  /// Remote API model (source: AliasSource::RemoteApi)
  Api(ApiAlias),
}

impl Alias {
  /// Check if this alias can serve the requested model
  pub fn can_serve(&self, model: &str) -> bool {
    match self {
      Alias::User(alias) => alias.alias == model,
      Alias::Model(alias) => alias.alias == model,
      Alias::Api(api_alias) => api_alias.models.contains(&model.to_string()),
    }
  }

  /// Get the alias name for this model
  pub fn alias_name(&self) -> &str {
    match self {
      Alias::User(alias) => &alias.alias,
      Alias::Model(alias) => &alias.alias,
      Alias::Api(api_alias) => &api_alias.id,
    }
  }

  /// Get the source of this alias
  pub fn source(&self) -> &AliasSource {
    match self {
      Alias::User(alias) => &alias.source,
      Alias::Model(alias) => &alias.source,
      Alias::Api(api_alias) => &api_alias.source,
    }
  }
}
```

**Test Updates:**
Update test cases in the same file to use `ModelAlias` or `ModelAliasBuilder`:

```rust
#[test]
fn test_model_alias_model_can_serve() {
  let alias = ModelAliasBuilder::default()
    .alias("testalias:instruct")
    .repo(Repo::from_str("test/testalias").unwrap())
    .filename("testalias.gguf")
    .snapshot("main")
    .build()
    .unwrap();

  let model_alias = Alias::Model(alias);

  assert!(model_alias.can_serve("testalias:instruct"));
  assert!(!model_alias.can_serve("llama3:instruct"));
  assert_eq!(model_alias.alias_name(), "testalias:instruct");
}
```

**Actions:**

1. Update `Alias::Model` variant to use `ModelAlias`
2. Update helper methods to handle `ModelAlias`
3. Update test cases to use `ModelAlias` or `ModelAliasBuilder`
4. Run `cargo check -p objs`
5. Run `cargo test -p objs`

### Phase 3: Update services Crate

**File: `crates/services/src/hub_service.rs`**

Update trait definition and implementation:

```rust
#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait HubService: std::fmt::Debug + Send + Sync {
  // ... other methods ...

  fn list_model_aliases(&self) -> Result<Vec<ModelAlias>>; // Changed return type
}

impl HubService for HfHubService {
  // ... other methods ...

  fn list_model_aliases(&self) -> Result<Vec<ModelAlias>> {
    let cache = self.hf_cache();
    let mut aliases = WalkDir::new(&cache)
      .follow_links(true)
      .into_iter()
      .filter_map(|e| e.ok())
      .filter(|entry| entry.file_type().is_file())
      .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "gguf"))
      .filter(|entry| {
        entry
          .path()
          .parent()
          .and_then(|p| p.parent())
          .is_some_and(|p| p.ends_with("snapshots"))
      })
      .filter_map(|entry| {
        let path = entry.path();
        let models_dir = path.ancestors().find(|p| {
          p.file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|name| name.starts_with("models--"))
        })?;

        let dir_name = models_dir.file_name()?.to_str()?;
        let repo_path = dir_name.strip_prefix("models--")?;
        let (owner, repo_name) = repo_path.split_once("--")?;
        let repo = Repo::from_str(&format!("{}/{}", owner, repo_name)).ok()?;

        let filename = path.file_name()?.to_str()?.to_string();
        let snapshot = path.parent()?.file_name()?.to_str()?.to_string();

        Some(HubFile::new(cache.clone(), repo, filename, snapshot, None))
      })
      .filter_map(|hub_file| {
        let qualifier = hub_file
          .filename
          .split('.')
          .nth_back(1)
          .and_then(|s| s.split('-').nth_back(0))
          .unwrap_or_else(|| &hub_file.filename);
        let alias = ModelAliasBuilder::default() // Changed to ModelAliasBuilder
          .alias(format!("{}:{}", hub_file.repo, qualifier))
          .repo(hub_file.repo)
          .filename(hub_file.filename)
          .snapshot(hub_file.snapshot)
          .build()
          .ok()?;
        Some(alias)
      })
      .collect::<Vec<_>>();

    // Sort by alias name and then by snapshot, remove duplicates keeping latest snapshot
    aliases.sort_by(|a, b| (&a.alias, &b.snapshot).cmp(&(&b.alias, &a.snapshot)));
    aliases.dedup_by(|a, b| a.alias == b.alias);

    Ok(aliases)
  }
}
```

**File: `crates/services/src/data_service.rs`**

Update trait and implementation:

```rust
use objs::{UserAlias, ModelAlias, ApiAlias, Alias, ...};

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait DataService: Send + Sync + std::fmt::Debug {
  fn list_aliases(&self) -> Result<Vec<Alias>>; // Changed return type
  fn save_alias(&self, alias: &UserAlias) -> Result<PathBuf>; // Keep UserAlias for saving
  fn find_alias(&self, alias: &str) -> Option<Alias>; // Changed return type
  // ... other methods unchanged ...
}

impl DataService for LocalDataService {
  fn list_aliases(&self) -> Result<Vec<Alias>> {
    let user_aliases = self._list_aliases()?;
    let mut result: Vec<Alias> = user_aliases
      .into_values()
      .map(|a| Alias::User(a))
      .collect();

    result.sort_by(|a, b| a.alias_name().cmp(b.alias_name()));

    let model_aliases = self.hub_service.list_model_aliases()?;
    result.extend(model_aliases.into_iter().map(|a| Alias::Model(a)));

    Ok(result)
  }

  fn find_alias(&self, alias: &str) -> Option<Alias> {
    let aliases = self.list_aliases();
    let aliases = aliases.unwrap_or_default();
    aliases.into_iter().find(|obj| obj.alias_name() == alias)
  }

  fn copy_alias(&self, alias: &str, new_alias: &str) -> Result<()> {
    let alias_obj = self
      .find_alias(alias)
      .ok_or_else(|| AliasNotFoundError(alias.to_string()))?;

    // Only UserAlias can be copied
    let mut user_alias = match alias_obj {
      Alias::User(u) => u,
      _ => return Err(DataServiceError::from(
        BadRequestError::new("Only user aliases can be copied".to_string())
      )),
    };

    match self.find_alias(new_alias) {
      Some(_) => Err(AliasExistsError(new_alias.to_string()))?,
      None => {
        user_alias.alias = new_alias.to_string();
        self.save_alias(&user_alias)?;
        Ok(())
      }
    }
  }

  // ... other methods ...
}
```

**Update Mock Implementations:**
Update any mock implementations in test_utils to match new signatures.

**Actions:**

1. Update `HubService` trait and implementation
2. Update `DataService` trait and implementation
3. Update mock implementations
4. Run `cargo check -p services`
5. Run `cargo test -p services`

### Phase 4: Update server_core Crate

**File: `crates/server_core/src/model_router.rs`**

Update to work with `Alias` enum:

```rust
use objs::{Alias, UserAlias, ModelAlias, ApiAlias, ...};

impl ModelRouter {
  pub async fn resolve_model(&self, model: &str) -> Result<ResolvedModel, ModelRouterError> {
    // First check local aliases
    if let Some(alias) = self.data_service.find_alias(model) {
      match alias {
        Alias::User(user_alias) | Alias::Model(model_alias) => {
          // Handle local models
          let filepath = self.resolve_local_file(&alias)?;
          return Ok(ResolvedModel::Local { filepath, alias });
        }
        Alias::Api(api_alias) => {
          // Handle API models
          return Ok(ResolvedModel::Api { config: api_alias });
        }
      }
    }

    // Check API models from database
    if let Some(api_alias) = self.db_service.get_api_model_alias(model).await? {
      return Ok(ResolvedModel::Api { config: api_alias });
    }

    Err(ModelRouterError::ModelNotFound(model.to_string()))
  }

  fn resolve_local_file(&self, alias: &Alias) -> Result<PathBuf, ModelRouterError> {
    let (repo, filename, snapshot) = match alias {
      Alias::User(u) => (&u.repo, &u.filename, &u.snapshot),
      Alias::Model(m) => (&m.repo, &m.filename, &m.snapshot),
      _ => return Err(ModelRouterError::InvalidAliasType),
    };

    self.hub_service
      .find_local_file(repo, filename, Some(snapshot.clone()))
      .map(|hub_file| hub_file.filepath())
      .map_err(|e| ModelRouterError::from(e))
  }
}
```

**Actions:**

1. Update model router to handle `Alias` enum
2. Update helper methods and error handling
3. Run `cargo check -p server_core`
4. Run `cargo test -p server_core`

### Phase 5: Update routes_oai Crate

**File: `crates/routes_oai/src/routes_oai_models.rs`**

Update model listing to work with `Alias`:

```rust
use objs::{Alias, UserAlias, ModelAlias, ApiAlias, ...};

fn to_oai_model(state: Arc<dyn RouterState>, alias: Alias) -> OAIModel {
  let (id, owned_by) = match &alias {
    Alias::User(u) => (u.alias.clone(), "user".to_string()),
    Alias::Model(m) => (m.alias.clone(), "system".to_string()),
    Alias::Api(a) => (a.id.clone(), a.provider.clone()),
  };

  OAIModel {
    id,
    created: 1677610602,
    object: "model".to_string(),
    owned_by,
  }
}

pub async fn oai_models_handler(
  State(state): State<Arc<dyn RouterState>>,
) -> Result<Json<OAIModelListResponse>, ApiError> {
  // Get all aliases from DataService (returns Vec<Alias>)
  let aliases = state
    .app_service()
    .data_service()
    .list_aliases()?;

  let mut models = aliases
    .into_iter()
    .map(|alias| to_oai_model(state.clone(), alias))
    .collect::<Vec<_>>();

  // Get API models from DbService
  let api_models = state
    .app_service()
    .db_service()
    .list_api_model_aliases()
    .await
    .unwrap_or_else(|_| vec![])
    .into_iter()
    .map(|api_alias| api_model_to_oai_model(api_alias))
    .collect::<Vec<_>>();

  // Combine both lists
  models.extend(api_models);

  Ok(Json(OAIModelListResponse {
    object: "list".to_string(),
    data: models,
  }))
}

pub async fn oai_model_handler(
  State(state): State<Arc<dyn RouterState>>,
  Path(model_id): Path<String>,
) -> Result<Json<OAIModel>, ApiError> {
  // Try to find the model as an alias
  if let Some(alias) = state.app_service().data_service().find_alias(&model_id) {
    return Ok(Json(to_oai_model(state, alias)));
  }

  // Try API models
  if let Some(api_alias) = state
    .app_service()
    .db_service()
    .get_api_model_alias(&model_id)
    .await?
  {
    return Ok(Json(api_model_to_oai_model(api_alias)));
  }

  Err(ApiError::from(AliasNotFoundError(model_id)))
}
```

**Actions:**

1. Update handlers to work with `Alias` enum
2. Update conversion functions
3. Run `cargo check -p routes_oai`
4. Run `cargo test -p routes_oai`

### Phase 6: Remove UnifiedModelResponse from routes_app

**File: `crates/routes_app/src/objs.rs`**

Remove `UnifiedModelResponse` enum and update response types:

```rust
// Remove these lines:
// #[derive(Serialize, Deserialize, Debug, PartialEq, ToSchema)]
// #[serde(tag = "model_type")]
// pub enum UnifiedModelResponse {
//   #[serde(rename = "local")]
//   Local(AliasResponse),
//   #[serde(rename = "api")]
//   Api(crate::api_models_dto::ApiModelResponse),
// }
//
// impl From<AliasResponse> for UnifiedModelResponse { ... }
// impl From<crate::api_models_dto::ApiModelResponse> for UnifiedModelResponse { ... }

// Update to use Alias directly:
#[derive(Serialize, Deserialize, ToSchema)]
pub struct PaginatedAliasResponseNew {
  pub data: Vec<objs::Alias>, // Use Alias directly from objs crate
  pub total: usize,
  pub page: usize,
  pub page_size: usize,
}

// Keep AliasResponse for backwards compatibility if needed, or remove if not used
```

**File: `crates/routes_app/src/routes_models.rs`**

Update handlers to use `Alias` directly:

```rust
use objs::{Alias, ApiAlias, ...};

/// List all model aliases (both local aliases and API models)
#[utoipa::path(
    get,
    path = ENDPOINT_MODELS,
    tag = API_TAG_MODELS,
    operation_id = "listAllModels",
    params(PaginationSortParams),
    responses(
        (status = 200, description = "List of all configured models", body = PaginatedAliasResponseNew),
        (status = 500, description = "Internal server error", body = OpenAIApiError)
    ),
    security(("bearer_auth" = [])),
)]
pub async fn list_local_aliases_handler(
  State(state): State<Arc<dyn RouterState>>,
  Query(params): Query<PaginationSortParams>,
) -> Result<Json<PaginatedAliasResponseNew>, ApiError> {
  let (page, page_size, sort, sort_order) = extract_pagination_sort_params(params);

  // Get all aliases (now returns Vec<Alias>)
  let mut all_aliases = state.app_service().data_service().list_aliases()?;

  // Add API aliases
  let db_service = state.app_service().db_service();
  let api_aliases = db_service.list_api_model_aliases().await?;
  all_aliases.extend(api_aliases.into_iter().map(|a| Alias::Api(a)));

  // Sort aliases based on requested field
  sort_aliases(&mut all_aliases, &sort, &sort_order);

  let total = all_aliases.len();
  let (start, end) = calculate_pagination(page, page_size, total);
  let data = all_aliases.into_iter().skip(start).take(end - start).collect();

  Ok(Json(PaginatedAliasResponseNew {
    data,
    total,
    page,
    page_size,
  }))
}

fn sort_aliases(aliases: &mut Vec<Alias>, sort: &Option<String>, sort_order: &str) {
  match sort.as_deref() {
    Some("alias") | Some("name") => {
      aliases.sort_by(|a, b| {
        let cmp = a.alias_name().cmp(b.alias_name());
        if sort_order == "desc" { cmp.reverse() } else { cmp }
      });
    }
    Some("source") | Some("type") => {
      aliases.sort_by(|a, b| {
        let cmp = a.source().to_string().cmp(&b.source().to_string());
        if sort_order == "desc" { cmp.reverse() } else { cmp }
      });
    }
    _ => {
      // Default sort by alias name
      aliases.sort_by(|a, b| a.alias_name().cmp(b.alias_name()));
    }
  }
}
```

**Update OpenAPI Generation:**
Update `crates/routes_app/src/openapi.rs` to include the `Alias` schema from objs crate.

**Actions:**

1. Remove `UnifiedModelResponse` from objs.rs
2. Update response types to use `Alias`
3. Update all handlers in routes_models.rs
4. Update sorting and pagination logic
5. Update OpenAPI schema registration
6. Run `cargo check -p routes_app`
7. Run `cargo test -p routes_app`

### Phase 7: Update commands Crate

**File: `crates/commands/src/cmd_create.rs`**

Update to work with new `Alias` structure:

```rust
use objs::{Alias, UserAlias, ModelAlias, ...};

impl CreateCommand {
  pub async fn execute(&self, state: Arc<dyn RouterState>) -> Result<()> {
    // Check if alias already exists
    if let Some(existing) = state.app_service().data_service().find_alias(&self.alias) {
      match existing {
        Alias::User(_) => {
          return Err(anyhow!("User alias '{}' already exists", self.alias));
        }
        Alias::Model(_) => {
          return Err(anyhow!("Model alias '{}' already exists (auto-discovered)", self.alias));
        }
        Alias::Api(_) => {
          return Err(anyhow!("API alias '{}' already exists", self.alias));
        }
      }
    }

    // Create new UserAlias (only user can create UserAlias)
    let new_alias = UserAliasBuilder::default()
      .alias(self.alias.clone())
      .repo(self.repo.clone())
      .filename(self.filename.clone())
      .snapshot(self.snapshot.clone().unwrap_or_else(|| "main".to_string()))
      .source(AliasSource::User)
      .build()?;

    state.app_service().data_service().save_alias(&new_alias)?;

    println!("Created alias '{}' successfully", self.alias);
    Ok(())
  }
}
```

**File: `crates/commands/src/cmd_pull.rs`**

Similar updates for pull command and other commands that work with aliases.

**Actions:**

1. Review and update command implementations
2. Update error handling for different alias types
3. Run `cargo check -p commands`
4. Run `cargo test -p commands`

### Phase 8: Regenerate TypeScript Types

**Actions:**

1. Generate OpenAPI specification:
   ```bash
   cargo run --package xtask openapi
   ```
2. Navigate to ts-client:
   ```bash
   cd ts-client
   ```
3. Generate TypeScript types:
   ```bash
   npm run generate
   ```
4. Build the package:
   ```bash
   npm run build
   ```
5. Run tests:
   ```bash
   npm test
   ```

Expected changes in generated types:

- `UnifiedModelResponse` type will be removed
- New `Alias` discriminated union type with three variants
- `ModelAlias` interface with minimal fields
- Updated API response types

### Phase 9: Update Frontend

**File: `crates/bodhi/src/app/ui/models/page.tsx`**

Update to work with new `Alias` type:

```tsx
'use client';

import { useState } from 'react';
import { useRouter } from 'next/navigation';
import { DataTable, Pagination } from '@/components/DataTable';
import { TableCell } from '@/components/ui/table';
import { Alias, UserAlias, ModelAlias, ApiAlias } from '@bodhiapp/ts-client'; // Updated import
import { SortState } from '@/types/models';
// ... other imports ...

// Helper function to determine alias type
const getAliasType = (alias: Alias): 'user' | 'model' | 'api' => {
  // TypeScript discriminated union will handle this
  if ('models' in alias) return 'api';
  if ('context_params' in alias) return 'user';
  return 'model';
};

const SourceBadge = ({ alias }: { alias: Alias }) => {
  const type = getAliasType(alias);

  if (type === 'api') {
    return (
      <Badge variant="outline" className="bg-purple-500/10 text-purple-600 border-purple-200">
        <Cloud className="h-3 w-3 mr-1" />
        API
      </Badge>
    );
  }

  const colorClass = type === 'model' ? 'bg-green-500/10 text-green-500' : 'bg-blue-500/10 text-blue-500';

  return (
    <span className={`inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium w-fit ${colorClass}`}>
      {type}
    </span>
  );
};

// Update data handling
function ModelsPageContent() {
  const router = useRouter();
  const [page, setPage] = useState(1);
  const [pageSize] = useState(10);
  const [sort, setSort] = useState<SortState>({
    column: 'alias',
    direction: 'asc',
  });

  // Updated query hook - response.data is now Alias[]
  const {
    data: response,
    isLoading,
    error,
  } = useModels({
    page,
    pageSize,
    sort: sort.column,
    sortOrder: sort.direction,
  });

  // ... rest of component

  const renderAliasCell = (alias: Alias) => {
    const type = getAliasType(alias);

    if (type === 'api') {
      const apiAlias = alias as ApiAlias;
      return (
        <>
          <div className="font-medium">{apiAlias.id}</div>
          <div className="text-sm text-muted-foreground">
            {apiAlias.models.length} model{apiAlias.models.length !== 1 ? 's' : ''}
          </div>
        </>
      );
    }

    const localAlias = alias as UserAlias | ModelAlias;
    return (
      <>
        <div className="font-medium">{localAlias.alias}</div>
        <div className="text-sm text-muted-foreground">{localAlias.repo}</div>
      </>
    );
  };

  // ... rest of component
}
```

**File: `crates/bodhi/src/hooks/useQuery.ts`**

Update query hooks:

```tsx
import { Alias } from '@bodhiapp/ts-client';

export const useModels = (params: ModelQueryParams) => {
  return useQuery<PaginatedResponse<Alias>>({
    queryKey: ['models', params],
    queryFn: () => client.models.listAllModels(params),
  });
};
```

**File: `crates/bodhi/src/app/ui/chat/`** components

Update any chat components that use model information to work with the new `Alias` type structure.

**Actions:**

1. Update imports from `UnifiedModelResponse` to `Alias`
2. Update type checks and component logic
3. Update all affected components in `/app/ui/chat/`
4. Update hooks in `/hooks/useQuery.ts`
5. Test UI:
   ```bash
   cd crates/bodhi
   npm run dev
   ```
6. Run tests:
   ```bash
   npm test
   ```

### Phase 10: Integration Testing

**Actions:**

1. Run full test suite:
   ```bash
   make test
   ```
2. Run backend tests:
   ```bash
   make test.backend
   ```
3. Run frontend tests:
   ```bash
   make test.ui
   ```
4. Run NAPI tests:
   ```bash
   make test.napi
   ```
5. Test Tauri app:
   ```bash
   cd crates/bodhi/src-tauri
   cargo tauri dev
   ```
6. Manual testing checklist:
   - [ ] Create a new user alias via UI
   - [ ] Create a new user alias via CLI
   - [ ] List all models (should show user, model, and api types)
   - [ ] Delete a user alias
   - [ ] Copy a user alias
   - [ ] Create an API model configuration
   - [ ] Update API model configuration
   - [ ] Delete API model
   - [ ] Chat with a user-created alias
   - [ ] Chat with an auto-discovered model
   - [ ] Chat with an API model
   - [ ] Verify OpenAI API compatibility endpoints work
   - [ ] Test model selection in chat interface
   - [ ] Verify model filtering/sorting in models page

## Error Handling Considerations

### Phase-Specific Error Cases

1. **Alias Type Mismatches:**

   - Attempting to copy a ModelAlias or ApiAlias (only UserAlias can be copied)
   - Trying to save a ModelAlias (only UserAlias can be saved to disk)
   - Attempting to delete an auto-discovered ModelAlias

2. **Serialization/Deserialization:**

   - The `#[serde(untagged)]` attribute means deserialization will try variants in order
   - Need to ensure distinct schemas for each variant to avoid ambiguity

3. **Database Operations:**
   - API aliases are stored in database, not filesystem
   - Need to handle database unavailability gracefully

## Testing Strategy

### Unit Tests

Each crate will have updated unit tests:

- `objs`: Test new `ModelAlias` struct and updated `Alias` enum
- `services`: Test listing and finding different alias types
- `server_core`: Test model routing with all alias types
- `routes_*`: Test API endpoints with new types

### Integration Tests

- End-to-end tests for creating, listing, using aliases
- API compatibility tests
- Frontend interaction tests

### Manual Testing

Comprehensive manual testing checklist in Phase 10

## Rollback Strategy

If issues arise at any phase:

1. Git reset to previous commit
2. Review and fix the specific issue
3. Re-apply changes with fixes
4. Continue with remaining phases

Each phase is designed to be atomic and independently testable.

## Success Criteria

The refactoring is considered successful when:

1. All tests pass (`make test`)
2. TypeScript types are generated without errors
3. Frontend works with new types without runtime errors
4. All manual testing checklist items pass
5. No regression in existing functionality
6. Code is cleaner and more maintainable

## Timeline

Estimated time: 4-6 hours

- Phase 1-2: 1 hour (objs crate changes)
- Phase 3-4: 1 hour (services and server_core)
- Phase 5-6: 1 hour (routes updates)
- Phase 7-8: 1 hour (commands and TypeScript generation)
- Phase 9: 1 hour (frontend updates)
- Phase 10: 1 hour (integration testing)

## Notes for Resumption

If work is interrupted, to resume:

1. Check current phase completion with `cargo check -p <crate>`
2. Review git status to see modified files
3. Run tests for completed phases to ensure stability
4. Continue from the next uncompleted phase
5. Use this document's detailed instructions for each phase

## Conclusion

This refactoring will result in:

- Clear separation of concerns with three distinct alias types
- Better memory efficiency (ModelAlias has fewer fields)
- Improved type safety in both Rust and TypeScript
- Removal of redundant `UnifiedModelResponse` wrapper
- Better maintainability and extensibility
