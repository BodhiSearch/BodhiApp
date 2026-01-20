# Toolset Multi-Instance: Remaining Implementation Work

## Completed Phases ✅

- **Phase objs-types**: Domain types updated (Toolset, validation functions added)
- **Phase services-schema**: Database migration updated (toolsets table with UUID, name uniqueness)
- **Phase services-row**: ToolsetRow structure renamed and updated
- **Phase services-db**: DbService methods implemented (get_toolset, create_toolset, update_toolset, etc.)
- **Phase services-tool**: PARTIALLY COMPLETE
  - Error variants added (NameExists, InvalidName, InvalidToolsetType)
  - Error messages added to en-US/messages.ftl
  - Imports updated
  - ToolService trait updated with new method signatures
  - `builtin_toolsets()` converted to return `Vec<ToolsetDefinition>`
  - Helper function `toolset_row_to_model()` added

## Remaining Work in Phase services-tool

### File: crates/services/src/tool_service.rs

#### 1. Remove Old Trait Implementations (lines ~450-621)

Methods to remove:
- `get_user_toolset_config` (lines 450-469)
- `update_user_toolset_config` (lines 471-526)
- `delete_user_toolset_config` (lines 528-547)
- `execute_toolset_tool` (lines 549-599)
- `is_toolset_available_for_user` (lines 601-621)

#### 2. Update Existing Methods

**list_tools_for_user** (line 364):
```rust
async fn list_tools_for_user(&self, user_id: &str) -> Result<Vec<ToolDefinition>, ToolsetError> {
  // Get user's enabled toolsets
  let toolsets = self.db_service.list_toolsets(user_id).await?;

  // Filter to only enabled toolsets with API keys
  let enabled_types: Vec<String> = toolsets
    .into_iter()
    .filter(|t| t.enabled && t.encrypted_api_key.is_some())
    .map(|t| t.toolset_type)
    .collect();

  // Get tool definitions for enabled types
  let mut tools = Vec::new();
  for toolset_def in Self::builtin_toolsets() {
    if enabled_types.contains(&toolset_def.toolset_id) {
      tools.extend(toolset_def.tools);
    }
  }

  Ok(tools)
}
```

**list_all_toolsets** (line 388):
```rust
async fn list_all_toolsets(&self) -> Result<Vec<ToolsetWithTools>, ToolsetError> {
  let mut result = Vec::new();

  for toolset_def in Self::builtin_toolsets() {
    // Get app-level enabled status
    let app_enabled = self
      .is_toolset_enabled_for_app(&toolset_def.toolset_id)
      .await
      .unwrap_or(false);

    result.push(ToolsetWithTools {
      toolset_id: toolset_def.toolset_id,
      name: toolset_def.name,
      description: toolset_def.description,
      app_enabled,
      tools: toolset_def.tools,
    });
  }

  Ok(result)
}
```

#### 3. Add New Trait Implementations

**list** - List all toolset instances for user:
```rust
async fn list(&self, user_id: &str) -> Result<Vec<Toolset>, ToolsetError> {
  let rows = self.db_service.list_toolsets(user_id).await?;
  Ok(rows.into_iter().map(Self::toolset_row_to_model).collect())
}
```

**get** - Get specific toolset:
```rust
async fn get(&self, user_id: &str, id: &str) -> Result<Option<Toolset>, ToolsetError> {
  let row = self.db_service.get_toolset(id).await?;

  // Return None if not found OR if user doesn't own it (hide existence)
  match row {
    Some(r) if r.user_id == user_id => Ok(Some(Self::toolset_row_to_model(r))),
    _ => Ok(None),
  }
}
```

**create** - Create new toolset:
```rust
async fn create(
  &self,
  user_id: &str,
  toolset_type: &str,
  name: &str,
  description: Option<String>,
  enabled: bool,
  api_key: String,
) -> Result<Toolset, ToolsetError> {
  // Validate name format
  objs::validate_toolset_name(name)
    .map_err(|reason| ToolsetError::InvalidName(reason))?;

  // Validate description if provided
  if let Some(ref desc) = description {
    objs::validate_toolset_description(desc)
      .map_err(|reason| ToolsetError::InvalidName(reason))?;
  }

  // Validate toolset type exists
  self.validate_type(toolset_type)?;

  // Check app-level enabled
  if !self.is_type_enabled(toolset_type).await? {
    return Err(ToolsetError::ToolsetAppDisabled);
  }

  // Check name uniqueness (case-insensitive)
  if let Some(_) = self.db_service.get_toolset_by_name(user_id, name).await? {
    return Err(ToolsetError::NameExists(name.to_string()));
  }

  // Encrypt API key
  let (encrypted, salt, nonce) = encrypt_api_key(&api_key, &self.db_service.secret_key())?;

  // Create row
  let now = self.time_service.now().timestamp();
  let row = ToolsetRow {
    id: Uuid::new_v4().to_string(),
    user_id: user_id.to_string(),
    toolset_type: toolset_type.to_string(),
    name: name.to_string(),
    description,
    enabled,
    encrypted_api_key: Some(encrypted),
    salt: Some(salt),
    nonce: Some(nonce),
    created_at: now,
    updated_at: now,
  };

  let result = self.db_service.create_toolset(&row).await?;
  Ok(Self::toolset_row_to_model(result))
}
```

**update** - Update existing toolset:
```rust
async fn update(
  &self,
  user_id: &str,
  id: &str,
  name: &str,
  description: Option<String>,
  enabled: bool,
  api_key_update: ApiKeyUpdate,
) -> Result<Toolset, ToolsetError> {
  // Fetch existing
  let existing = self.db_service.get_toolset(id).await?
    .ok_or_else(|| ToolsetError::ToolsetNotFound(id.to_string()))?;

  // Verify ownership
  if existing.user_id != user_id {
    return Err(ToolsetError::ToolsetNotFound(id.to_string()));
  }

  // Validate name format
  objs::validate_toolset_name(name)
    .map_err(|reason| ToolsetError::InvalidName(reason))?;

  // Validate description if provided
  if let Some(ref desc) = description {
    objs::validate_toolset_description(desc)
      .map_err(|reason| ToolsetError::InvalidName(reason))?;
  }

  // Check name uniqueness if changed (case-insensitive)
  if name.to_lowercase() != existing.name.to_lowercase() {
    if let Some(_) = self.db_service.get_toolset_by_name(user_id, name).await? {
      return Err(ToolsetError::NameExists(name.to_string()));
    }
  }

  // Check app-level enabled if trying to enable
  if enabled {
    if !self.is_type_enabled(&existing.toolset_type).await? {
      return Err(ToolsetError::ToolsetAppDisabled);
    }
  }

  // Handle API key update
  let (encrypted, salt, nonce, api_key_update_param) = match api_key_update {
    ApiKeyUpdate::Keep => (existing.encrypted_api_key, existing.salt, existing.nonce, ApiKeyUpdate::Keep),
    ApiKeyUpdate::Set(Some(new_key)) => {
      let (enc, s, n) = encrypt_api_key(&new_key, &self.db_service.secret_key())?;
      (Some(enc.clone()), Some(s.clone()), Some(n.clone()), ApiKeyUpdate::Set(Some(enc)))
    },
    ApiKeyUpdate::Set(None) => (None, None, None, ApiKeyUpdate::Set(None)),
  };

  // Update row
  let now = self.time_service.now().timestamp();
  let row = ToolsetRow {
    id: id.to_string(),
    user_id: user_id.to_string(),
    toolset_type: existing.toolset_type,
    name: name.to_string(),
    description,
    enabled,
    encrypted_api_key: encrypted,
    salt,
    nonce,
    created_at: existing.created_at,
    updated_at: now,
  };

  let result = self.db_service.update_toolset(&row, api_key_update_param).await?;
  Ok(Self::toolset_row_to_model(result))
}
```

**delete** - Delete toolset:
```rust
async fn delete(&self, user_id: &str, id: &str) -> Result<(), ToolsetError> {
  // Fetch and verify ownership
  let existing = self.db_service.get_toolset(id).await?
    .ok_or_else(|| ToolsetError::ToolsetNotFound(id.to_string()))?;

  if existing.user_id != user_id {
    return Err(ToolsetError::ToolsetNotFound(id.to_string()));
  }

  // Allow delete even if app-level type disabled
  self.db_service.delete_toolset(id).await?;
  Ok(())
}
```

**execute** - Execute tool on toolset:
```rust
async fn execute(
  &self,
  user_id: &str,
  id: &str,
  method: &str,
  request: ToolsetExecutionRequest,
) -> Result<ToolsetExecutionResponse, ToolsetError> {
  // Fetch toolset and verify ownership
  let toolset = self.db_service.get_toolset(id).await?
    .ok_or_else(|| ToolsetError::ToolsetNotFound(id.to_string()))?;

  if toolset.user_id != user_id {
    return Err(ToolsetError::ToolsetNotFound(id.to_string()));
  }

  // Check app-level type enabled
  if !self.is_type_enabled(&toolset.toolset_type).await? {
    return Err(ToolsetError::ToolsetAppDisabled);
  }

  // Check toolset enabled
  if !toolset.enabled {
    return Err(ToolsetError::ToolsetDisabled);
  }

  // Get decrypted API key
  let api_key = self.db_service.get_toolset_api_key(id).await?
    .ok_or(ToolsetError::ToolsetNotConfigured)?;

  // Execute via Exa service
  match toolset.toolset_type.as_str() {
    "builtin-exa-web-search" => {
      self.exa_service.execute_tool(&api_key, method, &request.params)
        .await
        .map(|result| ToolsetExecutionResponse {
          tool_call_id: request.tool_call_id,
          result: Some(result),
          error: None,
        })
        .map_err(|e| e.into())
    },
    _ => Err(ToolsetError::ToolsetNotFound(toolset.toolset_type)),
  }
}
```

**is_available** - Check if toolset is available:
```rust
async fn is_available(&self, user_id: &str, id: &str) -> Result<bool, ToolsetError> {
  let toolset = self.db_service.get_toolset(id).await?;

  Ok(match toolset {
    Some(i) if i.user_id == user_id => {
      i.enabled
        && i.encrypted_api_key.is_some()
        && self.is_type_enabled(&i.toolset_type).await.unwrap_or(false)
    },
    _ => false,
  })
}
```

**Type management methods**:
```rust
fn list_types(&self) -> Vec<ToolsetDefinition> {
  Self::builtin_toolsets()
}

fn get_type(&self, toolset_type: &str) -> Option<ToolsetDefinition> {
  Self::builtin_toolsets()
    .into_iter()
    .find(|def| def.toolset_id == toolset_type)
}

fn validate_type(&self, toolset_type: &str) -> Result<(), ToolsetError> {
  if self.get_type(toolset_type).is_some() {
    Ok(())
  } else {
    Err(ToolsetError::InvalidToolsetType(toolset_type.to_string()))
  }
}

async fn is_type_enabled(&self, toolset_type: &str) -> Result<bool, ToolsetError> {
  self.is_toolset_enabled_for_app(toolset_type).await
}
```

## Next Steps

1. Apply all changes to tool_service.rs
2. Run `cargo fmt --all`
3. Run `cargo test -p services` to see what tests need updating
4. Update/replace tests in Phase services-tests
5. Final compilation and test verification

## Notes

- Need to import `encrypt_api_key` in tool_service.rs
- Need to handle the secret_key access (may need to add method to DbService)
- All ownership checks return NotFound instead of permission error to hide existence
