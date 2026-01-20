use crate::db::{
  encryption::encrypt_api_key, ApiKeyUpdate, AppToolsetConfigRow, DbError, DbService, TimeService,
  ToolsetRow,
};
use crate::exa_service::{ExaError, ExaService};
use chrono::DateTime;
use objs::{
  AppError, AppToolsetConfig, ErrorType, FunctionDefinition, ToolDefinition, Toolset,
  ToolsetDefinition, ToolsetExecutionRequest, ToolsetExecutionResponse, ToolsetWithTools,
};
use serde_json::json;
use std::fmt::Debug;
use std::sync::Arc;
use uuid::Uuid;

// ============================================================================
// ToolsetError - Errors from toolset operations
// ============================================================================

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum ToolsetError {
  #[error("not_found")]
  #[error_meta(error_type = ErrorType::NotFound)]
  ToolsetNotFound(String),

  #[error("method_not_found")]
  #[error_meta(error_type = ErrorType::NotFound)]
  MethodNotFound(String),

  #[error("not_configured")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  ToolsetNotConfigured,

  #[error("disabled")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  ToolsetDisabled,

  #[error("execution_failed")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  ExecutionFailed(String),

  #[error("app_disabled")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  ToolsetAppDisabled,

  #[error("name_exists")]
  #[error_meta(error_type = ErrorType::Conflict)]
  NameExists(String),

  #[error("invalid_name")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  InvalidName(String),

  #[error("invalid_toolset_type")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  InvalidToolsetType(String),

  #[error(transparent)]
  DbError(#[from] DbError),

  #[error(transparent)]
  ExaError(#[from] ExaError),
}

// ============================================================================
// ToolService - Service for managing and executing toolsets
// ============================================================================

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
    toolset_type: &str,
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

  /// Get toolset type definition by ID
  fn get_type(&self, toolset_type: &str) -> Option<ToolsetDefinition>;

  /// Validate toolset type exists
  fn validate_type(&self, toolset_type: &str) -> Result<(), ToolsetError>;

  /// Check if toolset type is enabled at app level
  async fn is_type_enabled(&self, toolset_type: &str) -> Result<bool, ToolsetError>;

  // ============================================================================
  // App-level toolset configuration (admin-controlled)
  // ============================================================================

  /// Get app-level toolset config by ID
  async fn get_app_toolset_config(
    &self,
    toolset_id: &str,
  ) -> Result<Option<AppToolsetConfig>, ToolsetError>;

  /// Check if toolset is enabled at app level
  async fn is_toolset_enabled_for_app(&self, toolset_id: &str) -> Result<bool, ToolsetError>;

  /// Set app-level toolset enabled status (admin only)
  /// Updates local DB only (no longer syncs with auth server)
  async fn set_app_toolset_enabled(
    &self,
    admin_token: &str,
    toolset_id: &str,
    enabled: bool,
    updated_by: &str,
  ) -> Result<AppToolsetConfig, ToolsetError>;

  /// List all app-level toolset configs
  async fn list_app_toolset_configs(&self) -> Result<Vec<AppToolsetConfig>, ToolsetError>;

  // ============================================================================
  // App-client toolset configuration (cached from auth server)
  // ============================================================================

  /// Check if an external app-client is registered for a specific toolset
  /// Looks up the cached app_client_toolset_configs table
  async fn is_app_client_registered_for_toolset(
    &self,
    app_client_id: &str,
    toolset_id: &str,
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
  fn builtin_toolsets() -> Vec<ToolsetDefinition> {
    vec![ToolsetDefinition {
      toolset_id: "builtin-exa-web-search".to_string(),
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
  fn toolset_row_to_model(row: ToolsetRow) -> Toolset {
    Toolset {
      id: row.id,
      name: row.name,
      toolset_type: row.toolset_type,
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
      toolset_id: row.toolset_id,
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

  fn list_all_tool_definitions(&self) -> Vec<ToolDefinition> {
    Self::builtin_tool_definitions()
  }

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

  // ============================================================================
  // Toolset instance management
  // ============================================================================

  async fn list(&self, user_id: &str) -> Result<Vec<Toolset>, ToolsetError> {
    let rows = self.db_service.list_toolsets(user_id).await?;
    Ok(rows.into_iter().map(Self::toolset_row_to_model).collect())
  }

  async fn get(&self, user_id: &str, id: &str) -> Result<Option<Toolset>, ToolsetError> {
    let row = self.db_service.get_toolset(id).await?;

    // Return None if not found OR user doesn't own it (hide existence)
    Ok(match row {
      Some(r) if r.user_id == user_id => Some(Self::toolset_row_to_model(r)),
      _ => None,
    })
  }

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
    objs::validate_toolset_name(name).map_err(|reason| ToolsetError::InvalidName(reason))?;

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

    let result = self
      .db_service
      .update_toolset(&row, db_api_key_update)
      .await?;
    Ok(Self::toolset_row_to_model(result))
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
    if !self.is_type_enabled(&instance.toolset_type).await? {
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

    // Verify method exists for toolset type
    let toolset_def = self
      .get_type(&instance.toolset_type)
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

  // ============================================================================
  // Toolset type management
  // ============================================================================

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

  // ============================================================================
  // App-level toolset configuration (admin-controlled)
  // ============================================================================

  async fn get_app_toolset_config(
    &self,
    toolset_id: &str,
  ) -> Result<Option<AppToolsetConfig>, ToolsetError> {
    // Verify toolset exists
    if !Self::builtin_tool_definitions()
      .iter()
      .any(|def| def.function.name == toolset_id)
    {
      return Err(ToolsetError::ToolsetNotFound(toolset_id.to_string()));
    }

    let config = self.db_service.get_app_toolset_config(toolset_id).await?;
    Ok(config.map(Self::app_row_to_config))
  }

  async fn is_toolset_enabled_for_app(&self, toolset_id: &str) -> Result<bool, ToolsetError> {
    let config = self.db_service.get_app_toolset_config(toolset_id).await?;
    // No row means disabled (default state)
    Ok(config.map(|c| c.enabled).unwrap_or(false))
  }

  async fn set_app_toolset_enabled(
    &self,
    admin_token: &str,
    toolset_id: &str,
    enabled: bool,
    updated_by: &str,
  ) -> Result<AppToolsetConfig, ToolsetError> {
    // Verify toolset exists
    if !Self::builtin_tool_definitions()
      .iter()
      .any(|def| def.function.name == toolset_id)
    {
      return Err(ToolsetError::ToolsetNotFound(toolset_id.to_string()));
    }

    // Note: admin_token is now unused since we removed auth server integration
    // App-level enable/disable is now purely local DB operation
    let _ = admin_token;

    let now = self.time_service.utc_now();
    let now_ts = now.timestamp();

    // Get existing config
    let existing = self.db_service.get_app_toolset_config(toolset_id).await?;

    let config = AppToolsetConfigRow {
      id: existing.as_ref().map(|e| e.id).unwrap_or(0),
      toolset_id: toolset_id.to_string(),
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

  async fn is_app_client_registered_for_toolset(
    &self,
    app_client_id: &str,
    toolset_id: &str,
  ) -> Result<bool, ToolsetError> {
    // Look up cached app-client toolset config
    let config = self
      .db_service
      .get_app_client_toolset_config(app_client_id)
      .await?;

    let Some(config) = config else {
      return Ok(false);
    };

    // Parse the toolsets_json to check if toolset_id is registered
    let toolsets: Vec<serde_json::Value> =
      serde_json::from_str(&config.toolsets_json).unwrap_or_default();

    Ok(toolsets.iter().any(|t| {
      t.get("id")
        .and_then(|v| v.as_str())
        .map(|id| id == toolset_id)
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
              tool_call_id: request.tool_call_id,
              result: Some(json!({
                "results": results,
                "query_used": response.autoprompt_string,
              })),
              error: None,
            })
          }
          Err(e) => Ok(ToolsetExecutionResponse {
            tool_call_id: request.tool_call_id,
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
              tool_call_id: request.tool_call_id,
              result: Some(json!({"results": results})),
              error: None,
            })
          }
          Err(e) => Ok(ToolsetExecutionResponse {
            tool_call_id: request.tool_call_id,
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
              tool_call_id: request.tool_call_id,
              result: Some(json!({"results": results})),
              error: None,
            })
          }
          Err(e) => Ok(ToolsetExecutionResponse {
            tool_call_id: request.tool_call_id,
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
              tool_call_id: request.tool_call_id,
              result: Some(json!({
                "answer": response.answer,
                "sources": sources,
              })),
              error: None,
            })
          }
          Err(e) => Ok(ToolsetExecutionResponse {
            tool_call_id: request.tool_call_id,
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

#[cfg(test)]
mod tests {
  use crate::db::{ApiKeyUpdate, AppToolsetConfigRow, MockDbService, MockTimeService, ToolsetRow};
  use crate::exa_service::MockExaService;
  use crate::tool_service::{DefaultToolService, ToolService, ToolsetError};
  use chrono::{TimeZone, Utc};
  use mockall::predicate::eq;
  use rstest::rstest;
  use std::sync::Arc;

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

  // ============================================================================
  // Static Method Tests
  // ============================================================================

  #[rstest]
  fn test_list_all_tool_definitions() {
    let db = MockDbService::new();
    let exa = MockExaService::new();
    let time = MockTimeService::new();

    let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time));
    let defs = service.list_all_tool_definitions();

    assert_eq!(1, defs.len());
    assert_eq!("builtin-exa-web-search", defs[0].function.name);
  }

  #[rstest]
  fn test_list_types_returns_builtin_toolsets() {
    let db = MockDbService::new();
    let exa = MockExaService::new();
    let time = MockTimeService::new();

    let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time));
    let types = service.list_types();

    assert_eq!(1, types.len());
    assert_eq!("builtin-exa-web-search", types[0].toolset_id);
    assert_eq!(4, types[0].tools.len());
  }

  #[rstest]
  fn test_get_type_returns_toolset_definition() {
    let db = MockDbService::new();
    let exa = MockExaService::new();
    let time = MockTimeService::new();

    let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time));
    let def = service.get_type("builtin-exa-web-search");

    assert!(def.is_some());
    let def = def.unwrap();
    assert_eq!("Exa Web Search", def.name);
    assert_eq!(4, def.tools.len());
  }

  #[rstest]
  fn test_get_type_returns_none_for_unknown() {
    let db = MockDbService::new();
    let exa = MockExaService::new();
    let time = MockTimeService::new();

    let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time));
    let def = service.get_type("unknown");

    assert!(def.is_none());
  }

  #[rstest]
  fn test_validate_type_success() {
    let db = MockDbService::new();
    let exa = MockExaService::new();
    let time = MockTimeService::new();

    let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time));
    let result = service.validate_type("builtin-exa-web-search");

    assert!(result.is_ok());
  }

  #[rstest]
  fn test_validate_type_fails_for_unknown() {
    let db = MockDbService::new();
    let exa = MockExaService::new();
    let time = MockTimeService::new();

    let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time));
    let result = service.validate_type("unknown");

    assert!(result.is_err());
    assert!(matches!(
      result.unwrap_err(),
      ToolsetError::InvalidToolsetType(_)
    ));
  }

  // ============================================================================
  // list_tools_for_user Tests
  // ============================================================================

  #[rstest]
  #[tokio::test]
  async fn test_list_tools_for_user_returns_tools_for_enabled_instances() -> anyhow::Result<()> {
    let mut db = MockDbService::new();
    let exa = MockExaService::new();
    let time = MockTimeService::new();

    db.expect_list_toolsets()
      .with(eq("user123"))
      .returning(|_| {
        Ok(vec![
          test_toolset_row("id1", "user123", "my-exa-1"),
          ToolsetRow {
            enabled: false,
            ..test_toolset_row("id2", "user123", "my-exa-2")
          },
        ])
      });

    let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time));
    let tools = service.list_tools_for_user("user123").await?;

    assert_eq!(4, tools.len());
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_list_tools_for_user_returns_empty_when_no_instances() -> anyhow::Result<()> {
    let mut db = MockDbService::new();
    let exa = MockExaService::new();
    let time = MockTimeService::new();

    db.expect_list_toolsets()
      .with(eq("user123"))
      .returning(|_| Ok(vec![]));

    let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time));
    let tools = service.list_tools_for_user("user123").await?;

    assert!(tools.is_empty());
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_list_tools_for_user_deduplicates_same_type() -> anyhow::Result<()> {
    let mut db = MockDbService::new();
    let exa = MockExaService::new();
    let time = MockTimeService::new();

    db.expect_list_toolsets()
      .with(eq("user123"))
      .returning(|_| {
        Ok(vec![
          test_toolset_row("id1", "user123", "my-exa-1"),
          test_toolset_row("id2", "user123", "my-exa-2"),
        ])
      });

    let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time));
    let tools = service.list_tools_for_user("user123").await?;

    assert_eq!(4, tools.len());
    Ok(())
  }

  // ============================================================================
  // list_all_toolsets Tests
  // ============================================================================

  #[rstest]
  #[tokio::test]
  async fn test_list_all_toolsets_returns_toolsets_with_app_status() -> anyhow::Result<()> {
    let mut db = MockDbService::new();
    let exa = MockExaService::new();
    let time = MockTimeService::new();

    db.expect_get_app_toolset_config()
      .with(eq("builtin-exa-web-search"))
      .returning(|_| Ok(Some(app_config_enabled())));

    let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time));
    let toolsets = service.list_all_toolsets().await?;

    assert_eq!(1, toolsets.len());
    assert!(toolsets[0].app_enabled);
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_list_all_toolsets_shows_disabled_when_no_config() -> anyhow::Result<()> {
    let mut db = MockDbService::new();
    let exa = MockExaService::new();
    let time = MockTimeService::new();

    db.expect_get_app_toolset_config()
      .with(eq("builtin-exa-web-search"))
      .returning(|_| Ok(None));

    let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time));
    let toolsets = service.list_all_toolsets().await?;

    assert_eq!(1, toolsets.len());
    assert!(!toolsets[0].app_enabled);
    Ok(())
  }

  // ============================================================================
  // list Tests
  // ============================================================================

  #[rstest]
  #[tokio::test]
  async fn test_list_returns_user_toolsets() -> anyhow::Result<()> {
    let mut db = MockDbService::new();
    let exa = MockExaService::new();
    let time = MockTimeService::new();

    db.expect_list_toolsets()
      .with(eq("user123"))
      .returning(|_| {
        Ok(vec![
          test_toolset_row("id1", "user123", "my-exa-1"),
          test_toolset_row("id2", "user123", "my-exa-2"),
        ])
      });

    let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time));
    let toolsets = service.list("user123").await?;

    assert_eq!(2, toolsets.len());
    assert_eq!("my-exa-1", toolsets[0].name);
    assert_eq!("my-exa-2", toolsets[1].name);
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_list_returns_empty_for_user_with_no_toolsets() -> anyhow::Result<()> {
    let mut db = MockDbService::new();
    let exa = MockExaService::new();
    let time = MockTimeService::new();

    db.expect_list_toolsets()
      .with(eq("user123"))
      .returning(|_| Ok(vec![]));

    let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time));
    let toolsets = service.list("user123").await?;

    assert!(toolsets.is_empty());
    Ok(())
  }

  // ============================================================================
  // get Tests
  // ============================================================================

  #[rstest]
  #[tokio::test]
  async fn test_get_returns_owned_toolset() -> anyhow::Result<()> {
    let mut db = MockDbService::new();
    let exa = MockExaService::new();
    let time = MockTimeService::new();

    db.expect_get_toolset()
      .with(eq("id1"))
      .returning(|_| Ok(Some(test_toolset_row("id1", "user123", "my-exa-1"))));

    let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time));
    let toolset = service.get("user123", "id1").await?;

    assert!(toolset.is_some());
    assert_eq!("my-exa-1", toolset.unwrap().name);
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_get_returns_none_for_other_users_toolset() -> anyhow::Result<()> {
    let mut db = MockDbService::new();
    let exa = MockExaService::new();
    let time = MockTimeService::new();

    db.expect_get_toolset()
      .with(eq("id1"))
      .returning(|_| Ok(Some(test_toolset_row("id1", "user999", "other-exa"))));

    let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time));
    let toolset = service.get("user123", "id1").await?;

    assert!(toolset.is_none());
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_get_returns_none_when_not_found() -> anyhow::Result<()> {
    let mut db = MockDbService::new();
    let exa = MockExaService::new();
    let time = MockTimeService::new();

    db.expect_get_toolset()
      .with(eq("id999"))
      .returning(|_| Ok(None));

    let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time));
    let toolset = service.get("user123", "id999").await?;

    assert!(toolset.is_none());
    Ok(())
  }

  // ============================================================================
  // create Tests
  // ============================================================================

  #[rstest]
  #[tokio::test]
  async fn test_create_success() -> anyhow::Result<()> {
    let mut db = MockDbService::new();
    let exa = MockExaService::new();
    let mut time = MockTimeService::new();

    time
      .expect_utc_now()
      .returning(|| Utc.timestamp_opt(1700000000, 0).unwrap());

    db.expect_get_toolset_by_name()
      .with(eq("user123"), eq("my-exa"))
      .returning(|_, _| Ok(None));
    db.expect_get_app_toolset_config()
      .with(eq("builtin-exa-web-search"))
      .returning(|_| Ok(Some(app_config_enabled())));
    db.expect_encryption_key()
      .return_const(b"0123456789abcdef".to_vec());
    db.expect_create_toolset().returning(|row| Ok(row.clone()));

    let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time));
    let toolset = service
      .create(
        "user123",
        "builtin-exa-web-search",
        "my-exa",
        Some("My Exa".to_string()),
        true,
        "test-api-key".to_string(),
      )
      .await?;

    assert_eq!("my-exa", toolset.name);
    assert!(toolset.enabled);
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_create_fails_when_name_already_exists() -> anyhow::Result<()> {
    let mut db = MockDbService::new();
    let exa = MockExaService::new();
    let time = MockTimeService::new();

    db.expect_get_app_toolset_config()
      .with(eq("builtin-exa-web-search"))
      .returning(|_| Ok(Some(app_config_enabled())));
    db.expect_get_toolset_by_name()
      .with(eq("user123"), eq("my-exa"))
      .returning(|_, _| Ok(Some(test_toolset_row("existing", "user123", "my-exa"))));

    let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time));
    let result = service
      .create(
        "user123",
        "builtin-exa-web-search",
        "my-exa",
        None,
        true,
        "test-api-key".to_string(),
      )
      .await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ToolsetError::NameExists(_)));
    Ok(())
  }

  #[rstest]
  #[case("", "empty")]
  #[case("my_toolset", "special chars")]
  #[tokio::test]
  async fn test_create_fails_with_invalid_name(
    #[case] name: &str,
    #[case] _reason: &str,
  ) -> anyhow::Result<()> {
    let db = MockDbService::new();
    let exa = MockExaService::new();
    let time = MockTimeService::new();

    let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time));
    let result = service
      .create(
        "user123",
        "builtin-exa-web-search",
        name,
        None,
        true,
        "test-api-key".to_string(),
      )
      .await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ToolsetError::InvalidName(_)));
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_create_fails_with_too_long_name() -> anyhow::Result<()> {
    let db = MockDbService::new();
    let exa = MockExaService::new();
    let time = MockTimeService::new();

    let long_name = "a".repeat(25);
    let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time));
    let result = service
      .create(
        "user123",
        "builtin-exa-web-search",
        &long_name,
        None,
        true,
        "test-api-key".to_string(),
      )
      .await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ToolsetError::InvalidName(_)));
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_create_fails_with_invalid_toolset_type() -> anyhow::Result<()> {
    let db = MockDbService::new();
    let exa = MockExaService::new();
    let time = MockTimeService::new();

    let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time));
    let result = service
      .create(
        "user123",
        "unknown-type",
        "my-exa",
        None,
        true,
        "test-api-key".to_string(),
      )
      .await;

    assert!(result.is_err());
    assert!(matches!(
      result.unwrap_err(),
      ToolsetError::InvalidToolsetType(_)
    ));
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_create_fails_when_app_disabled() -> anyhow::Result<()> {
    let mut db = MockDbService::new();
    let exa = MockExaService::new();
    let time = MockTimeService::new();

    db.expect_get_app_toolset_config()
      .with(eq("builtin-exa-web-search"))
      .returning(|_| Ok(None));

    let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time));
    let result = service
      .create(
        "user123",
        "builtin-exa-web-search",
        "my-exa",
        None,
        true,
        "test-api-key".to_string(),
      )
      .await;

    assert!(result.is_err());
    assert!(matches!(
      result.unwrap_err(),
      ToolsetError::ToolsetAppDisabled
    ));
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_create_same_name_different_user_succeeds() -> anyhow::Result<()> {
    let mut db = MockDbService::new();
    let exa = MockExaService::new();
    let mut time = MockTimeService::new();

    time
      .expect_utc_now()
      .returning(|| Utc.timestamp_opt(1700000000, 0).unwrap());

    db.expect_get_toolset_by_name()
      .with(eq("user456"), eq("my-exa"))
      .returning(|_, _| Ok(None));
    db.expect_get_app_toolset_config()
      .with(eq("builtin-exa-web-search"))
      .returning(|_| Ok(Some(app_config_enabled())));
    db.expect_encryption_key()
      .return_const(b"0123456789abcdef".to_vec());
    db.expect_create_toolset().returning(|row| Ok(row.clone()));

    let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time));
    let toolset = service
      .create(
        "user456",
        "builtin-exa-web-search",
        "my-exa",
        None,
        true,
        "test-api-key".to_string(),
      )
      .await?;

    assert_eq!("my-exa", toolset.name);
    Ok(())
  }

  // ============================================================================
  // update Tests
  // ============================================================================

  #[rstest]
  #[tokio::test]
  async fn test_update_success() -> anyhow::Result<()> {
    let mut db = MockDbService::new();
    let exa = MockExaService::new();
    let mut time = MockTimeService::new();

    time
      .expect_utc_now()
      .returning(|| Utc.timestamp_opt(1700000001, 0).unwrap());

    db.expect_get_toolset()
      .with(eq("id1"))
      .returning(|_| Ok(Some(test_toolset_row("id1", "user123", "my-exa"))));
    db.expect_get_toolset_by_name()
      .with(eq("user123"), eq("my-exa-updated"))
      .returning(|_, _| Ok(None));
    db.expect_get_app_toolset_config()
      .with(eq("builtin-exa-web-search"))
      .returning(|_| Ok(Some(app_config_enabled())));
    db.expect_encryption_key()
      .return_const(b"0123456789abcdef".to_vec());
    db.expect_update_toolset()
      .returning(|row, _| Ok(row.clone()));

    let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time));
    let toolset = service
      .update(
        "user123",
        "id1",
        "my-exa-updated",
        Some("Updated".to_string()),
        true,
        ApiKeyUpdate::Keep,
      )
      .await?;

    assert_eq!("my-exa-updated", toolset.name);
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_update_fails_when_not_found() -> anyhow::Result<()> {
    let mut db = MockDbService::new();
    let exa = MockExaService::new();
    let time = MockTimeService::new();

    db.expect_get_toolset()
      .with(eq("id999"))
      .returning(|_| Ok(None));

    let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time));
    let result = service
      .update("user123", "id999", "my-exa", None, true, ApiKeyUpdate::Keep)
      .await;

    assert!(result.is_err());
    assert!(matches!(
      result.unwrap_err(),
      ToolsetError::ToolsetNotFound(_)
    ));
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_update_fails_when_not_owned() -> anyhow::Result<()> {
    let mut db = MockDbService::new();
    let exa = MockExaService::new();
    let time = MockTimeService::new();

    db.expect_get_toolset()
      .with(eq("id1"))
      .returning(|_| Ok(Some(test_toolset_row("id1", "user999", "other-exa"))));

    let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time));
    let result = service
      .update("user123", "id1", "my-exa", None, true, ApiKeyUpdate::Keep)
      .await;

    assert!(result.is_err());
    assert!(matches!(
      result.unwrap_err(),
      ToolsetError::ToolsetNotFound(_)
    ));
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_update_fails_when_name_conflicts() -> anyhow::Result<()> {
    let mut db = MockDbService::new();
    let exa = MockExaService::new();
    let time = MockTimeService::new();

    db.expect_get_toolset()
      .with(eq("id1"))
      .returning(|_| Ok(Some(test_toolset_row("id1", "user123", "my-exa-1"))));
    db.expect_get_toolset_by_name()
      .with(eq("user123"), eq("my-exa-2"))
      .returning(|_, _| Ok(Some(test_toolset_row("id2", "user123", "my-exa-2"))));

    let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time));
    let result = service
      .update("user123", "id1", "my-exa-2", None, true, ApiKeyUpdate::Keep)
      .await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ToolsetError::NameExists(_)));
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_update_same_name_different_case_succeeds() -> anyhow::Result<()> {
    let mut db = MockDbService::new();
    let exa = MockExaService::new();
    let mut time = MockTimeService::new();

    time
      .expect_utc_now()
      .returning(|| Utc.timestamp_opt(1700000001, 0).unwrap());

    db.expect_get_toolset()
      .with(eq("id1"))
      .returning(|_| Ok(Some(test_toolset_row("id1", "user123", "MyExa"))));
    db.expect_get_app_toolset_config()
      .with(eq("builtin-exa-web-search"))
      .returning(|_| Ok(Some(app_config_enabled())));
    db.expect_encryption_key()
      .return_const(b"0123456789abcdef".to_vec());
    db.expect_update_toolset()
      .returning(|row, _| Ok(row.clone()));

    let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time));
    let toolset = service
      .update("user123", "id1", "myexa", None, true, ApiKeyUpdate::Keep)
      .await?;

    assert_eq!("myexa", toolset.name);
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_update_enable_fails_when_app_disabled() -> anyhow::Result<()> {
    let mut db = MockDbService::new();
    let exa = MockExaService::new();
    let time = MockTimeService::new();

    db.expect_get_toolset().with(eq("id1")).returning(|_| {
      Ok(Some(ToolsetRow {
        enabled: false,
        ..test_toolset_row("id1", "user123", "my-exa")
      }))
    });
    db.expect_get_app_toolset_config()
      .with(eq("builtin-exa-web-search"))
      .returning(|_| Ok(None));

    let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time));
    let result = service
      .update("user123", "id1", "my-exa", None, true, ApiKeyUpdate::Keep)
      .await;

    assert!(result.is_err());
    assert!(matches!(
      result.unwrap_err(),
      ToolsetError::ToolsetAppDisabled
    ));
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_update_with_api_key_set() -> anyhow::Result<()> {
    let mut db = MockDbService::new();
    let exa = MockExaService::new();
    let mut time = MockTimeService::new();

    time
      .expect_utc_now()
      .returning(|| Utc.timestamp_opt(1700000001, 0).unwrap());

    db.expect_get_toolset()
      .with(eq("id1"))
      .returning(|_| Ok(Some(test_toolset_row("id1", "user123", "my-exa"))));
    db.expect_get_app_toolset_config()
      .with(eq("builtin-exa-web-search"))
      .returning(|_| Ok(Some(app_config_enabled())));
    db.expect_encryption_key()
      .return_const(b"0123456789abcdef".to_vec());
    db.expect_update_toolset()
      .returning(|row, _| Ok(row.clone()));

    let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time));
    let toolset = service
      .update(
        "user123",
        "id1",
        "my-exa",
        None,
        true,
        ApiKeyUpdate::Set(Some("new-key".to_string())),
      )
      .await?;

    assert_eq!("my-exa", toolset.name);
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_update_with_api_key_keep() -> anyhow::Result<()> {
    let mut db = MockDbService::new();
    let exa = MockExaService::new();
    let mut time = MockTimeService::new();

    time
      .expect_utc_now()
      .returning(|| Utc.timestamp_opt(1700000001, 0).unwrap());

    db.expect_get_toolset()
      .with(eq("id1"))
      .returning(|_| Ok(Some(test_toolset_row("id1", "user123", "my-exa"))));
    db.expect_get_app_toolset_config()
      .with(eq("builtin-exa-web-search"))
      .returning(|_| Ok(Some(app_config_enabled())));
    db.expect_encryption_key()
      .return_const(b"0123456789abcdef".to_vec());
    db.expect_update_toolset()
      .returning(|row, _| Ok(row.clone()));

    let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time));
    let toolset = service
      .update("user123", "id1", "my-exa", None, true, ApiKeyUpdate::Keep)
      .await?;

    assert_eq!("my-exa", toolset.name);
    Ok(())
  }

  // ============================================================================
  // delete Tests
  // ============================================================================

  #[rstest]
  #[tokio::test]
  async fn test_delete_success() -> anyhow::Result<()> {
    let mut db = MockDbService::new();
    let exa = MockExaService::new();
    let time = MockTimeService::new();

    db.expect_get_toolset()
      .with(eq("id1"))
      .returning(|_| Ok(Some(test_toolset_row("id1", "user123", "my-exa"))));
    db.expect_delete_toolset()
      .with(eq("id1"))
      .returning(|_| Ok(()));

    let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time));
    let result = service.delete("user123", "id1").await;

    assert!(result.is_ok());
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_delete_fails_when_not_found() -> anyhow::Result<()> {
    let mut db = MockDbService::new();
    let exa = MockExaService::new();
    let time = MockTimeService::new();

    db.expect_get_toolset()
      .with(eq("id999"))
      .returning(|_| Ok(None));

    let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time));
    let result = service.delete("user123", "id999").await;

    assert!(result.is_err());
    assert!(matches!(
      result.unwrap_err(),
      ToolsetError::ToolsetNotFound(_)
    ));
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_delete_fails_when_not_owned() -> anyhow::Result<()> {
    let mut db = MockDbService::new();
    let exa = MockExaService::new();
    let time = MockTimeService::new();

    db.expect_get_toolset()
      .with(eq("id1"))
      .returning(|_| Ok(Some(test_toolset_row("id1", "user999", "other-exa"))));

    let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time));
    let result = service.delete("user123", "id1").await;

    assert!(result.is_err());
    assert!(matches!(
      result.unwrap_err(),
      ToolsetError::ToolsetNotFound(_)
    ));
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_delete_succeeds_even_when_app_disabled() -> anyhow::Result<()> {
    let mut db = MockDbService::new();
    let exa = MockExaService::new();
    let time = MockTimeService::new();

    db.expect_get_toolset()
      .with(eq("id1"))
      .returning(|_| Ok(Some(test_toolset_row("id1", "user123", "my-exa"))));
    db.expect_delete_toolset()
      .with(eq("id1"))
      .returning(|_| Ok(()));

    let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time));
    let result = service.delete("user123", "id1").await;

    assert!(result.is_ok());
    Ok(())
  }

  // ============================================================================
  // is_type_enabled Tests
  // ============================================================================

  #[rstest]
  #[tokio::test]
  async fn test_is_type_enabled_true() -> anyhow::Result<()> {
    let mut db = MockDbService::new();
    let exa = MockExaService::new();
    let time = MockTimeService::new();

    db.expect_get_app_toolset_config()
      .with(eq("builtin-exa-web-search"))
      .returning(|_| Ok(Some(app_config_enabled())));

    let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time));
    let enabled = service.is_type_enabled("builtin-exa-web-search").await?;

    assert!(enabled);
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_is_type_enabled_false_when_disabled() -> anyhow::Result<()> {
    let mut db = MockDbService::new();
    let exa = MockExaService::new();
    let time = MockTimeService::new();

    db.expect_get_app_toolset_config()
      .with(eq("builtin-exa-web-search"))
      .returning(|_| {
        Ok(Some(AppToolsetConfigRow {
          enabled: false,
          ..app_config_enabled()
        }))
      });

    let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time));
    let enabled = service.is_type_enabled("builtin-exa-web-search").await?;

    assert!(!enabled);
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_is_type_enabled_false_when_no_config() -> anyhow::Result<()> {
    let mut db = MockDbService::new();
    let exa = MockExaService::new();
    let time = MockTimeService::new();

    db.expect_get_app_toolset_config()
      .with(eq("builtin-exa-web-search"))
      .returning(|_| Ok(None));

    let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time));
    let enabled = service.is_type_enabled("builtin-exa-web-search").await?;

    assert!(!enabled);
    Ok(())
  }

  // ============================================================================
  // App-client toolset registration tests
  // ============================================================================

  #[rstest]
  #[tokio::test]
  async fn test_is_app_client_registered_for_toolset_returns_true_when_registered(
  ) -> anyhow::Result<()> {
    let mut db = MockDbService::new();
    let exa = MockExaService::new();
    let time = MockTimeService::new();

    db.expect_get_app_client_toolset_config()
      .with(eq("external-app"))
      .returning(|_| {
        Ok(Some(crate::db::AppClientToolsetConfigRow {
          id: 1,
          app_client_id: "external-app".to_string(),
          config_version: Some("v1.0.0".to_string()),
          toolsets_json: r#"[{"id":"builtin-exa-web-search","toolset_scope":"scope_toolset-builtin-exa-web-search"}]"#.to_string(),
          resource_scope: "scope_resource-bodhi".to_string(),
          created_at: 0,
          updated_at: 0,
        }))
      });

    let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time));
    let registered = service
      .is_app_client_registered_for_toolset("external-app", "builtin-exa-web-search")
      .await?;

    assert!(registered);
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_is_app_client_registered_for_toolset_returns_false_when_toolset_not_in_config(
  ) -> anyhow::Result<()> {
    let mut db = MockDbService::new();
    let exa = MockExaService::new();
    let time = MockTimeService::new();

    db.expect_get_app_client_toolset_config()
      .with(eq("external-app"))
      .returning(|_| {
        Ok(Some(crate::db::AppClientToolsetConfigRow {
          id: 1,
          app_client_id: "external-app".to_string(),
          config_version: Some("v1.0.0".to_string()),
          toolsets_json:
            r#"[{"toolset_id":"other-toolset","toolset_scope":"scope_toolset-other-toolset"}]"#
              .to_string(),
          resource_scope: "scope_resource-bodhi".to_string(),
          created_at: 0,
          updated_at: 0,
        }))
      });

    let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time));
    let registered = service
      .is_app_client_registered_for_toolset("external-app", "builtin-exa-web-search")
      .await?;

    assert!(!registered);
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_is_app_client_registered_for_toolset_returns_false_when_no_config(
  ) -> anyhow::Result<()> {
    let mut db = MockDbService::new();
    let exa = MockExaService::new();
    let time = MockTimeService::new();

    db.expect_get_app_client_toolset_config()
      .with(eq("unknown-app"))
      .returning(|_| Ok(None));

    let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time));
    let registered = service
      .is_app_client_registered_for_toolset("unknown-app", "builtin-exa-web-search")
      .await?;

    assert!(!registered);
    Ok(())
  }
}
