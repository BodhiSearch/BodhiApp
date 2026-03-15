---
title: 'OpenAPI Reference'
description: 'Interactive API documentation via Swagger UI — endpoint prefixes, CORS policy, OpenAI-compatible APIs, and curl examples'
order: 256
---

# OpenAPI Reference

Bodhi provides auto-generated, interactive API documentation via Swagger UI. The documentation covers all public and authenticated endpoints and is continuously updated as the backend evolves.

## Accessing Swagger UI

Visit the interactive API explorer at:

```
http://<your-bodhi-instance>/swagger-ui
```

For a default local installation, that is `http://localhost:1135/swagger-ui`.

You can also access it from within the Bodhi App by selecting **API Documentation** from the menu.

The Swagger UI lets you:

- Browse endpoint descriptions, request/response schemas, and authentication requirements
- Test endpoints interactively with real-time requests
- View available authentication methods (session-based and bearer token)

## Endpoint Prefixes

Bodhi organizes its API under two main prefixes:

### `/v1/` -- OpenAI-Compatible Endpoints

These endpoints follow the OpenAI API format, so existing OpenAI client libraries work with Bodhi:

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/v1/chat/completions` | POST | Chat completions (streaming and non-streaming) |
| `/v1/models` | GET | List available models |
| `/v1/embeddings` | POST | Generate text embeddings |

### `/bodhi/v1/` -- Bodhi-Specific Endpoints

Bodhi-specific functionality lives under the `/bodhi/v1/` prefix. This includes user management, MCP configuration, model management, settings, tokens, and more. See Swagger UI for the full list.

### `/bodhi/v1/apps/` -- External App Endpoints

Third-party apps that have completed the [access request flow](/docs/developer/app-access-requests) use endpoints under `/bodhi/v1/apps/`:

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/bodhi/v1/apps/request-access` | POST | Create an access request |
| `/bodhi/v1/apps/access-requests/{id}` | GET | Poll access request status |
| `/bodhi/v1/apps/mcps` | GET | List accessible MCP instances |
| `/bodhi/v1/apps/mcps/{id}` | GET | Get MCP instance details |
| `/bodhi/v1/apps/mcps/{id}/tools/refresh` | POST | Refresh MCP tool list |
| `/bodhi/v1/apps/mcps/{id}/tools/{tool_name}/execute` | POST | Execute an MCP tool |

## CORS Policy

Bodhi applies different CORS policies depending on the endpoint category:

- **Session endpoints** (login, logout, OAuth callbacks) have **restrictive CORS** -- only same-origin requests are allowed. This protects session-based authentication from cross-site attacks.
- **API and external app endpoints** (`/v1/*`, `/bodhi/v1/apps/*`) have **permissive CORS** -- cross-origin requests are allowed. This enables third-party web apps to call the Bodhi API from their own domains.

## Authentication

Bodhi supports two authentication methods:

- **Session auth** -- Browser cookie-based sessions, used by the Bodhi web UI and for review/approval flows.
- **Bearer token** -- OAuth2 bearer tokens in the `Authorization` header. Used by external apps after completing the access request and token exchange flow. Also used for API token-based access.

```bash
# Bearer token authentication
curl http://localhost:1135/v1/models \
  -H "Authorization: Bearer YOUR_TOKEN"
```

## Quick Start Examples

### Chat Completion

```bash
curl -X POST http://localhost:1135/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_API_TOKEN" \
  -d '{
    "model": "your-model-alias",
    "messages": [
      {"role": "system", "content": "You are a helpful assistant."},
      {"role": "user", "content": "Hello!"}
    ]
  }'
```

### List Models

```bash
curl http://localhost:1135/v1/models \
  -H "Authorization: Bearer YOUR_API_TOKEN"
```

### Generate Embeddings

```bash
curl -X POST http://localhost:1135/v1/embeddings \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_API_TOKEN" \
  -d '{
    "model": "your-embedding-model",
    "input": "Text to generate embeddings for"
  }'
```

## Keeping the Spec Current

The OpenAPI specification is auto-generated from the Rust backend. As the codebase evolves, the Swagger UI always reflects the current API surface. Developers building against the API can rely on the Swagger UI as the authoritative reference.
