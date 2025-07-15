# Bodhi Platform Architecture

## Overview

The Bodhi Platform is a comprehensive Web AI system that enables secure access to local Large Language Models (LLMs) from web applications. It solves the fundamental challenge of browser security restrictions that prevent web pages from directly accessing localhost services, while maintaining industry-standard security practices through OAuth2 authentication and authorization.

The platform consists of multiple integrated components that work together to provide a seamless, secure, and user-friendly experience for both developers and end-users who want to use local AI in their web applications.

## Platform Components

### 1. Bodhi App (Core Server)
**Location**: This repository (`BodhiApp/`)
**Purpose**: Central LLM inference server and application management hub

**Key Capabilities**:
- Local LLM inference via llama.cpp integration
- OpenAI-compatible API endpoints (`/v1/chat/completions`, `/v1/models`)
- Ollama-compatible API endpoints (`/api/tags`, `/api/show`, `/api/chat`)
- Built-in web chat interface
- Model management and downloading from HuggingFace
- OAuth2 resource server with role-based access control
- Multi-user support with authentication
- Desktop application via Tauri integration

**Architecture**: Multi-crate Rust backend with Next.js frontend
**Deployment**: Standalone server or desktop application

### 2. Bodhi Browser Extension
**Repository**: `bodhi-browser/`
**Purpose**: Bridge between web pages and local Bodhi App server

**Key Capabilities**:
- Circumvents browser security sandbox restrictions
- Proxies API requests from web pages to localhost
- Supports both streaming and non-streaming responses
- Extension-to-extension communication for advanced integrations
- Configurable backend URL management
- Built-in settings UI with real-time validation

**Architecture**: Chrome Manifest V3 extension with background service worker
**Components**:
- Background script (service worker)
- Content script (injected into web pages)
- Injection script (provides `window.bodhiext` API)
- Settings UI (Next.js-based popup)

### 3. Bodhi.js Library
**Repository**: `bodhi-browser/` (NPM package `@bodhiapp/bodhijs`)
**Purpose**: JavaScript/TypeScript library for web applications

**Key Capabilities**:
- Framework-agnostic API for web applications
- Extension detection and setup guidance
- Structured error handling with specific error codes
- OpenAI-compatible chat completions interface
- Automatic fallback and retry mechanisms
- TypeScript definitions for full type safety

**Architecture**: Browser-only library with no React dependencies
**Distribution**: NPM package with TypeScript definitions

### 4. Authentication Server (Keycloak Extension)
**Repository**: `keycloak-bodhi-ext/`
**Purpose**: OAuth2.1-compliant authentication and authorization server

**Key Capabilities**:
- Multi-tenant OAuth2 token exchange system
- App client management (public clients for frontends)
- Resource server client management (confidential clients for APIs)
- Custom audience-based authorization policies
- Role-based access control with hierarchical permissions
- User and group management APIs
- Environment-aware client configurations

**Architecture**: Keycloak SPI (Service Provider Interface) extension
**Components**:
- Resource providers for REST endpoints
- Policy providers for authorization rules
- Admin API integration
- Comprehensive UI testing framework

## System Architecture

### High-Level Data Flow

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Web Page      │    │ Browser Ext     │    │   Bodhi App     │    │  Auth Server    │
│                 │    │                 │    │                 │    │                 │
│ - Uses bodhijs  │◄──►│ - Proxies API   │◄──►│ - LLM Inference │◄──►│ - OAuth2 Auth   │
│ - Detects ext   │    │ - Handles auth  │    │ - Model Mgmt    │    │ - Token Exchange│
│ - Makes API     │    │ - Configurable  │    │ - Web UI        │    │ - Role-based    │
│   calls         │    │   backend       │    │ - API endpoints │    │   access        │
└─────────────────┘    └─────────────────┘    └─────────────────┘    └─────────────────┘
```

### Authentication & Authorization Flow

```
1. Web Page → Extension → Bodhi App → Auth Server
   ┌─ User Authentication (OAuth2)
   ├─ Token Exchange (App → Resource Server)
   ├─ API Authorization (Role-based)
   └─ Secure API Access

2. Security Model:
   ┌─ App Clients (Public) → User Authentication
   ├─ Resource Clients (Confidential) → API Access
   ├─ Token Exchange → Cross-client authorization
   └─ Role Hierarchy → Permission management
```

### Communication Patterns

#### 1. Web Page to LLM (Standard Flow)
```
Web Page (bodhijs) 
    ↓ window.bodhiext API
Browser Extension (inject.js)
    ↓ postMessage
Browser Extension (content.js)
    ↓ chrome.runtime.sendMessage
Browser Extension (background.js)
    ↓ HTTP fetch
Bodhi App Server
    ↓ OAuth2 validation
Auth Server
```

#### 2. Streaming Response Flow
```
Bodhi App (SSE stream)
    ↓ Server-Sent Events
Browser Extension (background.js)
    ↓ Chunked messages
Browser Extension (content.js)
    ↓ postMessage chunks
Browser Extension (inject.js)
    ↓ AsyncIterator
Web Page (for-await loop)
```

#### 3. Extension-to-Extension Communication
```
External Extension
    ↓ chrome.runtime.sendMessage
Bodhi Extension (background.js)
    ↓ Message handling
Bodhi App Server
    ↓ API response
External Extension
```

## Security Architecture

### Multi-Layer Security Model

#### 1. Browser Security Layer
- **Content Security Policy**: Strict CSP in extension manifest
- **Origin Verification**: Content script validates message origins
- **Minimal Permissions**: Only necessary Chrome permissions requested
- **Interface Protection**: `window.bodhiext` is frozen and non-configurable

#### 2. OAuth2 Authentication Layer
- **Standard Protocols**: OAuth2.1 and OpenID Connect compliance
- **Token Exchange**: Secure cross-client authorization
- **Role-Based Access**: Hierarchical permission system
- **Multi-Tenant Support**: Isolated client environments

#### 3. API Authorization Layer
- **Resource Server Protection**: All API endpoints require valid tokens
- **Scope-Based Access**: Fine-grained permission control
- **Audience Validation**: Custom policies prevent unauthorized access
- **Service Account Authentication**: Secure machine-to-machine communication

### Token Management System

#### App Client Tokens (Public Clients)
- **Purpose**: User authentication for frontend applications
- **Flow**: Authorization Code with PKCE
- **Scope**: User identity and basic permissions
- **Security**: No client secrets, public client model

#### Resource Server Tokens (Confidential Clients)
- **Purpose**: API access for backend services
- **Flow**: Client credentials and token exchange
- **Scope**: Resource-specific permissions
- **Security**: Client secrets, service account enabled

#### Token Exchange Flow
```
1. User authenticates with App Client → App Token
2. Frontend requests API access → Extension
3. Extension exchanges App Token → Resource Token
4. Resource Token used for API calls → Bodhi App
5. Bodhi App validates token → Auth Server
```

## Component Integration Patterns

### 1. Extension Detection and Setup
```javascript
// Web application integration
import { isInstalled, waitForExtension } from '@bodhiapp/bodhijs';

// Detect extension
if (isInstalled()) {
  // Extension available
} else {
  // Guide user through setup
}

// Wait for extension with timeout
const state = await waitForExtension(10000);
```

### 2. API Communication
```javascript
// Chat completion with streaming
const stream = await window.bodhiext.chat.completions.create({
  model: 'llama2:7b',
  messages: [{ role: 'user', content: 'Hello!' }],
  stream: true
});

for await (const chunk of stream) {
  console.log(chunk.choices[0]?.delta?.content);
}
```

### 3. Error Handling and Fallbacks
```javascript
// Structured error handling
try {
  const response = await ping();
} catch (error) {
  if (error.code === 'EXTENSION_NOT_INSTALLED') {
    // Guide user to install extension
  } else if (error.code === 'SERVER_UNAVAILABLE') {
    // Guide user to start Bodhi App
  }
}
```

## Deployment Scenarios

### 1. Personal Desktop Use
**Components**:
- Bodhi App (Tauri desktop application)
- Browser Extension (Chrome/Edge)
- Local Keycloak instance (optional, for multi-user)

**Characteristics**:
- Single user, simplified setup
- Local model storage and inference
- Desktop-native experience
- Optional authentication for security

### 2. Development Environment
**Components**:
- Bodhi App (standalone server)
- Browser Extension (development mode)
- Bodhi.js library (local development)
- Keycloak development server

**Characteristics**:
- Hot reloading and development tools
- Test data and mock configurations
- Comprehensive logging and debugging
- Automated testing infrastructure

### 3. Team/Enterprise Deployment
**Components**:
- Bodhi App (server deployment)
- Browser Extension (enterprise distribution)
- Keycloak (production instance)
- Load balancing and monitoring

**Characteristics**:
- Multi-user authentication
- Role-based access control
- Centralized model management
- Audit logging and compliance

### 4. Web Application Integration
**Components**:
- Web application with bodhijs integration
- Browser Extension (user-installed)
- Bodhi App (user's local instance)
- Authentication flow integration

**Characteristics**:
- Seamless user experience
- Automatic setup guidance
- Fallback mechanisms
- Error recovery flows

## Development Patterns

### 1. Component Communication
- **Standardized Messages**: Common message format across all components
- **Request/Response Correlation**: Unique IDs for tracking requests
- **Error Propagation**: Structured error handling throughout the chain
- **Timeout Management**: Configurable timeouts for all operations

### 2. Configuration Management
- **Environment-Aware**: Different configurations for dev/test/prod
- **User-Configurable**: Settings UI for runtime configuration
- **Validation**: Real-time validation of configuration changes
- **Persistence**: Secure storage of configuration data

### 3. Testing Strategy
- **Unit Testing**: Individual component testing
- **Integration Testing**: Cross-component communication testing
- **UI Testing**: Browser automation with Playwright
- **Real Server Testing**: End-to-end testing with actual LLM servers

## Performance Considerations

### 1. Streaming Optimization
- **Chunked Transfer**: Efficient streaming response handling
- **Queue Management**: Proper buffering and flow control
- **Memory Management**: Cleanup of streaming resources
- **Timeout Handling**: Graceful handling of stalled streams

### 2. Connection Management
- **Connection Pooling**: Efficient HTTP connection reuse
- **Retry Logic**: Automatic retry with exponential backoff
- **Circuit Breaker**: Prevent cascading failures
- **Health Checks**: Monitoring of component health

### 3. Resource Optimization
- **Lazy Loading**: Load components only when needed
- **Caching**: Intelligent caching of responses and metadata
- **Compression**: Efficient data transfer
- **Background Processing**: Non-blocking operations

## Monitoring and Observability

### 1. Logging Strategy
- **Structured Logging**: JSON-formatted logs with consistent fields
- **Log Levels**: Appropriate log levels for different environments
- **Correlation IDs**: Request tracing across components
- **Error Context**: Rich error information for debugging

### 2. Metrics Collection
- **Performance Metrics**: Response times, throughput, error rates
- **Usage Metrics**: API usage patterns, feature adoption
- **System Metrics**: Resource utilization, health status
- **Business Metrics**: User engagement, conversion rates

### 3. Health Monitoring
- **Component Health**: Individual component status monitoring
- **End-to-End Health**: Full system health checks
- **Dependency Monitoring**: External service health tracking
- **Alerting**: Proactive issue detection and notification

## Future Evolution

### 1. Platform Expansion
- **Multi-Model Support**: Support for different LLM architectures
- **Cloud Integration**: Hybrid local/cloud model deployment
- **Mobile Support**: Mobile browser and app integration
- **API Gateway**: Centralized API management and routing

### 2. Security Enhancements
- **Zero-Trust Architecture**: Enhanced security model
- **Advanced Authentication**: Multi-factor authentication, biometrics
- **Audit and Compliance**: Enhanced logging and compliance features
- **Threat Detection**: Advanced security monitoring

### 3. Developer Experience
- **SDK Expansion**: SDKs for additional programming languages
- **Developer Tools**: Enhanced debugging and development tools
- **Documentation**: Interactive documentation and tutorials
- **Community**: Open-source community and ecosystem

## Integration Guidelines

### For Web Developers
1. **Install bodhijs**: `npm install @bodhiapp/bodhijs`
2. **Detect Extension**: Use `isInstalled()` or `waitForExtension()`
3. **Handle Setup**: Guide users through extension installation
4. **Implement Fallbacks**: Graceful degradation when platform unavailable
5. **Error Handling**: Use structured error codes for user guidance

### For Extension Developers
1. **Message Format**: Use standardized message types
2. **Extension Discovery**: Use `chrome.runtime.sendMessage` for communication
3. **Streaming Support**: Implement proper streaming message handling
4. **Error Handling**: Propagate errors with context
5. **Testing**: Use provided test extensions and patterns

### For System Administrators
1. **Authentication Setup**: Configure Keycloak for multi-user scenarios
2. **Network Configuration**: Ensure proper firewall and proxy settings
3. **Monitoring**: Implement health checks and alerting
4. **Backup**: Regular backup of configuration and user data
5. **Updates**: Establish update procedures for all components

## Related Documentation

- **[System Overview](system-overview.md)** - Bodhi App core architecture
- **[Authentication](authentication.md)** - OAuth2 security implementation
- **[API Integration](api-integration.md)** - Frontend-backend integration patterns
- **[Development Conventions](development-conventions.md)** - Coding standards and practices

## External References

- **[Bodhi Browser Extension Documentation](https://github.com/BodhiSearch/bodhi-browser/tree/main/ai-docs)** - Complete extension documentation
- **[Keycloak Bodhi Extension Documentation](https://github.com/BodhiSearch/keycloak-bodhi-ext/tree/main/ai-docs)** - Authentication server documentation
- **[Bodhi.js API Reference](https://github.com/BodhiSearch/bodhi-browser/blob/main/ai-docs/bodhijs.md)** - JavaScript library documentation

---

*This document provides a comprehensive technical overview of the Bodhi Platform architecture for AI coding assistants. For implementation details, refer to the component-specific documentation and the related architecture documents.* 