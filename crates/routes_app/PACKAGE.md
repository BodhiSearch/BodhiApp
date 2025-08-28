# PACKAGE.md - routes_app

This document provides detailed technical information for the `routes_app` crate, focusing on BodhiApp's application API orchestration architecture, sophisticated HTTP endpoint implementation, and comprehensive OpenAPI documentation generation patterns.

## Application API Orchestration Architecture

The `routes_app` crate serves as BodhiApp's **application API orchestration layer**, implementing comprehensive HTTP endpoints for model management, authentication, API token management, and application configuration with sophisticated service coordination.

### RouterState Integration Architecture
Application API routes coordinate extensively with HTTP infrastructure:

```rust
// Pattern structure (see src/routes_create.rs:45-67 for complete implementation)
#[utoipa::path(
    post,
    path = ENDPOINT_MODELS,
    tag = API_TAG_MODELS,
    operation_id = "createModelAlias",
    request_body = CreateAliasRequest,
    responses(
        (status = 201, description = "Model alias created successfully", body = AliasResponse),
        (status = 400, description = "Invalid request parameters", body = OpenAIApiError),
        (status = 500, description = "Internal server error", body = OpenAIApiError)
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_alias_handler(
    State(router_state): State<Arc<dyn RouterState>>,
    WithRejection(Json(request), _): WithRejection<Json<CreateAliasRequest>, ApiError>,
) -> Result<(StatusCode, Json<AliasResponse>), ApiError> {
    // Command orchestration through RouterState
    let command = CreateCommand::new(
        request.alias,
        Repo::new(&request.repo)?,
        request.filename,
        request.snapshot,
        true, // auto_download
        false, // update
        request.request_params.unwrap_or_default(),
        request.context_params.unwrap_or_default(),
    );
    
    command.execute(router_state.app_service()).await?;
    
    // Response generation with alias lookup
    let alias = router_state.app_service()
        .data_service()
        .find_alias(&request.alias)
        .ok_or_else(|| AliasNotFoundError(request.alias.clone()))?;
    
    Ok((StatusCode::CREATED, Json(AliasResponse::from(alias))))
}
```

### Command Layer Integration Architecture
Direct integration with commands crate for complex operations:

```rust
// Model pull orchestration (see src/routes_pull.rs:89-123 for complete implementation)
pub async fn create_pull_request_handler(
    State(router_state): State<Arc<dyn RouterState>>,
    WithRejection(Json(request), _): WithRejection<Json<NewDownloadRequest>, ApiError>,
) -> Result<(StatusCode, Json<DownloadRequest>), ApiError> {
    let pull_command = match request.alias {
        Some(alias) => PullCommand::ByAlias { alias },
        None => PullCommand::ByRepoFile {
            repo: Repo::new(&request.repo)?,
            filename: request.filename,
            snapshot: request.snapshot,
        },
    };
    
    // Execute command with progress tracking
    pull_command.execute(router_state.app_service(), None).await?;
    
    // Create download request record
    let download_request = DownloadRequest::new(
        request.repo,
        request.filename,
        request.snapshot.unwrap_or_else(|| "main".to_string()),
        DownloadStatus::Completed,
    );
    
    Ok((StatusCode::CREATED, Json(download_request)))
}
```

**Key Command Integration Features**:
- Direct command execution through HTTP endpoints with async coordination
- Progress tracking and error propagation from command layer to HTTP responses
- Service registry access through RouterState for consistent business logic coordination
- Command result translation to appropriate HTTP status codes and response objects

## Authentication Flow Implementation

### OAuth2 Authentication Orchestration
Sophisticated OAuth2 flow implementation with comprehensive security:

```rust
// OAuth initiation (see src/routes_login.rs:67-89 for complete implementation)
pub async fn auth_initiate_handler(
    State(router_state): State<Arc<dyn RouterState>>,
    session: Session,
    headers: HeaderMap,
) -> Result<(StatusCode, Json<RedirectResponse>), ApiError> {
    let app_status = app_status_or_default(router_state.app_service()).await;
    
    match app_status {
        AppStatus::Ready => {
            // Check existing authentication
            if let Some(access_token) = session.get::<String>(SESSION_KEY_ACCESS_TOKEN).await? {
                if let Ok(claims) = extract_claims(&access_token) {
                    if !claims.is_expired() {
                        return Ok((StatusCode::OK, Json(RedirectResponse {
                            location: CHAT_PATH.to_string(),
                        })));
                    }
                }
            }
            
            // Generate OAuth authorization URL with PKCE
            let app_reg_info = router_state.app_service()
                .secret_service()
                .get_app_reg_info().await?
                .ok_or(LoginError::AppRegInfoNotFound)?;
            
            let (auth_url, csrf_token, pkce_verifier) = generate_oauth_url(&app_reg_info, &headers)?;
            
            // Store PKCE verifier and state in session
            session.insert("pkce_verifier", pkce_verifier.secret()).await?;
            session.insert("csrf_state", csrf_token.secret()).await?;
            
            Ok((StatusCode::CREATED, Json(RedirectResponse {
                location: auth_url.to_string(),
            })))
        }
        _ => Err(LoginError::AppStatusInvalid(app_status).into()),
    }
}
```

### Session Management Integration
HTTP session coordination with Tower Sessions:

```rust
// OAuth callback processing (see src/routes_login.rs:156-189 for complete implementation)
pub async fn auth_callback_handler(
    State(router_state): State<Arc<dyn RouterState>>,
    Query(params): Query<HashMap<String, String>>,
    session: Session,
) -> Result<(StatusCode, Json<RedirectResponse>), ApiError> {
    // Validate OAuth callback parameters
    let code = params.get("code").ok_or_else(|| {
        BadRequestError::new("authorization code missing from callback".to_string())
    })?;
    
    let state = params.get("state").ok_or_else(|| {
        BadRequestError::new("state parameter missing from callback".to_string())
    })?;
    
    // Validate CSRF state
    let stored_state = session.get::<String>("csrf_state").await?
        .ok_or(LoginError::SessionInfoNotFound)?;
    
    if state != &stored_state {
        return Err(LoginError::StateDigestMismatch.into());
    }
    
    // Exchange authorization code for tokens
    let pkce_verifier = session.get::<String>("pkce_verifier").await?
        .ok_or(LoginError::SessionInfoNotFound)?;
    
    let app_reg_info = router_state.app_service()
        .secret_service()
        .get_app_reg_info().await?
        .ok_or(LoginError::AppRegInfoNotFound)?;
    
    let (access_token, refresh_token) = router_state.app_service()
        .auth_service()
        .exchange_auth_code(
            AuthorizationCode::new(code.clone()),
            ClientId::new(app_reg_info.client_id),
            ClientSecret::new(app_reg_info.client_secret),
            RedirectUrl::new(app_reg_info.redirect_uri)?,
            PkceCodeVerifier::new(pkce_verifier),
        ).await?;
    
    // Store tokens in session
    session.insert(SESSION_KEY_ACCESS_TOKEN, access_token.secret()).await?;
    session.insert(SESSION_KEY_REFRESH_TOKEN, refresh_token.secret()).await?;
    
    // Clear temporary session data
    session.remove::<String>("pkce_verifier").await?;
    session.remove::<String>("csrf_state").await?;
    
    Ok((StatusCode::OK, Json(RedirectResponse {
        location: CHAT_PATH.to_string(),
    })))
}
```

**Authentication Flow Features**:
- PKCE-based OAuth2 flow with proper state validation and CSRF protection
- Session-based token storage with secure cookie configuration
- Automatic token refresh with session updates and error recovery
- Comprehensive error handling with actionable user guidance

## API Token Management Implementation

### JWT Token Generation and Validation
Sophisticated API token management with database integration:

```rust
// API token creation (see src/routes_api_token.rs:45-78 for complete implementation)
pub async fn create_token_handler(
    State(router_state): State<Arc<dyn RouterState>>,
    WithRejection(Json(request), _): WithRejection<Json<CreateApiTokenRequest>, ApiError>,
) -> Result<(StatusCode, Json<ApiTokenResponse>), ApiError> {
    // Extract user information from session or bearer token
    let user_id = extract_user_id_from_auth_context(&router_state).await?;
    
    // Generate JWT token with specified scopes
    let token_id = uuid::Uuid::new_v4().to_string();
    let token_claims = TokenClaims {
        sub: user_id.clone(),
        token_id: token_id.clone(),
        scopes: request.scopes.clone(),
        exp: calculate_expiration(request.expires_in),
        iat: chrono::Utc::now().timestamp(),
    };
    
    let jwt_token = generate_jwt_token(&token_claims)?;
    
    // Store token metadata in database
    let token_digest = calculate_sha256_digest(&jwt_token);
    let api_token = ApiToken {
        id: token_id.clone(),
        user_id,
        name: request.name,
        token_digest,
        scopes: request.scopes,
        status: TokenStatus::Active,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        expires_at: token_claims.exp.map(|exp| {
            chrono::DateTime::from_timestamp(exp, 0).unwrap()
        }),
    };
    
    router_state.app_service()
        .db_service()
        .create_api_token(&api_token).await?;
    
    Ok((StatusCode::CREATED, Json(ApiTokenResponse {
        token: jwt_token,
        token_id,
        expires_at: api_token.expires_at,
    })))
}
```

### Token Lifecycle Management
Comprehensive token management with status tracking:

```rust
// Token status update (see src/routes_api_token.rs:123-145 for complete implementation)
pub async fn update_token_handler(
    State(router_state): State<Arc<dyn RouterState>>,
    Path(token_id): Path<String>,
    WithRejection(Json(request), _): WithRejection<Json<UpdateTokenRequest>, ApiError>,
) -> Result<(StatusCode, Json<ApiToken>), ApiError> {
    let user_id = extract_user_id_from_auth_context(&router_state).await?;
    
    // Verify token ownership
    let mut api_token = router_state.app_service()
        .db_service()
        .get_api_token_by_id(&token_id).await?
        .ok_or_else(|| ApiTokenError::TokenNotFound(token_id.clone()))?;
    
    if api_token.user_id != user_id {
        return Err(ApiTokenError::TokenNotFound(token_id).into());
    }
    
    // Update token status
    if let Some(status) = request.status {
        api_token.status = status;
        api_token.updated_at = chrono::Utc::now();
        
        router_state.app_service()
            .db_service()
            .update_api_token(&api_token).await?;
    }
    
    Ok((StatusCode::OK, Json(api_token)))
}
```

**API Token Management Features**:
- JWT token generation with configurable scopes and expiration
- Database-backed token storage with digest-based lookup for security
- Token lifecycle management with activation, deactivation, and expiration
- User-based token ownership with proper authorization checks

## OpenAPI Documentation Generation

### Utoipa Integration Architecture
Comprehensive OpenAPI specification generation with environment-specific configuration:

```rust
// OpenAPI document configuration (see src/openapi.rs:89-156 for complete implementation)
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

## Authentication
This API supports two authentication methods:

1. **Browser Session** (Default)
   - Login via `/bodhi/v1/auth/initiate` endpoint
   - Session cookie will be used automatically
   - Best for browser-based access

2. **API Token**
   - Create API Token using the app Menu > Settings > API Tokens
   - Use the API Token as the Authorization Bearer token in API calls
   - Best for programmatic access
"#
    ),
    components(
        schemas(
            OpenAIApiError,
            AppInfo,
            AppStatus,
            UserInfo,
            RedirectResponse,
            PaginatedDownloadResponse,
            PaginatedAliasResponse,
            PaginatedApiTokenResponse,
            PaginatedLocalModelResponse,
            SetupRequest,
            SetupResponse,
            CreateAliasRequest,
            UpdateAliasRequest,
            AliasResponse,
            ApiTokenResponse,
            ApiToken,
            CreateApiTokenRequest,
            TokenStatus,
            SettingInfo,
            SettingMetadata,
            SettingSource,
            UpdateSettingRequest
        ),
    ),
    paths(
        // System endpoints
        ping_handler,
        health_handler,
        app_info_handler,
        // Authentication endpoints
        auth_initiate_handler,
        auth_callback_handler,
        logout_handler,
        user_info_handler,
        // Model management endpoints
        create_alias_handler,
        update_alias_handler,
        list_local_aliases_handler,
        get_alias_handler,
        list_local_modelfiles_handler,
        // API token endpoints
        create_token_handler,
        list_tokens_handler,
        update_token_handler,
        // Settings endpoints
        list_settings_handler,
        update_setting_handler,
        delete_setting_handler,
    )
)]
pub struct BodhiOpenAPIDoc;
```

### Environment-Specific Configuration
Dynamic OpenAPI configuration based on application environment:

```rust
// Environment modifier (see src/openapi.rs:234-267 for complete implementation)
#[derive(Debug, derive_new::new)]
pub struct OpenAPIEnvModifier {
    setting_service: Arc<dyn SettingService>,
}

impl Modify for OpenAPIEnvModifier {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        // Add environment-specific server configuration
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
        
        // Add security schemes
        if let Some(components) = &mut openapi.components {
            components.security_schemes.insert(
                "bearer_auth".to_string(),
                SecurityScheme::Http(
                    HttpBuilder::default()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .description(Some(
                            "Enter the API token obtained from /bodhi/v1/tokens endpoint".to_string(),
                        ))
                        .build(),
                ),
            );
        }
    }
}
```

**OpenAPI Documentation Features**:
- Automatic OpenAPI 3.0 specification generation with comprehensive schema coverage
- Environment-specific server configuration with development and production variants
- Interactive API documentation with authentication flow guidance
- Comprehensive request/response examples with validation schemas

## Cross-Crate Integration Implementation

### Service Layer Coordination
Application API routes coordinate extensively with BodhiApp's service layer:

```rust
// Model listing with service coordination (see src/routes_models.rs:45-78 for complete implementation)
pub async fn list_local_aliases_handler(
    State(router_state): State<Arc<dyn RouterState>>,
    Query(params): Query<PaginationSortParams>,
) -> Result<Json<PaginatedAliasResponse>, ApiError> {
    let data_service = router_state.app_service().data_service();
    
    // Get all aliases with pagination
    let aliases = data_service.list_aliases()?;
    let total = aliases.len();
    
    // Apply sorting
    let mut sorted_aliases = aliases;
    if let Some(sort_field) = &params.sort {
        match sort_field.as_str() {
            "repo" => sorted_aliases.sort_by(|a, b| a.repo.cmp(&b.repo)),
            "filename" => sorted_aliases.sort_by(|a, b| a.filename.cmp(&b.filename)),
            "alias" => sorted_aliases.sort_by(|a, b| a.alias.cmp(&b.alias)),
            _ => {} // Default ordering
        }
        
        if params.sort_order == "desc" {
            sorted_aliases.reverse();
        }
    }
    
    // Apply pagination
    let start = (params.page - 1) * params.page_size;
    let end = std::cmp::min(start + params.page_size, total);
    let paginated_aliases = sorted_aliases.into_iter()
        .skip(start)
        .take(params.page_size)
        .map(AliasResponse::from)
        .collect();
    
    let response = PaginatedResponse {
        data: paginated_aliases,
        total,
        page: params.page,
        page_size: params.page_size,
    };
    
    Ok(Json(PaginatedAliasResponse::from(response)))
}
```

### Error Translation Architecture
Comprehensive error handling with HTTP status code mapping:

```rust
// Application-specific error types (see src/error.rs:15-45 for complete implementation)
#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum LoginError {
    #[error("app_reg_info_not_found")]
    #[error_meta(error_type = ErrorType::InvalidAppState)]
    AppRegInfoNotFound,
    
    #[error("app_status_invalid")]
    #[error_meta(error_type = ErrorType::InvalidAppState)]
    AppStatusInvalid(AppStatus),
    
    #[error(transparent)]
    SecretServiceError(#[from] SecretServiceError),
    
    #[error(transparent)]
    #[error_meta(error_type = ErrorType::Authentication, code = "login_error-session_error", args_delegate = false)]
    SessionError(#[from] tower_sessions::session::Error),
    
    #[error("oauth_error")]
    #[error_meta(error_type = ErrorType::BadRequest)]
    OAuthError(String),
    
    #[error(transparent)]
    AuthServiceError(#[from] AuthServiceError),
    
    #[error("state_digest_mismatch")]
    #[error_meta(error_type = ErrorType::BadRequest)]
    StateDigestMismatch,
}
```

**Cross-Crate Integration Features**:
- Service registry access through RouterState for consistent business logic coordination
- Command layer integration for complex multi-service operations
- Comprehensive error translation with localized messages and HTTP status code mapping
- Domain object integration for consistent validation and serialization patterns

## Request/Response Object Architecture

### Comprehensive API Object Definitions
Sophisticated request/response objects with validation and OpenAPI integration:

```rust
// Model creation request (see src/objs.rs:45-67 for complete implementation)
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateAliasRequest {
    #[validate(length(min = 1, message = "Alias name cannot be empty"))]
    alias: String,
    
    #[validate(length(min = 1, message = "Repository name cannot be empty"))]
    repo: String,
    
    #[validate(length(min = 1, message = "Filename cannot be empty"))]
    filename: String,
    
    snapshot: Option<String>,
    request_params: Option<OAIRequestParams>,
    context_params: Option<Vec<String>>,
}

// Paginated response wrapper (see src/objs.rs:89-123 for complete implementation)
#[derive(Serialize, Deserialize, ToSchema)]
pub struct PaginatedAliasResponse {
    pub data: Vec<AliasResponse>,
    pub total: usize,
    pub page: usize,
    pub page_size: usize,
}

impl From<PaginatedResponse<AliasResponse>> for PaginatedAliasResponse {
    fn from(paginated: PaginatedResponse<AliasResponse>) -> Self {
        PaginatedAliasResponse {
            data: paginated.data,
            total: paginated.total,
            page: paginated.page,
            page_size: paginated.page_size,
        }
    }
}
```

### Pagination and Sorting Support
Comprehensive pagination and sorting infrastructure:

```rust
// Pagination parameters (see src/objs.rs:15-35 for complete implementation)
#[derive(Debug, Deserialize, IntoParams)]
pub struct PaginationSortParams {
    /// Page number (1-based)
    #[serde(default = "default_page")]
    pub page: usize,

    /// Number of items per page (max 100)
    #[serde(default = "default_page_size")]
    pub page_size: usize,

    /// Field to sort by (repo, filename, size, updated_at, snapshot)
    #[serde(default)]
    pub sort: Option<String>,

    /// Sort order (asc or desc)
    #[serde(default = "default_sort_order")]
    pub sort_order: String,
}

fn default_page() -> usize { 1 }
fn default_page_size() -> usize { 30 }
fn default_sort_order() -> String { "asc".to_string() }
```

**API Object Features**:
- Comprehensive validation using validator crate with custom validation rules
- OpenAPI schema generation with ToSchema derive for interactive documentation
- Pagination and sorting support for large data sets with configurable parameters
- Type-safe request/response handling with serde serialization and deserialization

## Testing Infrastructure

### Application API Testing Patterns
Comprehensive testing infrastructure for HTTP endpoints:

```rust
// Test utilities (see src/test_utils/alias_response.rs:15-35 for complete implementation)
impl AliasResponse {
    pub fn llama3() -> Self {
        AliasResponseBuilder::default()
            .alias("llama3:instruct")
            .repo(Repo::LLAMA3)
            .filename(Repo::LLAMA3_Q8)
            .snapshot("5007652f7a641fe7170e0bad4f63839419bd9213")
            .source("user")
            .model_params(HashMap::new())
            .request_params(
                OAIRequestParamsBuilder::default()
                    .stop(vec![
                        "<|start_header_id|>".to_string(),
                        "<|end_header_id|>".to_string(),
                        "<|eot_id|>".to_string(),
                    ])
                    .build()
                    .unwrap(),
            )
            .context_params(vec!["--n-keep 24".to_string()])
            .build()
            .unwrap()
    }
}

// Test event coordination macro (see src/test_utils/mod.rs:5-15 for complete implementation)
#[macro_export]
macro_rules! wait_for_event {
    ($rx:expr, $event_name:expr, $timeout:expr) => {{
        loop {
            tokio::select! {
                event = $rx.recv() => {
                    match event {
                        Ok(e) if e == $event_name => break true,
                        _ => continue
                    }
                }
                _ = tokio::time::sleep($timeout) => break false
            }
        }
    }};
}
```

**Testing Infrastructure Features**:
- Test fixture creation with builder patterns for consistent test data
- Event coordination macros for testing async operations and state changes
- HTTP endpoint testing with axum-test integration for realistic request/response testing
- Service mocking coordination for isolated endpoint testing scenarios

## Extension Guidelines

### Adding New Application Endpoints
When creating new application API endpoints:

1. **Request/Response Design**: Define comprehensive API objects with validation using validator crate and ToSchema for OpenAPI
2. **Service Coordination**: Use RouterState for consistent AppService access and business logic coordination
3. **Command Integration**: Leverage commands crate for complex operations requiring multi-service coordination
4. **Error Handling**: Implement endpoint-specific errors with transparent service error wrapping and HTTP status mapping
5. **OpenAPI Documentation**: Add comprehensive Utoipa annotations with examples, security requirements, and proper schema definitions

### Authentication and Authorization Extensions
For new authentication and authorization patterns:

1. **OAuth2 Integration**: Follow established PKCE patterns with proper state validation and CSRF protection
2. **Session Management**: Integrate with Tower Sessions for consistent session handling and secure cookie configuration
3. **API Token Management**: Extend JWT token system with new scopes and authorization patterns while maintaining security
4. **Authorization Middleware**: Coordinate with auth_middleware for consistent security across all endpoints
5. **User Management**: Design user profile and account management features with proper privacy controls and data protection

### Cross-Service Integration Patterns
For features requiring coordination across multiple services:

1. **Command Orchestration**: Use commands crate for complex multi-service workflows with proper error boundaries and rollback
2. **Service Registry**: Coordinate through AppService registry for consistent business logic access and dependency injection
3. **Error Translation**: Convert service errors to appropriate HTTP responses with OpenAI-compatible error formats and localization
4. **Progress Tracking**: Implement progress feedback for long-running operations with cancellation support and status updates
5. **Transaction Management**: Ensure data consistency across service boundaries with proper transaction coordination and rollback capabilities

## Commands

**Application API Tests**: `cargo test -p routes_app` (includes HTTP endpoint and integration testing)  
**OpenAPI Validation**: `cargo test -p routes_app openapi` (includes OpenAPI specification validation)  
**Authentication Tests**: `cargo test -p routes_app auth` (includes OAuth2 flow and session management testing)