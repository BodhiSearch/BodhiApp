use crate::db::{DbError, DbService, TimeService, UserToolConfigRow};
use crate::exa_service::{ExaError, ExaService};
use chrono::DateTime;
use objs::{
  AppError, ErrorType, FunctionDefinition, ToolDefinition, ToolExecutionRequest,
  ToolExecutionResponse, UserToolConfig,
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

  #[error(transparent)]
  DbError(#[from] DbError),

  #[error(transparent)]
  ExaError(#[from] ExaError),
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

  /// Check if tool is available for user (enabled + has API key)
  async fn is_tool_available_for_user(
    &self,
    user_id: &str,
    tool_id: &str,
  ) -> Result<bool, ToolError>;
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
  fn row_to_config(row: UserToolConfigRow) -> UserToolConfig {
    UserToolConfig {
      tool_id: row.tool_id,
      enabled: row.enabled,
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
    let config = self
      .db_service
      .get_user_tool_config(user_id, tool_id)
      .await?;

    Ok(match config {
      Some(c) => c.enabled && c.encrypted_api_key.is_some(),
      None => false,
    })
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

    let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time));
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

    let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time));
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

    let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time));
    let result = service
      .get_user_tool_config("user123", "unknown-tool")
      .await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ToolError::ToolNotFound(_)));

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_is_tool_available_returns_true_when_enabled_with_key() -> anyhow::Result<()> {
    let mut db = MockDbService::new();
    let exa = MockExaService::new();
    let time = MockTimeService::new();

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

    let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time));
    let available = service
      .is_tool_available_for_user("user123", "builtin-exa-web-search")
      .await?;

    assert!(available);

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_is_tool_available_returns_false_when_disabled() -> anyhow::Result<()> {
    let mut db = MockDbService::new();
    let exa = MockExaService::new();
    let time = MockTimeService::new();

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

    let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time));
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

    let service = DefaultToolService::new(Arc::new(db), Arc::new(exa), Arc::new(time));
    let available = service
      .is_tool_available_for_user("user123", "builtin-exa-web-search")
      .await?;

    assert!(!available);

    Ok(())
  }
}
