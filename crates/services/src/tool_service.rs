use crate::auth_service::AuthService;
use crate::db::{AppToolConfigRow, DbError, DbService, TimeService, UserToolConfigRow};
use crate::exa_service::{ExaError, ExaService};
use crate::AuthServiceError;
use chrono::DateTime;
use objs::{
  AppError, AppToolConfig, ErrorType, FunctionDefinition, ToolDefinition, ToolExecutionRequest,
  ToolExecutionResponse, ToolScope, UserToolConfig,
};
use serde_json::json;
use std::fmt::Debug;
use std::sync::Arc;

// ============================================================================
// ToolError - Errors from tool operations
// ============================================================================

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum ToolError {
  #[error("not_found")]
  #[error_meta(error_type = ErrorType::NotFound)]
  ToolNotFound(String),

  #[error("not_configured")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  ToolNotConfigured,

  #[error("disabled")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  ToolDisabled,

  #[error("execution_failed")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  ExecutionFailed(String),

  #[error("app_disabled")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  ToolAppDisabled,

  #[error(transparent)]
  DbError(#[from] DbError),

  #[error(transparent)]
  ExaError(#[from] ExaError),

  #[error(transparent)]
  AuthServiceError(#[from] AuthServiceError),
}

// ============================================================================
// ToolService - Service for managing and executing tools
// ============================================================================

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait ToolService: Debug + Send + Sync {
  /// List tool definitions for user (only configured and enabled tools)
  async fn list_tools_for_user(&self, user_id: &str) -> Result<Vec<ToolDefinition>, ToolError>;

  /// Get all available tool definitions (for UI listing)
  fn list_all_tool_definitions(&self) -> Vec<ToolDefinition>;

  /// Get user's tool config by ID
  async fn get_user_tool_config(
    &self,
    user_id: &str,
    tool_id: &str,
  ) -> Result<Option<UserToolConfig>, ToolError>;

  /// Update user's tool config (enable/disable, API key)
  async fn update_user_tool_config(
    &self,
    user_id: &str,
    tool_id: &str,
    enabled: bool,
    api_key: Option<String>,
  ) -> Result<UserToolConfig, ToolError>;

  /// Execute a tool for user
  async fn execute_tool(
    &self,
    user_id: &str,
    tool_id: &str,
    request: ToolExecutionRequest,
  ) -> Result<ToolExecutionResponse, ToolError>;

  /// Check if tool is available for user (app enabled + user enabled + has API key)
  async fn is_tool_available_for_user(
    &self,
    user_id: &str,
    tool_id: &str,
  ) -> Result<bool, ToolError>;

  // ============================================================================
  // App-level tool configuration (admin-controlled)
  // ============================================================================

  /// Get app-level tool config by ID
  async fn get_app_tool_config(&self, tool_id: &str) -> Result<Option<AppToolConfig>, ToolError>;

  /// Check if tool is enabled at app level
  async fn is_tool_enabled_for_app(&self, tool_id: &str) -> Result<bool, ToolError>;

  /// Set app-level tool enabled status (admin only)
  /// This calls Keycloak to sync the client scope, then updates local DB
  async fn set_app_tool_enabled(
    &self,
    admin_token: &str,
    tool_id: &str,
    enabled: bool,
    updated_by: &str,
  ) -> Result<AppToolConfig, ToolError>;

  /// List all app-level tool configs
  async fn list_app_tool_configs(&self) -> Result<Vec<AppToolConfig>, ToolError>;
}

// ============================================================================
// DefaultToolService - Implementation
// ============================================================================

#[derive(Debug)]
pub struct DefaultToolService {
  db_service: Arc<dyn DbService>,
  exa_service: Arc<dyn ExaService>,
  time_service: Arc<dyn TimeService>,
  auth_service: Arc<dyn AuthService>,
}

impl DefaultToolService {
  pub fn new(
    db_service: Arc<dyn DbService>,
    exa_service: Arc<dyn ExaService>,
    time_service: Arc<dyn TimeService>,
    auth_service: Arc<dyn AuthService>,
  ) -> Self {
    Self {
      db_service,
      exa_service,
      time_service,
      auth_service,
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
  fn row_to_config(row: UserToolConfigRow) -> UserToolConfig {
    UserToolConfig {
      tool_id: row.tool_id,
      enabled: row.enabled,
      created_at: DateTime::from_timestamp(row.created_at, 0).unwrap(),
      updated_at: DateTime::from_timestamp(row.updated_at, 0).unwrap(),
    }
  }

  /// Convert app tool config row to public model
  fn app_row_to_config(row: AppToolConfigRow) -> AppToolConfig {
    AppToolConfig {
      tool_id: row.tool_id,
      enabled: row.enabled,
      updated_by: row.updated_by,
      created_at: DateTime::from_timestamp(row.created_at, 0).unwrap(),
      updated_at: DateTime::from_timestamp(row.updated_at, 0).unwrap(),
    }
  }
}

#[async_trait::async_trait]
impl ToolService for DefaultToolService {
  async fn list_tools_for_user(&self, user_id: &str) -> Result<Vec<ToolDefinition>, ToolError> {
    // Get user's configured tools
    let configs = self.db_service.list_user_tool_configs(user_id).await?;

    // Filter to only enabled tools with API keys
    let enabled_tool_ids: Vec<String> = configs
      .into_iter()
      .filter(|c| c.enabled && c.encrypted_api_key.is_some())
      .map(|c| c.tool_id)
      .collect();

    // Return definitions for enabled tools
    Ok(
      Self::builtin_tool_definitions()
        .into_iter()
        .filter(|def| enabled_tool_ids.contains(&def.function.name))
        .collect(),
    )
  }

  fn list_all_tool_definitions(&self) -> Vec<ToolDefinition> {
    Self::builtin_tool_definitions()
  }

  async fn get_user_tool_config(
    &self,
    user_id: &str,
    tool_id: &str,
  ) -> Result<Option<UserToolConfig>, ToolError> {
    // Verify tool exists
    if !Self::builtin_tool_definitions()
      .iter()
      .any(|def| def.function.name == tool_id)
    {
      return Err(ToolError::ToolNotFound(tool_id.to_string()));
    }

    let config = self
      .db_service
      .get_user_tool_config(user_id, tool_id)
      .await?;

    Ok(config.map(Self::row_to_config))
  }

  async fn update_user_tool_config(
    &self,
    user_id: &str,
    tool_id: &str,
    enabled: bool,
    api_key: Option<String>,
  ) -> Result<UserToolConfig, ToolError> {
    // Verify tool exists
    if !Self::builtin_tool_definitions()
      .iter()
      .any(|def| def.function.name == tool_id)
    {
      return Err(ToolError::ToolNotFound(tool_id.to_string()));
    }

    let now = self.time_service.utc_now();
    let now_ts = now.timestamp();

    // Get existing config or create new one
    let existing = self
      .db_service
      .get_user_tool_config(user_id, tool_id)
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

    let config = UserToolConfigRow {
      id: existing.as_ref().map(|e| e.id).unwrap_or(0),
      user_id: user_id.to_string(),
      tool_id: tool_id.to_string(),
      enabled,
      encrypted_api_key,
      salt,
      nonce,
      created_at: existing.as_ref().map(|e| e.created_at).unwrap_or(now_ts),
      updated_at: now_ts,
    };

    let result = self.db_service.upsert_user_tool_config(&config).await?;
    Ok(Self::row_to_config(result))
  }

  async fn execute_tool(
    &self,
    user_id: &str,
    tool_id: &str,
    request: ToolExecutionRequest,
  ) -> Result<ToolExecutionResponse, ToolError> {
    // Verify tool is available for user
    if !self.is_tool_available_for_user(user_id, tool_id).await? {
      return Err(ToolError::ToolNotConfigured);
    }

    // Get decrypted API key
    let config = self
      .db_service
      .get_user_tool_config(user_id, tool_id)
      .await?
      .ok_or(ToolError::ToolNotConfigured)?;

    let (encrypted, salt, nonce) = match (config.encrypted_api_key, config.salt, config.nonce) {
      (Some(e), Some(s), Some(n)) => (e, s, n),
      _ => return Err(ToolError::ToolNotConfigured),
    };

    let master_key = self.db_service.encryption_key();
    let api_key = crate::db::encryption::decrypt_api_key(master_key, &encrypted, &salt, &nonce)
      .map_err(|e| DbError::EncryptionError(e.to_string()))?;

    // Execute based on tool type
    match tool_id {
      "builtin-exa-web-search" => self.execute_exa_search(&api_key, request).await,
      _ => Err(ToolError::ToolNotFound(tool_id.to_string())),
    }
  }

  async fn is_tool_available_for_user(
    &self,
    user_id: &str,
    tool_id: &str,
  ) -> Result<bool, ToolError> {
    // Check app-level first
    if !self.is_tool_enabled_for_app(tool_id).await? {
      return Ok(false);
    }

    // Then check user-level
    let config = self
      .db_service
      .get_user_tool_config(user_id, tool_id)
      .await?;

    Ok(match config {
      Some(c) => c.enabled && c.encrypted_api_key.is_some(),
      None => false,
    })
  }

  // ============================================================================
  // App-level tool configuration (admin-controlled)
  // ============================================================================

  async fn get_app_tool_config(&self, tool_id: &str) -> Result<Option<AppToolConfig>, ToolError> {
    // Verify tool exists
    if !Self::builtin_tool_definitions()
      .iter()
      .any(|def| def.function.name == tool_id)
    {
      return Err(ToolError::ToolNotFound(tool_id.to_string()));
    }

    let config = self.db_service.get_app_tool_config(tool_id).await?;
    Ok(config.map(Self::app_row_to_config))
  }

  async fn is_tool_enabled_for_app(&self, tool_id: &str) -> Result<bool, ToolError> {
    let config = self.db_service.get_app_tool_config(tool_id).await?;
    // No row means disabled (default state)
    Ok(config.map(|c| c.enabled).unwrap_or(false))
  }

  async fn set_app_tool_enabled(
    &self,
    admin_token: &str,
    tool_id: &str,
    enabled: bool,
    updated_by: &str,
  ) -> Result<AppToolConfig, ToolError> {
    // Verify tool exists
    if !Self::builtin_tool_definitions()
      .iter()
      .any(|def| def.function.name == tool_id)
    {
      return Err(ToolError::ToolNotFound(tool_id.to_string()));
    }

    // Get the tool scope string
    let tool_scope = ToolScope::scope_for_tool_id(tool_id)
      .ok_or_else(|| ToolError::ToolNotFound(tool_id.to_string()))?
      .to_string();

    // Call Keycloak first
    if enabled {
      self
        .auth_service
        .enable_tool_scope(admin_token, &tool_scope)
        .await?;
    } else {
      self
        .auth_service
        .disable_tool_scope(admin_token, &tool_scope)
        .await?;
    }

    // On Keycloak success, update local DB
    let now = self.time_service.utc_now();
    let now_ts = now.timestamp();

    // Get existing config
    let existing = self.db_service.get_app_tool_config(tool_id).await?;

    let config = AppToolConfigRow {
      id: existing.as_ref().map(|e| e.id).unwrap_or(0),
      tool_id: tool_id.to_string(),
      enabled,
      updated_by: updated_by.to_string(),
      created_at: existing.as_ref().map(|e| e.created_at).unwrap_or(now_ts),
      updated_at: now_ts,
    };

    // Note: If DB write fails after Keycloak success, we log but return success
    // as Keycloak is the source of truth (per spec requirement)
    match self.db_service.upsert_app_tool_config(&config).await {
      Ok(result) => Ok(Self::app_row_to_config(result)),
      Err(e) => {
        tracing::error!(
          "DB write failed after Keycloak success for tool {}: {}. Keycloak state is source of truth.",
          tool_id,
          e
        );
        // Return a synthetic config since Keycloak succeeded
        Ok(AppToolConfig {
          tool_id: tool_id.to_string(),
          enabled,
          updated_by: updated_by.to_string(),
          created_at: DateTime::from_timestamp(config.created_at, 0).unwrap(),
          updated_at: DateTime::from_timestamp(now_ts, 0).unwrap(),
        })
      }
    }
  }

  async fn list_app_tool_configs(&self) -> Result<Vec<AppToolConfig>, ToolError> {
    let configs = self.db_service.list_app_tool_configs().await?;
    Ok(configs.into_iter().map(Self::app_row_to_config).collect())
  }
}

impl DefaultToolService {
  async fn execute_exa_search(
    &self,
    api_key: &str,
    request: ToolExecutionRequest,
  ) -> Result<ToolExecutionResponse, ToolError> {
    // Parse arguments
    let query = request.arguments["query"]
      .as_str()
      .ok_or_else(|| ToolError::ExecutionFailed("Missing 'query' parameter".to_string()))?;

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

        Ok(ToolExecutionResponse {
          tool_call_id: request.tool_call_id,
          result: Some(json!({
            "results": results,
            "query_used": response.autoprompt_string,
          })),
          error: None,
        })
      }
      Err(e) => Ok(ToolExecutionResponse {
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
  use crate::auth_service::MockAuthService;
  use crate::db::{MockDbService, MockTimeService};
  use crate::exa_service::MockExaService;
  use mockall::predicate::*;
  use rstest::rstest;

  #[rstest]
  fn test_list_all_tool_definitions() {
    let db = MockDbService::new();
    let exa = MockExaService::new();
    let time = MockTimeService::new();
    let auth = MockAuthService::new();

    // No expectations needed for list_all

    let service =
      DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time), Arc::new(auth));
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
    let auth = MockAuthService::new();

    db.expect_list_user_tool_configs()
      .with(eq("user123"))
      .returning(|_| {
        Ok(vec![
          UserToolConfigRow {
            id: 1,
            user_id: "user123".to_string(),
            tool_id: "builtin-exa-web-search".to_string(),
            enabled: true,
            encrypted_api_key: Some("encrypted".to_string()),
            salt: Some("salt".to_string()),
            nonce: Some("nonce".to_string()),
            created_at: 1700000000,
            updated_at: 1700000000,
          },
          UserToolConfigRow {
            id: 2,
            user_id: "user123".to_string(),
            tool_id: "another-tool".to_string(),
            enabled: false,
            encrypted_api_key: Some("encrypted2".to_string()),
            salt: Some("salt2".to_string()),
            nonce: Some("nonce2".to_string()),
            created_at: 1700000000,
            updated_at: 1700000000,
          },
        ])
      });

    let service =
      DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time), Arc::new(auth));
    let tools = service.list_tools_for_user("user123").await?;

    assert_eq!(1, tools.len());
    assert_eq!("builtin-exa-web-search", tools[0].function.name);

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_get_user_tool_config_returns_config() -> anyhow::Result<()> {
    let mut db = MockDbService::new();
    let exa = MockExaService::new();
    let time = MockTimeService::new();
    let auth = MockAuthService::new();

    db.expect_get_user_tool_config()
      .with(eq("user123"), eq("builtin-exa-web-search"))
      .returning(|_, _| {
        Ok(Some(UserToolConfigRow {
          id: 1,
          user_id: "user123".to_string(),
          tool_id: "builtin-exa-web-search".to_string(),
          enabled: true,
          encrypted_api_key: Some("encrypted".to_string()),
          salt: Some("salt".to_string()),
          nonce: Some("nonce".to_string()),
          created_at: 1700000000,
          updated_at: 1700000000,
        }))
      });

    let service =
      DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time), Arc::new(auth));
    let config = service
      .get_user_tool_config("user123", "builtin-exa-web-search")
      .await?;

    assert!(config.is_some());
    let config = config.unwrap();
    assert_eq!("builtin-exa-web-search", config.tool_id);
    assert!(config.enabled);

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_get_user_tool_config_returns_error_for_unknown_tool() -> anyhow::Result<()> {
    let db = MockDbService::new();
    let exa = MockExaService::new();
    let time = MockTimeService::new();
    let auth = MockAuthService::new();

    let service =
      DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time), Arc::new(auth));
    let result = service
      .get_user_tool_config("user123", "unknown-tool")
      .await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ToolError::ToolNotFound(_)));

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_is_tool_available_returns_true_when_app_and_user_enabled() -> anyhow::Result<()> {
    let mut db = MockDbService::new();
    let exa = MockExaService::new();
    let time = MockTimeService::new();
    let auth = MockAuthService::new();

    // App-level check
    db.expect_get_app_tool_config()
      .with(eq("builtin-exa-web-search"))
      .returning(|_| {
        Ok(Some(crate::db::AppToolConfigRow {
          id: 1,
          tool_id: "builtin-exa-web-search".to_string(),
          enabled: true,
          updated_by: "admin".to_string(),
          created_at: 1700000000,
          updated_at: 1700000000,
        }))
      });

    // User-level check
    db.expect_get_user_tool_config()
      .with(eq("user123"), eq("builtin-exa-web-search"))
      .returning(|_, _| {
        Ok(Some(UserToolConfigRow {
          id: 1,
          user_id: "user123".to_string(),
          tool_id: "builtin-exa-web-search".to_string(),
          enabled: true,
          encrypted_api_key: Some("encrypted".to_string()),
          salt: Some("salt".to_string()),
          nonce: Some("nonce".to_string()),
          created_at: 1700000000,
          updated_at: 1700000000,
        }))
      });

    let service =
      DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time), Arc::new(auth));
    let available = service
      .is_tool_available_for_user("user123", "builtin-exa-web-search")
      .await?;

    assert!(available);

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_is_tool_available_returns_false_when_app_disabled() -> anyhow::Result<()> {
    let mut db = MockDbService::new();
    let exa = MockExaService::new();
    let time = MockTimeService::new();
    let auth = MockAuthService::new();

    // App-level disabled - should short circuit, never check user config
    db.expect_get_app_tool_config()
      .with(eq("builtin-exa-web-search"))
      .returning(|_| {
        Ok(Some(crate::db::AppToolConfigRow {
          id: 1,
          tool_id: "builtin-exa-web-search".to_string(),
          enabled: false,
          updated_by: "admin".to_string(),
          created_at: 1700000000,
          updated_at: 1700000000,
        }))
      });

    let service =
      DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time), Arc::new(auth));
    let available = service
      .is_tool_available_for_user("user123", "builtin-exa-web-search")
      .await?;

    assert!(!available);

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_is_tool_available_returns_false_when_user_disabled() -> anyhow::Result<()> {
    let mut db = MockDbService::new();
    let exa = MockExaService::new();
    let time = MockTimeService::new();
    let auth = MockAuthService::new();

    // App-level enabled
    db.expect_get_app_tool_config()
      .with(eq("builtin-exa-web-search"))
      .returning(|_| {
        Ok(Some(crate::db::AppToolConfigRow {
          id: 1,
          tool_id: "builtin-exa-web-search".to_string(),
          enabled: true,
          updated_by: "admin".to_string(),
          created_at: 1700000000,
          updated_at: 1700000000,
        }))
      });

    // User-level disabled
    db.expect_get_user_tool_config()
      .with(eq("user123"), eq("builtin-exa-web-search"))
      .returning(|_, _| {
        Ok(Some(UserToolConfigRow {
          id: 1,
          user_id: "user123".to_string(),
          tool_id: "builtin-exa-web-search".to_string(),
          enabled: false,
          encrypted_api_key: Some("encrypted".to_string()),
          salt: Some("salt".to_string()),
          nonce: Some("nonce".to_string()),
          created_at: 1700000000,
          updated_at: 1700000000,
        }))
      });

    let service =
      DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time), Arc::new(auth));
    let available = service
      .is_tool_available_for_user("user123", "builtin-exa-web-search")
      .await?;

    assert!(!available);

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_is_tool_available_returns_false_when_no_api_key() -> anyhow::Result<()> {
    let mut db = MockDbService::new();
    let exa = MockExaService::new();
    let time = MockTimeService::new();
    let auth = MockAuthService::new();

    // App-level enabled
    db.expect_get_app_tool_config()
      .with(eq("builtin-exa-web-search"))
      .returning(|_| {
        Ok(Some(crate::db::AppToolConfigRow {
          id: 1,
          tool_id: "builtin-exa-web-search".to_string(),
          enabled: true,
          updated_by: "admin".to_string(),
          created_at: 1700000000,
          updated_at: 1700000000,
        }))
      });

    // User enabled but no API key
    db.expect_get_user_tool_config()
      .with(eq("user123"), eq("builtin-exa-web-search"))
      .returning(|_, _| {
        Ok(Some(UserToolConfigRow {
          id: 1,
          user_id: "user123".to_string(),
          tool_id: "builtin-exa-web-search".to_string(),
          enabled: true,
          encrypted_api_key: None,
          salt: None,
          nonce: None,
          created_at: 1700000000,
          updated_at: 1700000000,
        }))
      });

    let service =
      DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time), Arc::new(auth));
    let available = service
      .is_tool_available_for_user("user123", "builtin-exa-web-search")
      .await?;

    assert!(!available);

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_is_tool_available_returns_false_when_no_app_config() -> anyhow::Result<()> {
    let mut db = MockDbService::new();
    let exa = MockExaService::new();
    let time = MockTimeService::new();
    let auth = MockAuthService::new();

    // No app config means disabled
    db.expect_get_app_tool_config()
      .with(eq("builtin-exa-web-search"))
      .returning(|_| Ok(None));

    let service =
      DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time), Arc::new(auth));
    let available = service
      .is_tool_available_for_user("user123", "builtin-exa-web-search")
      .await?;

    assert!(!available);

    Ok(())
  }
}
