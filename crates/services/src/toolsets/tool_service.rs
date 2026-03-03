use super::exa_service::ExaService;
use super::toolset_entity::ToolsetEntity;
use super::toolset_objs::{
  AppToolsetConfig, FunctionDefinition, ToolDefinition, ToolsetDefinition, ToolsetExecutionRequest,
  ToolsetExecutionResponse, ToolsetRequest,
};
use super::ToolsetError;
use crate::db::{encryption::encrypt_api_key, DbError, DbService, TimeService};
use crate::RawApiKeyUpdate;
use serde_json::json;
use std::fmt::Debug;
use std::sync::Arc;
use ulid::Ulid;

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait ToolService: Debug + Send + Sync {
  /// List tool definitions for user (only configured and enabled toolsets)
  async fn list_tools_for_user(
    &self,
    tenant_id: &str,
    user_id: &str,
  ) -> Result<Vec<ToolDefinition>, ToolsetError>;

  /// List all available toolsets with their tools (nested structure)
  async fn list_all_toolsets(&self) -> Result<Vec<ToolsetDefinition>, ToolsetError>;

  // ============================================================================
  // Toolset instance management
  // ============================================================================

  /// List all toolset instances for user
  async fn list(&self, tenant_id: &str, user_id: &str) -> Result<Vec<ToolsetEntity>, ToolsetError>;

  /// Get a specific toolset instance by ID
  async fn get(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
  ) -> Result<Option<ToolsetEntity>, ToolsetError>;

  /// Create a new toolset instance
  async fn create(
    &self,
    tenant_id: &str,
    user_id: &str,
    request: ToolsetRequest,
  ) -> Result<ToolsetEntity, ToolsetError>;

  /// Update an existing toolset instance
  async fn update(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
    request: ToolsetRequest,
  ) -> Result<ToolsetEntity, ToolsetError>;

  /// Delete a toolset instance
  async fn delete(&self, tenant_id: &str, user_id: &str, id: &str) -> Result<(), ToolsetError>;

  /// Execute a tool on a toolset instance
  async fn execute(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
    method: &str,
    request: ToolsetExecutionRequest,
  ) -> Result<ToolsetExecutionResponse, ToolsetError>;

  // ============================================================================
  // Toolset type management
  // ============================================================================

  /// List all available toolset types
  fn list_types(&self) -> Vec<ToolsetDefinition>;

  /// Get toolset type definition by toolset_type
  fn get_type(&self, toolset_type: &str) -> Option<ToolsetDefinition>;

  /// Validate toolset toolset_type exists
  fn validate_type(&self, toolset_type: &str) -> Result<(), ToolsetError>;

  /// Check if toolset type is enabled at app level
  async fn is_type_enabled(
    &self,
    tenant_id: &str,
    toolset_type: &str,
  ) -> Result<bool, ToolsetError>;

  // ============================================================================
  // App-level toolset type configuration (Admin only)
  // ============================================================================

  /// Set app-level enable/disable for a toolset type
  async fn set_app_toolset_enabled(
    &self,
    tenant_id: &str,
    toolset_type: &str,
    enabled: bool,
    updated_by: &str,
  ) -> Result<AppToolsetConfig, ToolsetError>;

  /// List all app-level toolset configurations
  async fn list_app_toolset_configs(
    &self,
    tenant_id: &str,
  ) -> Result<Vec<AppToolsetConfig>, ToolsetError>;

  /// Get app-level config for specific toolset type
  async fn get_app_toolset_config(
    &self,
    tenant_id: &str,
    toolset_type: &str,
  ) -> Result<Option<AppToolsetConfig>, ToolsetError>;
}

// ============================================================================
// DefaultToolService - Implementation
// ============================================================================

#[derive(Debug)]
pub struct DefaultToolService {
  db_service: Arc<dyn DbService>,
  exa_service: Arc<dyn ExaService>,
  time_service: Arc<dyn TimeService>,
}

impl DefaultToolService {
  pub fn new(
    db_service: Arc<dyn DbService>,
    exa_service: Arc<dyn ExaService>,
    time_service: Arc<dyn TimeService>,
  ) -> Self {
    Self {
      db_service,
      exa_service,
      time_service,
    }
  }

  /// Static registry of built-in toolsets with nested tools
  fn builtin_toolsets() -> Vec<ToolsetDefinition> {
    vec![ToolsetDefinition {
      toolset_type: "builtin-exa-search".to_string(),
      name: "Exa Web Search".to_string(),
      description: "Search and analyze web content using Exa AI".to_string(),
      tools: vec![
        ToolDefinition {
          tool_type: "function".to_string(),
          function: FunctionDefinition {
            name: "search".to_string(),
            description: "Search the web for current information using Exa AI semantic search. Returns relevant web pages with titles, URLs, and content snippets.".to_string(),
            parameters: json!({
              "type": "object",
              "properties": {
                "query": {
                  "type": "string",
                  "description": "The search query to find relevant web pages"
                },
                "num_results": {
                  "type": "integer",
                  "description": "Number of results to return (default: 5, max: 10)",
                  "minimum": 1,
                  "maximum": 10,
                  "default": 5
                }
              },
              "required": ["query"]
            }),
          },
        },
        ToolDefinition {
          tool_type: "function".to_string(),
          function: FunctionDefinition {
            name: "findSimilar".to_string(),
            description: "Find web pages similar to a given URL using Exa AI. Returns pages with similar content and topics.".to_string(),
            parameters: json!({
              "type": "object",
              "properties": {
                "url": {
                  "type": "string",
                  "description": "The URL to find similar pages for"
                },
                "num_results": {
                  "type": "integer",
                  "description": "Number of results to return (default: 5, max: 10)",
                  "minimum": 1,
                  "maximum": 10,
                  "default": 5
                }
              },
              "required": ["url"]
            }),
          },
        },
        ToolDefinition {
          tool_type: "function".to_string(),
          function: FunctionDefinition {
            name: "contents".to_string(),
            description: "Get the full text content of specific web pages using Exa AI. Returns the main content of each URL.".to_string(),
            parameters: json!({
              "type": "object",
              "properties": {
                "urls": {
                  "type": "array",
                  "items": {"type": "string"},
                  "description": "List of URLs to get content for"
                },
                "text": {
                  "type": "boolean",
                  "description": "Whether to include text content (default: true)",
                  "default": true
                }
              },
              "required": ["urls"]
            }),
          },
        },
        ToolDefinition {
          tool_type: "function".to_string(),
          function: FunctionDefinition {
            name: "answer".to_string(),
            description: "Get an AI-powered answer to a question using Exa AI. Returns a direct answer with supporting sources.".to_string(),
            parameters: json!({
              "type": "object",
              "properties": {
                "query": {
                  "type": "string",
                  "description": "The question to get an answer for"
                },
                "text": {
                  "type": "boolean",
                  "description": "Whether to include supporting text (default: true)",
                  "default": true
                }
              },
              "required": ["query"]
            }),
          },
        },
      ],
    }]
  }
}

#[async_trait::async_trait]
impl ToolService for DefaultToolService {
  async fn list_tools_for_user(
    &self,
    tenant_id: &str,
    user_id: &str,
  ) -> Result<Vec<ToolDefinition>, ToolsetError> {
    let toolsets = self.db_service.list_toolsets(tenant_id, user_id).await?;

    // Collect unique toolset_types that are enabled with API keys
    let enabled_toolset_types: std::collections::HashSet<String> = toolsets
      .into_iter()
      .filter(|t| t.enabled && t.encrypted_api_key.is_some())
      .map(|t| t.toolset_type)
      .collect();

    // Return tool definitions for enabled types
    let mut tools = Vec::new();
    for toolset_def in Self::builtin_toolsets() {
      if enabled_toolset_types.contains(&toolset_def.toolset_type) {
        tools.extend(toolset_def.tools);
      }
    }

    Ok(tools)
  }

  async fn list_all_toolsets(&self) -> Result<Vec<ToolsetDefinition>, ToolsetError> {
    Ok(Self::builtin_toolsets())
  }

  // ============================================================================
  // Toolset instance management
  // ============================================================================

  async fn list(&self, tenant_id: &str, user_id: &str) -> Result<Vec<ToolsetEntity>, ToolsetError> {
    let rows = self.db_service.list_toolsets(tenant_id, user_id).await?;
    Ok(rows)
  }

  async fn get(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
  ) -> Result<Option<ToolsetEntity>, ToolsetError> {
    let row = self.db_service.get_toolset(tenant_id, id).await?;

    // Return None if not found OR user doesn't own it (hide existence)
    Ok(match row {
      Some(r) if r.user_id == user_id => Some(r),
      _ => None,
    })
  }

  async fn create(
    &self,
    tenant_id: &str,
    user_id: &str,
    request: ToolsetRequest,
  ) -> Result<ToolsetEntity, ToolsetError> {
    let toolset_type = request
      .toolset_type
      .as_deref()
      .ok_or_else(|| ToolsetError::InvalidToolsetType("toolset_type is required".to_string()))?;

    // Validate toolset_type exists
    self.validate_type(toolset_type)?;

    // Check app-level enabled
    if !self.is_type_enabled(tenant_id, toolset_type).await? {
      return Err(ToolsetError::ToolsetAppDisabled);
    }

    // Check slug uniqueness (case-insensitive)
    if self
      .db_service
      .get_toolset_by_slug(tenant_id, user_id, &request.slug)
      .await?
      .is_some()
    {
      return Err(ToolsetError::SlugExists(request.slug.clone()));
    }

    // Extract and encrypt API key
    let api_key_raw = request.api_key.into_raw_update();
    let (encrypted, salt, nonce) = match api_key_raw {
      RawApiKeyUpdate::Set(Some(ref key)) => {
        let (enc, s, n) = encrypt_api_key(self.db_service.encryption_key(), key)
          .map_err(|e| DbError::EncryptionError(e.to_string()))?;
        (Some(enc), Some(s), Some(n))
      }
      _ => (None, None, None),
    };

    // Create entity
    let now = self.time_service.utc_now();
    let entity = ToolsetEntity {
      id: Ulid::new().to_string(),
      tenant_id: tenant_id.to_string(),
      user_id: user_id.to_string(),
      toolset_type: toolset_type.to_string(),
      slug: request.slug,
      description: request.description,
      enabled: request.enabled,
      encrypted_api_key: encrypted,
      salt,
      nonce,
      created_at: now,
      updated_at: now,
    };

    let result = self.db_service.create_toolset(tenant_id, &entity).await?;
    Ok(result)
  }

  async fn update(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
    request: ToolsetRequest,
  ) -> Result<ToolsetEntity, ToolsetError> {
    // Fetch existing
    let existing = self
      .db_service
      .get_toolset(tenant_id, id)
      .await?
      .ok_or_else(|| ToolsetError::ToolsetNotFound(id.to_string()))?;

    // Verify ownership
    if existing.user_id != user_id {
      return Err(ToolsetError::ToolsetNotFound(id.to_string()));
    }

    // Check slug uniqueness if changed (case-insensitive)
    if request.slug.to_lowercase() != existing.slug.to_lowercase()
      && self
        .db_service
        .get_toolset_by_slug(tenant_id, user_id, &request.slug)
        .await?
        .is_some()
    {
      return Err(ToolsetError::SlugExists(request.slug.clone()));
    }

    // Check app-level enabled if trying to enable
    if request.enabled
      && !self
        .is_type_enabled(tenant_id, &existing.toolset_type)
        .await?
    {
      return Err(ToolsetError::ToolsetAppDisabled);
    }

    // Handle API key update
    let api_key_update = request.api_key.into_raw_update();
    let (encrypted, salt, nonce, db_api_key_update) = match api_key_update {
      RawApiKeyUpdate::Keep => (
        existing.encrypted_api_key,
        existing.salt,
        existing.nonce,
        RawApiKeyUpdate::Keep,
      ),
      RawApiKeyUpdate::Set(Some(new_key)) => {
        let (enc, s, n) = encrypt_api_key(self.db_service.encryption_key(), &new_key)
          .map_err(|e| DbError::EncryptionError(e.to_string()))?;
        (
          Some(enc.clone()),
          Some(s.clone()),
          Some(n.clone()),
          RawApiKeyUpdate::Set(Some(enc)),
        )
      }
      RawApiKeyUpdate::Set(None) => (None, None, None, RawApiKeyUpdate::Set(None)),
    };

    // Update entity
    let now = self.time_service.utc_now();
    let entity = ToolsetEntity {
      id: id.to_string(),
      tenant_id: tenant_id.to_string(),
      user_id: user_id.to_string(),
      toolset_type: existing.toolset_type,
      slug: request.slug,
      description: request.description,
      enabled: request.enabled,
      encrypted_api_key: encrypted,
      salt,
      nonce,
      created_at: existing.created_at,
      updated_at: now,
    };

    let result = self
      .db_service
      .update_toolset(tenant_id, &entity, db_api_key_update)
      .await?;
    Ok(result)
  }

  async fn delete(&self, tenant_id: &str, user_id: &str, id: &str) -> Result<(), ToolsetError> {
    // Fetch and verify ownership
    let existing = self
      .db_service
      .get_toolset(tenant_id, id)
      .await?
      .ok_or_else(|| ToolsetError::ToolsetNotFound(id.to_string()))?;

    if existing.user_id != user_id {
      return Err(ToolsetError::ToolsetNotFound(id.to_string()));
    }

    // Allow delete even if app-level type disabled
    self.db_service.delete_toolset(tenant_id, id).await?;
    Ok(())
  }

  async fn execute(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
    method: &str,
    request: ToolsetExecutionRequest,
  ) -> Result<ToolsetExecutionResponse, ToolsetError> {
    // Fetch instance and verify ownership
    let instance = self
      .db_service
      .get_toolset(tenant_id, id)
      .await?
      .ok_or_else(|| ToolsetError::ToolsetNotFound(id.to_string()))?;

    if instance.user_id != user_id {
      return Err(ToolsetError::ToolsetNotFound(id.to_string()));
    }

    // Check app-level type enabled
    if !self
      .is_type_enabled(tenant_id, &instance.toolset_type)
      .await?
    {
      return Err(ToolsetError::ToolsetAppDisabled);
    }

    // Check instance enabled
    if !instance.enabled {
      return Err(ToolsetError::ToolsetDisabled);
    }

    // Get decrypted API key
    let api_key = self
      .db_service
      .get_toolset_api_key(tenant_id, id)
      .await?
      .ok_or(ToolsetError::ToolsetNotConfigured)?;

    // Verify method exists for toolset type
    let toolset_def = self
      .get_type(&instance.toolset_type)
      .ok_or_else(|| ToolsetError::ToolsetNotFound(instance.toolset_type.clone()))?;

    if !toolset_def.tools.iter().any(|t| t.function.name == method) {
      return Err(ToolsetError::MethodNotFound(format!(
        "Method '{}' not found in toolset type '{}'",
        method, toolset_def.toolset_type
      )));
    }

    // Execute based on toolset_type
    match toolset_def.toolset_type.as_str() {
      "builtin-exa-search" => {
        super::execution::execute_exa(self.exa_service.as_ref(), &api_key, method, request).await
      }
      _ => Err(ToolsetError::ToolsetNotFound(instance.toolset_type)),
    }
  }

  // ============================================================================
  // Toolset type management
  // ============================================================================

  fn list_types(&self) -> Vec<ToolsetDefinition> {
    Self::builtin_toolsets()
  }

  fn get_type(&self, toolset_type: &str) -> Option<ToolsetDefinition> {
    Self::builtin_toolsets()
      .into_iter()
      .find(|def| def.toolset_type == toolset_type)
  }

  fn validate_type(&self, toolset_type: &str) -> Result<(), ToolsetError> {
    if self.get_type(toolset_type).is_some() {
      Ok(())
    } else {
      Err(ToolsetError::InvalidToolsetType(toolset_type.to_string()))
    }
  }

  async fn is_type_enabled(
    &self,
    tenant_id: &str,
    toolset_type: &str,
  ) -> Result<bool, ToolsetError> {
    // Check database for app-level configuration
    match self
      .db_service
      .get_app_toolset_config(tenant_id, toolset_type)
      .await?
    {
      Some(config) => Ok(config.enabled),
      None => Ok(false), // Default to disabled if no config exists (security-by-default)
    }
  }

  // ============================================================================
  // App-level toolset type configuration (Admin only)
  // ============================================================================

  async fn set_app_toolset_enabled(
    &self,
    tenant_id: &str,
    toolset_type: &str,
    enabled: bool,
    updated_by: &str,
  ) -> Result<AppToolsetConfig, ToolsetError> {
    // Validate toolset_type exists in ToolsetDefinition registry
    self.validate_type(toolset_type)?;

    // Call db_service to upsert app_toolset_configs record
    let db_row = self
      .db_service
      .set_app_toolset_enabled(tenant_id, toolset_type, enabled, updated_by)
      .await?;

    // Enrich with name/description from ToolsetDefinition
    let type_def = self
      .get_type(toolset_type)
      .ok_or_else(|| ToolsetError::InvalidToolsetType(toolset_type.to_string()))?;

    Ok(AppToolsetConfig {
      toolset_type: db_row.toolset_type,
      name: type_def.name,
      description: type_def.description,
      enabled: db_row.enabled,
      updated_by: db_row.updated_by,
      created_at: db_row.created_at,
      updated_at: db_row.updated_at,
    })
  }

  async fn list_app_toolset_configs(
    &self,
    tenant_id: &str,
  ) -> Result<Vec<AppToolsetConfig>, ToolsetError> {
    let db_rows = self.db_service.list_app_toolset_configs(tenant_id).await?;

    // Enrich each row with name/description from ToolsetDefinition
    let configs = db_rows
      .into_iter()
      .filter_map(|row| {
        // Only include configs for toolset types that still exist in the registry
        self
          .get_type(&row.toolset_type)
          .map(|type_def| AppToolsetConfig {
            toolset_type: row.toolset_type,
            name: type_def.name,
            description: type_def.description,
            enabled: row.enabled,
            updated_by: row.updated_by,
            created_at: row.created_at,
            updated_at: row.updated_at,
          })
      })
      .collect();

    Ok(configs)
  }

  async fn get_app_toolset_config(
    &self,
    tenant_id: &str,
    toolset_type: &str,
  ) -> Result<Option<AppToolsetConfig>, ToolsetError> {
    let db_row = self
      .db_service
      .get_app_toolset_config(tenant_id, toolset_type)
      .await?;

    Ok(db_row.and_then(|row| {
      self
        .get_type(&row.toolset_type)
        .map(|type_def| AppToolsetConfig {
          toolset_type: row.toolset_type,
          name: type_def.name,
          description: type_def.description,
          enabled: row.enabled,
          updated_by: row.updated_by,
          created_at: row.created_at,
          updated_at: row.updated_at,
        })
    }))
  }
}

#[cfg(test)]
#[path = "test_tool_service.rs"]
mod test_tool_service;
