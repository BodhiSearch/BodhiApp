# BodhiApp Developer User Guide

**A comprehensive guide for developers integrating with BodhiApp APIs and using the embedded UI**

## Overview

BodhiApp is a local Large Language Model (LLM) inference server that provides OpenAI-compatible APIs, comprehensive model management, and a built-in chat interface. This guide helps third-party developers, AI tool builders, and individual developers successfully integrate with BodhiApp's capabilities.

## Target Audience

- **Third-party app developers** integrating with BodhiApp APIs
- **AI tool developers** building on top of BodhiApp
- **Individual developers** creating personal AI applications

## Prerequisites

### System Requirements
- **Memory**: 8GB RAM minimum (16GB recommended)
- **GPU**: 8GB iGPU minimum for optimal performance
- **Processor**: 8-core CPU minimum
- **Storage**: 5GB+ for model downloads
- **Operating System**: macOS 14.0+ (ARM64), Windows/Linux support coming soon

### Account Setup
- Download BodhiApp from [https://getbodhi.app](https://getbodhi.app)
- Account creation handled during initial setup via bodhi-auth-server
- First user automatically becomes admin

### Development Setup
```bash
# Install TypeScript client library
npm install @bodhiapp/ts-client
```

## Quick Start Path

**New to BodhiApp?** Follow this recommended path:

1. **[Getting Started](getting-started.md)** - Install and set up BodhiApp
2. **[Embedded UI Guide](embedded-ui.md)** - Learn the built-in interface
3. **[Authentication](authentication.md)** - Understand API tokens and permissions
4. **[OpenAI API](openai-api.md)** - Start with familiar OpenAI-compatible endpoints
5. **[Model Management](model-management.md)** - Download and configure models

## Guide Structure

### Core Sections

| Section | Purpose | Best For |
|---------|---------|----------|
| **[Overview](overview.md)** | System architecture and key concepts | Understanding BodhiApp capabilities |
| **[Getting Started](getting-started.md)** | Installation and initial setup | First-time users |
| **[Embedded UI](embedded-ui.md)** | Built-in React interface guide | Users of the desktop app |
| **[Authentication](authentication.md)** | API tokens and authorization | API developers |
| **[OpenAI API](openai-api.md)** | OpenAI-compatible endpoints | Existing OpenAI integrations |

### API Documentation

| Section | Purpose | Best For |
|---------|---------|----------|
| **[Model Management](model-management.md)** | Model download and aliases | Advanced model workflows |
| **[BodhiApp API](bodhi-api.md)** | BodhiApp-specific endpoints | Full platform integration |
| **[Ollama API](ollama-api.md)** | Ollama-compatible endpoints | Ollama migrations |

### Reference & Support

| Section | Purpose | Best For |
|---------|---------|----------|
| **[Error Handling](error-handling.md)** | Troubleshooting and error codes | Debugging issues |
| **[Examples](examples.md)** | Integration patterns | Implementation guidance |
| **[API Reference](api-reference.md)** | Quick endpoint reference | Development reference |

## Key Features Covered

### API Compatibility
- **OpenAI Compatible**: Drop-in replacement for `/v1/chat/completions` and `/v1/models`
- **Ollama Compatible**: Support for `/api/chat`, `/api/tags`, and `/api/show`
- **BodhiApp Native**: Advanced features via `/bodhi/v1/*` endpoints

### Authentication & Security
- **Role-based Access Control**: Admin → Manager → PowerUser → User hierarchy
- **API Token Management**: Long-lived tokens for programmatic access
- **Scope-based Permissions**: Fine-grained access control

### Model Management
- **HuggingFace Integration**: Direct model downloads from repositories
- **Model Aliases**: Consistent model references with custom parameters
- **Local Model Support**: Use existing GGUF models from HuggingFace cache

### Built-in Interface
- **Chat UI**: Real-time streaming chat with markdown support
- **Model Configuration**: Visual model parameter adjustment
- **User Management**: Token and settings management

## Additional Resources

### Official Documentation
- **OpenAPI Specification**: Available at `/docs` when BodhiApp is running
- **Interactive API Explorer**: Built-in Swagger UI at `/swagger-ui`
- **TypeScript Client**: Auto-generated types from OpenAPI spec

### External Links
- **Download BodhiApp**: [https://getbodhi.app](https://getbodhi.app)
- **TypeScript Client**: `@bodhiapp/ts-client` npm package
- **Support**: GitHub Issues and community forums

## Navigation Tips

### For API Integration
1. Start with [Authentication](authentication.md) to understand tokens
2. Try [OpenAI API](openai-api.md) for familiar endpoints
3. Explore [Model Management](model-management.md) for advanced features
4. Reference [Error Handling](error-handling.md) for troubleshooting

### For UI Usage
1. Follow [Getting Started](getting-started.md) for installation
2. Complete [Embedded UI Guide](embedded-ui.md) for full interface tour
3. Use [Examples](examples.md) for integration patterns

### For Troubleshooting
1. Check [Error Handling](error-handling.md) for common issues
2. Use built-in Swagger UI for API testing
3. Review [API Reference](api-reference.md) for quick lookup

---

*This guide focuses on practical API usage and embedded UI functionality. For deployment and enterprise scenarios, refer to the main BodhiApp documentation.* 