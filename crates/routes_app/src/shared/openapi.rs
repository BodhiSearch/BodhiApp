use crate::{
  AccessRequestActionResponse, AccessRequestReviewResponse, AccessRequestStatusResponse,
  ApiFormatsResponse, ApiKey, ApiKeyUpdateAction, ApiModelResponse, ApproveAccessRequestBody,
  AuthCallbackRequest, CreateAccessRequestBody, CreateAccessRequestResponse, CreateApiModelRequest,
  FetchModelsRequest, FetchModelsResponse, LocalModelResponse, PaginatedApiModelResponse,
  PaginationSortParams, PingResponse, TestCreds, TestPromptRequest, TestPromptResponse,
  UpdateApiModelRequest, UpdateApiTokenRequest,
};
use crate::{
  ApiTokenResponse, AppInfo, ApproveUserAccessRequest, ChangeRoleRequest, CopyAliasRequest,
  CreateAliasRequest, CreateApiTokenRequest, ListUsersParams, NewDownloadRequest,
  PaginatedAliasResponse, PaginatedApiTokenResponse, PaginatedDownloadResponse,
  PaginatedLocalModelResponse, PaginatedUserAccessResponse, PaginatedUserAliasResponse,
  QueueStatusResponse, RedirectResponse, RefreshRequest, RefreshResponse, RefreshSource,
  SetupRequest, SetupResponse, UpdateAliasRequest, UpdateSettingRequest, UserAccessStatusResponse,
  UserAliasResponse, UserResponse, __path_app_info_handler, __path_approve_access_request_handler,
  __path_approve_request_handler, __path_auth_callback_handler, __path_auth_initiate_handler,
  __path_change_user_role_handler, __path_copy_alias_handler, __path_create_access_request_handler,
  __path_create_alias_handler, __path_create_api_model_handler, __path_create_pull_request_handler,
  __path_create_token_handler, __path_delete_alias_handler, __path_delete_api_model_handler,
  __path_delete_setting_handler, __path_deny_access_request_handler, __path_fetch_models_handler,
  __path_get_access_request_review_handler, __path_get_access_request_status_handler,
  __path_get_api_formats_handler, __path_get_api_model_handler, __path_get_download_status_handler,
  __path_get_user_alias_handler, __path_health_handler, __path_list_aliases_handler,
  __path_list_all_requests_handler, __path_list_api_models_handler, __path_list_downloads_handler,
  __path_list_local_modelfiles_handler, __path_list_pending_requests_handler,
  __path_list_settings_handler, __path_list_tokens_handler, __path_list_users_handler,
  __path_logout_handler, __path_ping_handler, __path_queue_status_handler,
  __path_refresh_metadata_handler, __path_reject_request_handler, __path_remove_user_handler,
  __path_request_status_handler, __path_setup_handler, __path_sync_models_handler,
  __path_test_api_model_handler, __path_update_alias_handler, __path_update_api_model_handler,
  __path_update_setting_handler, __path_update_token_handler, __path_user_info_handler,
  __path_user_request_access_handler,
};
// Toolsets DTOs and handlers
use crate::routes_oai::{
  __path_chat_completions_handler, __path_embeddings_handler, __path_oai_model_handler,
  __path_oai_models_handler,
};
use crate::routes_ollama::{
  __path_ollama_model_chat_handler, __path_ollama_model_show_handler, __path_ollama_models_handler,
};
use crate::{
  ApiKeyUpdateDto, CreateToolsetRequest, ExecuteToolsetRequest, ListToolsetTypesResponse,
  ListToolsetsResponse, ToolsetResponse, UpdateToolsetRequest, __path_create_toolset_handler,
  __path_delete_toolset_handler, __path_disable_type_handler, __path_enable_type_handler,
  __path_execute_toolset_handler, __path_get_toolset_handler, __path_list_toolset_types_handler,
  __path_list_toolsets_handler, __path_update_toolset_handler,
};
// MCP DTOs and handlers
use crate::{
  CreateMcpRequest, CreateMcpServerRequest, ListMcpServersResponse, ListMcpsResponse,
  McpExecuteRequest, McpExecuteResponse, McpResponse, McpServerResponse, McpToolsResponse,
  UpdateMcpRequest, UpdateMcpServerRequest, __path_create_mcp_handler,
  __path_create_mcp_server_handler, __path_delete_mcp_handler, __path_execute_mcp_tool_handler,
  __path_get_mcp_handler, __path_get_mcp_server_handler, __path_list_mcp_servers_handler,
  __path_list_mcp_tools_handler, __path_list_mcps_handler, __path_refresh_mcp_tools_handler,
  __path_update_mcp_handler, __path_update_mcp_server_handler,
};
use async_openai::types::{
  chat::{
    ChatChoice, ChatChoiceStream, ChatCompletionRequestMessage, ChatCompletionResponseMessage,
    CompletionUsage, CreateChatCompletionRequest, CreateChatCompletionResponse,
    CreateChatCompletionStreamResponse,
  },
  embeddings::{
    CreateEmbeddingRequest, CreateEmbeddingResponse, Embedding, EmbeddingInput, EmbeddingUsage,
  },
  models::{ListModelResponse, Model},
};
use objs::{
  Alias, ApiFormat, AppRole, McpServer, McpServerInfo, McpTool, OAIRequestParams, OpenAIApiError,
  ResourceRole, SettingInfo, SettingMetadata, SettingSource, TokenScope, ToolDefinition, Toolset,
  ToolsetDefinition, ToolsetExecutionResponse, UserInfo, UserScope, API_TAG_API_KEYS,
  API_TAG_API_MODELS, API_TAG_AUTH, API_TAG_MCPS, API_TAG_MODELS, API_TAG_OLLAMA, API_TAG_OPENAI,
  API_TAG_SETTINGS, API_TAG_SETUP, API_TAG_SYSTEM, API_TAG_TOOLSETS,
};
use services::db::DownloadStatus;
use services::{
  db::{ApiToken, DownloadRequest, TokenStatus},
  AppStatus, SettingService, UserListResponse,
};
use std::sync::Arc;
use utoipa::{
  openapi::security::{
    AuthorizationCode, Flow, HttpAuthScheme, HttpBuilder, OAuth2, Scopes, SecurityScheme,
  },
  Modify, OpenApi,
};

macro_rules! make_ui_endpoint {
  ($name:ident, $path:expr) => {
    pub const $name: &str = concat!("/bodhi/v1/", $path);
  };
}

pub const ENDPOINT_PING: &str = "/ping";
pub const ENDPOINT_HEALTH: &str = "/health";

make_ui_endpoint!(ENDPOINT_LOGOUT, "logout");
make_ui_endpoint!(ENDPOINT_APP_INFO, "info");
make_ui_endpoint!(ENDPOINT_APP_SETUP, "setup");
make_ui_endpoint!(ENDPOINT_USER_INFO, "user");
make_ui_endpoint!(ENDPOINT_USER_REQUEST_ACCESS, "user/request-access");
make_ui_endpoint!(ENDPOINT_USER_REQUEST_STATUS, "user/request-status");
make_ui_endpoint!(ENDPOINT_ACCESS_REQUESTS_PENDING, "access-requests/pending");
make_ui_endpoint!(ENDPOINT_ACCESS_REQUESTS_ALL, "access-requests");
make_ui_endpoint!(ENDPOINT_USERS, "users");
make_ui_endpoint!(ENDPOINT_AUTH_INITIATE, "auth/initiate");
make_ui_endpoint!(ENDPOINT_AUTH_CALLBACK, "auth/callback");

make_ui_endpoint!(ENDPOINT_MODEL_FILES, "modelfiles");
make_ui_endpoint!(ENDPOINT_MODEL_PULL, "modelfiles/pull");
make_ui_endpoint!(ENDPOINT_MODELS, "models");
make_ui_endpoint!(ENDPOINT_MODELS_REFRESH, "models/refresh");
make_ui_endpoint!(ENDPOINT_QUEUE, "queue");
make_ui_endpoint!(ENDPOINT_CHAT_TEMPLATES, "chat_templates");
make_ui_endpoint!(ENDPOINT_TOKENS, "tokens");
make_ui_endpoint!(ENDPOINT_API_MODELS, "api-models");
make_ui_endpoint!(ENDPOINT_API_MODELS_TEST, "api-models/test");
make_ui_endpoint!(ENDPOINT_API_MODELS_FETCH_MODELS, "api-models/fetch-models");
make_ui_endpoint!(ENDPOINT_API_MODELS_API_FORMATS, "api-models/api-formats");
make_ui_endpoint!(ENDPOINT_SETTINGS, "settings");
make_ui_endpoint!(ENDPOINT_TOOLSETS, "toolsets");
make_ui_endpoint!(ENDPOINT_TOOLSET_TYPES, "toolset_types");
make_ui_endpoint!(ENDPOINT_MCPS, "mcps");
make_ui_endpoint!(ENDPOINT_MCP_SERVERS, "mcp_servers");

// dev-only debugging info endpoint
pub const ENDPOINT_DEV_SECRETS: &str = "/dev/secrets";
pub const ENDPOINT_DEV_ENVS: &str = "/dev/envs";
pub const ENDPOINT_DEV_DB_RESET: &str = "/dev/db-reset";

/// API documentation configuration
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Bodhi App APIs",
        version = env!("CARGO_PKG_VERSION"),
        contact(
            name = "Bodhi API Support",
            url = "https://github.com/BodhiSearch/BodhiApp/issues",
            email = "support@getbodhi.app"
        ),
        description = r#"API documentation for Bodhi App.

## Authentication Methods

Bodhi App supports three sophisticated authentication methods:

### 1. API Token Authentication (Recommended for API access)
- **Method**: Bearer token in Authorization header
- **Format**: `Authorization: Bearer bodhiapp_<random_string>`
- **Obtain**: Create via web interface at Menu > Settings > API Tokens
- **Scopes**: Token-based scopes with hierarchical permissions
  - `scope_token_user`: Basic API access (read operations, OpenAI/Ollama APIs)
  - `scope_token_power_user`: Advanced operations (model creation, downloads)
  - `scope_token_manager`: User management operations
  - `scope_token_admin`: Full administrative access
- **Usage**: Best for programmatic access, CI/CD, and external integrations

### 2. Session Authentication (Browser-based)
- **Method**: HTTP cookies (automatic for browsers)
- **Login**: Initiate via `POST /bodhi/v1/auth/initiate`
- **Callback**: Complete via `POST /bodhi/v1/auth/callback`
- **Roles**: Resource-based roles with hierarchical permissions
  - `resource_user`: Basic authenticated access
  - `resource_power_user`: Advanced operations
  - `resource_manager`: User management (session-only)
  - `resource_admin`: Full administration (session-only)
- **Usage**: Best for web browsers and interactive applications
- **Session-Only Operations**: Some sensitive operations (token management, settings, user management) require session authentication only for security

### 3. OAuth 2.1 Token Exchange (External integrations)
- **Method**: Bearer token with exchanged OAuth credentials
- **Format**: `Authorization: Bearer <oauth_exchanged_token>`
- **Scopes**: User-based scopes via OAuth 2.1 token exchange
  - `scope_user_user`: Basic user access via OAuth exchange
  - `scope_user_power_user`: Advanced user operations via OAuth exchange  
  - `scope_user_manager`: Manager operations via OAuth exchange
  - `scope_user_admin`: Admin operations via OAuth exchange
- **Usage**: For external OAuth 2.1 providers integrating with Bodhi

## Authorization Hierarchy

**Permission Hierarchy** (higher levels include all lower permissions):
```
Admin > Manager > PowerUser > User
```

**Scope/Role Mapping**:
| Permission Level | API Token Scope | OAuth User Scope | Session Role |
|------------------|----------------|------------------|-------------|
| User | `scope_token_user` | `scope_user_user` | `resource_user` |
| PowerUser | `scope_token_power_user` | `scope_user_power_user` | `resource_power_user` |
| Manager | `scope_token_manager` | `scope_user_manager` | `resource_manager` |
| Admin | `scope_token_admin` | `scope_user_admin` | `resource_admin` |

## Endpoint Access Patterns

- **Public Endpoints**: No authentication required (ping, health, app_info, setup)
- **Optional Auth Endpoints**: Work with or without authentication, providing different data (user_info)
- **Multi-Auth Endpoints**: Accept any of the three authentication methods (most API endpoints)
- **Session-Only Endpoints**: Require browser session authentication only:
  - Token management (`/bodhi/v1/tokens/*`) - PowerUser+ session required
  - Settings management (`/bodhi/v1/settings/*`) - Admin session required  
  - User management (`/bodhi/v1/users/*`) - Manager+ session required

## Security Examples

### API Token Usage:
```bash
curl -H "Authorization: Bearer bodhiapp_1234567890abcdef" \
     https://api.example.com/v1/models
```

### Session Authentication:
```bash
# Login first
curl -X POST https://api.example.com/bodhi/v1/auth/initiate \
     -H "Content-Type: application/json" \
     -d '{"provider": "github"}'

# Then use session cookie automatically
curl https://api.example.com/v1/models \
     --cookie-jar cookies.txt --cookie cookies.txt
```

### OAuth 2.1 Token Exchange:
```bash
curl -H "Authorization: Bearer <oauth_exchanged_token>" \
     https://api.example.com/v1/models
```

## Security Best Practices

1. **Use HTTPS**: Always use HTTPS in production
2. **Scope Principle**: Request minimal required scopes/roles
3. **Token Rotation**: Regularly rotate API tokens
4. **Session Security**: Session-only operations prevent token-based access to sensitive functions
5. **Hierarchical Permissions**: Higher roles automatically include lower role permissions
"#
    ),
    external_docs(
        url = "https://getbodhi.app/docs/api",
        description = "Find more info here"
    ),
    servers(
        (url = "http://localhost:1135", description = "Local running instance"),
    ),
    tags(
        (name = API_TAG_SYSTEM, description = "System information and operations"),
        (name = API_TAG_SETUP, description = "Application setup and initialization"),
        (name = API_TAG_AUTH, description = "Authentication and session management"),
        (name = API_TAG_API_KEYS, description = "API keys management"),
        (name = API_TAG_API_MODELS, description = "Remote AI API model configuration"),
        (name = API_TAG_MODELS, description = "Model files and aliases"),
        (name = API_TAG_SETTINGS, description = "Application settings management"),
        (name = API_TAG_TOOLSETS, description = "AI toolsets configuration and execution"),
        (name = API_TAG_MCPS, description = "MCP server management and tool execution"),
        (name = API_TAG_OPENAI, description = "OpenAI-compatible API endpoints"),
        (name = API_TAG_OLLAMA, description = "Ollama-compatible API endpoints"),
    ),
    components(
        schemas(
            // common
            OpenAIApiError,
            AppStatus,
            RedirectResponse,
            // system
            AppInfo,
            PingResponse,
            // setup
            SetupRequest,
            SetupResponse,
            // auth
            AuthCallbackRequest,
            UserResponse,
            AppRole,
            ResourceRole,
            TokenScope,
            UserScope,
            // access requests
            UserAccessStatusResponse,
            ApproveUserAccessRequest,
            PaginatedUserAccessResponse,
            // app access requests
            CreateAccessRequestBody,
            CreateAccessRequestResponse,
            AccessRequestStatusResponse,
            AccessRequestReviewResponse,
            ApproveAccessRequestBody,
            AccessRequestActionResponse,
            // user management
            ListUsersParams,
            UserListResponse,
            UserInfo,
            ChangeRoleRequest,
            // api keys/token
            CreateApiTokenRequest,
            ApiTokenResponse,
            UpdateApiTokenRequest,
            TokenStatus,
            PaginatedApiTokenResponse,
            ApiToken,
            // api models
            PaginatedApiModelResponse,
            ApiModelResponse,
            CreateApiModelRequest,
            UpdateApiModelRequest,
            ApiKey,
            ApiKeyUpdateAction,
            TestCreds,
            TestPromptRequest,
            TestPromptResponse,
            FetchModelsRequest,
            FetchModelsResponse,
            ApiFormatsResponse,
            ApiFormat,
            // models
            CreateAliasRequest,
            CopyAliasRequest,
            OAIRequestParams,
            PaginatedUserAliasResponse,
            UserAliasResponse,
            UpdateAliasRequest,
            PaginationSortParams,
            PaginatedAliasResponse,
            Alias,
            PaginatedLocalModelResponse,
            LocalModelResponse,
            PaginatedDownloadResponse,
            DownloadRequest,
            DownloadStatus,
            NewDownloadRequest,
            RefreshRequest,
            RefreshSource,
            RefreshResponse,
            QueueStatusResponse,
            // settings
            SettingInfo,
            SettingMetadata,
            SettingSource,
            UpdateSettingRequest,
            // openai
            ListModelResponse,
            Model,
            CreateChatCompletionRequest,
            CreateChatCompletionResponse,
            CreateChatCompletionStreamResponse,
            ChatCompletionRequestMessage,
            ChatCompletionResponseMessage,
            ChatChoice,
            ChatChoiceStream,
            CompletionUsage,
            CreateEmbeddingRequest,
            CreateEmbeddingResponse,
            Embedding,
            EmbeddingInput,
            EmbeddingUsage,
            // toolsets
            CreateToolsetRequest,
            UpdateToolsetRequest,
            ApiKeyUpdateDto,
            ToolsetResponse,
            ListToolsetsResponse,
            ListToolsetTypesResponse,
            ToolsetDefinition,
            ExecuteToolsetRequest,
            ToolDefinition,
            Toolset,
            ToolsetExecutionResponse,
            // mcps
            CreateMcpRequest,
            UpdateMcpRequest,
            CreateMcpServerRequest,
            UpdateMcpServerRequest,
            McpResponse,
            McpServerResponse,
            ListMcpsResponse,
            ListMcpServersResponse,
            McpServer,
            McpServerInfo,
            McpTool,
            McpToolsResponse,
            McpExecuteRequest,
            McpExecuteResponse,
        ),
        responses( ),
    ),
    paths(
        // System endpoints
        ping_handler,
        health_handler,
        app_info_handler,

        // Setup endpoints
        setup_handler,

        // Authentication endpoints
        auth_initiate_handler,
        auth_callback_handler,
        logout_handler,
        user_info_handler,

        // API Keys endpoints
        create_token_handler,
        update_token_handler,
        list_tokens_handler,

        // API Models endpoints
        list_api_models_handler,
        get_api_model_handler,
        create_api_model_handler,
        update_api_model_handler,
        delete_api_model_handler,
        sync_models_handler,
        test_api_model_handler,
        fetch_models_handler,
        get_api_formats_handler,

        // Models endpoints
        create_alias_handler,
        update_alias_handler,
        delete_alias_handler,
        copy_alias_handler,
        list_aliases_handler,
        list_local_modelfiles_handler,
        get_user_alias_handler,
        list_downloads_handler,
        create_pull_request_handler,
        get_download_status_handler,
        refresh_metadata_handler,
        queue_status_handler,

        // Settings endpoints
        list_settings_handler,
        update_setting_handler,
        delete_setting_handler,

        // OpenAI endpoints
        oai_models_handler,
        oai_model_handler,
        chat_completions_handler,
        embeddings_handler,

        // Ollama endpoints
        ollama_models_handler,
        ollama_model_show_handler,
        ollama_model_chat_handler,

        // User request endpoints
        user_request_access_handler,
        request_status_handler,
        list_pending_requests_handler,
        list_all_requests_handler,
        approve_request_handler,
        reject_request_handler,

        // App access request endpoints
        create_access_request_handler,
        get_access_request_status_handler,
        get_access_request_review_handler,
        approve_access_request_handler,
        deny_access_request_handler,

        // User management endpoints
        list_users_handler,
        change_user_role_handler,
        remove_user_handler,

        // Toolsets endpoints
        list_toolsets_handler,
        create_toolset_handler,
        get_toolset_handler,
        update_toolset_handler,
        delete_toolset_handler,
        execute_toolset_handler,
        list_toolset_types_handler,
        enable_type_handler,
        disable_type_handler,

        // MCP endpoints
        list_mcps_handler,
        create_mcp_handler,
        get_mcp_handler,
        update_mcp_handler,
        delete_mcp_handler,
        list_mcp_tools_handler,
        refresh_mcp_tools_handler,
        execute_mcp_tool_handler,
        // MCP server admin endpoints
        list_mcp_servers_handler,
        get_mcp_server_handler,
        create_mcp_server_handler,
        update_mcp_server_handler
    )
)]
pub struct BodhiOpenAPIDoc;

/// Modifies OpenAPI documentation with environment-specific settings
#[derive(Debug, derive_new::new)]
pub struct OpenAPIEnvModifier {
  setting_service: Arc<dyn SettingService>,
}

impl Modify for OpenAPIEnvModifier {
  fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
    // Add environment-specific server
    let server_url = self.setting_service.public_server_url();
    let desc = if self.setting_service.is_production() {
      ""
    } else {
      " - Development"
    };
    let server = utoipa::openapi::ServerBuilder::default()
      .url(server_url)
      .description(Some(format!("Bodhi App {}", desc)))
      .build();
    openapi.servers = Some(vec![server]);

    if let Some(components) = &mut openapi.components {
      // 1. API Token Authentication (Database-stored tokens with TokenScope)
      components.security_schemes.insert(
        "bearer_api_token".to_string(),
        SecurityScheme::Http(
          HttpBuilder::new()
            .scheme(HttpAuthScheme::Bearer)
            .bearer_format("bodhiapp_<token>")
            .description(Some("API token authentication. Create tokens via web interface at Menu > Settings > API Tokens. Format: 'bodhiapp_<random>'. Use as: Authorization: Bearer <token>\n\nScopes:\n- scope_token_user: Basic API access - read operations\n- scope_token_power_user: Advanced operations - create/update models, downloads\n- scope_token_manager: User management operations\n- scope_token_admin: Full administrative access"))
            .build()
        ),
      );

      // 2. OAuth 2.1 Token Exchange (External tokens with UserScope)
      components.security_schemes.insert(
        "bearer_oauth_token".to_string(),
        SecurityScheme::Http(
          HttpBuilder::new()
            .scheme(HttpAuthScheme::Bearer)
            .bearer_format("JWT")
            .description(Some("OAuth 2.1 token exchange authentication. External OAuth providers can exchange tokens with UserScope claims for access to Bodhi resources. Use as: Authorization: Bearer <oauth_exchanged_token>\n\nScopes:\n- scope_user_user: Basic user access via OAuth 2.1 token exchange\n- scope_user_power_user: Advanced user operations via OAuth 2.1 token exchange\n- scope_user_manager: Manager operations via OAuth 2.1 token exchange\n- scope_user_admin: Admin operations via OAuth 2.1 token exchange"))
            .build()
        ),
      );

      // 3. Session Cookie Authentication (Browser sessions with ResourceRole)
      components.security_schemes.insert(
        "session_auth".to_string(),
        SecurityScheme::OAuth2(
          OAuth2::with_description(
            [Flow::AuthorizationCode(
              AuthorizationCode::new(
                "/bodhi/v1/auth/initiate".to_string(),
                "/bodhi/v1/auth/callback".to_string(),
                Scopes::from_iter([
                  ("resource_user".to_string(), "Basic authenticated user access via browser session".to_string()),
                  ("resource_power_user".to_string(), "Power user operations via browser session".to_string()),
                  ("resource_manager".to_string(), "Manager operations via browser session (session-only)".to_string()),
                  ("resource_admin".to_string(), "Admin operations via browser session (session-only)".to_string()),
                ]),
              ),
            )],
            "Browser session authentication. Login via /bodhi/v1/auth/initiate. Some operations (token management, settings, user management) require session authentication only.",
          )
        ),
      );
    }
  }
}

/// Modifies OpenAPI documentation to add security schemes
#[derive(Debug)]
pub struct SecurityModifier;

impl Modify for SecurityModifier {
  fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
    if openapi.components.is_none() {
      openapi.components = Some(Default::default());
    }

    if let Some(components) = &mut openapi.components {
      // 1. API Token Authentication (Database-stored tokens with TokenScope)
      components.security_schemes.insert(
        "bearer_api_token".to_string(),
        SecurityScheme::Http(
          HttpBuilder::new()
            .scheme(HttpAuthScheme::Bearer)
            .bearer_format("bodhiapp_<token>")
            .description(Some("API token authentication. Create tokens via web interface at Menu > Settings > API Tokens. Format: 'bodhiapp_<random>'. Use as: Authorization: Bearer <token>\n\nScopes:\n- scope_token_user: Basic API access - read operations\n- scope_token_power_user: Advanced operations - create/update models, downloads\n- scope_token_manager: User management operations\n- scope_token_admin: Full administrative access"))
            .build()
        ),
      );

      // 2. OAuth 2.1 Token Exchange (External tokens with UserScope)
      components.security_schemes.insert(
        "bearer_oauth_token".to_string(),
        SecurityScheme::Http(
          HttpBuilder::new()
            .scheme(HttpAuthScheme::Bearer)
            .bearer_format("JWT")
            .description(Some("OAuth 2.1 token exchange authentication. External OAuth providers can exchange tokens with UserScope claims for access to Bodhi resources. Use as: Authorization: Bearer <oauth_exchanged_token>\n\nScopes:\n- scope_user_user: Basic user access via OAuth 2.1 token exchange\n- scope_user_power_user: Advanced user operations via OAuth 2.1 token exchange\n- scope_user_manager: Manager operations via OAuth 2.1 token exchange\n- scope_user_admin: Admin operations via OAuth 2.1 token exchange"))
            .build()
        ),
      );

      // 3. Session Cookie Authentication (Browser sessions with ResourceRole)
      components.security_schemes.insert(
        "session_auth".to_string(),
        SecurityScheme::OAuth2(
          OAuth2::with_description(
            [Flow::AuthorizationCode(
              AuthorizationCode::new(
                "/bodhi/v1/auth/initiate".to_string(),
                "/bodhi/v1/auth/callback".to_string(),
                Scopes::from_iter([
                  ("resource_user".to_string(), "Basic authenticated user access via browser session".to_string()),
                  ("resource_power_user".to_string(), "Power user operations via browser session".to_string()),
                  ("resource_manager".to_string(), "Manager operations via browser session (session-only)".to_string()),
                  ("resource_admin".to_string(), "Admin operations via browser session (session-only)".to_string()),
                ]),
              ),
            )],
            "Browser session authentication. Login via /bodhi/v1/auth/initiate. Some operations (token management, settings, user management) require session authentication only.",
          )
        ),
      );
    }
  }
}

/// Modifies OpenAPI documentation to add common error responses to all endpoints
#[derive(Debug)]
pub struct GlobalErrorResponses;

impl Modify for GlobalErrorResponses {
  fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
    use utoipa::openapi::{ContentBuilder, Ref, RefOr, ResponseBuilder};

    // Define public endpoints that don't require authentication
    let public_endpoints = [
      ENDPOINT_PING,
      ENDPOINT_HEALTH,
      ENDPOINT_APP_INFO,
      ENDPOINT_APP_SETUP,
    ];

    for (path, path_item) in openapi.paths.paths.iter_mut() {
      // Check if this is a public endpoint
      let is_public = public_endpoints.iter().any(|&public| path == public);

      // Add errors to all operations (GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS)
      let operations = [
        &mut path_item.get,
        &mut path_item.post,
        &mut path_item.put,
        &mut path_item.delete,
        &mut path_item.patch,
        &mut path_item.head,
        &mut path_item.options,
      ];

      for operation in operations.into_iter().flatten() {
        // Add 400 Bad Request to all endpoints
        operation.responses.responses.insert(
          "400".to_string(),
          RefOr::T(
            ResponseBuilder::new()
              .description("Invalid request parameters")
              .content(
                "application/json",
                ContentBuilder::new()
                  .schema(Some(Ref::from_schema_name("OpenAIApiError")))
                  .build(),
              )
              .build(),
          ),
        );

        // Add 401 and 403 only to non-public endpoints
        if !is_public {
          operation.responses.responses.insert(
            "401".to_string(),
            RefOr::T(
              ResponseBuilder::new()
                .description("Not authenticated")
                .content(
                  "application/json",
                  ContentBuilder::new()
                    .schema(Some(Ref::from_schema_name("OpenAIApiError")))
                    .build(),
                )
                .build(),
            ),
          );

          operation.responses.responses.insert(
            "403".to_string(),
            RefOr::T(
              ResponseBuilder::new()
                .description("Insufficient permissions")
                .content(
                  "application/json",
                  ContentBuilder::new()
                    .schema(Some(Ref::from_schema_name("OpenAIApiError")))
                    .build(),
                )
                .build(),
            ),
          );
        }

        // Add 500 Internal Server Error to all endpoints
        operation.responses.responses.insert(
          "500".to_string(),
          RefOr::T(
            ResponseBuilder::new()
              .description("Internal server error")
              .content(
                "application/json",
                ContentBuilder::new()
                  .schema(Some(Ref::from_schema_name("OpenAIApiError")))
                  .build(),
              )
              .build(),
          ),
        );
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    BodhiOpenAPIDoc, GlobalErrorResponses, ENDPOINT_APP_INFO, ENDPOINT_APP_SETUP, ENDPOINT_LOGOUT,
    ENDPOINT_MODELS, ENDPOINT_MODEL_FILES, ENDPOINT_MODEL_PULL, ENDPOINT_PING, ENDPOINT_TOKENS,
    ENDPOINT_USER_INFO,
  };
  use pretty_assertions::assert_eq;
  use serde_json::json;
  use utoipa::{
    openapi::{path::ParameterIn, OpenApi as OpenApiSpec, RefOr},
    Modify, OpenApi,
  };

  /// Helper function to get OpenAPI spec with GlobalErrorResponses modifier applied
  /// This ensures tests validate against the same spec used in production
  fn get_openapi_with_modifiers() -> OpenApiSpec {
    let mut spec = BodhiOpenAPIDoc::openapi();
    let modifier = GlobalErrorResponses;
    modifier.modify(&mut spec);
    spec
  }

  #[test]
  fn test_openapi_basic_info() {
    let api_doc = BodhiOpenAPIDoc::openapi();

    // Test API Info
    let info = &api_doc.info;
    assert_eq!(info.title, "Bodhi App APIs");

    // Test Contact Info
    let contact = info.contact.as_ref().unwrap();
    assert_eq!(contact.name.as_deref().unwrap(), "Bodhi API Support");
    assert_eq!(
      contact.url.as_deref().unwrap(),
      "https://github.com/BodhiSearch/BodhiApp/issues"
    );
    assert_eq!(contact.email.as_deref().unwrap(), "support@getbodhi.app");

    // Test Servers
    let servers = api_doc.servers.as_ref().unwrap();
    assert_eq!(servers.len(), 1);
    assert_eq!(servers[0].url, "http://localhost:1135");
    assert_eq!(
      servers[0].description.as_deref().unwrap(),
      "Local running instance"
    );
  }

  #[test]
  fn test_app_info_endpoint() {
    let api_doc = get_openapi_with_modifiers();

    // Verify tags
    let tags = api_doc.tags.as_ref().unwrap();
    assert!(tags.iter().any(|t| t.name == "system"));

    // Verify endpoint
    let paths = &api_doc.paths;
    let app_info = paths
      .paths
      .get(ENDPOINT_APP_INFO)
      .expect("App info endpoint not found");
    let get_op = app_info.get.as_ref().expect("GET operation not found");

    // Check operation details
    assert_eq!(get_op.tags.as_ref().unwrap()[0], "system");
    assert_eq!(get_op.operation_id.as_ref().unwrap(), "getAppInfo");

    // Check responses
    let responses = &get_op.responses;
    assert!(responses.responses.contains_key("200"));
    assert!(responses.responses.contains_key("500"));

    // Verify response schema references AppInfo
    let success_response = responses.responses.get("200").unwrap();
    if let RefOr::T(response) = success_response {
      assert!(response.content.get("application/json").is_some());
    }
  }

  #[test]
  fn test_setup_endpoint() {
    let api_doc = get_openapi_with_modifiers();

    // Verify tags
    let tags = api_doc.tags.as_ref().unwrap();
    assert!(tags.iter().any(|t| t.name == "setup"));

    // Verify endpoint
    let paths = &api_doc.paths;
    let setup = paths
      .paths
      .get(ENDPOINT_APP_SETUP)
      .expect("Setup endpoint not found");
    let post_op = setup.post.as_ref().expect("POST operation not found");

    // Check operation details
    assert_eq!(post_op.tags.as_ref().unwrap()[0], "setup");
    assert_eq!(post_op.operation_id.as_ref().unwrap(), "setupApp");

    // Check responses
    let responses = &post_op.responses;
    assert!(responses.responses.contains_key("200"));
    assert!(responses.responses.contains_key("400"));
    assert!(responses.responses.contains_key("500"));
  }

  #[test]
  fn test_logout_endpoint() {
    let api_doc = get_openapi_with_modifiers();

    // Verify tags
    let tags = api_doc.tags.as_ref().unwrap();
    assert!(tags.iter().any(|t| t.name == "auth"));

    // Verify endpoint
    let paths = &api_doc.paths;
    let logout = paths
      .paths
      .get(ENDPOINT_LOGOUT)
      .expect("Logout endpoint not found");
    let post_op = logout.post.as_ref().expect("POST operation not found");

    // Check operation details
    assert_eq!(post_op.tags.as_ref().unwrap()[0], "auth");
    assert_eq!(post_op.operation_id.as_ref().unwrap(), "logoutUser");

    // Check responses
    let responses = &post_op.responses;
    assert!(responses.responses.contains_key("200"));
    assert!(responses.responses.contains_key("500"));

    // Verify JSON response in 200 response
    let success_response = responses.responses.get("200").unwrap();
    if let RefOr::T(response) = success_response {
      assert!(response.content.contains_key("application/json"));
    }
  }

  #[test]
  fn test_ping_endpoint() {
    let api_doc = BodhiOpenAPIDoc::openapi();

    // Verify endpoint
    let paths = &api_doc.paths;
    let ping = paths
      .paths
      .get(ENDPOINT_PING)
      .expect("Ping endpoint not found");
    let get_op = ping.get.as_ref().expect("GET operation not found");

    // Check operation details
    assert_eq!(get_op.tags.as_ref().unwrap()[0], "system");
    assert_eq!(get_op.operation_id.as_ref().unwrap(), "pingServer");

    // Check response
    let responses = &get_op.responses;
    let success = responses.responses.get("200").unwrap();
    if let RefOr::T(response) = success {
      let content = response.content.get("application/json").unwrap();
      if let Some(example) = &content.example {
        assert_eq!(example, &json!({"message": "pong"}));
      } else {
        panic!("No example found for 200 status");
      }
    } else {
      panic!("No response found for 200 status");
    }
  }

  #[test]
  fn test_user_info_endpoint() {
    let api_doc = get_openapi_with_modifiers();

    // Verify endpoint
    let paths = &api_doc.paths;
    let user_info = paths
      .paths
      .get(ENDPOINT_USER_INFO)
      .expect("User info endpoint not found");
    let get_op = user_info.get.as_ref().expect("GET operation not found");

    // Check operation details
    assert_eq!(get_op.tags.as_ref().unwrap()[0], "auth");
    assert_eq!(get_op.operation_id.as_ref().unwrap(), "getCurrentUser");

    // Check responses
    let responses = &get_op.responses;
    assert!(responses.responses.contains_key("200"));
    assert!(responses.responses.contains_key("500"));

    // Verify response schema references UserResponse
    let success_response = responses.responses.get("200").unwrap();
    if let RefOr::T(response) = success_response {
      assert!(response.content.get("application/json").is_some());
    }
  }

  #[test]
  fn test_modelfiles_endpoint() {
    let api_doc = get_openapi_with_modifiers();

    // Verify tags
    let tags = api_doc.tags.as_ref().unwrap();
    assert!(tags.iter().any(|t| t.name == "models"));

    // Verify endpoint
    let paths = &api_doc.paths;
    let modelfiles = paths
      .paths
      .get(ENDPOINT_MODEL_FILES)
      .expect("Modelfiles endpoint not found");
    let get_op = modelfiles.get.as_ref().expect("GET operation not found");

    // Check operation details
    assert_eq!(get_op.tags.as_ref().unwrap()[0], "models");
    assert_eq!(get_op.operation_id.as_ref().unwrap(), "listModelFiles");

    // Check query parameters
    let params = get_op.parameters.as_ref().unwrap();
    assert!(params.iter().any(|p| p.name == "page"));
    assert!(params.iter().any(|p| p.name == "page_size"));
    assert!(params.iter().any(|p| p.name == "sort"));
    assert!(params.iter().any(|p| p.name == "sort_order"));

    // Check responses
    let responses = &get_op.responses;
    assert!(responses.responses.contains_key("200"));
    assert!(responses.responses.contains_key("500"));

    // Verify response schema references PaginatedResponse<LocalModelResponse>
    let success_response = responses.responses.get("200").unwrap();
    if let RefOr::T(response) = success_response {
      let content = response.content.get("application/json").unwrap();
      if let Some(example) = &content.example {
        // Verify example has correct structure
        assert!(example.get("data").is_some());
        assert!(example.get("total").is_some());
        assert!(example.get("page").is_some());
        assert!(example.get("page_size").is_some());
      } else {
        panic!("No example found for 200 status");
      }
    } else {
      panic!("No response found for 200 status");
    }
  }

  #[test]
  fn test_download_endpoints() {
    let api_doc = get_openapi_with_modifiers();

    // Verify tags
    let tags = api_doc.tags.as_ref().unwrap();
    assert!(tags.iter().any(|t| t.name == "models"));

    let paths = &api_doc.paths;

    // Test GET /modelfiles/pull endpoint
    let downloads = paths
      .paths
      .get(ENDPOINT_MODEL_PULL)
      .expect("Downloads endpoint not found");

    // Check GET operation
    let get_op = downloads.get.as_ref().expect("GET operation not found");
    assert_eq!(get_op.tags.as_ref().unwrap()[0], "models");
    assert_eq!(get_op.operation_id.as_ref().unwrap(), "listDownloads");

    // Check query parameters
    let params = get_op.parameters.as_ref().unwrap();
    assert!(params.iter().any(|p| p.name == "page"));
    assert!(params.iter().any(|p| p.name == "page_size"));
    assert!(params.iter().any(|p| p.name == "sort"));
    assert!(params.iter().any(|p| p.name == "sort_order"));

    // Check GET responses
    let get_responses = &get_op.responses;
    let get_200 = get_responses.responses.get("200").unwrap();
    if let RefOr::T(response) = get_200 {
      let content = response.content.get("application/json").unwrap();
      if let Some(example) = &content.example {
        assert!(example.get("data").is_some());
        assert!(example.get("total").is_some());
        assert!(example.get("page").is_some());
        assert!(example.get("page_size").is_some());
      } else {
        panic!("No example found for GET 200 status");
      }
    }

    // Check POST operation
    let post_op = downloads.post.as_ref().expect("POST operation not found");
    assert_eq!(post_op.tags.as_ref().unwrap()[0], "models");
    assert_eq!(post_op.operation_id.as_ref().unwrap(), "pullModelFile");

    // Verify request body schema
    assert!(post_op.request_body.is_some());

    // Check POST responses
    let post_responses = &post_op.responses;
    assert!(post_responses.responses.contains_key("200"));
    assert!(post_responses.responses.contains_key("500"));

    // Verify response schema references DownloadRequest
    let success_response = post_responses.responses.get("200").unwrap();
    if let RefOr::T(response) = success_response {
      let content = response.content.get("application/json").unwrap();
      assert!(content.schema.is_some());
    } else {
      panic!("No response found for POST 200 status");
    }
  }

  #[test]
  fn test_model_aliases_endpoint() {
    let api_doc = get_openapi_with_modifiers();

    // Verify endpoint
    let paths = &api_doc.paths;
    let aliases = paths
      .paths
      .get(ENDPOINT_MODELS)
      .expect("Model aliases endpoint not found");
    let get_op = aliases.get.as_ref().expect("GET operation not found");

    // Check operation details
    assert_eq!(get_op.tags.as_ref().unwrap()[0], "models");
    assert_eq!(get_op.operation_id.as_ref().unwrap(), "listAllModels");

    // Check query parameters
    let params = get_op.parameters.as_ref().unwrap();
    assert!(params.iter().any(|p| p.name == "page"));
    assert!(params.iter().any(|p| p.name == "page_size"));
    assert!(params.iter().any(|p| p.name == "sort"));
    assert!(params.iter().any(|p| p.name == "sort_order"));

    // Check responses
    let responses = &get_op.responses;
    let success = responses.responses.get("200").unwrap();
    if let RefOr::T(response) = success {
      let content = response.content.get("application/json").unwrap();
      if let Some(example) = &content.example {
        // Verify example has correct structure
        assert!(example.get("data").is_some());
        assert!(example.get("total").is_some());
        assert!(example.get("page").is_some());
        assert!(example.get("page_size").is_some());
      }
    }
  }

  #[test]
  fn test_create_token_endpoint() {
    let api_doc = get_openapi_with_modifiers();

    // Verify endpoint
    let paths = &api_doc.paths;
    let tokens = paths
      .paths
      .get(ENDPOINT_TOKENS)
      .expect("Tokens endpoint not found");
    let post_op = tokens.post.as_ref().expect("POST operation not found");

    // Check operation details
    assert_eq!(post_op.tags.as_ref().unwrap()[0], "api-keys");
    assert_eq!(post_op.operation_id.as_ref().unwrap(), "createApiToken");

    // Verify request body schema
    let request_body = post_op.request_body.as_ref().unwrap();
    let content = request_body.content.get("application/json").unwrap();
    if let Some(example) = &content.example {
      assert!(example.get("name").is_some());
    }

    // Check responses
    let responses = &post_op.responses;
    assert!(responses.responses.contains_key("201"));
    assert!(responses.responses.contains_key("400"));
    assert!(responses.responses.contains_key("500"));

    // Verify response schema
    let success_response = responses.responses.get("201").unwrap();
    if let RefOr::T(response) = success_response {
      let content = response.content.get("application/json").unwrap();
      assert!(content.schema.is_some());
    }
  }

  #[test]
  fn test_get_download_status_endpoint() {
    let api_doc = get_openapi_with_modifiers();
    let paths = &api_doc.paths;

    // Verify endpoint
    let status_path = paths
      .paths
      .get("/bodhi/v1/modelfiles/pull/{id}")
      .expect("Download status endpoint not found");

    let get_op = status_path.get.as_ref().expect("GET operation not found");

    // Check operation details
    assert_eq!(get_op.tags.as_ref().unwrap()[0], "models");
    assert_eq!(get_op.operation_id.as_ref().unwrap(), "getDownloadStatus");

    // Check path parameters
    let params = get_op.parameters.as_ref().unwrap();
    let id_param = params
      .iter()
      .find(|p| p.name == "id")
      .expect("ID parameter not found");
    assert_eq!(
      serde_json::to_string(&id_param.parameter_in).unwrap(),
      serde_json::to_string(&ParameterIn::Path).unwrap()
    );
    assert!(id_param.description.is_some());

    // Check responses
    let responses = &get_op.responses;

    // Check 200 response
    let success = responses.responses.get("200").unwrap();
    if let RefOr::T(response) = success {
      let content = response.content.get("application/json").unwrap();
      if let Some(example) = &content.example {
        // Verify example has correct structure
        assert!(example.get("id").is_some());
        assert!(example.get("repo").is_some());
        assert!(example.get("filename").is_some());
        assert!(example.get("status").is_some());
        assert!(example.get("created_at").is_some());
        assert!(example.get("updated_at").is_some());

        // Verify status is "completed" in example
        assert_eq!(example.get("status").unwrap(), "completed");
      } else {
        panic!("No example found for 200 status");
      }
    }

    // Check 404 response
    let not_found = responses.responses.get("404").unwrap();
    if let RefOr::T(response) = not_found {
      let content = response.content.get("application/json").unwrap();
      if let Some(example) = &content.example {
        let error = example.get("error").unwrap();
        assert_eq!(error.get("type").unwrap(), "not_found_error");
        assert_eq!(error.get("code").unwrap(), "db_error-item_not_found");
      } else {
        panic!("No example found for 404 status");
      }
    }

    // Check 500 response exists
    assert!(responses.responses.contains_key("500"));
  }

  #[test]
  fn test_list_tokens_endpoint() {
    let api_doc = get_openapi_with_modifiers();
    let paths = &api_doc.paths;

    // Verify endpoint
    let tokens_path = paths
      .paths
      .get(ENDPOINT_TOKENS)
      .expect("Tokens endpoint not found");

    let get_op = tokens_path.get.as_ref().expect("GET operation not found");

    // Check operation details
    assert_eq!(get_op.tags.as_ref().unwrap()[0], "api-keys");
    assert_eq!(get_op.operation_id.as_ref().unwrap(), "listApiTokens");

    // Check pagination parameters
    let params = get_op.parameters.as_ref().unwrap();
    assert!(params.iter().any(|p| p.name == "page"));
    assert!(params.iter().any(|p| p.name == "page_size"));
    assert!(params.iter().any(|p| p.name == "sort"));
    assert!(params.iter().any(|p| p.name == "sort_order"));

    // Check responses
    let responses = &get_op.responses;

    // Check 200 response
    let success = responses.responses.get("200").unwrap();
    if let RefOr::T(response) = success {
      let content = response.content.get("application/json").unwrap();
      if let Some(example) = &content.example {
        // Verify paginated response structure
        assert!(example.get("data").is_some());
        assert!(example.get("total").is_some());
        assert!(example.get("page").is_some());
        assert!(example.get("page_size").is_some());

        // Verify token data structure
        let data = example.get("data").unwrap().as_array().unwrap();
        let token = &data[0];
        assert!(token.get("id").is_some());
        assert!(token.get("user_id").is_some());
        assert!(token.get("name").is_some());
        assert!(token.get("token_id").is_some());
        assert!(token.get("status").is_some());
        assert!(token.get("created_at").is_some());
        assert!(token.get("updated_at").is_some());
      } else {
        panic!("No example found for 200 status");
      }
    }

    // Check 401 response - added by GlobalErrorResponses
    let unauthorized = responses.responses.get("401").unwrap();
    if let RefOr::T(response) = unauthorized {
      let content = response.content.get("application/json").unwrap();
      // Verify schema reference instead of example (GlobalErrorResponses uses schema refs)
      assert!(content.schema.is_some());
    }

    // Check 500 response exists
    assert!(responses.responses.contains_key("500"));
  }

  #[test]
  fn test_update_token_endpoint() {
    let api_doc = get_openapi_with_modifiers();
    let paths = &api_doc.paths;

    // Verify endpoint
    let update_path = paths
      .paths
      .get("/bodhi/v1/tokens/{id}")
      .expect("Update token endpoint not found");

    let put_op = update_path.put.as_ref().expect("PUT operation not found");

    // Check operation details
    assert_eq!(put_op.tags.as_ref().unwrap()[0], "api-keys");
    assert_eq!(put_op.operation_id.as_ref().unwrap(), "updateApiToken");

    // Check path parameters
    let params = put_op.parameters.as_ref().unwrap();
    let id_param = params
      .iter()
      .find(|p| p.name == "id")
      .expect("ID parameter not found");
    assert_eq!(
      serde_json::to_string(&id_param.parameter_in).unwrap(),
      serde_json::to_string(&ParameterIn::Path).unwrap()
    );
    assert!(id_param.description.is_some());

    // Check request body
    let request_body = put_op.request_body.as_ref().unwrap();
    let content = request_body.content.get("application/json").unwrap();
    if let Some(example) = &content.example {
      assert!(example.get("name").is_some());
      assert!(example.get("status").is_some());
      assert_eq!(example.get("status").unwrap(), "inactive");
    } else {
      panic!("No example found for request body");
    }

    // Check responses
    let responses = &put_op.responses;

    // Check 200 response
    let success = responses.responses.get("200").unwrap();
    if let RefOr::T(response) = success {
      let content = response.content.get("application/json").unwrap();
      if let Some(example) = &content.example {
        // Verify token structure
        assert!(example.get("id").is_some());
        assert!(example.get("user_id").is_some());
        assert!(example.get("name").is_some());
        assert!(example.get("token_id").is_some());
        assert!(example.get("status").is_some());
        assert!(example.get("created_at").is_some());
        assert!(example.get("updated_at").is_some());

        // Verify updated values
        assert_eq!(example.get("name").unwrap(), "Updated Token Name");
        assert_eq!(example.get("status").unwrap(), "inactive");
      } else {
        panic!("No example found for 200 status");
      }
    }

    // Check 401 response - added by GlobalErrorResponses
    let unauthorized = responses.responses.get("401").unwrap();
    if let RefOr::T(response) = unauthorized {
      let content = response.content.get("application/json").unwrap();
      // Verify schema reference instead of example (GlobalErrorResponses uses schema refs)
      assert!(content.schema.is_some());
    }

    // Check 404 response
    let not_found = responses.responses.get("404").unwrap();
    if let RefOr::T(response) = not_found {
      let content = response.content.get("application/json").unwrap();
      if let Some(example) = &content.example {
        let error = example.get("error").unwrap();
        assert_eq!(error.get("type").unwrap(), "not_found_error");
        assert_eq!(error.get("code").unwrap(), "entity_error-not_found");
      } else {
        panic!("No example found for 404 status");
      }
    }

    // Check 500 response exists
    assert!(responses.responses.contains_key("500"));
  }

  #[test]
  fn test_oai_models_endpoint() {
    let api_doc = get_openapi_with_modifiers();
    let paths = &api_doc.paths;

    // Verify endpoint
    let models_path = paths
      .paths
      .get("/v1/models")
      .expect("OpenAI models endpoint not found");

    let get_op = models_path.get.as_ref().expect("GET operation not found");

    // Check operation details
    assert_eq!(get_op.tags.as_ref().unwrap()[0], "openai");
    assert_eq!(get_op.operation_id.as_ref().unwrap(), "listModels");

    // Check responses
    let responses = &get_op.responses;

    // Check 200 response
    let success = responses.responses.get("200").unwrap();
    if let RefOr::T(response) = success {
      let content = response.content.get("application/json").unwrap();
      if let Some(example) = &content.example {
        // Verify response structure
        assert_eq!(example.get("object").unwrap(), "list");
        let data = example.get("data").unwrap().as_array().unwrap();

        // Check first model in the list
        let model = &data[0];
        assert!(model.get("id").is_some());
        assert_eq!(model.get("object").unwrap(), "model");
        assert!(model.get("created").is_some());
        assert!(model.get("owned_by").is_some());

        // Verify example values
        assert_eq!(model.get("id").unwrap(), "llama2:chat");
        assert_eq!(model.get("owned_by").unwrap(), "bodhi");
      } else {
        panic!("No example found for 200 status");
      }
    }

    // Check 401 response - added by GlobalErrorResponses
    let unauthorized = responses.responses.get("401").unwrap();
    if let RefOr::T(response) = unauthorized {
      let content = response.content.get("application/json").unwrap();
      // Verify schema reference instead of example (GlobalErrorResponses uses schema refs)
      assert!(content.schema.is_some());
    }

    // Check 500 response exists
    assert!(responses.responses.contains_key("500"));
  }

  /// Test that runtime OpenAPI spec matches the generated openapi.json file
  #[test]
  fn test_all_endpoints_match_spec() {
    let runtime_spec = BodhiOpenAPIDoc::openapi();
    let runtime_value = serde_json::to_value(&runtime_spec).unwrap();

    // Load the generated openapi.json file
    let spec_content = include_str!("../../../../openapi.json");
    let generated_spec: serde_json::Value = serde_json::from_str(spec_content).unwrap();

    // Compare key sections to ensure they're in sync
    assert_eq!(
      runtime_value["info"]["title"], generated_spec["info"]["title"],
      "API title mismatch between runtime and generated spec"
    );

    // Compare paths - ensure all paths exist in both specs
    let runtime_paths = runtime_value["paths"].as_object().unwrap();
    let generated_paths = generated_spec["paths"].as_object().unwrap();

    for (path, _) in runtime_paths {
      assert!(
        generated_paths.contains_key(path),
        "Path '{}' exists in runtime spec but missing from generated openapi.json",
        path
      );
    }

    for (path, _) in generated_paths {
      assert!(
        runtime_paths.contains_key(path),
        "Path '{}' exists in generated openapi.json but missing from runtime spec",
        path
      );
    }

    // Compare number of endpoints
    assert_eq!(
      runtime_paths.len(),
      generated_paths.len(),
      "Number of paths mismatch: runtime={}, generated={}",
      runtime_paths.len(),
      generated_paths.len()
    );
  }
}
