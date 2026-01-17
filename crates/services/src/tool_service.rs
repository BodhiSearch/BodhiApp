use crate::db::{AppToolsetConfigRow, DbError, DbService, TimeService, UserToolsetConfigRow};
use crate::exa_service::{ExaError, ExaService};
use chrono::DateTime;
use objs::{
  AppError, AppToolsetConfig, ErrorType, FunctionDefinition, ToolDefinition,
  ToolsetExecutionRequest, ToolsetExecutionResponse, UserToolsetConfig,
};
use serde_json::json;
use std::fmt::Debug;
use std::sync::Arc;

// ============================================================================
// ToolsetError - Errors from toolset operations
// ============================================================================

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum ToolsetError {
  #[error("not_found")]
  #[error_meta(error_type = ErrorType::NotFound)]
  ToolsetNotFound(String),

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

  /// Get user's toolset config by ID
  async fn get_user_toolset_config(
    &self,
    user_id: &str,
    toolset_id: &str,
  ) -> Result<Option<UserToolsetConfig>, ToolsetError>;

  /// Update user's toolset config (enable/disable, API key)
  async fn update_user_toolset_config(
    &self,
    user_id: &str,
    toolset_id: &str,
    enabled: bool,
    api_key: Option<String>,
  ) -> Result<UserToolsetConfig, ToolsetError>;

  /// Delete user's toolset config (clears API key)
  async fn delete_user_toolset_config(
    &self,
    user_id: &str,
    toolset_id: &str,
  ) -> Result<(), ToolsetError>;

  /// Execute a tool within a toolset for user
  async fn execute_toolset_tool(
    &self,
    user_id: &str,
    toolset_id: &str,
    request: ToolsetExecutionRequest,
  ) -> Result<ToolsetExecutionResponse, ToolsetError>;

  /// Check if toolset is available for user (app enabled + user enabled + has API key)
  async fn is_toolset_available_for_user(
    &self,
    user_id: &str,
    toolset_id: &str,
  ) -> Result<bool, ToolsetError>;

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

  /// Convert database row to public config model
  fn row_to_config(row: UserToolsetConfigRow) -> UserToolsetConfig {
    UserToolsetConfig {
      toolset_id: row.toolset_id,
      enabled: row.enabled,
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
    // Get user's configured toolsets
    let configs = self.db_service.list_user_toolset_configs(user_id).await?;

    // Filter to only enabled toolsets with API keys
    let enabled_toolset_ids: Vec<String> = configs
      .into_iter()
      .filter(|c| c.enabled && c.encrypted_api_key.is_some())
      .map(|c| c.toolset_id)
      .collect();

    // Return definitions for enabled toolsets
    Ok(
      Self::builtin_tool_definitions()
        .into_iter()
        .filter(|def| enabled_toolset_ids.contains(&def.function.name))
        .collect(),
    )
  }

  fn list_all_tool_definitions(&self) -> Vec<ToolDefinition> {
    Self::builtin_tool_definitions()
  }

  async fn get_user_toolset_config(
    &self,
    user_id: &str,
    toolset_id: &str,
  ) -> Result<Option<UserToolsetConfig>, ToolsetError> {
    // Verify toolset exists
    if !Self::builtin_tool_definitions()
      .iter()
      .any(|def| def.function.name == toolset_id)
    {
      return Err(ToolsetError::ToolsetNotFound(toolset_id.to_string()));
    }

    let config = self
      .db_service
      .get_user_toolset_config(user_id, toolset_id)
      .await?;

    Ok(config.map(Self::row_to_config))
  }

  async fn update_user_toolset_config(
    &self,
    user_id: &str,
    toolset_id: &str,
    enabled: bool,
    api_key: Option<String>,
  ) -> Result<UserToolsetConfig, ToolsetError> {
    // Verify toolset exists
    if !Self::builtin_tool_definitions()
      .iter()
      .any(|def| def.function.name == toolset_id)
    {
      return Err(ToolsetError::ToolsetNotFound(toolset_id.to_string()));
    }

    let now = self.time_service.utc_now();
    let now_ts = now.timestamp();

    // Get existing config or create new one
    let existing = self
      .db_service
      .get_user_toolset_config(user_id, toolset_id)
      .await?;

    let (encrypted_api_key, salt, nonce) = if let Some(ref key) = api_key {
      // Encrypt the API key
      let master_key = self.db_service.encryption_key();
      let (enc, s, n) = crate::db::encryption::encrypt_api_key(master_key, key)
        .map_err(|e| DbError::EncryptionError(e.to_string()))?;
      (Some(enc), Some(s), Some(n))
    } else if let Some(ref existing) = existing {
      // Keep existing encryption if not changing key
      (
        existing.encrypted_api_key.clone(),
        existing.salt.clone(),
        existing.nonce.clone(),
      )
    } else {
      (None, None, None)
    };

    let config = UserToolsetConfigRow {
      id: existing.as_ref().map(|e| e.id).unwrap_or(0),
      user_id: user_id.to_string(),
      toolset_id: toolset_id.to_string(),
      enabled,
      encrypted_api_key,
      salt,
      nonce,
      created_at: existing.as_ref().map(|e| e.created_at).unwrap_or(now_ts),
      updated_at: now_ts,
    };

    let result = self.db_service.upsert_user_toolset_config(&config).await?;
    Ok(Self::row_to_config(result))
  }

  async fn delete_user_toolset_config(
    &self,
    user_id: &str,
    toolset_id: &str,
  ) -> Result<(), ToolsetError> {
    // Verify toolset exists
    if !Self::builtin_tool_definitions()
      .iter()
      .any(|def| def.function.name == toolset_id)
    {
      return Err(ToolsetError::ToolsetNotFound(toolset_id.to_string()));
    }

    self
      .db_service
      .delete_user_toolset_config(user_id, toolset_id)
      .await?;

    Ok(())
  }

  async fn execute_toolset_tool(
    &self,
    user_id: &str,
    toolset_id: &str,
    request: ToolsetExecutionRequest,
  ) -> Result<ToolsetExecutionResponse, ToolsetError> {
    // Verify toolset is available for user
    if !self
      .is_toolset_available_for_user(user_id, toolset_id)
      .await?
    {
      return Err(ToolsetError::ToolsetNotConfigured);
    }

    // Get decrypted API key
    let config = self
      .db_service
      .get_user_toolset_config(user_id, toolset_id)
      .await?
      .ok_or(ToolsetError::ToolsetNotConfigured)?;

    let (encrypted, salt, nonce) = match (config.encrypted_api_key, config.salt, config.nonce) {
      (Some(e), Some(s), Some(n)) => (e, s, n),
      _ => return Err(ToolsetError::ToolsetNotConfigured),
    };

    let master_key = self.db_service.encryption_key();
    let api_key = crate::db::encryption::decrypt_api_key(master_key, &encrypted, &salt, &nonce)
      .map_err(|e| DbError::EncryptionError(e.to_string()))?;

    // Execute based on toolset type
    match toolset_id {
      "builtin-exa-web-search" => self.execute_exa_search(&api_key, request).await,
      _ => Err(ToolsetError::ToolsetNotFound(toolset_id.to_string())),
    }
  }

  async fn is_toolset_available_for_user(
    &self,
    user_id: &str,
    toolset_id: &str,
  ) -> Result<bool, ToolsetError> {
    // Check app-level first
    if !self.is_toolset_enabled_for_app(toolset_id).await? {
      return Ok(false);
    }

    // Then check user-level
    let config = self
      .db_service
      .get_user_toolset_config(user_id, toolset_id)
      .await?;

    Ok(match config {
      Some(c) => c.enabled && c.encrypted_api_key.is_some(),
      None => false,
    })
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
      t.get("toolset_id")
        .and_then(|v| v.as_str())
        .map(|id| id == toolset_id)
        .unwrap_or(false)
    }))
  }
}

impl DefaultToolService {
  async fn execute_exa_search(
    &self,
    api_key: &str,
    request: ToolsetExecutionRequest,
  ) -> Result<ToolsetExecutionResponse, ToolsetError> {
    // Parse arguments
    let query = request.arguments["query"]
      .as_str()
      .ok_or_else(|| ToolsetError::ExecutionFailed("Missing 'query' parameter".to_string()))?;

    let num_results = request.arguments["num_results"].as_u64().map(|n| n as u32);

    // Call Exa service
    match self.exa_service.search(api_key, query, num_results).await {
      Ok(response) => {
        // Format results for LLM
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
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::db::{MockDbService, MockTimeService};
  use crate::exa_service::MockExaService;
  use mockall::predicate::*;
  use rstest::rstest;

  #[rstest]
  fn test_list_all_tool_definitions() {
    let db = MockDbService::new();
    let exa = MockExaService::new();
    let time = MockTimeService::new();

    // No expectations needed for list_all

    let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time));
    let defs = service.list_all_tool_definitions();

    assert_eq!(1, defs.len());
    assert_eq!("builtin-exa-web-search", defs[0].function.name);
  }

  #[rstest]
  #[tokio::test]
  async fn test_list_tools_for_user_returns_only_enabled_configured() -> anyhow::Result<()> {
    let mut db = MockDbService::new();
    let exa = MockExaService::new();
    let time = MockTimeService::new();

    db.expect_list_user_toolset_configs()
      .with(eq("user123"))
      .returning(|_| {
        Ok(vec![
          UserToolsetConfigRow {
            id: 1,
            user_id: "user123".to_string(),
            toolset_id: "builtin-exa-web-search".to_string(),
            enabled: true,
            encrypted_api_key: Some("encrypted".to_string()),
            salt: Some("salt".to_string()),
            nonce: Some("nonce".to_string()),
            created_at: 1700000000,
            updated_at: 1700000000,
          },
          UserToolsetConfigRow {
            id: 2,
            user_id: "user123".to_string(),
            toolset_id: "another-toolset".to_string(),
            enabled: false,
            encrypted_api_key: Some("encrypted2".to_string()),
            salt: Some("salt2".to_string()),
            nonce: Some("nonce2".to_string()),
            created_at: 1700000000,
            updated_at: 1700000000,
          },
        ])
      });

    let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time));
    let tools = service.list_tools_for_user("user123").await?;

    assert_eq!(1, tools.len());
    assert_eq!("builtin-exa-web-search", tools[0].function.name);

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_get_user_toolset_config_returns_config() -> anyhow::Result<()> {
    let mut db = MockDbService::new();
    let exa = MockExaService::new();
    let time = MockTimeService::new();

    db.expect_get_user_toolset_config()
      .with(eq("user123"), eq("builtin-exa-web-search"))
      .returning(|_, _| {
        Ok(Some(UserToolsetConfigRow {
          id: 1,
          user_id: "user123".to_string(),
          toolset_id: "builtin-exa-web-search".to_string(),
          enabled: true,
          encrypted_api_key: Some("encrypted".to_string()),
          salt: Some("salt".to_string()),
          nonce: Some("nonce".to_string()),
          created_at: 1700000000,
          updated_at: 1700000000,
        }))
      });

    let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time));
    let config = service
      .get_user_toolset_config("user123", "builtin-exa-web-search")
      .await?;

    assert!(config.is_some());
    let config = config.unwrap();
    assert_eq!("builtin-exa-web-search", config.toolset_id);
    assert!(config.enabled);

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_get_user_toolset_config_returns_error_for_unknown_toolset() -> anyhow::Result<()> {
    let db = MockDbService::new();
    let exa = MockExaService::new();
    let time = MockTimeService::new();

    let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time));
    let result = service
      .get_user_toolset_config("user123", "unknown-toolset")
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
  async fn test_is_toolset_available_returns_true_when_app_and_user_enabled() -> anyhow::Result<()>
  {
    let mut db = MockDbService::new();
    let exa = MockExaService::new();
    let time = MockTimeService::new();

    // App-level check
    db.expect_get_app_toolset_config()
      .with(eq("builtin-exa-web-search"))
      .returning(|_| {
        Ok(Some(crate::db::AppToolsetConfigRow {
          id: 1,
          toolset_id: "builtin-exa-web-search".to_string(),
          enabled: true,
          updated_by: "admin".to_string(),
          created_at: 1700000000,
          updated_at: 1700000000,
        }))
      });

    // User-level check
    db.expect_get_user_toolset_config()
      .with(eq("user123"), eq("builtin-exa-web-search"))
      .returning(|_, _| {
        Ok(Some(UserToolsetConfigRow {
          id: 1,
          user_id: "user123".to_string(),
          toolset_id: "builtin-exa-web-search".to_string(),
          enabled: true,
          encrypted_api_key: Some("encrypted".to_string()),
          salt: Some("salt".to_string()),
          nonce: Some("nonce".to_string()),
          created_at: 1700000000,
          updated_at: 1700000000,
        }))
      });

    let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time));
    let available = service
      .is_toolset_available_for_user("user123", "builtin-exa-web-search")
      .await?;

    assert!(available);

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_is_toolset_available_returns_false_when_app_disabled() -> anyhow::Result<()> {
    let mut db = MockDbService::new();
    let exa = MockExaService::new();
    let time = MockTimeService::new();

    // App-level disabled - should short circuit, never check user config
    db.expect_get_app_toolset_config()
      .with(eq("builtin-exa-web-search"))
      .returning(|_| {
        Ok(Some(crate::db::AppToolsetConfigRow {
          id: 1,
          toolset_id: "builtin-exa-web-search".to_string(),
          enabled: false,
          updated_by: "admin".to_string(),
          created_at: 1700000000,
          updated_at: 1700000000,
        }))
      });

    let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time));
    let available = service
      .is_toolset_available_for_user("user123", "builtin-exa-web-search")
      .await?;

    assert!(!available);

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_is_toolset_available_returns_false_when_user_disabled() -> anyhow::Result<()> {
    let mut db = MockDbService::new();
    let exa = MockExaService::new();
    let time = MockTimeService::new();

    // App-level enabled
    db.expect_get_app_toolset_config()
      .with(eq("builtin-exa-web-search"))
      .returning(|_| {
        Ok(Some(crate::db::AppToolsetConfigRow {
          id: 1,
          toolset_id: "builtin-exa-web-search".to_string(),
          enabled: true,
          updated_by: "admin".to_string(),
          created_at: 1700000000,
          updated_at: 1700000000,
        }))
      });

    // User-level disabled
    db.expect_get_user_toolset_config()
      .with(eq("user123"), eq("builtin-exa-web-search"))
      .returning(|_, _| {
        Ok(Some(UserToolsetConfigRow {
          id: 1,
          user_id: "user123".to_string(),
          toolset_id: "builtin-exa-web-search".to_string(),
          enabled: false,
          encrypted_api_key: Some("encrypted".to_string()),
          salt: Some("salt".to_string()),
          nonce: Some("nonce".to_string()),
          created_at: 1700000000,
          updated_at: 1700000000,
        }))
      });

    let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time));
    let available = service
      .is_toolset_available_for_user("user123", "builtin-exa-web-search")
      .await?;

    assert!(!available);

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_is_toolset_available_returns_false_when_no_api_key() -> anyhow::Result<()> {
    let mut db = MockDbService::new();
    let exa = MockExaService::new();
    let time = MockTimeService::new();

    // App-level enabled
    db.expect_get_app_toolset_config()
      .with(eq("builtin-exa-web-search"))
      .returning(|_| {
        Ok(Some(crate::db::AppToolsetConfigRow {
          id: 1,
          toolset_id: "builtin-exa-web-search".to_string(),
          enabled: true,
          updated_by: "admin".to_string(),
          created_at: 1700000000,
          updated_at: 1700000000,
        }))
      });

    // User enabled but no API key
    db.expect_get_user_toolset_config()
      .with(eq("user123"), eq("builtin-exa-web-search"))
      .returning(|_, _| {
        Ok(Some(UserToolsetConfigRow {
          id: 1,
          user_id: "user123".to_string(),
          toolset_id: "builtin-exa-web-search".to_string(),
          enabled: true,
          encrypted_api_key: None,
          salt: None,
          nonce: None,
          created_at: 1700000000,
          updated_at: 1700000000,
        }))
      });

    let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time));
    let available = service
      .is_toolset_available_for_user("user123", "builtin-exa-web-search")
      .await?;

    assert!(!available);

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_is_toolset_available_returns_false_when_no_app_config() -> anyhow::Result<()> {
    let mut db = MockDbService::new();
    let exa = MockExaService::new();
    let time = MockTimeService::new();

    // No app config means disabled
    db.expect_get_app_toolset_config()
      .with(eq("builtin-exa-web-search"))
      .returning(|_| Ok(None));

    let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time));
    let available = service
      .is_toolset_available_for_user("user123", "builtin-exa-web-search")
      .await?;

    assert!(!available);

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
          config_version: "v1.0.0".to_string(),
          toolsets_json: r#"[{"toolset_id":"builtin-exa-web-search","toolset_scope":"scope_toolset-builtin-exa-web-search"}]"#.to_string(),
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
          config_version: "v1.0.0".to_string(),
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
