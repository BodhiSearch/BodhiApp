# Backend OpenAPI with Utoipa

> **AI Coding Assistant Guide**: This document provides project-specific patterns for OpenAPI documentation using utoipa in the Bodhi App backend. Focus on established conventions and project-specific implementation details.

## Required Documentation References

**MUST READ for OpenAPI implementation:**
- `ai-docs/01-architecture/rust-backend.md` - Rust backend development patterns
- `ai-docs/01-architecture/backend-testing.md` - Backend testing approaches
- `ai-docs/03-crates/routes_app.md` - Application routes crate structure

**FOR COMPLETE BACKEND CONTEXT:**
- `ai-docs/01-architecture/authentication.md` - Authentication patterns
- `ai-docs/01-architecture/development-conventions.md` - Coding standards

## Project-Specific OpenAPI Patterns

### API Tag Constants System

**CRITICAL**: Always use API tag constants from `objs::api_tags`:

```rust
use objs::{
    API_TAG_SYSTEM,      // System endpoints (ping, info)
    API_TAG_SETUP,       // App setup endpoints
    API_TAG_AUTH,        // Authentication endpoints
    API_TAG_API_KEYS,    // API token management
    API_TAG_MODELS,      // Model management
    API_TAG_SETTINGS,    // Application settings
    API_TAG_OPENAI,      // OpenAI-compatible endpoints
    API_TAG_OLLAMA,      // Ollama-compatible endpoints
};

#[utoipa::path(
    get,
    path = ENDPOINT_MODELS,
    tag = API_TAG_MODELS,  // Use constant, not "models"
    // ...
)]
```

### Endpoint Path Constants

**Bodhi App Endpoints** (prefix: `/bodhi/v1/`):
```rust
// From routes_app/src/openapi.rs
pub const ENDPOINT_PING: &str = "/ping";
pub const ENDPOINT_APP_INFO: &str = "/bodhi/v1/info";
pub const ENDPOINT_APP_SETUP: &str = "/bodhi/v1/setup";
pub const ENDPOINT_AUTH_INITIATE: &str = "/bodhi/v1/auth/initiate";
pub const ENDPOINT_AUTH_CALLBACK: &str = "/bodhi/v1/auth/callback";
pub const ENDPOINT_MODELS: &str = "/bodhi/v1/models";
pub const ENDPOINT_MODEL_FILES: &str = "/bodhi/v1/modelfiles";
pub const ENDPOINT_TOKENS: &str = "/bodhi/v1/tokens";
pub const ENDPOINT_SETTINGS: &str = "/bodhi/v1/settings";
```

**OpenAI-Compatible Endpoints** (prefix: `/v1/`):
```rust
// From routes_oai/src/lib.rs
pub const ENDPOINT_OAI_MODELS: &str = "/v1/models";
pub const ENDPOINT_OAI_CHAT_COMPLETIONS: &str = "/v1/chat/completions";
```

**Ollama-Compatible Endpoints** (prefix: `/api/`):
```rust
pub const ENDPOINT_OLLAMA_TAGS: &str = "/api/tags";
pub const ENDPOINT_OLLAMA_SHOW: &str = "/api/show";
pub const ENDPOINT_OLLAMA_CHAT: &str = "/api/chat";
```

### Error Response Patterns

**Standard Error Response**:
```rust
use objs::OpenAIApiError;

// In utoipa::path responses
responses(
    (status = 200, description = "Success", body = SuccessResponse),
    (status = 400, description = "Bad request", body = OpenAIApiError),
    (status = 401, description = "Unauthorized", body = OpenAIApiError),
    (status = 404, description = "Not found", body = OpenAIApiError),
    (status = 500, description = "Internal server error", body = OpenAIApiError)
)
```

**Actual Error Schema**:
```rust
// From objs/src/error/error_oai.rs
#[derive(Serialize, Deserialize, ToSchema)]
pub struct OpenAIApiError {
    pub error: ErrorBody,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ErrorBody {
    pub message: String,
    pub r#type: String,
    pub code: Option<String>,
    pub param: Option<String>,
}
```

**Ollama Error Pattern**:
```rust
#[derive(Serialize, Deserialize, ToSchema)]
pub struct OllamaError {
    error: String,
}
```

### Pagination Patterns

**Pagination Query Parameters**:
```rust
// From routes_app/src/objs.rs
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

**Pagination Response**:
```rust
#[derive(Serialize, Deserialize, ToSchema)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: usize,
    pub page: usize,
    pub page_size: usize,
}

// Usage in endpoint
#[utoipa::path(
    get,
    path = ENDPOINT_MODELS,
    tag = API_TAG_MODELS,
    params(PaginationSortParams),
    responses(
        (status = 200, description = "List of models", 
         body = PaginatedResponse<AliasResponse>),
        (status = 500, description = "Internal server error", 
         body = OpenAIApiError)
    )
)]
```

### Authentication Patterns

**Security Schemes**:
```rust
// From routes_app/src/openapi.rs - BodhiOpenAPIDoc
components(
    securitySchemes(
        ("bearer_auth" = SecurityScheme::Http(
            HttpBuilder::new()
                .scheme(HttpAuthScheme::Bearer)
                .bearer_format("JWT")
                .description(Some("API Token authentication"))
                .build()
        )),
        ("session_auth" = SecurityScheme::ApiKey(
            ApiKeyBuilder::new()
                .location(ApiKeyLocation::Cookie)
                .name("session")
                .description(Some("Browser session authentication"))
                .build()
        ))
    )
)
```

**Security Usage**:
```rust
#[utoipa::path(
    get,
    path = ENDPOINT_MODELS,
    tag = API_TAG_MODELS,
    security(
        ("bearer_auth" = []),    // API token
        ("session_auth" = [])    // Browser session
    )
)]
```

### Request/Response Schema Patterns

**Schema with Examples**:
```rust
#[derive(Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "alias": "llama3:instruct",
    "repo": "microsoft/DialoGPT-medium",
    "filename": "model.gguf"
}))]
pub struct AliasResponse {
    #[schema(example = "llama3:instruct")]
    pub alias: String,
    
    #[schema(example = "microsoft/DialoGPT-medium")]
    pub repo: String,
    
    #[schema(example = "model.gguf")]
    pub filename: String,
    
    pub context_params: GptContextParams,
    pub request_params: OAIRequestParams,
    pub model_params: HashMap<String, Value>,
    pub source: String,
    pub snapshot: String,
    pub chat_template: String,
}
```

**Request Body with Examples**:
```rust
#[utoipa::path(
    post,
    path = ENDPOINT_OAI_CHAT_COMPLETIONS,
    tag = API_TAG_OPENAI,
    request_body(
        content = serde_json::Value,
        example = json!({
            "model": "llama2:chat",
            "messages": [
                {
                    "role": "system",
                    "content": "You are a helpful assistant."
                },
                {
                    "role": "user",
                    "content": "Hello!"
                }
            ],
            "temperature": 0.7,
            "max_tokens": 100,
            "stream": false
        })
    )
)]
```

### OpenAPI Document Configuration

**Main OpenAPI Document**:
```rust
// From routes_app/src/openapi.rs
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

2. **API Token**
   - Create API Token using the app Menu > Settings > API Tokens
   - Use the API Token as the Authorization Bearer token
"#
    ),
    servers(
        (url = "http://localhost:1135", description = "Local running instance"),
    ),
    paths(
        // System endpoints
        ping_handler,
        app_info_handler,
        
        // Setup endpoints
        setup_handler,
        
        // Authentication endpoints
        auth_initiate_handler,
        auth_callback_handler,
        logout_handler,
        user_info_handler,
        
        // Models endpoints
        list_local_aliases_handler,
        get_alias_handler,
        list_local_modelfiles_handler,
        // ... more paths
    ),
    components(
        schemas(
            OpenAIApiError,
            ErrorBody,
            PaginatedResponse<AliasResponse>,
            AliasResponse,
            // ... more schemas
        )
    ),
    tags(
        (name = API_TAG_SYSTEM, description = "System information and health check endpoints"),
        (name = API_TAG_SETUP, description = "Application setup and configuration endpoints"),
        (name = API_TAG_AUTH, description = "Authentication and authorization endpoints"),
        (name = API_TAG_API_KEYS, description = "API key management endpoints"),
        (name = API_TAG_MODELS, description = "Model management and configuration endpoints"),
        (name = API_TAG_SETTINGS, description = "Application settings endpoints"),
        (name = API_TAG_OPENAI, description = "OpenAI-compatible API endpoints"),
        (name = API_TAG_OLLAMA, description = "Ollama-compatible API endpoints"),
    ),
    modifiers(&OpenAPIEnvModifier)
)]
pub struct BodhiOpenAPIDoc;
```

### Testing Patterns

**Contract Testing**:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_all_endpoints_match_spec() {
        let runtime_spec = BodhiOpenAPIDoc::openapi();
        let runtime_value = serde_json::to_value(&runtime_spec).unwrap();
        
        // Load the generated openapi.json file
        let spec_content = include_str!("../../../openapi.json");
        let generated_spec: serde_json::Value = serde_json::from_str(spec_content).unwrap();
        
        // Compare key sections to ensure they're in sync
        assert_eq!(
            runtime_value["info"]["title"],
            generated_spec["info"]["title"],
            "API title mismatch between runtime and generated spec"
        );
        
        // Verify all paths exist in both specs
        let runtime_paths = runtime_value["paths"].as_object().unwrap();
        let generated_paths = generated_spec["paths"].as_object().unwrap();
        
        assert_eq!(
            runtime_paths.len(),
            generated_paths.len(),
            "Path count mismatch: runtime has {}, generated has {}",
            runtime_paths.len(),
            generated_paths.len()
        );
        
        for path in runtime_paths.keys() {
            assert!(
                generated_paths.contains_key(path),
                "Path '{}' exists in runtime but not in generated spec",
                path
            );
        }
    }
}
```

## Implementation Checklist

### For New Endpoints:
1. **Define endpoint constant** in appropriate module
2. **Use API tag constant** from `objs::api_tags`
3. **Document all response codes** with appropriate schemas
4. **Add security requirements** (bearer_auth, session_auth)
5. **Include request/response examples** in schema definitions
6. **Add to OpenAPI paths** in `BodhiOpenAPIDoc`
7. **Update contract test** to verify spec consistency

### For Schema Updates:
1. **Use `ToSchema` derive** for automatic schema generation
2. **Add examples** using `#[schema(example = ...)]`
3. **Include in components** section of OpenAPI doc
4. **Test schema generation** with actual data structures

### For Error Handling:
1. **Use `OpenAIApiError`** for standard error responses
2. **Use `OllamaError`** for Ollama-compatible endpoints
3. **Document specific error codes** in response descriptions
4. **Include error examples** in OpenAPI responses

## Common Pitfalls to Avoid

1. **Don't hardcode tag strings** - always use `API_TAG_*` constants
2. **Don't forget security annotations** - all endpoints need authentication
3. **Don't skip error documentation** - include 400, 401, 404, 500 responses
4. **Don't use generic schemas** - create specific response types
5. **Don't forget examples** - they're crucial for API documentation quality

## Adding New Endpoints - Step-by-Step Guide

### Step 1: Define Endpoint Constant

**Add to appropriate module** (`routes_app/src/openapi.rs` or `routes_oai/src/lib.rs`):

```rust
// For Bodhi App endpoints
make_ui_endpoint!(ENDPOINT_NEW_FEATURE, "new-feature");
// Expands to: pub const ENDPOINT_NEW_FEATURE: &str = "/bodhi/v1/new-feature";

// For OpenAI-compatible endpoints
pub const ENDPOINT_OAI_NEW_FEATURE: &str = "/v1/new-feature";

// For Ollama-compatible endpoints  
pub const ENDPOINT_OLLAMA_NEW_FEATURE: &str = "/api/new-feature";
```

### Step 2: Create Request/Response Schemas

**Define schemas with utoipa annotations**:

```rust
// Request schema
#[derive(Debug, Deserialize, ToSchema)]
#[schema(example = json!({
    "name": "example-name",
    "description": "Example description"
}))]
pub struct CreateFeatureRequest {
    #[schema(example = "example-name")]
    pub name: String,
    
    #[schema(example = "Example description")]
    pub description: String,
}

// Response schema
#[derive(Debug, Serialize, ToSchema)]
#[schema(example = json!({
    "id": "feature-123",
    "name": "example-name",
    "description": "Example description",
    "created_at": "2024-01-20T12:00:00Z"
}))]
pub struct FeatureResponse {
    #[schema(example = "feature-123")]
    pub id: String,
    
    #[schema(example = "example-name")]
    pub name: String,
    
    #[schema(example = "Example description")]
    pub description: String,
    
    #[schema(example = "2024-01-20T12:00:00Z")]
    pub created_at: String,
}
```

### Step 3: Implement Handler with utoipa::path

**Create handler function with complete documentation**:

```rust
use objs::{ApiError, OpenAIApiError, API_TAG_MODELS}; // Use appropriate tag

/// Create a new feature
#[utoipa::path(
    post,
    path = ENDPOINT_NEW_FEATURE,
    tag = API_TAG_MODELS,  // Use appropriate API_TAG_* constant
    operation_id = "createFeature",
    request_body(
        content = CreateFeatureRequest,
        description = "Feature creation request",
        example = json!({
            "name": "my-feature",
            "description": "A new feature"
        })
    ),
    responses(
        (status = 201, description = "Feature created successfully", 
         body = FeatureResponse,
         example = json!({
             "id": "feature-456",
             "name": "my-feature", 
             "description": "A new feature",
             "created_at": "2024-01-20T12:00:00Z"
         })
        ),
        (status = 400, description = "Invalid request", body = OpenAIApiError),
        (status = 401, description = "Unauthorized", body = OpenAIApiError),
        (status = 500, description = "Internal server error", body = OpenAIApiError)
    ),
    security(
        ("bearer_auth" = []),    // API token auth
        ("session_auth" = [])    // Browser session auth
    )
)]
pub async fn create_feature_handler(
    State(state): State<Arc<dyn RouterState>>,
    Json(request): Json<CreateFeatureRequest>,
) -> Result<(StatusCode, Json<FeatureResponse>), ApiError> {
    // Implementation...
    let feature = state.app_service().feature_service().create(request).await?;
    Ok((StatusCode::CREATED, Json(feature)))
}
```

### Step 4: Add to Router

**Register the route in appropriate router**:

```rust
// In routes_all/src/routes.rs or appropriate router file
use routes_app::{create_feature_handler, ENDPOINT_NEW_FEATURE};

pub fn create_app_router(shared_context: SharedContext) -> Router {
    Router::new()
        // ... existing routes
        .route(ENDPOINT_NEW_FEATURE, post(create_feature_handler))
        // ... more routes
        .with_state(shared_context.router_state)
}
```

### Step 5: Add to OpenAPI Documentation

**Update `BodhiOpenAPIDoc` in `routes_app/src/openapi.rs`**:

```rust
// 1. Import the handler
use crate::{
    // ... existing imports
    create_feature_handler, __path_create_feature_handler,
};

// 2. Add to paths section
#[derive(OpenApi)]
#[openapi(
    // ... existing config
    paths(
        // ... existing paths grouped by tag
        
        // Models endpoints (if using API_TAG_MODELS)
        list_local_aliases_handler,
        get_alias_handler,
        create_feature_handler,  // Add new handler here
        // ... more model endpoints
    ),
    components(
        schemas(
            // ... existing schemas
            CreateFeatureRequest,    // Add request schema
            FeatureResponse,         // Add response schema
            // ... more schemas
        )
    ),
    // ... rest of config
)]
pub struct BodhiOpenAPIDoc;
```

### Step 6: Handle Pagination (if applicable)

**For list endpoints, use pagination pattern**:

```rust
/// List features with pagination
#[utoipa::path(
    get,
    path = ENDPOINT_NEW_FEATURE,
    tag = API_TAG_MODELS,
    operation_id = "listFeatures",
    params(PaginationSortParams),
    responses(
        (status = 200, description = "List of features", 
         body = PaginatedResponse<FeatureResponse>),
        (status = 500, description = "Internal server error", 
         body = OpenAIApiError)
    ),
    security(
        ("bearer_auth" = []),
        ("session_auth" = [])
    )
)]
pub async fn list_features_handler(
    State(state): State<Arc<dyn RouterState>>,
    Query(params): Query<PaginationSortParams>,
) -> Result<Json<PaginatedResponse<FeatureResponse>>, ApiError> {
    let (page, page_size, sort, sort_order) = extract_pagination_sort_params(params);
    let mut features = state.app_service().feature_service().list().await?;
    
    // Apply sorting and pagination
    sort_features(&mut features, &sort, &sort_order);
    let total = features.len();
    let (start, end) = calculate_pagination(page, page_size, total);
    
    let data: Vec<FeatureResponse> = features
        .into_iter()
        .skip(start)
        .take(end - start)
        .map(Into::into)
        .collect();

    Ok(Json(PaginatedResponse {
        data,
        total,
        page,
        page_size,
    }))
}
```

### Step 7: Add Error Handling

**Define custom errors if needed**:

```rust
#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum FeatureError {
    #[error("feature_not_found")]
    #[error_meta(error_type = ErrorType::NotFound)]
    NotFound(String),
    
    #[error("feature_already_exists")]
    #[error_meta(error_type = ErrorType::BadRequest)]
    AlreadyExists(String),
    
    #[error("invalid_feature_name")]
    #[error_meta(error_type = ErrorType::BadRequest)]
    InvalidName(String),
}
```

### Step 8: Write Tests

**Add endpoint tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{Method, StatusCode};
    use serde_json::json;
    
    #[rstest]
    #[awt]
    #[tokio::test]
    async fn test_create_feature_success(
        #[future] router_state_stub: DefaultRouterState,
    ) -> anyhow::Result<()> {
        let request_body = json!({
            "name": "test-feature",
            "description": "Test description"
        });

        let response = test_router(router_state_stub)
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri(ENDPOINT_NEW_FEATURE)
                    .header("Content-Type", "application/json")
                    .body(Body::from(request_body.to_string()))
                    .unwrap(),
            )
            .await?;

        assert_eq!(response.status(), StatusCode::CREATED);
        
        let feature: FeatureResponse = response.json().await?;
        assert_eq!(feature.name, "test-feature");
        assert_eq!(feature.description, "Test description");
        
        Ok(())
    }

    #[rstest]
    #[awt]
    #[tokio::test]
    async fn test_create_feature_validation_error(
        #[future] router_state_stub: DefaultRouterState,
    ) -> anyhow::Result<()> {
        let request_body = json!({
            "name": "",  // Invalid empty name
            "description": "Test description"
        });

        let response = test_router(router_state_stub)
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri(ENDPOINT_NEW_FEATURE)
                    .header("Content-Type", "application/json")
                    .body(Body::from(request_body.to_string()))
                    .unwrap(),
            )
            .await?;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        
        let error: OpenAIApiError = response.json().await?;
        assert_eq!(error.error.r#type, "invalid_request_error");
        
        Ok(())
    }
}
```

### Step 9: Update Contract Test

**Verify the new endpoint is included**:

```rust
#[test]
fn test_all_endpoints_match_spec() {
    let runtime_spec = BodhiOpenAPIDoc::openapi();
    let runtime_value = serde_json::to_value(&runtime_spec).unwrap();
    
    // Verify new endpoint exists
    let paths = runtime_value["paths"].as_object().unwrap();
    assert!(
        paths.contains_key(ENDPOINT_NEW_FEATURE),
        "New endpoint '{}' should be documented in OpenAPI spec",
        ENDPOINT_NEW_FEATURE
    );
    
    // ... rest of contract test
}
```

### Step 10: Generate and Verify OpenAPI JSON

**Run generation to update openapi.json**:

```bash
# If you have a generation script
cargo run --bin generate-openapi

# Or generate manually and save
curl http://localhost:1135/api-docs/openapi.json > openapi.json
```

### Common Endpoint Patterns

**GET with Path Parameter**:
```rust
#[utoipa::path(
    get,
    path = ENDPOINT_NEW_FEATURE.to_owned() + "/{id}",
    tag = API_TAG_MODELS,
    params(
        ("id" = String, Path, description = "Feature identifier")
    ),
    responses(
        (status = 200, description = "Feature details", body = FeatureResponse),
        (status = 404, description = "Feature not found", body = OpenAIApiError)
    )
)]
pub async fn get_feature_handler(
    Path(id): Path<String>,
    State(state): State<Arc<dyn RouterState>>,
) -> Result<Json<FeatureResponse>, ApiError> {
    let feature = state.app_service().feature_service().get(&id).await?;
    Ok(Json(feature))
}
```

**PUT/PATCH for Updates**:
```rust
#[utoipa::path(
    put,
    path = ENDPOINT_NEW_FEATURE.to_owned() + "/{id}",
    tag = API_TAG_MODELS,
    params(
        ("id" = String, Path, description = "Feature identifier")
    ),
    request_body(content = UpdateFeatureRequest),
    responses(
        (status = 200, description = "Feature updated", body = FeatureResponse),
        (status = 404, description = "Feature not found", body = OpenAIApiError)
    )
)]
```

**DELETE Endpoint**:
```rust
#[utoipa::path(
    delete,
    path = ENDPOINT_NEW_FEATURE.to_owned() + "/{id}",
    tag = API_TAG_MODELS,
    params(
        ("id" = String, Path, description = "Feature identifier")
    ),
    responses(
        (status = 204, description = "Feature deleted successfully"),
        (status = 404, description = "Feature not found", body = OpenAIApiError)
    )
)]
```

### Verification Checklist

After implementing a new endpoint:

- [ ] Endpoint constant defined and used consistently
- [ ] Handler function has complete `#[utoipa::path]` annotation
- [ ] All response codes documented (200/201, 400, 401, 404, 500)
- [ ] Request/response schemas have `ToSchema` derive and examples
- [ ] Security requirements specified (`bearer_auth`, `session_auth`)
- [ ] Handler added to router configuration
- [ ] Handler and schemas added to `BodhiOpenAPIDoc` paths and components
- [ ] Tests written for success and error cases
- [ ] Contract test passes (verifies endpoint in OpenAPI spec)
- [ ] API tag constant used (not hardcoded string)
- [ ] Error handling follows project patterns
- [ ] Pagination implemented if returning lists
- [ ] Generated openapi.json includes new endpoint 