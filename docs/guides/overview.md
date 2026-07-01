# System Overview

## What is BodhiApp?

BodhiApp is a comprehensive local Large Language Model (LLM) inference server that brings enterprise-grade AI capabilities to your desktop. It combines the power of llama.cpp for local inference with a modern web interface, providing both technical and non-technical users with seamless access to open-source language models.

### Core Philosophy

BodhiApp bridges the gap between complex AI infrastructure and user-friendly interfaces. While many local LLM solutions target only technical users, BodhiApp is designed to serve both developers building AI applications and end-users who want to interact with AI models directly.

## Key Features

### 🔌 Multiple API Compatibility Layers

**OpenAI Compatible APIs**
- Full compatibility with OpenAI's `/v1/chat/completions` and `/v1/models` endpoints
- Drop-in replacement for existing OpenAI integrations
- Support for streaming and non-streaming responses
- Compatible with OpenAI client libraries

**Anthropic Compatible APIs**
- Anthropic Messages format via `/anthropic/v1/messages` (and `/v1/messages`)
- Model listing via `/anthropic/v1/models`
- Accepts `x-api-key` as an alternative to `Authorization: Bearer`

**OpenAI Responses API**
- Stateful response lifecycle via `/v1/responses` (create, get, delete, cancel, input_items)

**BodhiApp Native APIs**
- Advanced features via `/bodhi/v1/*` endpoints
- Model management and download capabilities
- User and token management
- System configuration and settings

### 🔐 Enterprise-Grade Authentication

**Role-Based Access Control**
- Hierarchical permission system: Admin → Manager → PowerUser → User
- Fine-grained access control for different API endpoints
- Support for both individual and team usage scenarios

**Flexible Authentication Modes**
- API token-based authentication for programmatic access
- Session-based authentication for web interface
- Integration with bodhi-auth-server for centralized identity management

### 🤖 Local LLM Inference

**llama.cpp Integration**
- Native llama.cpp compilation and process management
- Support for GGUF model format
- Hardware acceleration support (CUDA, OpenCL, Metal)
- Optimized for local hardware configurations

**Model Management**
- Direct integration with HuggingFace ecosystem
- Automatic model downloading and caching
- Support for existing local GGUF models
- Model alias system for consistent configuration

### 🎨 Built-in User Interface

**Modern Chat Interface**
- Real-time streaming responses
- Markdown support with code block highlighting
- Multi-conversation management
- Customizable system prompts and parameters

**Management Interfaces**
- Model download and configuration
- API token management
- User and permission management
- System settings and monitoring

## Architecture Overview

### System Components

```
┌─────────────────────────────────────────────────────────────┐
│                    Frontend Layer                           │
│  React + TypeScript + Vite + TanStack Router + Shadcn UI   │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                    Routes Layer                             │
│  routes_all → routes_oai + routes_app                      │
│  (HTTP endpoints, OpenAPI docs, middleware)                │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                   Services Layer                            │
│  Domain types, errors, validation + business logic,        │
│  external integrations, data management                    │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                Infrastructure Layer                         │
│  llama_server_proc, database, file system, auth           │
└─────────────────────────────────────────────────────────────┘
```

### Technology Stack

**Backend (Rust)**
- **Web Framework**: Axum for HTTP server functionality
- **Database**: SeaORM (SQLite dev/desktop, PostgreSQL production/Docker)
- **Authentication**: OAuth2 + JWT token validation
- **LLM Integration**: Custom llama.cpp process management
- **API Documentation**: OpenAPI 3.1 with utoipa

**Frontend (React)**
- **Framework**: Vite + React (client-side SPA) with TanStack Router (file-based routing)
- **Language**: TypeScript for type safety
- **Styling**: TailwindCSS + Shadcn UI components
- **State Management**: TanStack Query v5 for API state
- **Testing**: Vitest with MSW for API mocking

**Desktop Integration**
- **Native App**: Tauri for cross-platform desktop application
- **System Integration**: Native OS features and file system access
- **Auto-launch**: Automatic browser opening on application start

## System Requirements

### Minimum Requirements
- **Memory**: 8GB RAM (16GB recommended for larger models)
- **GPU**: 8GB iGPU for hardware acceleration
- **CPU**: 8-core processor for optimal performance
- **Storage**: 5GB+ available space for model downloads
- **Network**: Internet connection for model downloads and authentication

### Supported Platforms
- **macOS**: 14.0+ on ARM64 (M-series chips) - Currently available
- **Windows**: Support planned (Intel/AMD x64)
- **Linux**: Support planned (Intel/AMD x64)

### Model Compatibility
- **Format**: GGUF models from HuggingFace
- **Size Range**: 1GB to 70GB+ depending on available system memory
- **Popular Models**: Llama 3, Llama 2, Mistral, Phi-3, Gemma, and more
- **Local Storage**: Models stored in HuggingFace cache directory

## TypeScript Client Library

BodhiApp provides a comprehensive TypeScript client library for developers:

### Installation
```bash
npm install @bodhiapp/ts-client
```

### Key Features
- **Auto-generated Types**: Generated from OpenAPI specification
- **Full API Coverage**: All endpoints with proper TypeScript types
- **Error Handling**: Structured error types for robust error handling
- **Framework Agnostic**: Works with any JavaScript/TypeScript project

### Basic Usage
```typescript
import { type AppInfo, type CreateChatCompletionData } from '@bodhiapp/ts-client';

// Type-safe API calls with proper error handling
const response = await fetch('http://localhost:1135/bodhi/v1/info');
const appInfo: AppInfo = await response.json();
```

## Development Workflow

### API-First Development
1. **OpenAPI Specification**: Complete API documentation available at `/docs`
2. **Interactive Testing**: Built-in Swagger UI for API exploration
3. **Type Generation**: Automatic TypeScript type generation
4. **Error Handling**: Comprehensive error codes and localized messages

### Local Development Setup
1. **Download BodhiApp**: Install from [https://getbodhi.app](https://getbodhi.app)
2. **Initial Setup**: Complete authentication and model download
3. **API Access**: Generate API tokens for programmatic access
4. **Client Integration**: Install and configure TypeScript client

## Unique Capabilities

### Model Alias System
- **Consistent References**: Use semantic names instead of file paths
- **Parameter Bundling**: Save inference configurations with aliases
- **Easy Switching**: Change models without code modifications
- **Default Configurations**: Pre-configured aliases for popular models

### Integrated Authentication
- **Centralized Identity**: Integration with bodhi-auth-server
- **Multi-tenant Support**: Support for team and enterprise usage
- **API Token Management**: Long-lived tokens for automation
- **Permission Granularity**: Fine-grained access control

### Hybrid Architecture
- **Desktop Native**: Full desktop application experience
- **Web Interface**: Modern browser-based UI
- **API Server**: Programmatic access for integrations
- **Local Processing**: Complete privacy with local inference

## Next Steps

Now that you understand BodhiApp's architecture and capabilities, you can:

1. **[Get Started](getting-started.md)** - Install and set up BodhiApp
2. **[Explore the UI](embedded-ui.md)** - Learn the built-in interface
3. **[Understand Authentication](authentication.md)** - Set up API access
4. **[Try the APIs](openai-api.md)** - Start with OpenAI-compatible endpoints

---

*This overview provides the foundation for understanding BodhiApp's capabilities. The following sections dive deeper into practical usage and integration patterns.* 