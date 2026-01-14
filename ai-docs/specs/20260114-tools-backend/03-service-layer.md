# Service Layer - Tools Feature

> Layer: `services` crate | Status: Planning

## ToolService Trait

```rust
// crates/services/src/tool_service.rs
#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait ToolService: std::fmt::Debug + Send + Sync {
    /// List tool definitions for user (only configured tools)
    async fn list_tools_for_user(&self, user_id: &str) -> Result<Vec<ToolDefinition>, ToolError>;

    /// Get all available tool definitions (for UI listing)
    fn list_all_tool_definitions(&self) -> Vec<ToolDefinition>;

    /// Get user's tool config by ID
    async fn get_user_tool_config(&self, user_id: &str, tool_id: &str) -> Result<Option<UserToolConfig>, ToolError>;

    /// Update user's tool config (enable/disable, API key)
    async fn update_user_tool_config(
        &self,
        user_id: &str,
        tool_id: &str,
        enabled: bool,
        api_key: Option<&str>,
    ) -> Result<UserToolConfig, ToolError>;

    /// Execute a tool for user
    async fn execute_tool(
        &self,
        user_id: &str,
        tool_id: &str,
        request: ToolExecutionRequest,
    ) -> Result<ToolExecutionResponse, ToolError>;

    /// Check if tool is available for user (enabled + has API key)
    async fn is_tool_available_for_user(&self, user_id: &str, tool_id: &str) -> Result<bool, ToolError>;
}
```

## DefaultToolService Implementation

```rust
#[derive(Debug)]
pub struct DefaultToolService {
    db_service: Arc<dyn DbService>,
    exa_service: Arc<dyn ExaService>,
    time_service: Arc<dyn TimeService>,
}

impl DefaultToolService {
    /// Static registry of built-in tools
    fn builtin_tool_definitions() -> Vec<ToolDefinition> {
        vec![
            ToolDefinition {
                tool_type: "function".to_string(),
                function: FunctionDefinition {
                    name: "builtin-exa-web-search".to_string(),
                    description: "Search the web for current information using Exa AI semantic search.".to_string(),
                    parameters: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "query": {
                                "type": "string",
                                "description": "The search query to find relevant web pages"
                            },
                            "num_results": {
                                "type": "integer",
                                "description": "Number of results to return (default: 5, max: 10)"
                            }
                        },
                        "required": ["query"]
                    }),
                },
            },
        ]
    }
}
```

## ExaService Trait

```rust
// crates/services/src/exa_service.rs
#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait ExaService: std::fmt::Debug + Send + Sync {
    async fn search(
        &self,
        api_key: &str,
        query: &str,
        num_results: Option<u32>,
    ) -> Result<ExaSearchResponse, ExaError>;
}

pub struct DefaultExaService {
    client: reqwest::Client,  // with 30s timeout
}
```

## Error Types

```rust
// crates/services/src/tool_error.rs
#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum ToolError {
    #[error("tool_not_found: {0}")]
    #[error_meta(error_type = ErrorType::NotFound)]
    ToolNotFound(String),

    #[error("tool_not_configured")]
    #[error_meta(error_type = ErrorType::BadRequest)]
    ToolNotConfigured,

    #[error("tool_disabled")]
    #[error_meta(error_type = ErrorType::BadRequest)]
    ToolDisabled,

    #[error("tool_execution_failed: {0}")]
    #[error_meta(error_type = ErrorType::InternalServer)]
    ExecutionFailed(String),

    #[error(transparent)]
    DbError(#[from] DbError),

    #[error(transparent)]
    ExaError(#[from] ExaError),
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum ExaError {
    #[error("exa_request_failed: {0}")]
    #[error_meta(error_type = ErrorType::BadGateway)]
    RequestFailed(String),

    #[error("exa_rate_limited")]
    #[error_meta(error_type = ErrorType::TooManyRequests)]
    RateLimited,

    #[error("exa_invalid_api_key")]
    #[error_meta(error_type = ErrorType::Unauthorized)]
    InvalidApiKey,

    #[error("exa_timeout")]
    #[error_meta(error_type = ErrorType::GatewayTimeout)]
    Timeout,
}
```

## Integration with AppService

```rust
// Add to AppService trait
pub trait AppService: ... {
    // ... existing methods ...
    fn tool_service(&self) -> Arc<dyn ToolService>;
}
```
