# Utoipa Integration Guide

## Overview

Utoipa is a powerful OpenAPI documentation generator for Rust REST APIs that takes a code-first approach. It provides automated OpenAPI documentation through simple macro annotations, making it easier to maintain API documentation alongside your code.

## Key Features

- OpenAPI 3.1 specification support
- Code-first approach with minimal configuration
- Automated schema generation from Rust types
- Framework agnostic with special support for Axum
- Runtime modification capabilities for OpenAPI documentation
- Built-in support for various OpenAPI visualization tools
- Automatic recursive schema collection from usage
- Support for generic types
- Seamless integration with Axum framework

## Table of Contents

1. Installation and Setup

   - Dependencies
   - Basic Configuration
   - Integration with Axum

2. Schema Generation

   - Deriving OpenAPI Schemas
   - Custom Types
   - Generic Types
   - Example Values
   - Schema Customization

3. Path Documentation

   - Route Handlers
   - Request/Response Documentation
   - Query Parameters
   - Path Parameters
   - Request Bodies
   - Response Types

4. Authentication & Security

   - Bearer Token Authentication
   - OAuth2 Integration with Keycloak
   - Session Management
   - Security Schemes
   - Securing Endpoints
   - Refresh Token Handling

5. UI Integration

   - Swagger UI Setup
   - Configuration Options
   - Custom Styling
   - Alternative UIs (RapiDoc, ReDoc)

6. Testing & Validation

   - Testing OpenAPI Specification
   - Schema Validation
   - Endpoint Testing
   - Documentation Coverage

7. Best Practices

   - API Documentation Guidelines
   - Schema Organization
   - Version Management
   - Common Patterns
   - Error Handling Documentation

8. Common Gotchas & Troubleshooting

   - Known Issues
   - Solutions
   - Performance Considerations

9. Advanced Usage

   - Custom Modifications
   - Dynamic Documentation
   - Middleware Integration
   - Complex Scenarios
   - Streaming Response Handling
   - External Resources
     \* WebSocket Integration
     \* File Upload Handling

## Installation and Setup

### Dependencies

Add the following dependencies to your `Cargo.toml`:

```toml
[dependencies]
utoipa = "5.3.1"
utoipa-swagger-ui = "8.1.0"

# Required for Axum integration
axum = "0.7"
tokio = { version = "1.0", features = ["full"] }
```

### Basic Configuration

1. Create the OpenAPI configuration:

```rust
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "My API",
        version = "1.0.0",
        description = "API documentation with utoipa",
    ),
    servers(
        (url = "http://localhost:3000", description = "Local development server"),
        (url = "https://api.production.com", description = "Production server")
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "api", description = "API endpoints")
    )
)]
struct ApiDoc;
```

### Integration with Axum

Set up the Swagger UI with Axum:

```rust
use axum::{Router, Server};
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() {
    // Create the API documentation
    let openapi = ApiDoc::openapi();

    // Create the router
    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui")
            .url("/api-docs/openapi.json", openapi)
        )
        .fallback(handler_404);

    // Start the server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("Swagger UI available at: http://localhost:3000/swagger-ui/");

    axum::serve(listener, app).await.unwrap();
}
```

## Schema Generation

### Deriving OpenAPI Schemas

Use the `#[derive(ToSchema)]` macro to generate OpenAPI schemas for your types:

```rust
use utoipa::ToSchema;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, ToSchema)]
struct User {
    #[schema(example = 1)]
    id: i32,
    #[schema(example = "john_doe")]
    username: String,
    #[schema(example = "john@example.com")]
    email: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "message": "User created successfully",
    "status": "success"
}))]
struct ApiResponse {
    message: String,
    status: String,
}
```

### Custom Types

For custom types or enums, you can provide additional schema information:

```rust
#[derive(Serialize, Deserialize, ToSchema)]
#[schema(example = "active")]
enum UserStatus {
    #[schema(rename = "active")]
    Active,
    #[schema(rename = "inactive")]
    Inactive,
    #[schema(rename = "suspended")]
    Suspended,
}

// Using custom DateTime format
#[derive(Serialize, Deserialize, ToSchema)]
struct UserDetails {
    #[schema(example = "2024-03-20T12:00:00Z")]
    created_at: chrono::DateTime<chrono::Utc>,
    #[schema(value_type = String, example = "active")]
    status: UserStatus,
}
```

### Generic Types

Handling generic types with proper documentation:

```rust
#[derive(Serialize, Deserialize, ToSchema)]
struct PagedResponse<T> {
    #[schema(example = 1)]
    page: u32,
    #[schema(example = 10)]
    per_page: u32,
    #[schema(example = 100)]
    total: u64,
    data: Vec<T>,
}

// Implementation for specific types
#[derive(OpenApi)]
#[openapi(components(schemas(PagedResponse<User>)))]
struct ApiDoc;
```

### Example Values

Adding meaningful examples to your schemas:

```rust
#[derive(Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "id": 1,
    "title": "Important Task",
    "description": "This needs to be done",
    "status": "pending",
    "due_date": "2024-12-31"
}))]
struct Task {
    id: i32,
    title: String,
    description: String,
    status: String,
    due_date: String,
}
```

### Schema Customization

Customizing schema generation with attributes:

```rust
#[derive(Serialize, Deserialize, ToSchema)]
#[schema(title = "User Creation Request",
        description = "Request payload for creating a new user")]
struct CreateUserRequest {
    #[schema(example = "john_doe",
            description = "Username must be unique and alphanumeric",
            pattern = "^[a-zA-Z0-9_]+$",
            min_length = 3,
            max_length = 30)]
    username: String,

    #[schema(example = "john@example.com",
            format = "email",
            description = "Valid email address")]
    email: String,

    #[schema(write_only = true,
            min_length = 8,
            description = "Password must be at least 8 characters")]
    password: String,
}
```

## Path Documentation

### Documenting Axum Routes

Utoipa provides comprehensive support for documenting Axum route handlers through the `#[utoipa::path]` attribute macro.

#### Basic Route Documentation

Here's a basic example of documenting an Axum route:

```rust
use axum::{Router, Json};
use utoipa::OpenApi;

#[utoipa::path(
    get,
    path = "/api/health",
    responses(
        (status = 200, description = "API is healthy", body = HealthResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "health"
)]
async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

#[derive(serde::Serialize, ToSchema)]
struct HealthResponse {
    status: String,
    version: String,
}
```

#### CRUD Operations Example

Here's a complete example of REST CRUD operations with Utoipa annotations:

```rust
#[derive(serde::Serialize, serde::Deserialize, ToSchema)]
struct Todo {
    id: Option<i32>,
    title: String,
    completed: bool,
}

// Create Todo
#[utoipa::path(
    post,
    path = "/api/todos",
    request_body = Todo,
    responses(
        (status = 201, description = "Todo created successfully", body = Todo),
        (status = 400, description = "Invalid input")
    ),
    tag = "todos"
)]
async fn create_todo(
    Json(todo): Json<Todo>,
    State(db): State<DbConnection>,
) -> Result<(StatusCode, Json<Todo>), ApiError> {
    // Implementation...
}

// Get Todo by ID
#[utoipa::path(
    get,
    path = "/api/todos/{id}",
    responses(
        (status = 200, description = "Todo found", body = Todo),
        (status = 404, description = "Todo not found")
    ),
    params(
        ("id" = i32, Path, description = "Todo identifier")
    ),
    tag = "todos"
)]
async fn get_todo(
    Path(id): Path<i32>,
    State(db): State<DbConnection>,
) -> Result<Json<Todo>, ApiError> {
    // Implementation...
}

// Update Todo
#[utoipa::path(
    put,
    path = "/api/todos/{id}",
    request_body = Todo,
    responses(
        (status = 200, description = "Todo updated", body = Todo),
        (status = 404, description = "Todo not found")
    ),
    params(
        ("id" = i32, Path, description = "Todo identifier")
    ),
    tag = "todos"
)]
async fn update_todo(
    Path(id): Path<i32>,
    Json(todo): Json<Todo>,
    State(db): State<DbConnection>,
) -> Result<Json<Todo>, ApiError> {
    // Implementation...
}
```

#### Streaming Response Example

Example of documenting a streaming endpoint:

```rust
use axum::response::sse::{Event, Sse};
use futures::stream::{Stream, StreamExt};
use std::convert::Infallible;

#[utoipa::path(
    get,
    path = "/api/events",
    responses(
        (status = 200, description = "Server-sent events stream",
         content_type = "text/event-stream",
         body = String,
         headers(
             ("X-Rate-Limit" = String, description = "Rate limit info"),
             ("X-Stream-Type" = String, description = "SSE")
         ))
    ),
    tag = "events"
)]
async fn event_stream() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = tokio_stream::iter(0..10)
        .map(|i| {
            Ok(Event::default().data(format!("data: Message {}", i)))
        });

    Sse::new(stream)
}
```

#### Error Handling

Documenting error responses and types:

```rust
#[derive(serde::Serialize, ToSchema)]
struct ApiError {
    code: String,
    message: String,
    #[schema(example = "2024-03-20T12:00:00Z")]
    timestamp: String,
}

#[utoipa::path(
    get,
    path = "/api/protected-resource",
    responses(
        (status = 200, description = "Success"),
        (status = 401, description = "Unauthorized", body = ApiError),
        (status = 403, description = "Forbidden", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    ),
    security(
        ("bearer_auth" = []),
    ),
    tag = "protected"
)]
async fn protected_resource(
    auth: AuthHeader,
) -> Result<Json<Resource>, ApiError> {
    // Implementation...
}
```

#### Query Parameters

Documenting query parameters with validation:

```rust
#[derive(serde::Deserialize, ToSchema)]
struct PaginationParams {
    #[schema(minimum = 1, default = 1, example = 1)]
    page: Option<u32>,
    #[schema(minimum = 1, maximum = 100, default = 10, example = 10)]
    per_page: Option<u32>,
    #[schema(example = "title")]
    sort_by: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/items",
    params(
        PaginationParams
    ),
    responses(
        (status = 200, description = "List of items", body = Vec<Item>)
    ),
    tag = "items"
)]
async fn list_items(
    Query(params): Query<PaginationParams>,
) -> Json<Vec<Item>> {
    // Implementation...
}
```

#### Request Body Validation

Example with request body validation:

```rust
#[derive(serde::Deserialize, ToSchema)]
struct CreateUser {
    #[schema(example = "john_doe", pattern = "^[a-zA-Z0-9_]+$",
            min_length = 3, max_length = 30)]
    username: String,
    #[schema(example = "john@example.com", format = "email")]
    email: String,
    #[schema(min_length = 8, write_only = true)]
    password: String,
}

#[utoipa::path(
    post,
    path = "/api/users",
    request_body(content = CreateUser,
                description = "User creation payload",
                content_type = "application/json"),
    responses(
        (status = 201, description = "User created", body = User),
        (status = 400, description = "Invalid input", body = ApiError),
        (status = 409, description = "Username already exists", body = ApiError)
    ),
    tag = "users"
)]
async fn create_user(
    Json(payload): Json<CreateUser>,
) -> Result<(StatusCode, Json<User>), ApiError> {
    // Implementation...
}
```

## Authentication & Security

Utoipa provides comprehensive support for documenting API security schemes and secured endpoints.

### Security Schemes

First, define your security schemes in the OpenAPI configuration:

```rust
#[derive(OpenApi)]
#[openapi(
    info(
        title = "My API",
        version = "1.0.0"
    ),
    components(
        schemas(User, ApiError),
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
            )),
            ("oauth2" = (
                type = "oauth2",
                flows(
                    password(
                        token_url = "https://auth.example.com/token",
                        scopes(
                            ("read" = "Read access"),
                            ("write" = "Write access")
                        )
                    ),
                    implicit(
                        authorization_url = "https://auth.example.com/authorize",
                        scopes(
                            ("read" = "Read access"),
                            ("write" = "Write access")
                        )
                    )
                )
            ))
        )
    )
)]
struct ApiDoc;
```

#### JWT Authentication

Example of documenting JWT-protected endpoints:

```rust
#[derive(serde::Deserialize, ToSchema)]
struct LoginCredentials {
    #[schema(example = "user@example.com")]
    email: String,
    #[schema(example = "password123", format = "password")]
    password: String,
}

#[derive(serde::Serialize, ToSchema)]
struct TokenResponse {
    #[schema(example = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...")]
    access_token: String,
    #[schema(example = "bearer")]
    token_type: String,
    #[schema(example = 3600)]
    expires_in: i32,
}

#[utoipa::path(
    post,
    path = "/auth/login",
    request_body = LoginCredentials,
    responses(
        (status = 200, description = "Login successful", body = TokenResponse),
        (status = 401, description = "Invalid credentials", body = ApiError)
    ),
    tag = "auth"
)]
async fn login(
    Json(credentials): Json<LoginCredentials>,
) -> Result<Json<TokenResponse>, ApiError> {
    // Implementation...
}

#[utoipa::path(
    get,
    path = "/auth/me",
    responses(
        (status = 200, description = "User profile retrieved", body = User),
        (status = 401, description = "Invalid or expired token", body = ApiError)
    ),
    security(
        ("jwt" = [])
    ),
    tag = "auth"
)]
async fn get_profile(
    auth: AuthHeader,
) -> Result<Json<User>, ApiError> {
    // Implementation...
}
```

#### API Key Authentication

Documenting endpoints secured with API keys:

```rust
#[utoipa::path(
    get,
    path = "/api/secure-resource",
    responses(
        (status = 200, description = "Resource retrieved"),
        (status = 401, description = "Invalid API key", body = ApiError)
    ),
    security(
        ("api_key" = [])
    ),
    tag = "secure"
)]
async fn get_secure_resource(
    TypedHeader(api_key): TypedHeader<XApiKey>,
) -> Result<Json<Resource>, ApiError> {
    // Implementation...
}
```

#### OAuth2 Authentication

Example of OAuth2-protected endpoints with scopes:

```rust
#[utoipa::path(
    post,
    path = "/api/resources",
    request_body = CreateResource,
    responses(
        (status = 201, description = "Resource created", body = Resource),
        (status = 401, description = "Unauthorized", body = ApiError),
        (status = 403, description = "Insufficient permissions", body = ApiError)
    ),
    security(
        ("oauth2" = ["write"])
    ),
    tag = "resources"
)]
async fn create_resource(
    auth: OAuth2Token,
    Json(payload): Json<CreateResource>,
) -> Result<(StatusCode, Json<Resource>), ApiError> {
    // Implementation...
}
```

#### Multiple Security Schemes

You can combine multiple security schemes for an endpoint:

```rust
#[utoipa::path(
    delete,
    path = "/api/resources/{id}",
    responses(
        (status = 204, description = "Resource deleted"),
        (status = 401, description = "Unauthorized", body = ApiError),
        (status = 403, description = "Forbidden", body = ApiError),
        (status = 404, description = "Resource not found", body = ApiError)
    ),
    params(
        ("id" = String, Path, description = "Resource identifier")
    ),
    security(
        (),
        ("jwt" = []),
        ("api_key" = []),
        ("oauth2" = ["write"])
    ),
    tag = "resources"
)]
async fn delete_resource(
    auth: AuthHeader,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    // Implementation...
}
```

#### Runtime Security Configuration

You can modify security schemes at runtime:

```rust
impl ApiDoc {
    fn with_auth_url(mut self, auth_url: &str) -> Self {
        if let Some(components) = &mut self.components {
            if let Some(security_schemes) = &mut components.security_schemes {
                if let Some(SecurityScheme::OAuth2(oauth2)) =
                    security_schemes.get_mut("oauth2") {
                        if let Some(flows) = &mut oauth2.flows {
                            if let Some(implicit) = &mut flows.implicit {
                                implicit.authorization_url = auth_url.to_string();
                            }
                        }
                }
            }
        }
        self
    }
}

// Usage
let api_doc = ApiDoc::openapi()
    .with_auth_url("https://new-auth.example.com/authorize");
```

## Testing & Validation

### Testing OpenAPI Documentation

Utoipa provides ways to validate and test your OpenAPI documentation. Here's how to test your API documentation:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn test_api_documentation() {
        let api_doc = ApiDoc::openapi();

        // Convert to JSON for validation
        let doc_json = serde_json::to_value(api_doc).unwrap();

        // Validate specific paths exist
        let paths = doc_json.get("paths").unwrap();
        assert!(paths.get("/api/health").is_some());
        assert!(paths.get("/api/todos").is_some());

        // Validate security schemes
        let components = doc_json.get("components").unwrap();
        let security_schemes = components.get("security_schemes").unwrap();
        assert!(security_schemes.get("jwt").is_some());
    }
}
```

### Schema Validation

Testing schema generation and validation:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use utoipa::OpenApi;

    #[test]
    fn test_todo_schema() {
        let api_doc = ApiDoc::openapi();
        let doc_json = serde_json::to_value(api_doc).unwrap();

        // Get the Todo schema
        let todo_schema = doc_json
            .get("components").unwrap()
            .get("schemas").unwrap()
            .get("Todo").unwrap();

        // Validate required fields
        let required = todo_schema
            .get("required").unwrap()
            .as_array().unwrap();

        assert!(required.contains(&json!("title")));
        assert!(required.contains(&json!("completed")));
    }
}
```

### Endpoint Testing

Testing endpoint documentation completeness:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use utoipa::OpenApi;

    #[test]
    fn test_todo_endpoints() {
        let api_doc = ApiDoc::openapi();
        let doc_json = serde_json::to_value(api_doc).unwrap();
        let paths = doc_json.get("paths").unwrap();

        // Test POST /todos endpoint
        let todos_post = paths
            .get("/api/todos").unwrap()
            .get("post").unwrap();

        // Verify request body
        assert!(todos_post
            .get("requestBody").unwrap()
            .get("content")
            .unwrap()
            .get("application/json")
            .is_some());

        // Verify responses
        let responses = todos_post.get("responses").unwrap();
        assert!(responses.get("201").is_some());
        assert!(responses.get("400").is_some());
    }
}
```

### Documentation Coverage

Ensuring complete documentation coverage:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use utoipa::OpenApi;

    #[test]
    fn test_documentation_coverage() {
        let api_doc = ApiDoc::openapi();

        // Verify all endpoints are documented
        let routes = [
            "/api/health",
            "/api/todos",
            "/api/todos/{id}",
            "/api/users",
            "/auth/login",
        ];

        let doc_json = serde_json::to_value(api_doc).unwrap();
        let paths = doc_json.get("paths").unwrap();

        for route in routes {
            assert!(paths.get(route).is_some(),
                   "Route {} is not documented", route);
        }

        // Verify all models are documented
        let schemas = doc_json
            .get("components").unwrap()
            .get("schemas").unwrap();

        let required_models = ["Todo", "User", "ApiError", "TokenResponse"];

        for model in required_models {
            assert!(schemas.get(model).is_some(),
                   "Model {} is not documented", model);
        }
    }
}
```

### Runtime Validation

Validating OpenAPI specification at runtime:

```rust
use utoipa::OpenApi;
use std::fs;

impl ApiDoc {
    /// Validate and save OpenAPI documentation to file
    pub fn save_and_validate(path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let api_doc = Self::openapi();

        // Convert to JSON for validation
        let doc_json = serde_json::to_string_pretty(&api_doc)?;

        // Optional: Validate against OpenAPI schema
        // You can use external crates like openapi-schema-validator

        // Save to file
        fs::write(path, doc_json)?;

        Ok(())
    }
}

// Usage in main or tests
#[tokio::main]
async fn main() {
    if let Err(e) = ApiDoc::save_and_validate("openapi.json") {
        eprintln!("Failed to validate and save API documentation: {}", e);
    }
}
```

## UI Integration

### SwaggerUI Setup

Utoipa provides built-in support for SwaggerUI through the `utoipa-swagger-ui` crate. Here's how to set it up with Axum:

```rust
use axum::{Router, Server};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

// Generate API documentation
#[derive(OpenApi)]
#[openapi(
    info(
        title = "My API",
        version = "1.0.0"
    ),
    tags(
        (name = "pets", description = "Pet management endpoints"),
        (name = "users", description = "User management endpoints")
    )
)]
struct ApiDoc;

#[tokio::main]
async fn main() {
    // Create the OpenAPI documentation
    let openapi = ApiDoc::openapi();

    // Create router with SwaggerUI
    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui")
            .url("/api-docs/openapi.json", openapi)
            // Optional: Configure SwaggerUI
            .config(|config| {
                config
                    .deep_linking(true)
                    .persist_authorization(true)
                    .default_models_expand_depth(2)
            })
        );

    // Start server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("SwaggerUI available at: http://localhost:3000/swagger-ui/");
    axum::serve(listener, app).await.unwrap();
}
```

### SwaggerUI Configuration

Customizing SwaggerUI appearance and behavior:

```rust
let app = Router::new()
    .merge(SwaggerUi::new("/swagger-ui")
        .url("/api-docs/openapi.json", openapi)
        .config(|config| {
            config
                // Display settings
                .deep_linking(true)
                .default_models_expand_depth(2)
                .default_model_expand_depth(2)
                .default_model_rendering(ModelRendering::Example)
                .doc_expansion(DocExpansion::None)
                .filter(true)
                .show_extensions(true)
                .show_common_extensions(true)

                // OAuth settings
                .oauth2_redirect_url("/swagger-ui/oauth2-redirect.html")
                .persist_authorization(true)

                // Customization
                .syntax_highlight(SyntaxHighlight::Monokai)
                .try_it_out_enabled(true)
        })
    );
```

### Multiple API Versions

Supporting multiple API versions in SwaggerUI:

```rust
let app = Router::new()
    .merge(SwaggerUi::new("/swagger-ui")
        .url("/api-docs/v1/openapi.json", ApiDocV1::openapi())
        .url("/api-docs/v2/openapi.json", ApiDocV2::openapi())
        .config(|config| {
            config
                .urls_primary_name("v2") // Set default version
                .display_request_duration(true)
        })
    );
```

### Using RapiDoc

To switch to RapiDoc, use the `utoipa-rapidoc` crate instead:

```rust
use utoipa::OpenApi;
use utoipa_rapidoc::RapiDoc;

#[tokio::main]
async fn main() {
    let openapi = ApiDoc::openapi();

    let app = Router::new()
        .merge(RapiDoc::new("/api-docs/openapi.json")
            .path("/rapidoc")
            .hide_header() // Optional: Hide the header
            .theme(RapiDocTheme::Dark) // Optional: Use dark theme
            .config(|config| {
                config
                    .show_header(true)
                    .layout(RapiDocLayout::SideNav)
                    .render_style(RenderStyle::Read)
                    .schema_style(SchemaStyle::Table)
            })
        );

    // Start server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("RapiDoc available at: http://localhost:3000/rapidoc/");
    axum::serve(listener, app).await.unwrap();
}
```

### Debug Mode Considerations

When running in debug mode, you might need to enable the `debug-embed` feature:

```toml
[dependencies]
utoipa-swagger-ui = { version = "8.1.0", features = ["debug-embed"] }
# Or for RapiDoc
utoipa-rapidoc = { version = "3.0.0", features = ["debug-embed"] }
```

This ensures the UI assets are embedded in debug builds. Alternatively, you can build in release mode:

```bash
cargo run --release
```

### Custom Styling

Customizing the UI appearance:

```rust
// For SwaggerUI
let app = Router::new()
    .merge(SwaggerUi::new("/swagger-ui")
        .url("/api-docs/openapi.json", openapi)
        .config(|config| {
            config
                .theme(SwaggerUiTheme::Material) // or .theme(SwaggerUiTheme::Custom("custom.css"))
                .custom_css(r#"
                    .swagger-ui .topbar {
                        background-color: #1b1b1b;
                    }
                    .swagger-ui .info .title {
                        color: #2d2d2d;
                    }
                "#)
        })
    );

// For RapiDoc
let app = Router::new()
    .merge(RapiDoc::new("/api-docs/openapi.json")
        .path("/rapidoc")
        .config(|config| {
            config
                .theme(RapiDocTheme::Custom)
                .css_file("/custom-rapidoc.css")
                .primary_color("#2d2d2d")
                .bg_color("#f5f5f5")
        })
    );
```

## Best Practices

### API Documentation Guidelines

Here are recommended practices for documenting your API with utoipa:

#### 1. Consistent Tag Organization

Group related endpoints under meaningful tags:

```rust
#[derive(OpenApi)]
#[openapi(
    info(
        title = "My API",
        version = "1.0.0"
    ),
    tags(
        (name = "users", description = "User management endpoints"),
        (name = "auth", description = "Authentication endpoints"),
        (name = "admin", description = "Administrative operations"),
        (name = "public", description = "Public endpoints")
    )
)]
struct ApiDoc;

#[utoipa::path(
    get,
    path = "/api/users",
    tag = "users",  // Consistent tag usage
    responses(
        (status = 200, description = "List of users", body = Vec<User>)
    )
)]
async fn list_users() -> Json<Vec<User>> {
    // Implementation...
}
```

#### 2. Schema Organization

Keep related schemas in modules and use references:

```rust
mod user {
    use utoipa::ToSchema;

    #[derive(ToSchema)]
    pub struct User {
        id: i32,
        name: String,
    }

    #[derive(ToSchema)]
    pub struct CreateUser {
        name: String,
    }

    #[derive(ToSchema)]
    pub struct UpdateUser {
        name: Option<String>,
    }
}

mod error {
    use utoipa::ToSchema;

    #[derive(ToSchema)]
    #[schema(example = json!({
        "code": "USER_NOT_FOUND",
        "message": "User not found",
        "details": null
    }))]
    pub struct ApiError {
        code: String,
        message: String,
        details: Option<serde_json::Value>,
    }
}
```

#### 3. Version Management

Implement versioning in your API documentation:

```rust
mod v1 {
    #[derive(OpenApi)]
    #[openapi(
        info(version = "1.0.0"),
        paths(
            list_users_v1,
            create_user_v1
        ),
        components(schemas(User))
    )]
    pub struct ApiDocV1;
}

mod v2 {
    #[derive(OpenApi)]
    #[openapi(
        info(version = "2.0.0"),
        paths(
            list_users_v2,
            create_user_v2
        ),
        components(schemas(UserV2))
    )]
    pub struct ApiDocV2;
}
```

#### 4. Common Response Patterns

Define reusable response patterns:

```rust
#[derive(ToSchema)]
struct PaginatedResponse<T> {
    items: Vec<T>,
    total: i64,
    page: i32,
    per_page: i32,
}

#[derive(ToSchema)]
struct SuccessResponse<T> {
    data: T,
    message: String,
}

// Usage in endpoints
#[utoipa::path(
    get,
    path = "/api/users",
    responses(
        (status = 200, description = "Users retrieved successfully",
         body = PaginatedResponse<User>),
        (status = 401, description = "Unauthorized", body = ApiError)
    )
)]
async fn list_users(
    Query(pagination): Query<PaginationParams>,
) -> Json<PaginatedResponse<User>> {
    // Implementation...
}
```

#### 5. Documentation Maintenance

Keep documentation up-to-date with code changes:

```rust
// Use constants for common descriptions
const USER_NOT_FOUND: &str = "User with specified ID was not found";
const INVALID_INPUT: &str = "The provided input data is invalid";

#[utoipa::path(
    get,
    path = "/api/users/{id}",
    responses(
        (status = 200, description = "User found", body = User),
        (status = 404, description = USER_NOT_FOUND, body = ApiError)
    ),
    params(
        ("id" = i32, Path, description = "User identifier")
    ),
    security(
        ("jwt" = [])
    )
)]
async fn get_user(Path(id): Path<i32>) -> Result<Json<User>, ApiError> {
    // Implementation...
}
```

#### 6. Error Documentation

Document errors consistently:

```rust
#[derive(ToSchema)]
enum ErrorCode {
    #[schema(rename = "NOT_FOUND")]
    NotFound,
    #[schema(rename = "UNAUTHORIZED")]
    Unauthorized,
    #[schema(rename = "BAD_REQUEST")]
    BadRequest,
}

#[derive(ToSchema)]
struct DetailedError {
    code: ErrorCode,
    message: String,
    #[schema(example = json!({
        "field": "email",
        "reason": "invalid format"
    }))]
    details: Option<serde_json::Value>,
    #[schema(example = "2024-03-20T12:00:00Z")]
    timestamp: String,
}
```

#### 7. Security Documentation

Document security requirements clearly:

```rust
#[derive(OpenApi)]
#[openapi(
    components(
        security_schemes(
            ("jwt" = (
                type = "http",
                scheme = "bearer",
                bearer_format = "JWT",
                description = "JWT token obtained from /auth/login"
            ))
        )
    ),
    modifiers(&SecurityAddon)
)]
struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = &mut openapi.components {
            if let Some(security_schemes) = &mut components.security_schemes {
                if let Some(SecurityScheme::Http(http)) =
                    security_schemes.get_mut("jwt") {
                        http.description = Some(
                            "Requires valid JWT token. Get one from /auth/login".to_string()
                        );
                }
            }
        }
    }
}
```

#### 8. Example Values

Provide meaningful examples:

```rust
#[derive(ToSchema)]
#[schema(example = json!({
    "id": 1,
    "name": "John Doe",
    "email": "john@example.com",
    "roles": ["user", "admin"],
    "settings": {
        "theme": "dark",
        "notifications": true
    }
}))]
struct UserProfile {
    id: i32,
    name: String,
    email: String,
    roles: Vec<String>,
    settings: UserSettings,
}
```

#### 9. Testing Documentation

Implement documentation tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_api_documentation() {
        let api = ApiDoc::openapi();

        // Verify all endpoints have descriptions
        for (_, path_item) in api.paths.iter() {
            for (_, operation) in path_item.operations() {
                assert!(operation.description.is_some(),
                       "All operations should have descriptions");
                assert!(operation.responses.responses().next().is_some(),
                       "All operations should have at least one response");
            }
        }

        // Verify all schemas have examples
        if let Some(schemas) = api.components.as_ref()
            .and_then(|c| c.schemas.as_ref()) {
            for (name, schema) in schemas {
                assert!(schema.example.is_some(),
                       "Schema {} should have an example", name);
            }
        }
    }
}
```

## Advanced Usage

### Dynamic Documentation

Modifying OpenAPI documentation at runtime:

```rust
use utoipa::openapi::*;

// Create a modifier for dynamic changes
struct DynamicModifier {
    environment: String,
    version: String,
}

impl utoipa::Modify for DynamicModifier {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        // Add environment-specific server
        openapi.servers = Some(vec![
            Server::new(format!("https://{}.api.example.com", self.environment))
                .description(format!("{} environment", self.environment))
        ]);

        // Update version
        if let Some(info) = &mut openapi.info {
            info.version = self.version.clone();
        }

        // Add global response
        let mut responses = Responses::new();
        responses.insert(
            "429",
            RefOr::Value(Response::new("Too Many Requests")
                .description("Rate limit exceeded")
                .content("application/json", MediaType::new(Schema::new(SchemaType::Object)))
            ),
        );

        // Add to all operations
        if let Some(paths) = &mut openapi.paths {
            for path_item in paths.paths.values_mut() {
                if let Some(operations) = path_item.operations_mut() {
                    for operation in operations.values_mut() {
                        operation.responses.responses.insert(
                            "429".to_string(),
                            responses.responses["429"].clone(),
                        );
                    }
                }
            }
        }
    }
}

// Usage
#[derive(OpenApi)]
#[openapi(modifiers(&DynamicModifier))]
struct ApiDoc;

let modifier = DynamicModifier {
    environment: std::env::var("APP_ENV").unwrap_or("dev".to_string()),
    version: env!("CARGO_PKG_VERSION").to_string(),
};

let openapi = ApiDoc::openapi().merge_modifier(&modifier);
```

### Middleware Integration

Documenting middleware and filters:

```rust
use axum::{
    middleware::{self, Next},
    response::Response,
};

// Rate limiting middleware
async fn rate_limit<B>(
    req: Request<B>,
    next: Next<B>,
) -> Response {
    // Implementation...
}

// Document rate limit headers
#[utoipa::path(
    get,
    path = "/api/resources",
    responses(
        (status = 200, description = "Success", body = Vec<Resource>,
         headers(
             ("X-RateLimit-Limit" = i32, description = "Rate limit per hour"),
             ("X-RateLimit-Remaining" = i32, description = "Remaining requests"),
             ("X-RateLimit-Reset" = i64, description = "Rate limit reset timestamp")
         ))
    )
)]
async fn list_resources() -> Json<Vec<Resource>> {
    // Implementation...
}

// Apply middleware to router
let app = Router::new()
    .route("/api/resources", get(list_resources))
    .layer(middleware::from_fn(rate_limit));
```

### Complex Response Types

Handling complex response scenarios:

```rust
use bytes::Bytes;
use futures::Stream;
use std::pin::Pin;

// Streaming response with different content types
#[utoipa::path(
    get,
    path = "/api/export/{format}",
    params(
        ("format" = String, Path, description = "Export format (csv or json)")
    ),
    responses(
        (status = 200, description = "Export stream",
         content(
             ("application/json" = Vec<Record>, description = "JSON stream"),
             ("text/csv" = String, description = "CSV stream")
         ),
         headers(
             ("Content-Disposition" = String,
              description = "Attachment filename")
         ))
    ),
    security(("jwt" = []))
)]
async fn export_data(
    format: Path<String>,
) -> impl Stream<Item = Result<Bytes, std::io::Error>> {
    // Implementation...
}
```

### External Type Integration

Documenting external types that you can't modify:

```rust
// External type from another crate
use external_crate::ExternalType;

// Implement ToSchema for external type
#[derive(ToSchema)]
#[schema(from_type = ExternalType)]
struct ExternalTypeSchema {
    #[schema(example = 123)]
    id: i32,
    #[schema(example = "external")]
    name: String,
}

// Use in endpoint
#[utoipa::path(
    get,
    path = "/api/external",
    responses(
        (status = 200, description = "External data",
         body = ExternalTypeSchema)
    )
)]
async fn get_external() -> Json<ExternalType> {
    // Implementation...
}
```

### File Operations

Handling file uploads and downloads:

```rust
use axum::{
    extract::Multipart,
    response::IntoResponse,
};

// File upload
#[utoipa::path(
    post,
    path = "/api/upload",
    request_body(
        content = Binary,
        description = "File to upload",
        content_type = "multipart/form-data"
    ),
    responses(
        (status = 200, description = "File uploaded successfully",
         body = FileUploadResponse),
        (status = 413, description = "File too large")
    )
)]
async fn upload_file(
    multipart: Multipart,
) -> Result<Json<FileUploadResponse>, ApiError> {
    // Implementation...
}

// File download
#[utoipa::path(
    get,
    path = "/api/download/{id}",
    params(
        ("id" = String, Path, description = "File identifier")
    ),
    responses(
        (status = 200, description = "File download",
         content_type = "application/octet-stream",
         headers(
             ("Content-Disposition" = String,
              description = "Attachment filename"),
             ("Content-Length" = i64,
              description = "File size in bytes")
         )),
        (status = 404, description = "File not found")
    )
)]
async fn download_file(
    id: Path<String>,
) -> impl IntoResponse {
    // Implementation...
}
```

### Runtime Schema Modification

Modifying schemas at runtime:

```rust
use utoipa::openapi::*;

struct SchemaModifier;

impl utoipa::Modify for SchemaModifier {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = &mut openapi.components {
            if let Some(schemas) = &mut components.schemas {
                // Add new properties to existing schema
                if let Some(Schema::Object(obj)) = schemas.get_mut("User") {
                    obj.properties.insert(
                        "created_at".to_string(),
                        Schema::new(SchemaType::String)
                            .format(Some(SchemaFormat::DateTime))
                            .description(Some("User creation timestamp"))
                    );
                }

                // Add new schema
                schemas.insert(
                    "AuditLog".to_string(),
                    Schema::new(SchemaType::Object)
                        .property("action", Schema::new(SchemaType::String))
                        .property("timestamp",
                            Schema::new(SchemaType::String)
                                .format(Some(SchemaFormat::DateTime))
                        )
                        .required("action")
                        .required("timestamp")
                );
            }
        }
    }
}

// Usage
let openapi = ApiDoc::openapi().merge_modifier(&SchemaModifier);
```

### Multiple UI Options

Utoipa supports multiple UI options that can be used simultaneously:

```rust
use utoipa_swagger_ui::SwaggerUi;
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::Redoc;
use utoipa_scalar::Scalar;

let router = router
    // Swagger UI
    .merge(SwaggerUi::new("/swagger-ui")
        .url("/api-docs/openapi.json", api.clone()))
    // ReDoc UI
    .merge(Redoc::with_url("/redoc", api.clone()))
    // RapiDoc UI
    .merge(RapiDoc::new("/api-docs/openapi.json")
        .path("/rapidoc"))
    // Scalar UI
    .merge(Scalar::with_url("/scalar", api));
```

### Nested API Documentation

You can nest multiple OpenAPI specifications:

```rust
#[derive(OpenApi)]
#[openapi(
    nest(
        (path = "/api/v1/ones", api = one::OneApi)
    )
)]
struct ApiDoc;

// You can also nest programmatically
let mut doc = ApiDoc::openapi();
doc = doc.nest("/hello", hello_api);
```

### OpenApiRouter Integration

Using OpenApiRouter for automatic route documentation:

```rust
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

// Create router with OpenAPI documentation
let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
    .routes(routes!(health))
    .nest("/api/customer", customer::router())
    .nest("/api/order", order::router())
    .split_for_parts();

// Add Swagger UI
let router = router.merge(
    SwaggerUi::new("/swagger-ui")
        .url("/apidoc/openapi.json", api)
);
```

### Multipart Form Handling

Documenting multipart form endpoints:

```rust
/// Schema for multipart form
#[derive(Deserialize, ToSchema)]
struct HelloForm {
    name: String,
    #[schema(format = Binary, content_media_type = "application/octet-stream")]
    file: String,
}

#[utoipa::path(
    post,
    path = "/hello",
    request_body(
        content = HelloForm,
        content_type = "multipart/form-data"
    )
)]
async fn hello_form(mut multipart: Multipart) -> String {
    // Implementation...
}
```

### In-Memory Store Example

Example of documenting a complete CRUD API with in-memory storage:

```rust
/// In-memory todo store
type Store = Mutex<Vec<Todo>>;

#[derive(Serialize, Deserialize, ToSchema, Clone)]
struct Todo {
    id: i32,
    #[schema(example = "Buy groceries")]
    value: String,
    done: bool,
}

#[derive(Serialize, Deserialize, ToSchema)]
enum TodoError {
    #[schema(example = "Todo already exists")]
    Conflict(String),
    #[schema(example = "id = 1")]
    NotFound(String),
    #[schema(example = "missing api key")]
    Unauthorized(String),
}

/// List all Todo items
#[utoipa::path(
    get,
    path = "",
    tag = TODO_TAG,
    responses(
        (status = 200, description = "List all todos successfully",
         body = [Todo])
    )
)]
async fn list_todos(
    State(store): State<Arc<Store>>
) -> Json<Vec<Todo>> {
    let todos = store.lock().await.clone();
    Json(todos)
}

/// Create new Todo
#[utoipa::path(
    post,
    path = "",
    tag = TODO_TAG,
    responses(
        (status = 201, description = "Todo created successfully",
         body = Todo),
        (status = 409, description = "Todo already exists",
         body = TodoError)
    )
)]
async fn create_todo(
    State(store): State<Arc<Store>>,
    Json(todo): Json<Todo>,
) -> impl IntoResponse {
    // Implementation...
}
```
