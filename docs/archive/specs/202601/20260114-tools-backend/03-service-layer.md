# Service Layer - Toolsets Feature

> Layer: `services` crate | Status: ✅ Complete

## ToolsetService Trait

```rust
// crates/services/src/toolset_service.rs
#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait ToolsetService: std::fmt::Debug + Send + Sync {
    /// List toolset definitions for user (only configured toolsets)
    async fn list_toolsets_for_user(&self, user_id: &str) -> Result<Vec<ToolsetDefinition>, ToolsetError>;

    /// Get all available toolset definitions (for UI listing)
    fn list_all_toolset_definitions(&self) -> Vec<ToolsetDefinition>;

    /// Get user's toolset config by ID
    async fn get_user_toolset_config(&self, user_id: &str, toolset_id: &str) 
        -> Result<Option<UserToolsetConfig>, ToolsetError>;

    /// Update user's toolset config (enable/disable, API key)
    async fn update_user_toolset_config(
        &self,
        user_id: &str,
        toolset_id: &str,
        enabled: bool,
        api_key: Option<String>,
    ) -> Result<UserToolsetConfig, ToolsetError>;

    /// Delete user's toolset config (clears API key)
    async fn delete_user_toolset_config(&self, user_id: &str, toolset_id: &str) 
        -> Result<(), ToolsetError>;

    /// Execute a tool within a toolset
    async fn execute_toolset_tool(
        &self,
        user_id: &str,
        toolset_id: &str,
        request: ToolsetExecutionRequest,
    ) -> Result<ToolsetExecutionResponse, ToolsetError>;

    /// Check if toolset is available for user (enabled + has API key)
    async fn is_toolset_available_for_user(&self, user_id: &str, toolset_id: &str) 
        -> Result<bool, ToolsetError>;

    // App-level toolset configuration (admin-controlled)
    async fn get_app_toolset_config(&self, toolset_id: &str) 
        -> Result<Option<AppToolsetConfig>, ToolsetError>;
    async fn is_toolset_enabled_for_app(&self, toolset_id: &str) 
        -> Result<bool, ToolsetError>;
    async fn set_app_toolset_enabled(
        &self,
        admin_token: &str,
        toolset_id: &str,
        enabled: bool,
        updated_by: &str,
    ) -> Result<AppToolsetConfig, ToolsetError>;
    async fn list_app_toolset_configs(&self) -> Result<Vec<AppToolsetConfig>, ToolsetError>;

    // App-client toolset configuration (cached from auth server)
    async fn is_app_client_registered_for_toolset(
        &self,
        app_client_id: &str,
        toolset_id: &str,
    ) -> Result<bool, ToolsetError>;
}
```

## DefaultToolsetService Implementation

```rust
#[derive(Debug)]
pub struct DefaultToolsetService {
    db_service: Arc<dyn DbService>,
    exa_service: Arc<dyn ExaService>,
    time_service: Arc<dyn TimeService>,
}

impl DefaultToolsetService {
    /// Static registry of built-in toolsets
    fn builtin_toolset_definitions() -> Vec<ToolsetDefinition> {
        vec![
            ToolsetDefinition {
                toolset_id: "builtin-exa-web-search".to_string(),
                name: "Exa Web Search".to_string(),
                description: "Search the web using Exa AI semantic search".to_string(),
                tools: vec![
                    // search tool
                    ToolDefinition {
                        tool_type: "function".to_string(),
                        function: FunctionDefinition {
                            name: "toolset__builtin-exa-web-search__search".to_string(),
                            description: "Search the web for current information".to_string(),
                            parameters: json!({
                                "type": "object",
                                "properties": {
                                    "query": { "type": "string", "description": "Search query" },
                                    "num_results": { "type": "integer", "default": 5, "maximum": 10 }
                                },
                                "required": ["query"]
                            }),
                        },
                    },
                    // find_similar tool
                    ToolDefinition {
                        tool_type: "function".to_string(),
                        function: FunctionDefinition {
                            name: "toolset__builtin-exa-web-search__find_similar".to_string(),
                            description: "Find web pages similar to a given URL".to_string(),
                            parameters: json!({
                                "type": "object",
                                "properties": {
                                    "url": { "type": "string" },
                                    "num_results": { "type": "integer", "default": 5 },
                                    "exclude_source_domain": { "type": "boolean", "default": true }
                                },
                                "required": ["url"]
                            }),
                        },
                    },
                    // get_contents tool
                    ToolDefinition {
                        tool_type: "function".to_string(),
                        function: FunctionDefinition {
                            name: "toolset__builtin-exa-web-search__get_contents".to_string(),
                            description: "Get full contents of web pages by URLs".to_string(),
                            parameters: json!({
                                "type": "object",
                                "properties": {
                                    "urls": { "type": "array", "items": { "type": "string" }, "maxItems": 10 },
                                    "max_characters": { "type": "integer", "default": 3000 }
                                },
                                "required": ["urls"]
                            }),
                        },
                    },
                    // answer tool
                    ToolDefinition {
                        tool_type: "function".to_string(),
                        function: FunctionDefinition {
                            name: "toolset__builtin-exa-web-search__answer".to_string(),
                            description: "Get AI-generated answer based on web search".to_string(),
                            parameters: json!({
                                "type": "object",
                                "properties": {
                                    "query": { "type": "string" },
                                    "num_results": { "type": "integer", "default": 5 }
                                },
                                "required": ["query"]
                            }),
                        },
                    },
                ],
            },
        ]
    }
}
```

## Tool Execution Logic

The `execute_toolset_tool` method:
1. Validates the toolset is available for user (app-enabled + user-enabled + has API key)
2. Extracts the tool name from the request (e.g., `toolset__builtin-exa-web-search__search`)
3. Parses toolset_id and tool_name from the fully qualified name
4. Decrypts the API key from user config
5. Dispatches to appropriate Exa service method based on tool_name
6. Returns formatted response for LLM

```rust
async fn execute_toolset_tool(
    &self,
    user_id: &str,
    toolset_id: &str,
    request: ToolsetExecutionRequest,
) -> Result<ToolsetExecutionResponse, ToolsetError> {
    // Validate toolset is available
    if !self.is_toolset_available_for_user(user_id, toolset_id).await? {
        return Err(ToolsetError::ToolsetNotConfigured);
    }

    // Get decrypted API key
    let api_key = self.get_decrypted_api_key(user_id, toolset_id).await?;

    // Parse tool_name to extract the actual tool (e.g., "search" from "toolset__builtin-exa-web-search__search")
    let tool_name = Self::extract_tool_name(&request.tool_name)?;

    // Dispatch to appropriate Exa method
    match (toolset_id, tool_name.as_str()) {
        ("builtin-exa-web-search", "search") => self.execute_exa_search(&api_key, request).await,
        ("builtin-exa-web-search", "find_similar") => self.execute_exa_find_similar(&api_key, request).await,
        ("builtin-exa-web-search", "get_contents") => self.execute_exa_get_contents(&api_key, request).await,
        ("builtin-exa-web-search", "answer") => self.execute_exa_answer(&api_key, request).await,
        _ => Err(ToolsetError::ToolNotFound(request.tool_name)),
    }
}
```

## ExaService Trait

```rust
// crates/services/src/exa_service.rs
#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait ExaService: std::fmt::Debug + Send + Sync {
    /// Search the web using Exa AI semantic search
    async fn search(
        &self,
        api_key: &str,
        query: &str,
        num_results: Option<u32>,
    ) -> Result<ExaSearchResponse, ExaError>;

    /// Find pages similar to a given URL
    async fn find_similar(
        &self,
        api_key: &str,
        url: &str,
        num_results: Option<u32>,
        exclude_source_domain: Option<bool>,
    ) -> Result<ExaSearchResponse, ExaError>;

    /// Get full contents of web pages
    async fn get_contents(
        &self,
        api_key: &str,
        urls: Vec<String>,
        max_characters: Option<u32>,
    ) -> Result<ExaContentsResponse, ExaError>;

    /// Get AI-generated answer from web search
    async fn answer(
        &self,
        api_key: &str,
        query: &str,
        num_results: Option<u32>,
    ) -> Result<ExaAnswerResponse, ExaError>;
}

pub struct DefaultExaService {
    client: reqwest::Client,  // with 30s timeout
}
```

## Error Types

```rust
// crates/services/src/toolset_service.rs
#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum ToolsetError {
    #[error("toolset_not_found: {0}")]
    #[error_meta(error_type = ErrorType::NotFound)]
    ToolsetNotFound(String),

    #[error("tool_not_found: {0}")]
    #[error_meta(error_type = ErrorType::NotFound)]
    ToolNotFound(String),

    #[error("toolset_not_configured")]
    #[error_meta(error_type = ErrorType::BadRequest)]
    ToolsetNotConfigured,

    #[error("toolset_disabled")]
    #[error_meta(error_type = ErrorType::BadRequest)]
    ToolsetDisabled,

    #[error("toolset_app_disabled")]
    #[error_meta(error_type = ErrorType::BadRequest)]
    ToolsetAppDisabled,

    #[error("toolset_execution_failed: {0}")]
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
    fn toolset_service(&self) -> Arc<dyn ToolsetService>;
}
```

## Test Coverage

Tests cover:
- Toolset execution flow with MockDbService and MockExaService
- Config validation (enabled + has API key)
- Tool name parsing from fully qualified format
- Error propagation from DB and Exa services
- Toolset availability checking
- All 4 Exa tool methods
