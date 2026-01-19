# Toolset Multi-Instance: Complete ToolService Implementation

## Summary

Complete the multi-instance toolset implementation in `crates/services/src/tool_service.rs`. The trait has been updated with new method signatures, but the implementation still has old methods. This plan removes old methods, implements new ones, and replaces tests.

## Context

**Already Completed:**
- `objs`: `Toolset` struct, validation functions, removed `UserToolsetConfig*` types
- `services/db`: `ToolsetRow`, new DbService methods (`get_toolset`, `create_toolset`, etc.)
- `services/tool_service.rs`: Trait updated, error variants added, imports updated, `builtin_toolsets()` returns `Vec<ToolsetDefinition>`, `toolset_row_to_model()` added

**Remaining:**
- Remove old trait implementations that no longer match trait
- Implement new multi-instance methods
- Update `list_tools_for_user` and `list_all_toolsets` implementations
- Replace all tests

**Key Design Decision (from user):**
- Tool execution uses UUID: `/toolsets/{uuid}/execute/{method}`
- Frontend composes tool names as `toolset__{name}__{method}` for LLM
- Routes compilation breaking is acceptable (fixed in routes phase)

---

## Phase remove-old: Remove Old Implementations

### File: `crates/services/src/tool_service.rs`

**Remove these method implementations from `impl ToolService for DefaultToolService`:**

1. `list_all_toolsets` (lines ~391-451) - takes `user_id: Option<String>`, uses old tuple format
2. `get_user_toolset_config` (lines ~453-472)
3. `update_user_toolset_config` (lines ~474-529)
4. `delete_user_toolset_config` (lines ~531-550)
5. `execute_toolset_tool` (lines ~552-602)
6. `is_toolset_available_for_user` (lines ~604-624)

**Remove helper method:**
- `row_to_config` - no longer used

**Remove unused imports if any remain after removal.**

---

## Phase update-existing: Update Existing Implementations

### `list_tools_for_user`

```rust
async fn list_tools_for_user(&self, user_id: &str) -> Result<Vec<ToolDefinition>, ToolsetError> {
  let toolsets = self.db_service.list_toolsets(user_id).await?;

  // Collect unique toolset types that are enabled with API keys
  let enabled_types: std::collections::HashSet<String> = toolsets
    .into_iter()
    .filter(|t| t.enabled && t.encrypted_api_key.is_some())
    .map(|t| t.toolset_type)
    .collect();

  // Return tool definitions for enabled types
  let mut tools = Vec::new();
  for toolset_def in Self::builtin_toolsets() {
    if enabled_types.contains(&toolset_def.toolset_id) {
      tools.extend(toolset_def.tools);
    }
  }

  Ok(tools)
}
```

### `list_all_toolsets`

```rust
async fn list_all_toolsets(&self) -> Result<Vec<ToolsetWithTools>, ToolsetError> {
  let mut result = Vec::new();

  for toolset_def in Self::builtin_toolsets() {
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

---

## Phase impl-new: Implement New Trait Methods

### `list`

```rust
async fn list(&self, user_id: &str) -> Result<Vec<Toolset>, ToolsetError> {
  let rows = self.db_service.list_toolsets(user_id).await?;
  Ok(rows.into_iter().map(Self::toolset_row_to_model).collect())
}
```

### `get`

```rust
async fn get(&self, user_id: &str, id: &str) -> Result<Option<Toolset>, ToolsetError> {
  let row = self.db_service.get_toolset(id).await?;

  // Return None if not found OR user doesn't own it (hide existence)
  Ok(match row {
    Some(r) if r.user_id == user_id => Some(Self::toolset_row_to_model(r)),
    _ => None,
  })
}
```

### `create`

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
  if self.db_service.get_toolset_by_name(user_id, name).await?.is_some() {
    return Err(ToolsetError::NameExists(name.to_string()));
  }

  // Encrypt API key
  let (encrypted, salt, nonce) = encrypt_api_key(self.db_service.encryption_key(), &api_key)
    .map_err(|e| DbError::EncryptionError(e.to_string()))?;

  // Create row
  let now = self.time_service.utc_now().timestamp();
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

### `update`

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
    if self.db_service.get_toolset_by_name(user_id, name).await?.is_some() {
      return Err(ToolsetError::NameExists(name.to_string()));
    }
  }

  // Check app-level enabled if trying to enable
  if enabled && !self.is_type_enabled(&existing.toolset_type).await? {
    return Err(ToolsetError::ToolsetAppDisabled);
  }

  // Handle API key update
  let (encrypted, salt, nonce, db_api_key_update) = match api_key_update {
    ApiKeyUpdate::Keep => (
      existing.encrypted_api_key,
      existing.salt,
      existing.nonce,
      ApiKeyUpdate::Keep,
    ),
    ApiKeyUpdate::Set(Some(new_key)) => {
      let (enc, s, n) = encrypt_api_key(self.db_service.encryption_key(), &new_key)
        .map_err(|e| DbError::EncryptionError(e.to_string()))?;
      (Some(enc.clone()), Some(s.clone()), Some(n.clone()), ApiKeyUpdate::Set(Some(enc)))
    }
    ApiKeyUpdate::Set(None) => (None, None, None, ApiKeyUpdate::Set(None)),
  };

  // Update row
  let now = self.time_service.utc_now().timestamp();
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

  let result = self.db_service.update_toolset(&row, db_api_key_update).await?;
  Ok(Self::toolset_row_to_model(result))
}
```

### `delete`

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

### `execute`

```rust
async fn execute(
  &self,
  user_id: &str,
  id: &str,
  method: &str,
  request: ToolsetExecutionRequest,
) -> Result<ToolsetExecutionResponse, ToolsetError> {
  // Fetch instance and verify ownership
  let instance = self.db_service.get_toolset(id).await?
    .ok_or_else(|| ToolsetError::ToolsetNotFound(id.to_string()))?;

  if instance.user_id != user_id {
    return Err(ToolsetError::ToolsetNotFound(id.to_string()));
  }

  // Check app-level type enabled
  if !self.is_type_enabled(&instance.toolset_type).await? {
    return Err(ToolsetError::ToolsetAppDisabled);
  }

  // Check instance enabled
  if !instance.enabled {
    return Err(ToolsetError::ToolsetDisabled);
  }

  // Get decrypted API key
  let api_key = self.db_service.get_toolset_api_key(id).await?
    .ok_or(ToolsetError::ToolsetNotConfigured)?;

  // Verify method exists for toolset type
  let toolset_def = self.get_type(&instance.toolset_type)
    .ok_or_else(|| ToolsetError::ToolsetNotFound(instance.toolset_type.clone()))?;

  if !toolset_def.tools.iter().any(|t| t.function.name == method) {
    return Err(ToolsetError::MethodNotFound(format!(
      "Method '{}' not found in toolset type '{}'",
      method, instance.toolset_type
    )));
  }

  // Execute based on toolset type
  match instance.toolset_type.as_str() {
    "builtin-exa-web-search" => self.execute_exa_method(&api_key, method, request).await,
    _ => Err(ToolsetError::ToolsetNotFound(instance.toolset_type)),
  }
}
```

### `is_available`

```rust
async fn is_available(&self, user_id: &str, id: &str) -> Result<bool, ToolsetError> {
  let instance = self.db_service.get_toolset(id).await?;

  Ok(match instance {
    Some(i) if i.user_id == user_id => {
      i.enabled
        && i.encrypted_api_key.is_some()
        && self.is_type_enabled(&i.toolset_type).await.unwrap_or(false)
    }
    _ => false,
  })
}
```

### `list_types`

```rust
fn list_types(&self) -> Vec<ToolsetDefinition> {
  Self::builtin_toolsets()
}
```

### `get_type`

```rust
fn get_type(&self, toolset_type: &str) -> Option<ToolsetDefinition> {
  Self::builtin_toolsets()
    .into_iter()
    .find(|def| def.toolset_id == toolset_type)
}
```

### `validate_type`

```rust
fn validate_type(&self, toolset_type: &str) -> Result<(), ToolsetError> {
  if self.get_type(toolset_type).is_some() {
    Ok(())
  } else {
    Err(ToolsetError::InvalidToolsetType(toolset_type.to_string()))
  }
}
```

### `is_type_enabled`

```rust
async fn is_type_enabled(&self, toolset_type: &str) -> Result<bool, ToolsetError> {
  self.is_toolset_enabled_for_app(toolset_type).await
}
```

---

## Phase replace-tests: Replace Tests

Delete entire `#[cfg(test)] mod tests` block and replace with new tests. Tests use mocks (MockDbService, MockTimeService, MockExaService) with explicit imports.

### Test Structure Pattern

```rust
#[cfg(test)]
mod tests {
  use crate::db::{
    ApiKeyUpdate, AppToolsetConfigRow, MockDbService, MockTimeService, ToolsetRow,
  };
  use crate::exa_service::MockExaService;
  use crate::tool_service::{DefaultToolService, ToolService, ToolsetError};
  use chrono::{TimeZone, Utc};
  use mockall::predicate::eq;
  use rstest::rstest;
  use std::sync::Arc;

  // Helper to create test toolset row
  fn test_toolset_row(id: &str, user_id: &str, name: &str) -> ToolsetRow {
    ToolsetRow {
      id: id.to_string(),
      user_id: user_id.to_string(),
      toolset_type: "builtin-exa-web-search".to_string(),
      name: name.to_string(),
      description: Some("Test toolset".to_string()),
      enabled: true,
      encrypted_api_key: Some("encrypted".to_string()),
      salt: Some("salt".to_string()),
      nonce: Some("nonce".to_string()),
      created_at: 1700000000,
      updated_at: 1700000000,
    }
  }

  fn app_config_enabled() -> AppToolsetConfigRow {
    AppToolsetConfigRow {
      id: 1,
      toolset_id: "builtin-exa-web-search".to_string(),
      enabled: true,
      updated_by: "admin".to_string(),
      created_at: 1700000000,
      updated_at: 1700000000,
    }
  }

  // ... tests follow
}
```

### Complete Test Coverage

**Static Method Tests:**
```rust
#[rstest]
fn test_list_all_tool_definitions() {
  // Verify builtin_tool_definitions returns expected tool
  // Assert: 1 tool with name "builtin-exa-web-search"
}

#[rstest]
fn test_list_types_returns_builtin_toolsets() {
  // list_types returns Vec<ToolsetDefinition>
  // Assert: contains "builtin-exa-web-search" with 4 tools (search, findSimilar, contents, answer)
}

#[rstest]
fn test_get_type_returns_toolset_definition() {
  // get_type("builtin-exa-web-search") returns Some
  // Assert: name, description, tools match expected
}

#[rstest]
fn test_get_type_returns_none_for_unknown() {
  // get_type("unknown") returns None
}

#[rstest]
fn test_validate_type_success() {
  // validate_type("builtin-exa-web-search") returns Ok
}

#[rstest]
fn test_validate_type_fails_for_unknown() {
  // validate_type("unknown") returns Err(InvalidToolsetType)
}
```

**list_tools_for_user Tests:**
```rust
#[rstest]
#[tokio::test]
async fn test_list_tools_for_user_returns_tools_for_enabled_instances() {
  // Mock: list_toolsets returns 2 instances, 1 enabled with key, 1 disabled
  // Assert: returns tools for enabled type only
}

#[rstest]
#[tokio::test]
async fn test_list_tools_for_user_returns_empty_when_no_instances() {
  // Mock: list_toolsets returns empty
  // Assert: empty vec
}

#[rstest]
#[tokio::test]
async fn test_list_tools_for_user_deduplicates_same_type() {
  // Mock: list_toolsets returns 2 instances of same type both enabled
  // Assert: returns tools once (not duplicated)
}
```

**list_all_toolsets Tests:**
```rust
#[rstest]
#[tokio::test]
async fn test_list_all_toolsets_returns_toolsets_with_app_status() {
  // Mock: get_app_toolset_config returns enabled config
  // Assert: returns ToolsetWithTools with app_enabled=true
}

#[rstest]
#[tokio::test]
async fn test_list_all_toolsets_shows_disabled_when_no_config() {
  // Mock: get_app_toolset_config returns None
  // Assert: app_enabled=false
}
```

**list Tests:**
```rust
#[rstest]
#[tokio::test]
async fn test_list_returns_user_toolsets() {
  // Mock: list_toolsets returns 2 rows
  // Assert: returns 2 Toolset models with correct fields
}

#[rstest]
#[tokio::test]
async fn test_list_returns_empty_for_user_with_no_toolsets() {
  // Mock: list_toolsets returns empty
  // Assert: empty vec
}
```

**get Tests:**
```rust
#[rstest]
#[tokio::test]
async fn test_get_returns_owned_toolset() {
  // Mock: get_toolset returns row with matching user_id
  // Assert: returns Some(Toolset)
}

#[rstest]
#[tokio::test]
async fn test_get_returns_none_for_other_users_toolset() {
  // Mock: get_toolset returns row with different user_id
  // Assert: returns None (hide existence)
}

#[rstest]
#[tokio::test]
async fn test_get_returns_none_when_not_found() {
  // Mock: get_toolset returns None
  // Assert: returns None
}
```

**create Tests:**
```rust
#[rstest]
#[tokio::test]
async fn test_create_success() {
  // Mock: get_toolset_by_name returns None, get_app_toolset_config returns enabled
  // Mock: create_toolset returns row, time_service returns timestamp
  // Assert: returns Toolset with UUID, correct fields
}

#[rstest]
#[tokio::test]
async fn test_create_fails_when_name_already_exists() {
  // Mock: get_toolset_by_name returns Some
  // Assert: Err(NameExists)
}

#[rstest]
#[tokio::test]
async fn test_create_fails_with_invalid_name_empty() {
  // No mocks needed - validation fails first
  // Assert: Err(InvalidName) for empty name
}

#[rstest]
#[tokio::test]
async fn test_create_fails_with_invalid_name_too_long() {
  // Assert: Err(InvalidName) for 25+ char name
}

#[rstest]
#[tokio::test]
async fn test_create_fails_with_invalid_name_special_chars() {
  // Assert: Err(InvalidName) for "my_toolset" (underscore not allowed)
}

#[rstest]
#[tokio::test]
async fn test_create_fails_with_invalid_toolset_type() {
  // Assert: Err(InvalidToolsetType) for unknown type
}

#[rstest]
#[tokio::test]
async fn test_create_fails_when_app_disabled() {
  // Mock: get_app_toolset_config returns disabled or None
  // Assert: Err(ToolsetAppDisabled)
}

#[rstest]
#[tokio::test]
async fn test_create_same_name_different_user_succeeds() {
  // Mock: get_toolset_by_name(user2, name) returns None
  // Assert: success - names unique per user only
}
```

**update Tests:**
```rust
#[rstest]
#[tokio::test]
async fn test_update_success() {
  // Mock: get_toolset returns owned row, update_toolset returns updated
  // Assert: returns updated Toolset
}

#[rstest]
#[tokio::test]
async fn test_update_fails_when_not_found() {
  // Mock: get_toolset returns None
  // Assert: Err(ToolsetNotFound)
}

#[rstest]
#[tokio::test]
async fn test_update_fails_when_not_owned() {
  // Mock: get_toolset returns row with different user_id
  // Assert: Err(ToolsetNotFound) - hide existence
}

#[rstest]
#[tokio::test]
async fn test_update_fails_when_name_conflicts() {
  // Mock: get_toolset returns owned, get_toolset_by_name returns different row
  // Assert: Err(NameExists)
}

#[rstest]
#[tokio::test]
async fn test_update_same_name_different_case_succeeds() {
  // Current name "MyToolset", update to "mytoolset"
  // Mock: get_toolset_by_name returns same row (case-insensitive)
  // Assert: success - same name different case is allowed
}

#[rstest]
#[tokio::test]
async fn test_update_enable_fails_when_app_disabled() {
  // Mock: get_toolset returns disabled instance, enabling it
  // Mock: get_app_toolset_config returns disabled
  // Assert: Err(ToolsetAppDisabled)
}

#[rstest]
#[tokio::test]
async fn test_update_with_api_key_set() {
  // Mock with ApiKeyUpdate::Set(Some(key))
  // Assert: encryption called, update succeeds
}

#[rstest]
#[tokio::test]
async fn test_update_with_api_key_keep() {
  // Mock with ApiKeyUpdate::Keep
  // Assert: existing encrypted key preserved
}
```

**delete Tests:**
```rust
#[rstest]
#[tokio::test]
async fn test_delete_success() {
  // Mock: get_toolset returns owned, delete_toolset succeeds
  // Assert: Ok(())
}

#[rstest]
#[tokio::test]
async fn test_delete_fails_when_not_found() {
  // Mock: get_toolset returns None
  // Assert: Err(ToolsetNotFound)
}

#[rstest]
#[tokio::test]
async fn test_delete_fails_when_not_owned() {
  // Mock: get_toolset returns row with different user_id
  // Assert: Err(ToolsetNotFound)
}

#[rstest]
#[tokio::test]
async fn test_delete_succeeds_even_when_app_disabled() {
  // Mock: get_toolset returns owned, app config disabled
  // Assert: Ok(()) - can delete disabled type instances
}
```

**execute Tests:**
```rust
#[rstest]
#[tokio::test]
async fn test_execute_search_success() {
  // Mock: get_toolset returns enabled owned instance
  // Mock: get_app_toolset_config returns enabled
  // Mock: get_toolset_api_key returns key
  // Mock: exa_service.search returns results
  // Assert: ToolsetExecutionResponse with results
}

#[rstest]
#[tokio::test]
async fn test_execute_fails_when_not_found() {
  // Mock: get_toolset returns None
  // Assert: Err(ToolsetNotFound)
}

#[rstest]
#[tokio::test]
async fn test_execute_fails_when_not_owned() {
  // Mock: get_toolset returns row with different user_id
  // Assert: Err(ToolsetNotFound)
}

#[rstest]
#[tokio::test]
async fn test_execute_fails_when_app_disabled() {
  // Mock: get_toolset returns owned, get_app_toolset_config returns disabled
  // Assert: Err(ToolsetAppDisabled)
}

#[rstest]
#[tokio::test]
async fn test_execute_fails_when_instance_disabled() {
  // Mock: get_toolset returns owned but disabled instance
  // Assert: Err(ToolsetDisabled)
}

#[rstest]
#[tokio::test]
async fn test_execute_fails_when_method_not_found() {
  // Mock: get_toolset returns enabled owned instance, method="unknown"
  // Assert: Err(MethodNotFound)
}

#[rstest]
#[tokio::test]
async fn test_execute_fails_when_no_api_key() {
  // Mock: get_toolset returns enabled, get_toolset_api_key returns None
  // Assert: Err(ToolsetNotConfigured)
}
```

**is_available Tests:**
```rust
#[rstest]
#[tokio::test]
async fn test_is_available_true_when_all_conditions_met() {
  // Mock: get_toolset returns enabled owned with api_key
  // Mock: get_app_toolset_config returns enabled
  // Assert: true
}

#[rstest]
#[tokio::test]
async fn test_is_available_false_when_not_enabled() {
  // Mock: get_toolset returns owned but disabled
  // Assert: false
}

#[rstest]
#[tokio::test]
async fn test_is_available_false_when_no_api_key() {
  // Mock: get_toolset returns enabled but no encrypted_api_key
  // Assert: false
}

#[rstest]
#[tokio::test]
async fn test_is_available_false_when_app_disabled() {
  // Mock: get_toolset returns enabled, get_app_toolset_config disabled
  // Assert: false
}

#[rstest]
#[tokio::test]
async fn test_is_available_false_when_not_owned() {
  // Mock: get_toolset returns row with different user_id
  // Assert: false
}

#[rstest]
#[tokio::test]
async fn test_is_available_false_when_not_found() {
  // Mock: get_toolset returns None
  // Assert: false
}
```

**is_type_enabled Tests:**
```rust
#[rstest]
#[tokio::test]
async fn test_is_type_enabled_true() {
  // Mock: get_app_toolset_config returns enabled
  // Assert: true
}

#[rstest]
#[tokio::test]
async fn test_is_type_enabled_false_when_disabled() {
  // Mock: get_app_toolset_config returns disabled
  // Assert: false
}

#[rstest]
#[tokio::test]
async fn test_is_type_enabled_false_when_no_config() {
  // Mock: get_app_toolset_config returns None
  // Assert: false (no config = disabled)
}
```

**App-client Tests (keep existing):**
```rust
#[rstest]
#[tokio::test]
async fn test_is_app_client_registered_for_toolset_returns_true_when_registered() { ... }

#[rstest]
#[tokio::test]
async fn test_is_app_client_registered_for_toolset_returns_false_when_toolset_not_in_config() { ... }

#[rstest]
#[tokio::test]
async fn test_is_app_client_registered_for_toolset_returns_false_when_no_config() { ... }
```

### Test Count Summary

| Category | Old Tests | New Tests |
|----------|-----------|-----------|
| Static/Type methods | 1 | 6 |
| list_tools_for_user | 1 | 3 |
| list_all_toolsets | 0 (combined) | 2 |
| list | 0 | 2 |
| get | 0 | 3 |
| create | 0 | 8 |
| update | 0 | 8 |
| delete | 0 | 4 |
| execute | 0 | 7 |
| is_available | 5 (old API) | 6 |
| is_type_enabled | 0 | 3 |
| App-client | 3 | 3 |
| **Total** | **10** | **55** |

---

## Files Modified

| File | Action |
|------|--------|
| `crates/services/src/tool_service.rs` | Remove old impls, add new impls, replace tests |

---

## Verification

After implementation:
```bash
cargo fmt --all
cargo test -p services --lib tool_service
cargo test -p services
make test.backend
```

**Note:** `cargo test -p routes_app` will fail due to removed types - expected, fixed in routes phase.

---

## Out of Scope

- `routes_app` crate updates (separate phase)
- Frontend changes
- `app_toolset_configs` table (unchanged)
- `app_client_toolset_configs` table (unchanged)
