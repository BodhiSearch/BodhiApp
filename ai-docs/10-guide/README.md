# BodhiApp Integration Guides

This directory contains comprehensive guides for integrating with BodhiApp APIs and building AI applications using local Large Language Models (LLMs).

## Available Guides

### [BodhiApp AI Integration Guide](./bodhiapp-ai-integration-guide.md)
**Target Audience**: AI coding assistants and developers building AI applications  
**Focus**: User and Power User accessible APIs for chat applications and model management

#### What's Covered:
- **Core Chat APIs**: Models listing, chat completions (streaming and non-streaming)
- **Model Management**: Model creation, updates, and downloads (Power User features)
- **Authentication**: OAuth2/OpenID Connect patterns and API token usage
- **Error Handling**: Standard error responses and recovery strategies
- **Ollama Compatibility**: Ollama-compatible endpoints for existing integrations
- **Best Practices**: Rate limiting, streaming optimization, and production considerations
- **Complete Examples**: Ready-to-use TypeScript client implementations

#### Quick Start:
```typescript
// Basic chat completion
const response = await fetch('/v1/chat/completions', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer YOUR_API_TOKEN',
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({
    model: "llama-3.1-8b-instruct",
    messages: [{ role: "user", content: "Hello!" }]
  })
});
```

## API Endpoints Summary

### OpenAI-Compatible Endpoints (`/v1/`)
- `GET /v1/models` - List available models
- `POST /v1/chat/completions` - Chat completions with streaming support

### Bodhi-Specific Endpoints (`/bodhi/v1/`)
- `GET /bodhi/v1/models` - Detailed model information with download status
- `POST /bodhi/v1/models` - Create/update models (Power User)
- `POST /bodhi/v1/models/{id}/download` - Download model files (Power User)

### Ollama-Compatible Endpoints (`/ollama/api/`)
- `POST /ollama/api/generate` - Generate completions
- `POST /ollama/api/chat` - Chat with Ollama format

## Authentication
- **Session-based**: Browser cookies for web applications
- **API Token**: Bearer tokens for programmatic access
- **Roles**: User (basic access) and Power User (enhanced model management)

## For More Information
- **OpenAPI Specification**: Available at `/bodhi/v1/openapi.json`
- **Architecture Documentation**: See `ai-docs/01-architecture/` for system design details
- **Implementation Examples**: See `ai-docs/02-features/` for feature-specific guides

## Related Documentation
- [API Integration Patterns](../01-architecture/api-integration.md)
- [Authentication Architecture](../01-architecture/authentication.md)
- [Frontend React Integration](../01-architecture/frontend-react.md)
- [Backend Architecture](../01-architecture/backend-architecture.md) 