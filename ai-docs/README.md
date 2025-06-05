# BodhiApp Documentation Index

This document provides a comprehensive overview of the BodhiApp project structure, documentation, and crate organization.

## Project Overview

BodhiApp is a Rust-based application with a Tauri frontend that provides:
- Local LLM server management via llama.cpp integration
- OpenAI-compatible API endpoints
- Web-based chat interface
- Model management and downloading
- Authentication and authorization
- Multi-user support

## Documentation Structure

### Current Documentation (ai-docs/)

#### WebChat Documentation
- **webchat-uxpilot.md**: UX design specifications for web chat interface
- **webchat/PRD.md**: Product Requirements Document for minimal web chat
- **webchat/01-project-setup.md**: Initial project setup instructions
- **webchat/02-chat-interface.md**: Chat interface implementation
- **webchat/03-api-key-handling.md**: API key management
- **webchat/03.1-model-selection.md**: Model selection functionality
- **webchat/04-api-integration.md**: API integration details
- **webchat/05-theming-responsiveness.md**: Theming and responsive design
- **webchat/06-error-handling.md**: Error handling implementation
- **webchat/07-chat-settings.md**: Chat settings configuration
- **webchat/08-utility-features.md**: Additional utility features
- **webchat/09-final-integration.md**: Final integration steps
- **webchat/user-stories.md**: User stories and requirements
- **webchat/webchat-single-prompt.md**: Single prompt implementation

#### Bodhi App Frontend Documentation (crates/bodhi/ai-docs/)
- **README.md**: Frontend documentation overview
- **app-features-overview.md**: High-level feature overview
- **app-navigation.md**: Navigation and information architecture
- **app-knowledgebase.md**: Architecture and component overview
- **utoipa-docs.md**: OpenAPI documentation integration
- **app-ui-modelfile.rst**: Model file UI specifications

## Rust Workspace Structure

### Core Crates

#### 1. **objs** - Domain Objects and Types
- **Purpose**: Shared data structures, domain models, error types
- **Key Components**: 
  - Alias management
  - Chat templates
  - GGUF file handling
  - OpenAI API types
  - Error handling
  - Localization
- **Dependencies**: Core types used across all other crates

#### 2. **services** - Business Logic Layer
- **Purpose**: Core business logic and external service integrations
- **Key Components**:
  - App service (application lifecycle)
  - Auth service (authentication/authorization)
  - Cache service (caching layer)
  - Data service (data management)
  - Hub service (HuggingFace integration)
  - Init service (initialization)
  - Setting service (configuration)
- **Dependencies**: Uses objs, integrates with external APIs

#### 3. **commands** - CLI Commands
- **Purpose**: Command-line interface implementation
- **Key Components**: CLI command handlers and argument parsing
- **Dependencies**: Uses services and objs

#### 4. **server_core** - HTTP Server Core
- **Purpose**: Core HTTP server functionality
- **Key Components**: Server setup, middleware, routing foundation
- **Dependencies**: Foundation for route crates

#### 5. **auth_middleware** - Authentication Middleware
- **Purpose**: HTTP authentication and authorization middleware
- **Key Components**: JWT handling, role-based access control
- **Dependencies**: Used by route crates

### Route Crates

#### 6. **routes_oai** - OpenAI API Routes
- **Purpose**: OpenAI-compatible API endpoints
- **Key Components**: Chat completions, model endpoints
- **Dependencies**: Uses services, auth_middleware

#### 7. **routes_app** - Application API Routes  
- **Purpose**: Application-specific API endpoints
- **Key Components**: Model management, user management, app configuration
- **Dependencies**: Uses services, auth_middleware

#### 8. **routes_all** - Combined Routes
- **Purpose**: Aggregates all route modules
- **Key Components**: Route composition, OpenAPI documentation
- **Dependencies**: Combines routes_oai and routes_app

### Application Crates

#### 9. **server_app** - Standalone Server
- **Purpose**: Standalone HTTP server application
- **Key Components**: Server binary, configuration
- **Dependencies**: Uses routes_all and server_core

#### 10. **bodhi/src-tauri** - Tauri Application
- **Purpose**: Desktop application with embedded server
- **Key Components**: Tauri integration, native features
- **Dependencies**: Uses all server components

### Utility Crates

#### 11. **llama_server_proc** - LLM Server Process
- **Purpose**: llama.cpp integration and process management
- **Key Components**: Process spawning, llama.cpp bindings
- **Dependencies**: Core LLM functionality

#### 12. **errmeta_derive** - Error Metadata Derive
- **Purpose**: Procedural macros for error handling
- **Key Components**: Derive macros for error metadata
- **Dependencies**: Used by objs and services

### Development Crates

#### 13. **integration-tests** - Integration Tests
- **Purpose**: End-to-end testing
- **Key Components**: API testing, workflow testing
- **Dependencies**: Tests all components

#### 14. **xtask** - Build Tasks
- **Purpose**: Build automation and development tasks
- **Key Components**: Custom build scripts, development workflows
- **Dependencies**: Development tooling

## Frontend Structure (crates/bodhi/)

The frontend is a Vite + React + TypeScript application with:
- **src/**: React components and application logic
- **src-tauri/**: Tauri backend integration
- **public/**: Static assets
- **ai-docs/**: Frontend-specific documentation

## Key Technologies

- **Backend**: Rust, Axum, SQLx, Tokio
- **Frontend**: React, TypeScript, Vite, TailwindCSS, Shadcn UI
- **Desktop**: Tauri
- **LLM**: llama.cpp integration
- **API**: OpenAI-compatible endpoints
- **Auth**: OAuth2, JWT
- **Database**: SQLite (via SQLx)
- **Documentation**: OpenAPI/Swagger via utoipa

## Detailed Crate Documentation

### Core Foundation Crates
- **[objs](./crates/objs.md)** - Domain objects, types, and error handling
- **[services](./crates/services.md)** - Business logic and external integrations
- **[errmeta_derive](./crates/errmeta_derive.md)** - Error metadata generation macros

### HTTP Server Infrastructure
- **[server_core](./crates/server_core.md)** - HTTP server foundation and utilities
- **[auth_middleware](./crates/auth_middleware.md)** - Authentication and authorization
- **[routes_oai](./crates/routes_oai.md)** - OpenAI-compatible API endpoints
- **[routes_app](./crates/routes_app.md)** - Application-specific API endpoints
- **[routes_all](./crates/routes_all.md)** - Unified route composition

### Application Layers
- **[server_app](./crates/server_app.md)** - Standalone HTTP server application
- **[bodhi-tauri](./crates/bodhi-tauri.md)** - Tauri desktop application
- **[commands](./crates/commands.md)** - CLI command interface

### Specialized Components
- **[llama_server_proc](./crates/llama_server_proc.md)** - LLM server process management
- **[integration-tests](./crates/integration-tests.md)** - End-to-end testing
- **[xtask](./crates/xtask.md)** - Build automation and development tasks

## Architecture Patterns

1. **Layered Architecture**: Clear separation between routes, services, and domain objects
2. **Dependency Injection**: Services are injected into route handlers
3. **Error Handling**: Centralized error types with metadata
4. **API-First**: OpenAPI documentation generated from code
5. **Modular Design**: Each crate has a specific responsibility
6. **Test-Driven**: Comprehensive testing at multiple levels
