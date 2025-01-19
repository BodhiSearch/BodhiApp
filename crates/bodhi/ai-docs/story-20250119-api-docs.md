# API Documentation Feature

## Requirements

### User Story
As a Developer using Bodhi App
I want to have comprehensive API documentation with an interactive UI
So that I can understand and test the API endpoints easily

### Core Requirements

#### OpenAPI Documentation
- Generate OpenAPI 3.1 documentation using Utoipa
- Document all public and authenticated API endpoints
- Include request/response schemas
- Document authentication requirements
- Include example values for better understanding
- Support both session and token-based auth in documentation
- Document error responses
- Include rate limiting information where applicable

#### Swagger UI Integration
- Provide interactive Swagger UI at `/swagger-ui` endpoint
- Support "Try it out" functionality for all endpoints
- Allow authentication via Bearer token
- Show proper error responses
- Support dark/light theme
- Mobile-responsive interface
- Clear grouping of endpoints by functionality
- Support for file upload operations

#### Documentation Organization
1. Endpoint Groups
   - Authentication endpoints
   - User management
   - Model/Alias management
   - Chat operations
   - Token management
   - System operations

2. Schema Documentation
   - Request bodies
   - Response structures
   - Error types
   - Common objects
   - Authentication objects

#### Security Documentation
- Document authentication methods
- Show required scopes for endpoints
- Document role-based access requirements
- Include security scheme definitions
- Show proper error responses for auth failures
- Document token requirements

### Not In Scope
- Custom documentation UI
- API versioning
- Documentation versioning
- API changelog
- Custom documentation styling
- Multiple documentation formats
- Documentation export
- API client generation

## Implementation Tasks

### Backend Tasks

#### Utoipa Integration
- [ ] Add Utoipa dependencies
- [ ] Configure OpenAPI information
- [ ] Set up security schemes
- [ ] Configure servers section
- [ ] Add common components

#### Schema Documentation
- [ ] Document common request types
- [ ] Document response types
- [ ] Document error types
- [ ] Add example values
- [ ] Document authentication types

#### Endpoint Documentation
- [ ] Document auth endpoints
- [ ] Document user endpoints
- [ ] Document model endpoints
- [ ] Document chat endpoints
- [ ] Document token endpoints
- [ ] Document system endpoints

#### Swagger UI Setup
- [ ] Add Swagger UI dependencies
- [ ] Configure UI options
- [ ] Set up authentication
- [ ] Configure theme
- [ ] Add custom configuration

### Testing Tasks
- [ ] Verify OpenAPI spec generation
- [ ] Test Swagger UI integration
- [ ] Validate schema documentation
- [ ] Test authentication in Swagger UI
- [ ] Verify example values
- [ ] Test API operations through UI

## File Overview

### Core Implementation
- `crates/routes_app/src/openapi.rs`: OpenAPI configuration and setup
- `crates/routes_app/src/lib.rs`: Route documentation and schemas
- `crates/objs/src/lib.rs`: Common type definitions and schemas

### Route Documentation
- `crates/routes_app/src/routes_auth.rs`: Authentication endpoint docs
- `crates/routes_app/src/routes_user.rs`: User management docs
- `crates/routes_app/src/routes_model.rs`: Model operation docs
- `crates/routes_app/src/routes_chat.rs`: Chat endpoint docs
- `crates/routes_app/src/routes_token.rs`: Token management docs
- `crates/routes_app/src/routes_system.rs`: System operation docs

### Schema Documentation
- `crates/objs/src/auth.rs`: Authentication type schemas
- `crates/objs/src/user.rs`: User related schemas
- `crates/objs/src/model.rs`: Model related schemas
- `crates/objs/src/chat.rs`: Chat related schemas
- `crates/objs/src/token.rs`: Token related schemas
- `crates/objs/src/error.rs`: Error type schemas

## Technical Details

### OpenAPI Configuration

```rust
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Bodhi API",
        version = "1.0.0",
        description = "API documentation for Bodhi application",
        contact(
            name = "Bodhi Support",
            url = "https://github.com/BodhiSearch/BodhiApp"
        ),
    ),
    servers(
        (url = "http://localhost:1135", description = "Local development"),
        (url = "https://cloud.getbodhi.app", description = "Cloud hosted")
    ),
    tags(
        (name = "auth", description = "Authentication endpoints"),
        (name = "users", description = "User management"),
        (name = "models", description = "Model operations"),
        (name = "chat", description = "Chat operations"),
        (name = "tokens", description = "Token management"),
        (name = "system", description = "System operations")
    ),
    components(
        schemas(User, Model, Chat, Token, Error),
        responses(
            (name = "UnauthorizedError", description = "Unauthorized"),
            (name = "ValidationError", description = "Validation failed"),
            (name = "NotFoundError", description = "Resource not found")
        ),
        security_schemes(
            ("jwt" = (
                type = "http",
                scheme = "bearer",
                bearer_format = "JWT"
            )),
            ("api_key" = (
                type = "apiKey",
                name = "X-API-KEY",
                in = "header"
            ))
        )
    )
)]
struct ApiDoc;
```

### Security Schemes

```rust
// JWT Authentication
#[derive(SecurityScheme)]
#[oai(
    type = "http",
    scheme = "bearer",
    bearer_format = "JWT"
)]
struct JwtAuth;

// API Key Authentication
#[derive(SecurityScheme)]
#[oai(
    type = "apiKey",
    in = "header",
    name = "X-API-KEY"
)]
struct ApiKeyAuth;
```

### Example Endpoint Documentation

```rust
/// Create a new chat completion
#[utoipa::path(
    post,
    path = "/v1/chat/completions",
    request_body = ChatCompletionRequest,
    responses(
        (status = 200, description = "Chat completion successful",
         body = ChatCompletionResponse),
        (status = 400, description = "Invalid request",
         body = ErrorResponse),
        (status = 401, description = "Unauthorized",
         body = ErrorResponse),
        (status = 429, description = "Too many requests",
         body = ErrorResponse)
    ),
    security(
        ("jwt" = []),
        ("api_key" = [])
    ),
    tag = "chat"
)]
async fn create_chat_completion(
    State(state): State<AppState>,
    Json(request): Json<ChatCompletionRequest>,
) -> Result<Json<ChatCompletionResponse>, ApiError> {
    // Implementation...
}
```

## Migration Plan

1. Initial Setup
   - Add Utoipa dependencies
   - Configure basic OpenAPI information
   - Set up security schemes

2. Schema Documentation
   - Document common types
   - Add example values
   - Document error responses

3. Endpoint Documentation
   - Document endpoints by group
   - Add authentication requirements
   - Include example requests/responses

4. Swagger UI Integration
   - Set up Swagger UI
   - Configure authentication
   - Test interactive features

5. Testing & Validation
   - Verify documentation accuracy
   - Test through Swagger UI
   - Fix any issues

6. Production Deployment
   - Deploy to staging
   - Verify functionality
   - Deploy to production

## Success Criteria
1. All API endpoints are properly documented
2. Swagger UI is accessible and functional
3. Authentication works in Swagger UI
4. Example values are provided for all schemas
5. Documentation is accurate and up-to-date
6. Interactive testing works for all endpoints

## Monitoring & Maintenance
1. Regular validation of OpenAPI spec
2. Update documentation with API changes
3. Monitor Swagger UI usage
4. Keep example values current
5. Update security documentation as needed

## Future Enhancements
1. API versioning support
2. Multiple documentation formats
3. Custom styling for Swagger UI
4. API client generation
5. Documentation search functionality
6. Interactive tutorials
