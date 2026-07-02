# API Reference

Quick reference guide for all BodhiApp API endpoints, including authentication requirements, request/response formats, and common usage patterns.

## Base URLs

- **BodhiApp Native API**: `http://localhost:1135/bodhi/v1/`
- **OpenAI Compatible API**: `http://localhost:1135/v1/`
- **Anthropic Compatible API**: `http://localhost:1135/anthropic/v1/`
- **Gemini Compatible API**: `http://localhost:1135/v1beta/`
- **System Endpoints**: `http://localhost:1135/`

## Authentication

All API endpoints (except public ones) require authentication:

```http
Authorization: Bearer bodhiapp_your-api-token-here.your-client-id
Content-Type: application/json
```

API tokens have the format `bodhiapp_<random>.<client_id>` and are shown only once at creation.

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
**Response**: Discriminated union on `auth_status` (`logged_out` | `logged_in` | `api_token`). Session responses carry `user_id`, `username`, `first_name?`, `last_name?`, `role?`, `id_token?`; token responses carry `role` plus an `access` envelope field (`ResourceAccessInfo` = `{ models, mcps }` of `ResourceAccess`) reflecting effective grants. `access` is present only for token-bearing principals.

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

### Responses API

Stateful response lifecycle (OpenAI Responses spec). `model` is required and routes to a `resp/*` alias.

```http
POST   /v1/responses                              # Create response
GET    /v1/responses/{response_id}                # Retrieve response
DELETE /v1/responses/{response_id}                # Delete response
POST   /v1/responses/{response_id}/cancel         # Cancel response
GET    /v1/responses/{response_id}/input_items    # List input items
```
**Auth**: User level

## Anthropic Compatible API

Anthropic Messages format. Accepts `x-api-key` as an alternative to `Authorization: Bearer`. `/v1/messages` is an alias for `/anthropic/v1/messages`.

### Create Message
```http
POST /anthropic/v1/messages
POST /v1/messages
```
**Auth**: User level  
**Request**:
```json
{
  "model": "claude-3-5-sonnet",
  "max_tokens": 1024,
  "messages": [{"role": "user", "content": "Hello"}],
  "stream": false
}
```

### List Models
```http
GET /anthropic/v1/models
```
**Auth**: User level

### Get Model
```http
GET /anthropic/v1/models/{model_id}
```
**Auth**: User level

## Gemini Compatible API

Google Gemini generateContent format, served under the `/v1beta/` base URL. The model and action are encoded in the path (e.g. `:generateContent`, `:streamGenerateContent`).

### Generate Content
```http
POST /v1beta/models/{model}:{action}
```
**Auth**: User level

### List / Get Models
```http
GET /v1beta/models
GET /v1beta/models/{model_id}
```
**Auth**: User level

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
**Request** (`CreateTokenRequest`): `name` optional (0–100), `scope` required, `grants` optional (omitted ⇒ deny-everything default):
```json
{
  "name": "My Token",
  "scope": "scope_token_power_user",
  "grants": {
    "version": "1",
    "models_list": false,
    "models": { "type": "specific", "ids": ["alias-1"] },
    "mcps_list": false,
    "mcps": { "type": "specific", "ids": [] }
  }
}
```
**Response**: **201**, `Cache-Control: no-store`, `{ "token": "bodhiapp_<random>.<client_id>" }` — shown once. A `User`-scoped caller may mint only `scope_token_user` (else 403).

#### Update API Token
```http
PUT /bodhi/v1/tokens/{token_id}
```
**Auth**: PowerUser (session only)  
**Request**: `{"name": "Updated Name", "status": "inactive"}` (only `name` + `status`; **grants are immutable** — delete + re-mint to change them)

#### Delete API Token
```http
DELETE /bodhi/v1/tokens/{token_id}
```
**Auth**: PowerUser (session only)  
Hard-deletes the token, revoking it immediately. **204** on success; **404** (`entity_error-not_found`) for an unknown/unowned id.

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
| `/v1/chat/completions` | POST | User | OpenAI chat (grant-guarded) |
| `/v1/embeddings` | POST | User | OpenAI embeddings (grant-guarded) |
| `/v1/responses` | POST | User | Create response (grant-guarded) |
| `/v1/responses/{id}` | GET/DELETE | User | Get/delete response |
| `/v1/responses/{id}/cancel` | POST | User | Cancel response |
| `/v1/responses/{id}/input_items` | GET | User | List input items |
| `/anthropic/v1/messages` | POST | User | Anthropic message (also `/v1/messages`, grant-guarded) |
| `/anthropic/v1/models` | GET | User | List Anthropic models |
| `/anthropic/v1/models/{id}` | GET | User | Get Anthropic model |
| `/v1beta/models` | GET | User | List Gemini models |
| `/v1beta/models/{id}` | GET | User | Get Gemini model |
| `/v1beta/models/{model}:{action}` | POST | User | Gemini generateContent (grant-guarded) |
| `/bodhi/v1/models` | GET | User | List model aliases |
| `/bodhi/v1/models/{id}` | GET | User | Get model alias |
| `/bodhi/v1/modelfiles` | GET | User | List model files |
| `/bodhi/v1/models` | POST | PowerUser | Create alias |
| `/bodhi/v1/models/{id}` | PUT | PowerUser | Update alias |
| `/bodhi/v1/modelfiles/pull` | POST | PowerUser | Download model |
| `/bodhi/v1/modelfiles/pull/{id}` | GET | PowerUser | Download status |
| `/bodhi/v1/tokens` | GET | PowerUser (session) | List tokens |
| `/bodhi/v1/tokens` | POST | PowerUser (session) | Create token |
| `/bodhi/v1/tokens/{id}` | PUT | PowerUser (session) | Update token (name + status only) |
| `/bodhi/v1/tokens/{id}` | DELETE | PowerUser (session) | Delete token (hard delete, 204) |
| `/bodhi/v1/settings` | GET | Admin (session) | List settings |
| `/bodhi/v1/settings/{key}` | PUT | Admin (session) | Update setting |
| `/bodhi/v1/settings/{key}` | DELETE | Admin (session) | Delete setting |
| `/bodhi/v1/apps/request-access` | POST | None | App creates access request |
| `/bodhi/v1/apps/access-requests/{id}` | GET | None | App polls request status |
| `/bodhi/v1/access-requests/{id}/review` | GET | User (session) | Owner reviews request |
| `/bodhi/v1/access-requests/{id}/approve` | PUT | User (session) | Owner approves (grants + role) |
| `/bodhi/v1/access-requests/{id}/deny` | POST | User (session) | Owner denies request |
| `/bodhi/v1/access-requests/apps` | GET | User (session) | Owner lists granted apps |
| `/bodhi/v1/access-requests/{id}/revoke` | POST | User (session) | Owner revokes app access |
| `/bodhi/v1/apps/mcps` | GET | OAuth app (User) | List grant-listable MCP instances |
| `/bodhi/v1/apps/mcps/{id}` | GET | OAuth app (User) | Get MCP instance (404 if not listable) |
| `/bodhi/v1/apps/mcps/{id}/mcp` | ANY | OAuth app (User) | MCP proxy (403 if not granted) |

App-flow endpoints implement the third-party OAuth access-request flow; see **[app-to-bodhi-oauth.md](app-to-bodhi-oauth.md)**.

## Request/Response Formats

### Standard Error Response

Bodhi management endpoints (everything except the `/v1/*` OpenAI-compatible routes) return the `BodhiError` envelope, which is a superset of OpenAI's shape: both structured `params` (map) and `param` (JSON-encoded string form of `params`) are emitted so OpenAI-only clients can still read `param`.

```json
{
  "error": {
    "message": "Human-readable error description",
    "type": "error_category",
    "code": "specific_error_code",
    "params": { "field": "field_name" },
    "param": "{\"field\":\"field_name\"}"
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

### Grant / Access Errors
- `token_grant_error-model_forbidden` (403): Token/app lacks access to the requested model
- `token_grant_error-mcp_forbidden` (403): Token/app lacks access to the requested MCP

> **404-hidden**: a direct GET of a model or MCP that the token/app is not granted returns **404** (`entity_error-not_found` / `model_not_found` / `alias_not_found`) rather than 403 — the resource's existence is hidden. List endpoints silently omit non-granted resources. Only inference/connect attempts surface an explicit **403** `token_grant_error-*`.

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