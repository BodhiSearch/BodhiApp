use crate::oai::{
  __path_chat_completions_handler, __path_embeddings_handler, __path_oai_model_handler,
  __path_oai_models_handler,
};
use crate::ollama::{
  __path_ollama_model_chat_handler, __path_ollama_model_show_handler, __path_ollama_models_handler,
};
use crate::{
  AccessRequestActionResponse, AccessRequestReviewResponse, AccessRequestStatusResponse,
  AuthCallbackRequest, AuthInitiateRequest, CreateAccessRequestResponse, LocalModelResponse,
  PaginationSortParams, PingResponse,
};
use crate::{
  AppInfo, DashboardUser, ListUsersParams, PaginatedLocalModelResponse, QueueStatusResponse,
  RedirectResponse, UserInfoEnvelope, UserResponse, __path_api_models_create,
  __path_api_models_destroy, __path_api_models_fetch_models, __path_api_models_formats,
  __path_api_models_show, __path_api_models_sync, __path_api_models_test, __path_api_models_update,
  __path_apps_approve_access_request, __path_apps_create_access_request,
  __path_apps_deny_access_request, __path_apps_get_access_request_review,
  __path_apps_get_access_request_status, __path_auth_callback, __path_auth_initiate,
  __path_auth_logout, __path_health_handler, __path_modelfiles_index, __path_models_copy,
  __path_models_create, __path_models_destroy, __path_models_index, __path_models_pull_create,
  __path_models_pull_index, __path_models_pull_show, __path_models_show, __path_models_update,
  __path_ping_handler, __path_queue_status_handler, __path_refresh_metadata_handler,
  __path_tokens_create, __path_tokens_index, __path_tokens_update,
  __path_users_access_request_approve, __path_users_access_request_reject,
  __path_users_access_requests_index, __path_users_access_requests_pending,
  __path_users_change_role, __path_users_destroy, __path_users_index, __path_users_info,
  __path_users_request_access, __path_users_request_status,
};
// MCP DTOs and handlers
use crate::{
  CreateAuthConfig, DynamicRegisterRequest, DynamicRegisterResponse, ListMcpServersResponse,
  ListMcpsResponse, McpServerResponse, OAuthDiscoverAsRequest, OAuthDiscoverAsResponse,
  OAuthDiscoverMcpRequest, OAuthDiscoverMcpResponse, OAuthLoginRequest, OAuthLoginResponse,
  OAuthTokenExchangeRequest, OAuthTokenResponse, __path_apps_mcps_index, __path_apps_mcps_show,
  __path_mcp_auth_configs_create, __path_mcp_auth_configs_destroy, __path_mcp_auth_configs_index,
  __path_mcp_auth_configs_show, __path_mcp_oauth_discover_as, __path_mcp_oauth_discover_mcp,
  __path_mcp_oauth_dynamic_register, __path_mcp_oauth_login, __path_mcp_oauth_token_exchange,
  __path_mcp_oauth_tokens_destroy, __path_mcp_oauth_tokens_show, __path_mcp_proxy_handler,
  __path_mcp_servers_create, __path_mcp_servers_index, __path_mcp_servers_show,
  __path_mcp_servers_update, __path_mcps_create, __path_mcps_destroy, __path_mcps_index,
  __path_mcps_show, __path_mcps_update,
};
// Settings and setup DTOs and handlers
use crate::OpenAIApiError;
use crate::{
  SetupRequest, SetupResponse, __path_settings_destroy, __path_settings_index,
  __path_settings_update, __path_setup_create, __path_setup_show,
};
// Tenant DTOs and handlers
use crate::{
  CreateTenantRequest, CreateTenantResponse, TenantListItem, TenantListResponse,
  __path_dashboard_auth_callback, __path_dashboard_auth_initiate, __path_tenants_activate,
  __path_tenants_create, __path_tenants_index,
};
use crate::{
  API_TAG_API_KEYS, API_TAG_APPS, API_TAG_AUTH, API_TAG_MCPS, API_TAG_MODELS, API_TAG_MODELS_ALIAS,
  API_TAG_MODELS_API, API_TAG_MODELS_FILES, API_TAG_OLLAMA, API_TAG_OPENAI, API_TAG_SETTINGS,
  API_TAG_SETUP, API_TAG_SYSTEM, API_TAG_TENANTS,
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
use services::{
  Alias, AliasResponse, ApiAliasResponse, ApiFormat, ApiFormatsResponse, ApiKey, ApiKeyUpdate,
  ApiModelRequest, AppAccessRequestStatus, AppRole, AppStatus, ApprovalStatus,
  ApproveAccessRequest, ApproveUserAccessRequest, ApprovedResources, ApprovedResourcesV1,
  ChangeRoleRequest, CopyAliasRequest, CreateAccessRequest, CreateMcpAuthConfigRequest,
  CreateTokenRequest, DownloadRequest, DownloadStatus, FetchModelsRequest, FetchModelsResponse,
  FlowType, Mcp, McpApproval, McpAuthConfigParam, McpAuthConfigParamInput, McpAuthConfigResponse,
  McpAuthConfigType, McpAuthConfigsListResponse, McpAuthParam, McpAuthParamInput, McpAuthParamType,
  McpAuthType, McpInstance, McpRequest, McpServer, McpServerInfo, McpServerRequest,
  ModelAliasResponse, NewDownloadRequest, OAIRequestParams, PaginatedAliasResponse,
  PaginatedDownloadResponse, PaginatedTokenResponse, PaginatedUserAccessResponse,
  PaginatedUserAliasResponse, RefreshRequest, RefreshResponse, RefreshSource, RequestedMcpServer,
  RequestedResources, RequestedResourcesV1, ResourceRole, SettingInfo, SettingMetadata,
  SettingService, SettingSource, TestCreds, TestPromptRequest, TestPromptResponse, TokenCreated,
  TokenDetail, TokenScope, TokenStatus, UpdateSettingRequest, UpdateTokenRequest,
  UserAccessStatusResponse, UserAliasRequest, UserAliasResponse, UserInfo, UserListResponse,
  UserScope,
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
make_ui_endpoint!(ENDPOINT_DASHBOARD_AUTH_INITIATE, "auth/dashboard/initiate");
make_ui_endpoint!(ENDPOINT_DASHBOARD_AUTH_CALLBACK, "auth/dashboard/callback");

make_ui_endpoint!(ENDPOINT_MODELS, "models");
make_ui_endpoint!(ENDPOINT_MODELS_ALIAS, "models/alias");
make_ui_endpoint!(ENDPOINT_MODELS_API, "models/api");
make_ui_endpoint!(ENDPOINT_MODELS_API_TEST, "models/api/test");
make_ui_endpoint!(ENDPOINT_MODELS_API_FETCH_MODELS, "models/api/fetch-models");
make_ui_endpoint!(ENDPOINT_MODELS_API_FORMATS, "models/api/formats");
make_ui_endpoint!(ENDPOINT_MODELS_FILES, "models/files");
make_ui_endpoint!(ENDPOINT_MODELS_FILES_PULL, "models/files/pull");
make_ui_endpoint!(ENDPOINT_MODELS_REFRESH, "models/refresh");
make_ui_endpoint!(ENDPOINT_QUEUE, "queue");
make_ui_endpoint!(ENDPOINT_CHAT_TEMPLATES, "chat_templates");
make_ui_endpoint!(ENDPOINT_TOKENS, "tokens");
make_ui_endpoint!(ENDPOINT_SETTINGS, "settings");
make_ui_endpoint!(ENDPOINT_APPS_MCPS, "apps/mcps");
make_ui_endpoint!(ENDPOINT_TENANTS, "tenants");
// MCP endpoint constants are defined in mcps/mod.rs

// dev-only debugging info endpoint
pub const ENDPOINT_DEV_SECRETS: &str = "/dev/secrets";
pub const ENDPOINT_DEV_ENVS: &str = "/dev/envs";
pub const ENDPOINT_DEV_DB_RESET: &str = "/dev/db-reset";
pub const ENDPOINT_DEV_CLIENTS_DAG: &str = "/dev/clients/{client_id}/dag";
pub const ENDPOINT_DEV_TENANTS_CLEANUP: &str = "/dev/tenants/cleanup";

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
        (name = API_TAG_MODELS, description = "Model files and aliases"),
        (name = API_TAG_MODELS_ALIAS, description = "User-created model aliases"),
        (name = API_TAG_MODELS_API, description = "Remote AI API model configuration"),
        (name = API_TAG_MODELS_FILES, description = "Local model files and downloads"),
        (name = API_TAG_SETTINGS, description = "Application settings management"),
        (name = API_TAG_MCPS, description = "MCP server management and tool execution"),
        (name = API_TAG_OPENAI, description = "OpenAI-compatible API endpoints"),
        (name = API_TAG_OLLAMA, description = "Ollama-compatible API endpoints"),
        (name = API_TAG_APPS, description = "External app API endpoints (OAuth token required)"),
        (name = API_TAG_TENANTS, description = "Tenant management endpoints"),
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
            AuthInitiateRequest,
            AuthCallbackRequest,
            UserResponse,
            UserInfoEnvelope,
            DashboardUser,
            AppRole,
            ResourceRole,
            TokenScope,
            UserScope,
            // access requests
            UserAccessStatusResponse,
            ApproveUserAccessRequest,
            PaginatedUserAccessResponse,
            // app access requests
            CreateAccessRequest,
            CreateAccessRequestResponse,
            AccessRequestStatusResponse,
            AccessRequestReviewResponse,
            ApproveAccessRequest,
            AccessRequestActionResponse,
            FlowType,
            AppAccessRequestStatus,
            ApprovalStatus,
            RequestedResources,
            RequestedResourcesV1,
            ApprovedResources,
            ApprovedResourcesV1,
            RequestedMcpServer,
            McpApproval,
            McpInstance,
            // user management
            ListUsersParams,
            UserListResponse,
            UserInfo,
            ChangeRoleRequest,
            // api keys/token
            CreateTokenRequest,
            TokenCreated,
            UpdateTokenRequest,
            TokenStatus,
            PaginatedTokenResponse,
            TokenDetail,
            // api models
            ApiModelRequest,
            ApiKey,
            ApiKeyUpdate,
            TestCreds,
            TestPromptRequest,
            TestPromptResponse,
            FetchModelsRequest,
            FetchModelsResponse,
            ApiFormatsResponse,
            ApiFormat,
            // models
            UserAliasRequest,
            CopyAliasRequest,
            OAIRequestParams,
            PaginatedUserAliasResponse,
            UserAliasResponse,
            ModelAliasResponse,
            ApiAliasResponse,
            PaginationSortParams,
            PaginatedAliasResponse,
            AliasResponse,
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
            // mcps
            McpRequest,
            McpServerRequest,
            Mcp,
            McpServerResponse,
            ListMcpsResponse,
            ListMcpServersResponse,
            McpServer,
            McpServerInfo,
            // unified auth configs
            CreateAuthConfig,
            McpAuthType,
            McpAuthParamType,
            McpAuthConfigType,
            McpAuthConfigParam,
            McpAuthConfigParamInput,
            McpAuthParam,
            McpAuthParamInput,
            CreateMcpAuthConfigRequest,
            McpAuthConfigResponse,
            McpAuthConfigsListResponse,
            // mcp oauth
            OAuthTokenResponse,
            OAuthLoginRequest,
            OAuthLoginResponse,
            OAuthTokenExchangeRequest,
            OAuthDiscoverAsRequest,
            OAuthDiscoverAsResponse,
            OAuthDiscoverMcpRequest,
            OAuthDiscoverMcpResponse,
            DynamicRegisterRequest,
            DynamicRegisterResponse,
            // tenants
            TenantListItem,
            TenantListResponse,
            CreateTenantRequest,
            CreateTenantResponse,
        ),
        responses( ),
    ),
    paths(
        // System endpoints
        ping_handler,
        health_handler,

        // Setup endpoints
        setup_show,
        setup_create,

        // Authentication endpoints
        auth_initiate,
        auth_callback,
        auth_logout,
        users_info,

        // API Keys endpoints
        tokens_create,
        tokens_update,
        tokens_index,

        // API Models endpoints
        api_models_show,
        api_models_create,
        api_models_update,
        api_models_destroy,
        api_models_sync,
        api_models_test,
        api_models_fetch_models,
        api_models_formats,

        // Models endpoints
        models_create,
        models_update,
        models_destroy,
        models_copy,
        models_index,
        modelfiles_index,
        models_show,
        models_pull_index,
        models_pull_create,
        models_pull_show,
        refresh_metadata_handler,
        queue_status_handler,

        // Settings endpoints
        settings_index,
        settings_update,
        settings_destroy,

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
        users_request_access,
        users_request_status,
        users_access_requests_pending,
        users_access_requests_index,
        users_access_request_approve,
        users_access_request_reject,

        // App access request endpoints
        apps_create_access_request,
        apps_get_access_request_status,
        apps_get_access_request_review,
        apps_approve_access_request,
        apps_deny_access_request,

        // User management endpoints
        users_index,
        users_change_role,
        users_destroy,

        // MCP endpoints
        mcps_index,
        mcps_create,
        mcps_show,
        mcps_update,
        mcps_destroy,
        // MCP server admin endpoints
        mcp_servers_index,
        mcp_servers_show,
        mcp_servers_create,
        mcp_servers_update,
        // Unified auth config endpoints
        mcp_auth_configs_create,
        mcp_auth_configs_index,
        mcp_auth_configs_show,
        mcp_auth_configs_destroy,
        // OAuth flow endpoints
        mcp_oauth_login,
        mcp_oauth_token_exchange,
        // OAuth discovery endpoints
        mcp_oauth_discover_as,
        mcp_oauth_discover_mcp,
        mcp_oauth_dynamic_register,
        // OAuth token endpoints
        mcp_oauth_tokens_show,
        mcp_oauth_tokens_destroy,

        // MCP proxy endpoint
        mcp_proxy_handler,

        // External app endpoints
        apps_mcps_index,
        apps_mcps_show,

        // Tenant management endpoints
        tenants_index,
        tenants_create,
        tenants_activate,

        // Dashboard auth endpoints
        dashboard_auth_initiate,
        dashboard_auth_callback
    )
)]
pub struct BodhiOpenAPIDoc;

fn apply_security_schemes(components: &mut utoipa::openapi::Components) {
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

/// Modifies OpenAPI documentation with environment-specific settings
#[derive(Debug, derive_new::new)]
pub struct OpenAPIEnvModifier {
  setting_service: Arc<dyn SettingService>,
}

impl OpenAPIEnvModifier {
  pub async fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
    let server_url = self.setting_service.public_server_url().await;
    let desc = if self.setting_service.is_production().await {
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
      apply_security_schemes(components);
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
      apply_security_schemes(components);
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
