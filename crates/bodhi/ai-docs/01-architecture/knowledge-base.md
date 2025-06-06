# Bodhi App Knowledge Base

## Architecture Overview

### Core Components
1. **Routes Layer**
   - Handles HTTP endpoints
   - Implements OpenAPI documentation
   - Manages authentication middleware
   - Routes grouped by functionality (auth, models, chat, etc.)

2. **Services Layer**
   - Business logic implementation
   - Database interactions
   - External service integrations (auth server, HuggingFace)

3. **Objects Layer**
   - Domain models and types
   - Shared data structures
   - Error types and handling

## Authentication System

### Setup Modes
1. **Authenticated Mode** (`authz: true`)
   ```mermaid
   sequenceDiagram
       participant User
       participant App
       participant AuthServer
       
       User->>App: POST /setup {authz: true}
       App->>AuthServer: Register as resource server
       AuthServer->>App: Client credentials
       App->>App: Store credentials
       App->>App: Set status: resource-admin
       App->>User: Redirect to admin setup
   ```

2. **Non-Authenticated Mode** (`authz: false`)
   ```mermaid
   sequenceDiagram
       participant User
       participant App
       
       User->>App: POST /setup {authz: false}
       App->>App: Set status: ready
       App->>User: Ready for use
   ```

### Application States
1. **Setup** (`setup`)
   - Initial state
   - Requires choosing auth mode
   - No API access except setup endpoints

2. **Resource Admin** (`resource-admin`)
   - Intermediate state for authenticated mode
   - Waiting for first admin user
   - Limited API access

3. **Ready** (`ready`)
   - Fully operational state
   - All APIs accessible
   - Authentication enforced if enabled

### Token System
1. **Session Tokens**
   - Used for web UI authentication
   - Short-lived with refresh capability
   - Stored in session cookie

2. **API Tokens**
   - Long-lived offline tokens
   - Used for programmatic access
   - Can be named and managed
   - Status tracking (active/inactive)

## Model Management

### Model Files
1. **Storage**
   - GGUF format models
   - Stored in local HuggingFace cache
   - Tracked in database

2. **Download Process**
   ```mermaid
   sequenceDiagram
       participant Client
       participant App
       participant HF
       
       Client->>App: POST /modelfiles/pull
       App->>App: Create download request
       App->>HF: Pull model file
       HF->>App: Download progress
       App->>App: Update status
       Client->>App: GET /modelfiles/pull/status/{id}
   ```

### Model Aliases
1. **Structure**
   ```json
   {
     "alias": "llama2:chat",
     "repo": "TheBloke/Llama-2-7B-Chat-GGUF",
     "filename": "llama-2-7b-chat.Q4_K_M.gguf",
     "source": "huggingface",
     "chat_template": "llama2",
     "model_params": {},
     "request_params": {
       "temperature": 0.7,
       "top_p": 0.95
     },
     "context_params": {
       "max_tokens": 4096
     }
   }
   ```

2. **Chat Templates**
   - Built-in templates (llama2, mistral, etc.)
   - Custom templates from repos
   - Embedded templates in models

## API Compatibility

### OpenAI Compatibility
- `/v1/models` - List available models
- `/v1/chat/completions` - Chat completion API
- Compatible with OpenAI client libraries

### Ollama Compatibility
- `/api/tags` - List model tags
- `/api/show` - Model information
- `/api/chat` - Chat completion
- Drop-in replacement for Ollama clients

## Common Patterns

### Pagination
All list endpoints support:
- `page` - Page number (1-based)
- `page_size` - Items per page (max 100)
- `sort` - Sort field
- `sort_order` - asc/desc

Example response:
```json
{
  "data": [...],
  "total": 100,
  "page": 1,
  "page_size": 10
}
```

### Error Handling
Standard error format:
```json
{
  "error": {
    "message": "Error description",
    "type": "error_type",
    "code": "specific_error_code"
  }
}
```

Common error types:
- `invalid_request_error`
- `not_found_error`
- `internal_server_error`

### Security
1. **Authentication Headers**
   - Session: Cookie-based
   - API: Bearer token
   - Resource token: `X-Resource-Token`

2. **Authorization**
   - Role-based access control
   - First user becomes admin
   - API tokens inherit creator's permissions

## Development Guidelines

### OpenAPI Documentation
1. **Handler Documentation**
   ```rust
   /// Handler description
   #[utoipa::path(
       method,
       path = "endpoint_path",
       tag = "category",
       request_body = RequestType,
       responses(
           (status = 200, body = ResponseType),
           (status = 400, body = ErrorType)
       )
   )]
   ```

2. **Schema Documentation**
   ```rust
   /// Type description
   #[derive(ToSchema)]
   #[schema(example = json!({...}))]
   pub struct DataType {
       /// Field description
       pub field: Type
   }
   ```

### Testing
1. **OpenAPI Tests**
   - Verify endpoint documentation
   - Check schema definitions
   - Validate examples

2. **Integration Tests**
   - Use test utilities
   - Mock external services
   - Test error cases

## Configuration

### Environment Variables
- `BODHI_HOST` - Server host
- `BODHI_PORT` - Server port
- `BODHI_AUTH_URL` - Auth server URL

### File Locations
- Models: `$BODHI_HOME/models`
- Database: `$BODHI_HOME/db`
- Aliases: `$BODHI_HOME/aliases` 