# API Reference

Quick reference guide for all BodhiApp API endpoints, including authentication requirements, request/response formats, and common usage patterns.

## Base URLs

- **BodhiApp Native API**: `http://localhost:1135/bodhi/v1/`
- **OpenAI Compatible API**: `http://localhost:1135/v1/`
- **Ollama Compatible API**: `http://localhost:1135/api/`
- **System Endpoints**: `http://localhost:1135/`

## Authentication

All API endpoints (except public ones) require authentication:

```http
Authorization: Bearer sk-bodhi-your-api-token-here
Content-Type: application/json
```

## System Endpoints

### Health Check
```http
GET /ping
```
**Auth**: None  
**Response**: `{"message": "pong"}`

### Application Info
```http
GET /bodhi/v1/info
```
**Auth**: None  
**Response**: `{"version": "0.1.0", "status": "ready"}`

### User Information
```http
GET /bodhi/v1/user
```
**Auth**: Optional  
**Response**: User info with authentication status

## OpenAI Compatible API

### Chat Completions
```http
POST /v1/chat/completions
```
**Auth**: User level  
**Request**:
```json
{
  "model": "llama3:instruct",
  "messages": [
    {"role": "system", "content": "You are a helpful assistant."},
    {"role": "user", "content": "Hello!"}
  ],
  "temperature": 0.7,
  "max_tokens": 1000,
  "stream": false
}
```

### List Models
```http
GET /v1/models
```
**Auth**: User level  
**Response**: OpenAI-compatible model list

## Ollama Compatible API

### List Models
```http
GET /api/tags
```
**Auth**: User level  
**Response**: Ollama-format model list

### Show Model
```http
POST /api/show
```
**Auth**: User level  
**Request**: `{"name": "llama3:instruct"}`

### Chat
```http
POST /api/chat
```
**Auth**: User level  
**Request**:
```json
{
  "model": "llama3:instruct",
  "messages": [{"role": "user", "content": "Hello"}],
  "stream": false,
  "options": {"temperature": 0.7}
}
```

## BodhiApp Native API

### Model Management

#### List Model Aliases
```http
GET /bodhi/v1/models
```
**Auth**: User level  
**Query Params**: `page`, `page_size`, `sort`, `sort_order`

#### Get Model Alias
```http
GET /bodhi/v1/models/{alias}
```
**Auth**: User level

#### Create Model Alias
```http
POST /bodhi/v1/models
```
**Auth**: PowerUser level  
**Request**:
```json
{
  "alias": "my-model",
  "repo": "microsoft/Phi-3-mini-4k-instruct-gguf",
  "filename": "Phi-3-mini-4k-instruct-q4.gguf",
  "snapshot": "main",
  "request_params": {"temperature": 0.7},
  "context_params": {"n_ctx": 4096}
}
```

#### Update Model Alias
```http
PUT /bodhi/v1/models/{alias}
```
**Auth**: PowerUser level

#### List Model Files
```http
GET /bodhi/v1/modelfiles
```
**Auth**: User level

#### Download Model
```http
POST /bodhi/v1/modelfiles/pull
```
**Auth**: PowerUser level  
**Request**: `{"repo": "owner/repo", "filename": "model.gguf"}`

#### Download by Alias
```http
POST /bodhi/v1/modelfiles/pull/{alias}
```
**Auth**: PowerUser level

#### Get Download Status
```http
GET /bodhi/v1/modelfiles/pull/{id}
```
**Auth**: PowerUser level

### Token Management

#### List API Tokens
```http
GET /bodhi/v1/tokens
```
**Auth**: PowerUser (session only)

#### Create API Token
```http
POST /bodhi/v1/tokens
```
**Auth**: PowerUser (session only)  
**Request**:
```json
{
  "name": "My Token",
  "scope": "scope_token_power_user",
  "expires_at": "2024-12-31T23:59:59Z"
}
```

#### Update API Token
```http
PUT /bodhi/v1/tokens/{token_id}
```
**Auth**: PowerUser (session only)  
**Request**: `{"name": "Updated Name", "status": "inactive"}`

### Settings Management

#### List Settings
```http
GET /bodhi/v1/settings
```
**Auth**: Admin (session only)

#### Update Setting
```http
PUT /bodhi/v1/settings/{key}
```
**Auth**: Admin (session only)  
**Request**: `{"value": "new_value"}`

#### Delete Setting
```http
DELETE /bodhi/v1/settings/{key}
```
**Auth**: Admin (session only)

## Authentication Endpoints

### OAuth Initiation
```http
POST /bodhi/v1/auth/initiate
```
**Auth**: None  
**Request**: `{"redirect_uri": "http://localhost:1135/ui/auth/callback"}`

### OAuth Callback
```http
POST /bodhi/v1/auth/callback
```
**Auth**: None  
**Request**: `{"code": "auth_code", "state": "state_value"}`

### Logout
```http
POST /bodhi/v1/logout
```
**Auth**: None

### Initial Setup
```http
POST /bodhi/v1/setup
```
**Auth**: None (setup mode only)  
**Request**: `{"name": "My App", "description": "Description"}`

## Authorization Matrix

| Endpoint | Method | Auth Level | Description |
|----------|--------|------------|-------------|
| `/ping` | GET | None | Health check |
| `/bodhi/v1/info` | GET | None | App information |
| `/bodhi/v1/user` | GET | Optional | User information |
| `/v1/models` | GET | User | List OpenAI models |
| `/v1/chat/completions` | POST | User | OpenAI chat |
| `/api/tags` | GET | User | List Ollama models |
| `/api/show` | POST | User | Show Ollama model |
| `/api/chat` | POST | User | Ollama chat |
| `/bodhi/v1/models` | GET | User | List model aliases |
| `/bodhi/v1/models/{id}` | GET | User | Get model alias |
| `/bodhi/v1/modelfiles` | GET | User | List model files |
| `/bodhi/v1/models` | POST | PowerUser | Create alias |
| `/bodhi/v1/models/{id}` | PUT | PowerUser | Update alias |
| `/bodhi/v1/modelfiles/pull` | POST | PowerUser | Download model |
| `/bodhi/v1/modelfiles/pull/{id}` | GET | PowerUser | Download status |
| `/bodhi/v1/tokens` | GET | PowerUser (session) | List tokens |
| `/bodhi/v1/tokens` | POST | PowerUser (session) | Create token |
| `/bodhi/v1/tokens/{id}` | PUT | PowerUser (session) | Update token |
| `/bodhi/v1/settings` | GET | Admin (session) | List settings |
| `/bodhi/v1/settings/{key}` | PUT | Admin (session) | Update setting |
| `/bodhi/v1/settings/{key}` | DELETE | Admin (session) | Delete setting |

## Request/Response Formats

### Standard Error Response
```json
{
  "error": {
    "message": "Human-readable error description",
    "type": "error_category",
    "code": "specific_error_code",
    "param": "field_name_if_applicable"
  }
}
```

### Paginated Response
```json
{
  "data": [...],
  "total": 100,
  "page": 1,
  "page_size": 30
}
```

### Chat Completion Response
```json
{
  "id": "chatcmpl-123",
  "object": "chat.completion",
  "created": 1677652288,
  "model": "llama3:instruct",
  "choices": [{
    "index": 0,
    "message": {
      "role": "assistant",
      "content": "Hello! How can I help you today?"
    },
    "finish_reason": "stop"
  }],
  "usage": {
    "prompt_tokens": 9,
    "completion_tokens": 12,
    "total_tokens": 21
  }
}
```

### Model Alias Response
```json
{
  "alias": "llama3:instruct",
  "repo": "microsoft/Phi-3-mini-4k-instruct-gguf",
  "filename": "Phi-3-mini-4k-instruct-q4.gguf",
  "snapshot": "main",
  "source": "user",
  "request_params": {
    "temperature": 0.7,
    "max_tokens": 2048
  },
  "context_params": {
    "n_ctx": 4096,
    "n_threads": 8
  }
}
```

## Common Parameters

### Chat Completion Parameters
| Parameter | Type | Range | Default | Description |
|-----------|------|-------|---------|-------------|
| `model` | string | - | Required | Model alias to use |
| `messages` | array | - | Required | Conversation messages |
| `temperature` | number | 0.0-2.0 | 1.0 | Sampling randomness |
| `max_tokens` | integer | 1-∞ | Model limit | Maximum tokens to generate |
| `top_p` | number | 0.0-1.0 | 1.0 | Nucleus sampling |
| `frequency_penalty` | number | -2.0-2.0 | 0.0 | Frequency penalty |
| `presence_penalty` | number | -2.0-2.0 | 0.0 | Presence penalty |
| `stop` | array | - | null | Stop sequences |
| `stream` | boolean | - | false | Enable streaming |

### Model Alias Parameters
| Parameter | Type | Description |
|-----------|------|-------------|
| `alias` | string | Unique identifier |
| `repo` | string | HuggingFace repository |
| `filename` | string | GGUF file name |
| `snapshot` | string | Git commit/branch |
| `request_params` | object | OpenAI parameters |
| `context_params` | object | llama.cpp parameters |

### Pagination Parameters
| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `page` | integer | 1 | Page number |
| `page_size` | integer | 30 | Items per page |
| `sort` | string | - | Sort field |
| `sort_order` | string | "asc" | Sort direction |

## HTTP Status Codes

| Code | Description | Common Causes |
|------|-------------|---------------|
| 200 | OK | Successful request |
| 201 | Created | Resource created successfully |
| 400 | Bad Request | Invalid request format or parameters |
| 401 | Unauthorized | Missing or invalid authentication |
| 403 | Forbidden | Insufficient permissions |
| 404 | Not Found | Resource doesn't exist |
| 409 | Conflict | Resource already exists |
| 500 | Internal Server Error | Server-side error |
| 503 | Service Unavailable | Service temporarily unavailable |

## Error Codes Reference

### Authentication Errors
- `token_error-invalid_token`: Invalid or expired token
- `unauthorized`: No authentication provided
- `forbidden`: Insufficient permissions

### Request Errors
- `bad_request_error`: Invalid request parameters
- `json_rejection_error`: JSON parsing failed
- `validation_errors`: Field validation failed

### Resource Errors
- `alias_not_found`: Model alias not found
- `entity_error-not_found`: Generic resource not found

### System Errors
- `internal_server_error`: Server error
- `service_unavailable_error`: Service unavailable
- `app_reg_info_missing`: App not configured

## Client Library Usage

### TypeScript Client
```bash
npm install @bodhiapp/ts-client
```

```typescript
import { BodhiClient } from '@bodhiapp/ts-client';

const client = new BodhiClient({
  apiKey: 'your-api-token',
  baseURL: 'http://localhost:1135'
});

// Chat completion
const response = await client.chat.completions.create({
  model: 'llama3:instruct',
  messages: [{ role: 'user', content: 'Hello!' }]
});
```

### OpenAI SDK Compatibility
```typescript
import OpenAI from 'openai';

const client = new OpenAI({
  apiKey: 'your-api-token',
  baseURL: 'http://localhost:1135/v1'
});

const response = await client.chat.completions.create({
  model: 'llama3:instruct',
  messages: [{ role: 'user', content: 'Hello!' }]
});
```

## Rate Limits and Quotas

BodhiApp currently does not enforce rate limits, but consider these guidelines:

- **Concurrent Requests**: Limit to 10 concurrent requests
- **Model Loading**: Allow time for model loading (30-60 seconds)
- **Memory Usage**: Monitor system memory with large models
- **Context Windows**: Respect model context limits

## Development Tools

### Built-in API Explorer
- **Swagger UI**: `http://localhost:1135/swagger-ui`
- **OpenAPI Spec**: `http://localhost:1135/docs`

### Health Monitoring
```bash
# Check server health
curl http://localhost:1135/ping

# Check app status
curl http://localhost:1135/bodhi/v1/info

# Validate token
curl -H "Authorization: Bearer $TOKEN" \
     http://localhost:1135/bodhi/v1/user
```

### Debugging
```bash
# View logs
tail -f ~/.cache/bodhi/logs/bodhi.log

# Check model files
ls -la ~/.cache/huggingface/hub/

# Test API endpoints
curl -X POST http://localhost:1135/v1/chat/completions \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"model":"llama3:instruct","messages":[{"role":"user","content":"test"}]}'
```

## Migration Guides

### From OpenAI
1. Change base URL to `http://localhost:1135/v1`
2. Add authentication header
3. Use BodhiApp model aliases
4. Handle additional error codes

### From Ollama
1. Change base URL to `http://localhost:1135/api`
2. Add authentication header
3. Model names may differ (use `/api/tags` to list)
4. Some endpoints not supported (use BodhiApp equivalents)

## Best Practices

### Performance
- Reuse HTTP connections
- Cache model lists
- Use streaming for long responses
- Monitor memory usage

### Security
- Store tokens securely
- Use minimum required scopes
- Rotate tokens regularly
- Validate all inputs

### Error Handling
- Implement retry logic for 5xx errors
- Handle specific error codes
- Provide user-friendly messages
- Log errors with context

### Development
- Use TypeScript for better type safety
- Test error scenarios
- Monitor API usage
- Document integration patterns

## Next Steps

- **[Getting Started](getting-started.md)** - Initial setup and first API call
- **[Authentication](authentication.md)** - Detailed authentication guide
- **[Examples](examples.md)** - Complete integration examples
- **[Error Handling](error-handling.md)** - Comprehensive error handling

---

*This reference provides quick access to all BodhiApp API endpoints and essential information for developers.* 