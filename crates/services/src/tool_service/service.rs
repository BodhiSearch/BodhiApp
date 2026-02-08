use super::ToolsetError;
use crate::db::{
  encryption::encrypt_api_key, ApiKeyUpdate, AppToolsetConfigRow, DbError, DbService, TimeService,
  ToolsetRow,
};
use crate::exa_service::ExaService;
use chrono::DateTime;
use objs::{
  AppToolsetConfig, FunctionDefinition, ToolDefinition, Toolset, ToolsetDefinition,
  ToolsetExecutionRequest, ToolsetExecutionResponse, ToolsetWithTools,
};
use serde_json::json;
use std::fmt::Debug;
use std::sync::Arc;
use uuid::Uuid;

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait ToolService: Debug + Send + Sync {
  /// List tool definitions for user (only configured and enabled toolsets)
  async fn list_tools_for_user(&self, user_id: &str) -> Result<Vec<ToolDefinition>, ToolsetError>;

  /// Get all available tool definitions (for UI listing)
  fn list_all_tool_definitions(&self) -> Vec<ToolDefinition>;

  /// List all available toolsets with their tools (nested structure)
  async fn list_all_toolsets(&self) -> Result<Vec<ToolsetWithTools>, ToolsetError>;

  // ============================================================================
  // Toolset instance management
  // ============================================================================

  /// List all toolset instances for user
  async fn list(&self, user_id: &str) -> Result<Vec<Toolset>, ToolsetError>;

  /// Get a specific toolset instance by ID
  async fn get(&self, user_id: &str, id: &str) -> Result<Option<Toolset>, ToolsetError>;

  /// Create a new toolset instance
  async fn create(
    &self,
    user_id: &str,
    scope_uuid: &str,
    name: &str,
    description: Option<String>,
    enabled: bool,
    api_key: String,
  ) -> Result<Toolset, ToolsetError>;

  /// Update an existing toolset instance
  async fn update(
    &self,
    user_id: &str,
    id: &str,
    name: &str,
    description: Option<String>,
    enabled: bool,
    api_key_update: ApiKeyUpdate,
  ) -> Result<Toolset, ToolsetError>;

  /// Delete a toolset instance
  async fn delete(&self, user_id: &str, id: &str) -> Result<(), ToolsetError>;

  /// Execute a tool on a toolset instance
  async fn execute(
    &self,
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

  /// Get toolset type definition by scope_uuid
  fn get_type(&self, scope_uuid: &str) -> Option<ToolsetDefinition>;

  /// Validate toolset scope_uuid exists
  fn validate_type(&self, scope_uuid: &str) -> Result<(), ToolsetError>;

  /// Check if toolset type is enabled at app level
  async fn is_type_enabled(&self, scope_uuid: &str) -> Result<bool, ToolsetError>;

  // ============================================================================
  // App-level toolset configuration (admin-controlled)
  // ============================================================================

  /// Get app-level toolset config by scope
  async fn get_app_toolset_config(
    &self,
    scope: &str,
  ) -> Result<Option<AppToolsetConfig>, ToolsetError>;

  /// Check if toolset is enabled at app level
  async fn is_toolset_enabled_for_app(&self, scope: &str) -> Result<bool, ToolsetError>;

  /// Set app-level toolset enabled status (admin only)
  /// Updates local DB only (no longer syncs with auth server)
  async fn set_app_toolset_enabled(
    &self,
    admin_token: &str,
    scope: &str,
    scope_uuid: &str,
    enabled: bool,
    updated_by: &str,
  ) -> Result<AppToolsetConfig, ToolsetError>;

  /// List all app-level toolset configs
  async fn list_app_toolset_configs(&self) -> Result<Vec<AppToolsetConfig>, ToolsetError>;

  /// List app-level toolset configs for specific scopes
  async fn list_app_toolset_configs_by_scopes(
    &self,
    scopes: &[String],
  ) -> Result<Vec<AppToolsetConfig>, ToolsetError>;

  // ============================================================================
  // App-client toolset configuration (cached from auth server)
  // ============================================================================

  /// Check if an external app-client is registered for a specific toolset
  /// Looks up the cached app_client_toolset_configs table
  async fn is_app_client_registered_for_toolset(
    &self,
    app_client_id: &str,
    scope_uuid: &str,
  ) -> Result<bool, ToolsetError>;
}

// ============================================================================
// DefaultToolService - Implementation
// ============================================================================

#[derive(Debug)]
pub struct DefaultToolService {
  db_service: Arc<dyn DbService>,
  exa_service: Arc<dyn ExaService>,
  time_service: Arc<dyn TimeService>,
  is_production: bool,
}

impl DefaultToolService {
  pub fn new(
    db_service: Arc<dyn DbService>,
    exa_service: Arc<dyn ExaService>,
    time_service: Arc<dyn TimeService>,
    is_production: bool,
  ) -> Self {
    Self {
      db_service,
      exa_service,
      time_service,
      is_production,
    }
  }

  /// Static registry of built-in tools
  fn builtin_tool_definitions() -> Vec<ToolDefinition> {
    vec![ToolDefinition {
      tool_type: "function".to_string(),
      function: FunctionDefinition {
        name: "builtin-exa-web-search".to_string(),
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
    }]
  }

  /// Static registry of built-in toolsets with nested tools
  fn builtin_toolsets(is_production: bool) -> Vec<ToolsetDefinition> {
    let scope_uuid = if is_production {
      "7a89e236-9d23-4856-aa77-b52823ff9972"
    } else {
      "4ff0e163-36fb-47d6-a5ef-26e396f067d6"
    };

    vec![ToolsetDefinition {
      scope_uuid: scope_uuid.to_string(),
      scope: "scope_toolset-builtin-exa-web-search".to_string(),
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

  /// Convert toolset row to public model
  fn toolset_row_to_model(&self, row: ToolsetRow) -> Toolset {
    // Lookup scope from registry using scope_uuid
    let scope = Self::builtin_toolsets(self.is_production)
      .iter()
      .find(|def| def.scope_uuid == row.scope_uuid)
      .map(|def| def.scope.clone())
      .unwrap_or_else(|| "scope_unknown".to_string());

    Toolset {
      id: row.id,
      name: row.name,
      scope_uuid: row.scope_uuid,
      scope,
      description: row.description,
      enabled: row.enabled,
      has_api_key: row.encrypted_api_key.is_some(),
      created_at: DateTime::from_timestamp(row.created_at, 0).unwrap(),
      updated_at: DateTime::from_timestamp(row.updated_at, 0).unwrap(),
    }
  }

  /// Convert app toolset config row to public model
  fn app_row_to_config(row: AppToolsetConfigRow) -> AppToolsetConfig {
    AppToolsetConfig {
      scope: row.scope,
      scope_uuid: row.scope_uuid,
      enabled: row.enabled,
      updated_by: row.updated_by,
      created_at: DateTime::from_timestamp(row.created_at, 0).unwrap(),
      updated_at: DateTime::from_timestamp(row.updated_at, 0).unwrap(),
    }
  }
}

#[async_trait::async_trait]
impl ToolService for DefaultToolService {
  async fn list_tools_for_user(&self, user_id: &str) -> Result<Vec<ToolDefinition>, ToolsetError> {
    let toolsets = self.db_service.list_toolsets(user_id).await?;

    // Collect unique scope_uuids that are enabled with API keys
    let enabled_scope_uuids: std::collections::HashSet<String> = toolsets
      .into_iter()
      .filter(|t| t.enabled && t.encrypted_api_key.is_some())
      .map(|t| t.scope_uuid)
      .collect();

    // Return tool definitions for enabled types
    let mut tools = Vec::new();
    for toolset_def in Self::builtin_toolsets(self.is_production) {
      if enabled_scope_uuids.contains(&toolset_def.scope_uuid) {
        tools.extend(toolset_def.tools);
      }
    }

    Ok(tools)
  }

  fn list_all_tool_definitions(&self) -> Vec<ToolDefinition> {
    Self::builtin_tool_definitions()
  }

  async fn list_all_toolsets(&self) -> Result<Vec<ToolsetWithTools>, ToolsetError> {
    let mut result = Vec::new();

    for toolset_def in Self::builtin_toolsets(self.is_production) {
      let app_enabled = self
        .is_toolset_enabled_for_app(&toolset_def.scope)
        .await
        .unwrap_or(false);

      result.push(ToolsetWithTools {
        scope_uuid: toolset_def.scope_uuid,
        scope: toolset_def.scope,
        name: toolset_def.name,
        description: toolset_def.description,
        app_enabled,
        tools: toolset_def.tools,
      });
    }

    Ok(result)
  }

  // ============================================================================
  // Toolset instance management
  // ============================================================================

  async fn list(&self, user_id: &str) -> Result<Vec<Toolset>, ToolsetError> {
    let rows = self.db_service.list_toolsets(user_id).await?;
    Ok(
      rows
        .into_iter()
        .map(|r| self.toolset_row_to_model(r))
        .collect(),
    )
  }

  async fn get(&self, user_id: &str, id: &str) -> Result<Option<Toolset>, ToolsetError> {
    let row = self.db_service.get_toolset(id).await?;

    // Return None if not found OR user doesn't own it (hide existence)
    Ok(match row {
      Some(r) if r.user_id == user_id => Some(self.toolset_row_to_model(r)),
      _ => None,
    })
  }

  async fn create(
    &self,
    user_id: &str,
    scope_uuid: &str,
    name: &str,
    description: Option<String>,
    enabled: bool,
    api_key: String,
  ) -> Result<Toolset, ToolsetError> {
    // Validate name format
    objs::validate_toolset_name(name).map_err(|reason| ToolsetError::InvalidName(reason))?;

    // Validate description if provided
    if let Some(ref desc) = description {
      objs::validate_toolset_description(desc)
        .map_err(|reason| ToolsetError::InvalidName(reason))?;
    }

    // Validate scope_uuid exists
    self.validate_type(scope_uuid)?;

    // Check app-level enabled
    if !self.is_type_enabled(scope_uuid).await? {
      return Err(ToolsetError::ToolsetAppDisabled);
    }

    // Check name uniqueness (case-insensitive)
    if self
      .db_service
      .get_toolset_by_name(user_id, name)
      .await?
      .is_some()
    {
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
      scope_uuid: scope_uuid.to_string(),
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
    Ok(self.toolset_row_to_model(result))
  }

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
    let existing = self
      .db_service
      .get_toolset(id)
      .await?
      .ok_or_else(|| ToolsetError::ToolsetNotFound(id.to_string()))?;

    // Verify ownership
    if existing.user_id != user_id {
      return Err(ToolsetError::ToolsetNotFound(id.to_string()));
    }

    // Validate name format
    objs::validate_toolset_name(name).map_err(|reason| ToolsetError::InvalidName(reason))?;

    // Validate description if provided
    if let Some(ref desc) = description {
      objs::validate_toolset_description(desc)
        .map_err(|reason| ToolsetError::InvalidName(reason))?;
    }

    // Check name uniqueness if changed (case-insensitive)
    if name.to_lowercase() != existing.name.to_lowercase() {
      if self
        .db_service
        .get_toolset_by_name(user_id, name)
        .await?
        .is_some()
      {
        return Err(ToolsetError::NameExists(name.to_string()));
      }
    }

    // Check app-level enabled if trying to enable
    if enabled && !self.is_type_enabled(&existing.scope_uuid).await? {
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
        (
          Some(enc.clone()),
          Some(s.clone()),
          Some(n.clone()),
          ApiKeyUpdate::Set(Some(enc)),
        )
      }
      ApiKeyUpdate::Set(None) => (None, None, None, ApiKeyUpdate::Set(None)),
    };

    // Update row
    let now = self.time_service.utc_now().timestamp();
    let row = ToolsetRow {
      id: id.to_string(),
      user_id: user_id.to_string(),
      scope_uuid: existing.scope_uuid,
      name: name.to_string(),
      description,
      enabled,
      encrypted_api_key: encrypted,
      salt,
      nonce,
      created_at: existing.created_at,
      updated_at: now,
    };

    let result = self
      .db_service
      .update_toolset(&row, db_api_key_update)
      .await?;
    Ok(self.toolset_row_to_model(result))
  }

  async fn delete(&self, user_id: &str, id: &str) -> Result<(), ToolsetError> {
    // Fetch and verify ownership
    let existing = self
      .db_service
      .get_toolset(id)
      .await?
      .ok_or_else(|| ToolsetError::ToolsetNotFound(id.to_string()))?;

    if existing.user_id != user_id {
      return Err(ToolsetError::ToolsetNotFound(id.to_string()));
    }

    // Allow delete even if app-level type disabled
    self.db_service.delete_toolset(id).await?;
    Ok(())
  }

  async fn execute(
    &self,
    user_id: &str,
    id: &str,
    method: &str,
    request: ToolsetExecutionRequest,
  ) -> Result<ToolsetExecutionResponse, ToolsetError> {
    // Fetch instance and verify ownership
    let instance = self
      .db_service
      .get_toolset(id)
      .await?
      .ok_or_else(|| ToolsetError::ToolsetNotFound(id.to_string()))?;

    if instance.user_id != user_id {
      return Err(ToolsetError::ToolsetNotFound(id.to_string()));
    }

    // Check app-level type enabled
    if !self.is_type_enabled(&instance.scope_uuid).await? {
      return Err(ToolsetError::ToolsetAppDisabled);
    }

    // Check instance enabled
    if !instance.enabled {
      return Err(ToolsetError::ToolsetDisabled);
    }

    // Get decrypted API key
    let api_key = self
      .db_service
      .get_toolset_api_key(id)
      .await?
      .ok_or(ToolsetError::ToolsetNotConfigured)?;

    // Verify method exists for toolset type and get scope
    let toolset_def = self
      .get_type(&instance.scope_uuid)
      .ok_or_else(|| ToolsetError::ToolsetNotFound(instance.scope_uuid.clone()))?;

    if !toolset_def.tools.iter().any(|t| t.function.name == method) {
      return Err(ToolsetError::MethodNotFound(format!(
        "Method '{}' not found in toolset scope '{}'",
        method, toolset_def.scope
      )));
    }

    // Execute based on scope
    match toolset_def.scope.as_str() {
      "scope_toolset-builtin-exa-web-search" => {
        self.execute_exa_method(&api_key, method, request).await
      }
      _ => Err(ToolsetError::ToolsetNotFound(instance.scope_uuid)),
    }
  }

  // ============================================================================
  // Toolset type management
  // ============================================================================

  fn list_types(&self) -> Vec<ToolsetDefinition> {
    Self::builtin_toolsets(self.is_production)
  }

  fn get_type(&self, scope_uuid: &str) -> Option<ToolsetDefinition> {
    Self::builtin_toolsets(self.is_production)
      .into_iter()
      .find(|def| def.scope_uuid == scope_uuid)
  }

  fn validate_type(&self, scope_uuid: &str) -> Result<(), ToolsetError> {
    if self.get_type(scope_uuid).is_some() {
      Ok(())
    } else {
      Err(ToolsetError::InvalidToolsetType(scope_uuid.to_string()))
    }
  }

  async fn is_type_enabled(&self, scope_uuid: &str) -> Result<bool, ToolsetError> {
    // Get scope from scope_uuid for database lookup
    let toolset_def = self
      .get_type(scope_uuid)
      .ok_or_else(|| ToolsetError::InvalidToolsetType(scope_uuid.to_string()))?;
    self.is_toolset_enabled_for_app(&toolset_def.scope).await
  }

  // ============================================================================
  // App-level toolset configuration (admin-controlled)
  // ============================================================================

  async fn get_app_toolset_config(
    &self,
    scope: &str,
  ) -> Result<Option<AppToolsetConfig>, ToolsetError> {
    // Verify scope exists in our toolset definitions
    if !Self::builtin_toolsets(self.is_production)
      .iter()
      .any(|def| def.scope == scope)
    {
      return Err(ToolsetError::ToolsetNotFound(scope.to_string()));
    }

    let config = self
      .db_service
      .get_app_toolset_config_by_scope(scope)
      .await?;
    Ok(config.map(Self::app_row_to_config))
  }

  async fn is_toolset_enabled_for_app(&self, scope: &str) -> Result<bool, ToolsetError> {
    let config = self
      .db_service
      .get_app_toolset_config_by_scope(scope)
      .await?;
    // No row means disabled (default state)
    Ok(config.map(|c| c.enabled).unwrap_or(false))
  }

  async fn set_app_toolset_enabled(
    &self,
    admin_token: &str,
    scope: &str,
    scope_uuid: &str,
    enabled: bool,
    updated_by: &str,
  ) -> Result<AppToolsetConfig, ToolsetError> {
    // Verify scope exists in our toolset definitions
    if !Self::builtin_toolsets(self.is_production)
      .iter()
      .any(|def| def.scope == scope)
    {
      return Err(ToolsetError::ToolsetNotFound(scope.to_string()));
    }

    // Note: admin_token is now unused since we removed auth server integration
    // App-level enable/disable is now purely local DB operation
    let _ = admin_token;

    let now = self.time_service.utc_now();
    let now_ts = now.timestamp();

    // Get existing config
    let existing = self
      .db_service
      .get_app_toolset_config_by_scope(scope)
      .await?;

    let config = AppToolsetConfigRow {
      id: existing.as_ref().map(|e| e.id).unwrap_or(0),
      scope: scope.to_string(),
      scope_uuid: scope_uuid.to_string(),
      enabled,
      updated_by: updated_by.to_string(),
      created_at: existing.as_ref().map(|e| e.created_at).unwrap_or(now_ts),
      updated_at: now_ts,
    };

    let result = self.db_service.upsert_app_toolset_config(&config).await?;
    Ok(Self::app_row_to_config(result))
  }

  async fn list_app_toolset_configs(&self) -> Result<Vec<AppToolsetConfig>, ToolsetError> {
    let configs = self.db_service.list_app_toolset_configs().await?;
    Ok(configs.into_iter().map(Self::app_row_to_config).collect())
  }

  async fn list_app_toolset_configs_by_scopes(
    &self,
    scopes: &[String],
  ) -> Result<Vec<AppToolsetConfig>, ToolsetError> {
    let configs = self
      .db_service
      .list_app_toolset_configs_by_scopes(scopes)
      .await?;
    Ok(configs.into_iter().map(Self::app_row_to_config).collect())
  }

  async fn is_app_client_registered_for_toolset(
    &self,
    app_client_id: &str,
    scope_uuid: &str,
  ) -> Result<bool, ToolsetError> {
    // Look up cached app-client toolset config
    let config = self
      .db_service
      .get_app_client_toolset_config(app_client_id)
      .await?;

    let Some(config) = config else {
      return Ok(false);
    };

    // Parse the toolsets_json to check if scope_uuid is registered
    let toolsets: Vec<serde_json::Value> =
      serde_json::from_str(&config.toolsets_json).unwrap_or_default();

    Ok(toolsets.iter().any(|t| {
      t.get("scope_id")
        .and_then(|v| v.as_str())
        .map(|id| id == scope_uuid)
        .unwrap_or(false)
    }))
  }
}

impl DefaultToolService {
  async fn execute_exa_method(
    &self,
    api_key: &str,
    method: &str,
    request: ToolsetExecutionRequest,
  ) -> Result<ToolsetExecutionResponse, ToolsetError> {
    match method {
      "search" => {
        let query = request.params["query"]
          .as_str()
          .ok_or_else(|| ToolsetError::ExecutionFailed("Missing 'query' parameter".to_string()))?;
        let num_results = request.params["num_results"].as_u64().map(|n| n as u32);

        match self.exa_service.search(api_key, query, num_results).await {
          Ok(response) => {
            let results: Vec<_> = response
              .results
              .into_iter()
              .map(|r| {
                json!({
                  "title": r.title,
                  "url": r.url,
                  "snippet": r.highlights.join(" ... "),
                  "published_date": r.published_date,
                  "score": r.score,
                })
              })
              .collect();

            Ok(ToolsetExecutionResponse {
              result: Some(json!({
                "results": results,
                "query_used": response.autoprompt_string,
              })),
              error: None,
            })
          }
          Err(e) => Ok(ToolsetExecutionResponse {
            result: None,
            error: Some(e.to_string()),
          }),
        }
      }
      "findSimilar" => {
        let url = request.params["url"]
          .as_str()
          .ok_or_else(|| ToolsetError::ExecutionFailed("Missing 'url' parameter".to_string()))?;
        let num_results = request.params["num_results"].as_u64().map(|n| n as u32);

        match self
          .exa_service
          .find_similar(api_key, url, num_results)
          .await
        {
          Ok(response) => {
            let results: Vec<_> = response
              .results
              .into_iter()
              .map(|r| {
                json!({
                  "title": r.title,
                  "url": r.url,
                  "snippet": r.highlights.join(" ... "),
                  "published_date": r.published_date,
                  "score": r.score,
                })
              })
              .collect();

            Ok(ToolsetExecutionResponse {
              result: Some(json!({"results": results})),
              error: None,
            })
          }
          Err(e) => Ok(ToolsetExecutionResponse {
            result: None,
            error: Some(e.to_string()),
          }),
        }
      }
      "contents" => {
        let urls_array = request.params["urls"]
          .as_array()
          .ok_or_else(|| ToolsetError::ExecutionFailed("Missing 'urls' parameter".to_string()))?;
        let urls: Vec<String> = urls_array
          .iter()
          .filter_map(|v| v.as_str().map(String::from))
          .collect();
        let text = request
          .params
          .get("text")
          .and_then(|v| v.as_bool())
          .unwrap_or(true);

        match self.exa_service.get_contents(api_key, urls, text).await {
          Ok(response) => {
            let results: Vec<_> = response
              .results
              .into_iter()
              .map(|r| {
                json!({
                  "url": r.url,
                  "title": r.title,
                  "text": r.text,
                })
              })
              .collect();

            Ok(ToolsetExecutionResponse {
              result: Some(json!({"results": results})),
              error: None,
            })
          }
          Err(e) => Ok(ToolsetExecutionResponse {
            result: None,
            error: Some(e.to_string()),
          }),
        }
      }
      "answer" => {
        let query = request.params["query"]
          .as_str()
          .ok_or_else(|| ToolsetError::ExecutionFailed("Missing 'query' parameter".to_string()))?;
        let text = request
          .params
          .get("text")
          .and_then(|v| v.as_bool())
          .unwrap_or(true);

        match self.exa_service.answer(api_key, query, text).await {
          Ok(response) => {
            let sources: Vec<_> = response
              .results
              .into_iter()
              .map(|r| {
                json!({
                  "title": r.title,
                  "url": r.url,
                  "snippet": r.highlights.join(" ... "),
                })
              })
              .collect();

            Ok(ToolsetExecutionResponse {
              result: Some(json!({
                "answer": response.answer,
                "sources": sources,
              })),
              error: None,
            })
          }
          Err(e) => Ok(ToolsetExecutionResponse {
            result: None,
            error: Some(e.to_string()),
          }),
        }
      }
      _ => Err(ToolsetError::MethodNotFound(format!(
        "Unknown method: {}",
        method
      ))),
    }
  }
}
