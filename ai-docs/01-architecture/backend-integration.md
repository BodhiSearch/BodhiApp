# Backend Integration & State Management

This document details the backend service architecture, data flow patterns, and integration strategies for the Bodhi App. For frontend API query patterns, see [Frontend Query Architecture](frontend-query.md).

## Backend Service Architecture

The Bodhi backend uses a layered service architecture:

```
┌─────────────────────────────────────────────────────────┐
│                 HTTP Layer                              │
│  ├── REST API Endpoints (/bodhi/v1/*)                   │
│  ├── OpenAI-compatible API (/v1/*)                      │
│  ├── Authentication Endpoints (/app/*)                  │
│  └── Static File Serving                                │
├─────────────────────────────────────────────────────────┤
│                Service Layer                            │
│  ├── Application Services                               │
│  ├── Model Management Services                          │
│  ├── Authentication Services                            │
│  └── Settings Management                                │
├─────────────────────────────────────────────────────────┤
│                Data Layer                               │
│  ├── Database Services (SQLite)                         │
│  ├── File System Operations                             │
│  ├── External API Integration                           │
│  └── LLM Process Management                             │
└─────────────────────────────────────────────────────────┘
```

## API Endpoint Structure

### Endpoint Categories

**Application APIs** (`/bodhi/v1/*`):
- `/bodhi/v1/info` - Application information
- `/bodhi/v1/setup` - Initial application setup
- `/bodhi/v1/user` - User information and authentication
- `/bodhi/v1/models` - Model management (CRUD operations)
- `/bodhi/v1/modelfiles` - Available model files
- `/bodhi/v1/tokens` - API token management
- `/bodhi/v1/settings` - Application settings

**OpenAI-compatible APIs** (`/v1/*`):
- `/v1/chat/completions` - Chat completion endpoint
- `/v1/models` - List available models (OpenAI format)

**Authentication APIs** (`/app/*`):
- `/app/login` - OAuth login initiation
- `/app/login/callback` - OAuth callback handling

### Response Format Standards

All API responses follow consistent patterns:

```json
// Success Response
{
  "data": [...],
  "total": 100,
  "page": 1,
  "page_size": 20
}

// Error Response
{
  "error": {
    "message": "User-friendly error message",
    "type": "validation_error",
    "code": "INVALID_INPUT",
    "param": "field_name"
  }
}
```

## Authentication & Authorization

### OAuth 2.0 Integration

The backend implements OAuth 2.0 Authorization Code flow with PKCE:

1. **Login Initiation** (`/app/login`):
   - Generates PKCE challenge
   - Redirects to OAuth provider
   - Maintains state for security

2. **Callback Handling** (`/app/login/callback`):
   - Validates state parameter
   - Exchanges authorization code for tokens
   - Establishes user session

3. **Session Management**:
   - Cookie-based sessions for web clients
   - Token-based authentication for API clients
   - Automatic token refresh handling

### API Token Management

**Token Types**:
- **Session Tokens**: Short-lived, cookie-based
- **API Tokens**: Long-lived, for programmatic access
- **Refresh Tokens**: For token renewal

**Token Scopes**:
- `read`: Read-only access to resources
- `write`: Full CRUD operations
- `admin`: Administrative functions

## Service Layer Architecture

### Database Services

**Location**: `crates/services/src/db/service.rs`

```rust
#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait DbService: std::fmt::Debug + Send + Sync {
    async fn create_conversation(&self, conversation: &Conversation) -> Result<(), DbError>;
    async fn get_conversation(&self, id: &str) -> Result<Option<Conversation>, DbError>;
    async fn list_conversations(&self, page: i64, page_size: i64) -> Result<(Vec<Conversation>, i64), DbError>;
    async fn update_conversation(&self, conversation: &Conversation) -> Result<(), DbError>;
    async fn delete_conversation(&self, id: &str) -> Result<(), DbError>;
}
```

**Service Patterns**:
- Trait-based design for testability
- Async/await throughout
- Consistent error handling
- Pagination support
- Transaction management

### Model Management Services

**Model Operations**:
- **Discovery**: Scan for available model files
- **Registration**: Create model aliases with configurations
- **Validation**: Verify model compatibility
- **Lifecycle**: Download, update, and removal

**Configuration Management**:
- **Request Parameters**: OpenAI-compatible settings
- **Context Parameters**: LLM-specific configurations
- **Template Management**: Chat templates and prompts

### Settings Service

**Setting Categories**:
- **System Settings**: Core application configuration
- **User Preferences**: Per-user customizations
- **Model Defaults**: Default parameters for models
- **Feature Flags**: Experimental feature toggles

**Setting Sources** (priority order):
1. Command line arguments
2. Environment variables
3. Configuration files
4. Database settings
5. Default values

## Real-time Communication

### Server-Sent Events (SSE)

**Chat Streaming Implementation**:
- Chunked response processing
- Real-time content delivery
- Metadata extraction
- Error handling during streaming
- Connection management

**SSE Message Format**:
```
data: {"choices":[{"delta":{"content":"Hello"}}]}

data: {"choices":[{"delta":{"content":" world"}}]}

data: {"choices":[{"finish_reason":"stop","usage":{"total_tokens":10}}]}

data: [DONE]
```

### WebSocket Considerations

While currently using SSE for chat streaming, the architecture supports WebSocket integration for:
- Bidirectional communication
- Real-time notifications
- Collaborative features
- System status updates

## Data Flow Architecture

### Backend Request Processing

```
HTTP Request
    ↓ (routing)
Route Handler
    ↓ (authentication)
Auth Middleware
    ↓ (business logic)
Service Layer
    ↓ (data operations)
Database/File System
    ↓ (response)
JSON/Stream Response
```

### Service Integration Flow

```
Frontend Request
    ↓ (HTTP/WebSocket)
API Gateway/Router
    ↓ (service routing)
Application Services
    ↓ (data access)
Database Services
    ↓ (external calls)
LLM Process/External APIs
    ↓ (aggregated response)
Formatted API Response
```

## Error Handling & Monitoring

### Error Response Standards

**Structured Error Format**:
```json
{
  "error": {
    "message": "User-friendly error description",
    "type": "validation_error|authentication_error|server_error",
    "code": "SPECIFIC_ERROR_CODE",
    "param": "field_name",
    "details": {
      "additional": "context"
    }
  }
}
```

**HTTP Status Code Mapping**:
- `200`: Success
- `201`: Created
- `400`: Bad Request (validation errors)
- `401`: Unauthorized (authentication required)
- `403`: Forbidden (insufficient permissions)
- `404`: Not Found
- `409`: Conflict (resource already exists)
- `422`: Unprocessable Entity (business logic errors)
- `500`: Internal Server Error

### Logging & Observability

**Structured Logging**:
- Request/response logging
- Performance metrics
- Error tracking
- User action auditing

**Log Levels**:
- `ERROR`: System errors requiring attention
- `WARN`: Recoverable issues
- `INFO`: General application flow
- `DEBUG`: Detailed debugging information
- `TRACE`: Fine-grained execution details

## Performance & Scalability

### Database Optimization

**Query Optimization**:
- Indexed columns for frequent queries
- Pagination for large result sets
- Connection pooling
- Query result caching

**Schema Design**:
- Normalized data structure
- Efficient foreign key relationships
- Optimized for read-heavy workloads
- Migration-friendly schema evolution

### Caching Strategies

**Application-Level Caching**:
- In-memory caching for frequently accessed data
- Model metadata caching
- Settings caching with invalidation
- User session caching

**HTTP Caching**:
- Static asset caching
- API response caching headers
- ETags for conditional requests
- Cache invalidation strategies

### Resource Management

**Memory Management**:
- Efficient data structures
- Garbage collection optimization
- Memory leak prevention
- Resource cleanup

**Process Management**:
- LLM process lifecycle management
- Resource allocation and limits
- Process monitoring and restart
- Graceful shutdown handling

## Integration Testing

### API Testing Strategies

**Unit Testing**:
- Service layer testing with mocked dependencies
- Database operation testing
- Business logic validation
- Error handling verification

**Integration Testing**:
- End-to-end API testing
- Database integration testing
- External service integration
- Authentication flow testing

**Performance Testing**:
- Load testing for high-traffic scenarios
- Memory usage profiling
- Database query performance
- Streaming response testing

### Test Infrastructure

**Test Database**:
- Isolated test environment
- Fresh data for each test
- Transaction rollback for cleanup
- Realistic test data generation

**Mock Services**:
- External API mocking
- LLM process simulation
- File system mocking
- Network condition simulation

## Security Considerations

### Data Protection

**Sensitive Data Handling**:
- API token encryption
- User data anonymization
- Secure session management
- PII data protection

**Input Validation**:
- Request parameter validation
- SQL injection prevention
- XSS protection
- File upload security

### Access Control

**Authorization Patterns**:
- Role-based access control (RBAC)
- Resource-level permissions
- API endpoint protection
- Admin function restrictions

**Audit Logging**:
- User action tracking
- Administrative operation logging
- Security event monitoring
- Compliance reporting

## Related Documentation

- **[Frontend Query Architecture](frontend-query.md)** - Frontend API integration patterns
- **[Authentication](authentication.md)** - Security implementation details
- **[Frontend Architecture](frontend-architecture.md)** - React component architecture
- **[App Overview](app-overview.md)** - High-level system architecture

## Future Improvements

1. **Service Architecture**
   - Microservice decomposition
   - Service mesh implementation
   - API versioning strategy
   - Service discovery

2. **Performance**
   - Horizontal scaling capabilities
   - Database sharding
   - CDN integration
   - Response compression

3. **Monitoring**
   - Real-time metrics dashboard
   - Alerting system
   - Performance analytics
   - Health check endpoints

4. **Security**
   - Advanced threat detection
   - Rate limiting implementation
   - API security scanning
   - Compliance automation
