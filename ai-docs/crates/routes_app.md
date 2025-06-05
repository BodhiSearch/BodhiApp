# routes_app - Application-Specific API Routes

## Overview

The `routes_app` crate implements application-specific HTTP API endpoints for BodhiApp's web interface and administrative functions. Unlike the OpenAI-compatible routes, these endpoints provide BodhiApp-specific functionality for model management, user administration, and application configuration.

## Purpose

- **Application API**: BodhiApp-specific API endpoints for web interface
- **Model Management**: Advanced model operations beyond OpenAI compatibility
- **User Administration**: User management and authentication flows
- **System Configuration**: Application setup and configuration endpoints
- **Development Tools**: Development and debugging endpoints

## Key Route Modules

### Authentication and User Management

#### Login Routes (`routes_login.rs`)
- **POST /api/login**: User authentication endpoint
- **POST /api/logout**: User logout and session termination
- **GET /api/auth/status**: Current authentication status
- **OAuth2 Integration**: OAuth2 authentication flows
- **Session Management**: User session creation and validation

#### API Token Management (`routes_api_token.rs`)
- **GET /api/tokens**: List user's API tokens
- **POST /api/tokens**: Create new API token
- **PUT /api/tokens/{id}**: Update API token
- **DELETE /api/tokens/{id}**: Revoke API token
- **Token Scopes**: Manage token permissions and scopes

### Model Management

#### Model Routes (`routes_models.rs`)
- **GET /api/models**: List available models with detailed metadata
- **GET /api/models/{id}**: Get specific model information
- **POST /api/models/alias**: Create model alias
- **PUT /api/models/alias/{name}**: Update model alias
- **DELETE /api/models/alias/{name}**: Delete model alias
- **Model Validation**: Validate model configurations

#### Pull Routes (`routes_pull.rs`)
- **POST /api/pull**: Initiate model download
- **GET /api/pull/status**: Get download status
- **DELETE /api/pull/{id}**: Cancel download
- **Progress Tracking**: Real-time download progress via SSE
- **Resume Support**: Resume interrupted downloads

### System Configuration

#### Setup Routes (`routes_setup.rs`)
- **GET /api/setup/status**: Application setup status
- **POST /api/setup/init**: Initialize application
- **PUT /api/setup/config**: Update configuration
- **GET /api/setup/requirements**: System requirements check
- **First-time Setup**: Guided setup process

#### Settings Routes (`routes_settings.rs`)
- **GET /api/settings**: Get application settings
- **PUT /api/settings**: Update application settings
- **GET /api/settings/user**: Get user preferences
- **PUT /api/settings/user**: Update user preferences
- **Configuration Validation**: Settings validation and defaults

### Development and Utilities

#### Development Routes (`routes_dev.rs`)
- **GET /api/dev/info**: Development information
- **POST /api/dev/reset**: Reset development state
- **GET /api/dev/logs**: Access application logs
- **Debug Endpoints**: Development and debugging utilities

#### Create Routes (`routes_create.rs`)
- **POST /api/create/workspace**: Create new workspace
- **POST /api/create/project**: Create new project
- **Resource Creation**: Various resource creation endpoints

#### UI Routes (`routes_ui.rs`)
- **GET /api/ui/config**: UI configuration
- **GET /api/ui/theme**: Theme configuration
- **Static Assets**: UI asset serving
- **Frontend Configuration**: Dynamic frontend configuration

## Directory Structure

```
src/
├── lib.rs                    # Main module exports and route registration
├── error.rs                  # App-specific error types
├── objs.rs                   # Request/response objects
├── openapi.rs                # OpenAPI documentation
├── openapi/                  # OpenAPI specification files
├── routes_login.rs           # Authentication endpoints
├── routes_api_token.rs       # API token management
├── routes_models.rs          # Model management endpoints
├── routes_pull.rs            # Model download endpoints
├── routes_setup.rs           # Application setup endpoints
├── routes_settings.rs        # Configuration endpoints
├── routes_dev.rs             # Development endpoints
├── routes_create.rs          # Resource creation endpoints
├── routes_ui.rs              # UI configuration endpoints
├── bin/                      # Binary utilities
├── resources/                # Localization resources
│   └── en-US/
└── test_utils/               # Testing utilities
    ├── mod.rs
    └── alias_response.rs
```

## Key Features

### Comprehensive Model Management
- **Advanced Operations**: Beyond basic OpenAI compatibility
- **Metadata Management**: Rich model metadata and tagging
- **Alias System**: User-friendly model naming
- **Download Management**: Robust download system with progress tracking

### User Administration
- **Role-Based Access**: Admin, PowerUser, BasicUser roles
- **Token Management**: Fine-grained API token control
- **Session Handling**: Secure session management
- **OAuth2 Integration**: External authentication provider support

### Real-Time Updates
- **Server-Sent Events**: Real-time progress updates
- **Status Monitoring**: Live system status updates
- **Download Progress**: Real-time download progress
- **System Notifications**: Live system notifications

### Configuration Management
- **Dynamic Configuration**: Runtime configuration updates
- **Validation**: Comprehensive configuration validation
- **Defaults**: Intelligent default settings
- **Environment Support**: Environment-specific configurations

## API Examples

### Model Management
```bash
# List models with metadata
GET /api/models

# Create model alias
POST /api/models/alias
{
  "name": "chat-model",
  "model_id": "microsoft/DialoGPT-medium",
  "description": "General chat model"
}

# Start model download
POST /api/pull
{
  "model": "microsoft/DialoGPT-medium",
  "version": "main"
}
```

### Authentication
```bash
# Login
POST /api/login
{
  "username": "user@example.com",
  "password": "password"
}

# Create API token
POST /api/tokens
{
  "name": "My API Token",
  "scopes": ["chat", "models"]
}
```

### Configuration
```bash
# Get application settings
GET /api/settings

# Update settings
PUT /api/settings
{
  "max_concurrent_downloads": 3,
  "default_model": "chat-model"
}
```

## Dependencies

### Core Dependencies
- **objs**: Domain objects and types
- **services**: Business logic services
- **server_core**: HTTP server infrastructure
- **auth_middleware**: Authentication and authorization

### API Documentation
- **utoipa**: OpenAPI documentation generation
- **utoipa-swagger-ui**: Swagger UI integration

### HTTP and Validation
- **axum**: HTTP framework
- **serde**: JSON serialization
- **validator**: Input validation

## OpenAPI Documentation

The routes_app crate generates comprehensive OpenAPI documentation:

### Documentation Features
- **Automatic Generation**: Generated from code annotations
- **Interactive UI**: Swagger UI for API exploration
- **Type Safety**: TypeScript types generated from schemas
- **Examples**: Request/response examples for all endpoints

### Documentation Structure
```rust
#[utoipa::path(
    post,
    path = "/api/models/alias",
    request_body = CreateAliasRequest,
    responses(
        (status = 200, description = "Alias created", body = AliasResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse)
    )
)]
async fn create_alias(/* ... */) -> Result<Json<AliasResponse>, ApiError> {
    // Implementation
}
```

## Error Handling

### Application-Specific Errors
- **Validation Errors**: Input validation failures
- **Business Logic Errors**: Domain-specific errors
- **Resource Errors**: Resource not found, conflicts
- **Permission Errors**: Authorization failures

### Error Response Format
```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid model configuration",
    "details": {
      "field": "model_id",
      "reason": "Model not found"
    }
  }
}
```

## Testing Support

The routes_app crate includes comprehensive testing:
- **Integration Tests**: Full API endpoint testing
- **Mock Services**: Business logic mocking
- **Authentication Tests**: Auth flow testing
- **Validation Tests**: Input validation testing

## Security Considerations

### Authentication Required
- Most endpoints require authentication
- Role-based access control
- API token validation
- Session security

### Input Validation
- Comprehensive input validation
- SQL injection prevention
- XSS protection
- CSRF protection

### Rate Limiting
- Per-user rate limiting
- API token rate limiting
- Resource-specific limits
- Abuse prevention

## Integration Points

- **Frontend**: Primary API for web interface
- **Services Layer**: Uses all business logic services
- **Authentication**: Integrates with auth middleware
- **Database**: Manages persistent application state
- **File System**: Handles model file operations

## Performance Considerations

### Caching
- **Response Caching**: Cacheable endpoint responses
- **Model Metadata**: Cached model information
- **Configuration**: Cached settings and preferences

### Async Operations
- **Non-blocking**: All operations are async
- **Streaming**: Large responses use streaming
- **Background Tasks**: Long operations run in background

## Future Extensions

The routes_app crate is designed to support:
- **Workspace Management**: Multi-workspace support
- **Advanced Analytics**: Usage analytics and metrics
- **Plugin System**: Extensible plugin architecture
- **Advanced Monitoring**: System monitoring and alerting
- **Backup/Restore**: Data backup and restore functionality
