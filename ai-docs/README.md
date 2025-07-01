# Bodhi App Documentation Index

## Overview

This documentation index provides comprehensive navigation for the Bodhi App documentation system. The documentation is organized into logical sections to help you quickly find the specific information you need.

## Documentation Organization

The documentation is structured into the following main sections:
- **🏗️ Architecture** - Technical foundation, design system, and development standards
- **⚡ Features** - Current and planned application capabilities
- **🔧 Crates** - Individual Rust crate documentation
- **📖 Reference** - Technical reference materials
- **📢 Marketing** - Product positioning and community outreach
- **📚 Knowledge Transfer** - Implementation guides and tutorials
- **🔬 Research** - Technical research and architectural analysis
- **📦 Archive** - Historical materials and deprecated content

### Recent Documentation Updates

**Architecture Content Recovery (January 2025)**: The architecture documentation has been comprehensively updated to recover valuable knowledge that was lost during documentation reorganization. Key recovered content includes:

- **Enhanced System Overview**: Restored key features section with OpenAI/Ollama compatibility details, authentication flow diagrams, token system explanations, and model aliases concept
- **New Architectural Decisions Document**: Captures rationale for key design choices including embedded web server architecture, configuration hierarchy, and technology trade-offs
- **Strategic Roadmap**: Comprehensive future improvements plan including microservices evolution, performance enhancements, monitoring & observability, and security improvements
- **Enhanced Backend Documentation**: Added real-time communication patterns, logging guidance, and settings service architecture
- **Improved API Integration**: Added local storage patterns, implementation anomalies documentation, and comprehensive best practices summary
- **Verification & References**: Restored source code references and verification status for improved documentation reliability

## Quick Navigation

### 🏗️ [Architecture](01-architecture/) - Technical Foundation & Development Standards
Modular technical architecture documentation organized by technology stack and development concerns for efficient discovery and focused guidance. This section has been comprehensively updated to recover valuable architectural knowledge and strategic context that was previously lost during documentation reorganization.

#### Core System Architecture
- **[System Overview](01-architecture/system-overview.md)** - High-level application architecture, crate organization, data flows, key features (including OpenAI/Ollama compatibility), authentication flows, token system, and model aliases
- **[App Status & Lifecycle](01-architecture/app-status.md)** - Application state machine, status transitions, and lifecycle management
- **[Architectural Decisions](01-architecture/architectural-decisions.md)** - Key architectural decisions, rationale, and design patterns including embedded web server choice, configuration hierarchy, and technology trade-offs
- **[Roadmap](01-architecture/roadmap.md)** - Strategic direction, planned improvements, and future architecture evolution including microservices, performance enhancements, and security improvements

#### Technology Stack Guides
- **[Frontend Next.js](01-architecture/frontend-react.md)** - Next.js v14 development patterns, "dumb frontend" architecture, component structure, TypeScript conventions, and backend-driven validation patterns
- **[Rust Backend](01-architecture/rust-backend.md)** - Rust backend development, service patterns, database integration, real-time communication (SSE/WebSocket), logging & observability, and settings service architecture
- **[Backend OpenAPI](01-architecture/backend-openapi-utoipa.md)** - OpenAPI documentation with utoipa, API tag constants, authentication patterns, and project-specific conventions
- **[Backend Settings Service](01-architecture/backend-settings-service.md)** - Comprehensive settings architecture with cascaded configuration, environment-specific settings, API integration, and testing infrastructure
- **[Backend Error & L10n](01-architecture/backend-error-l10n.md)** - Error handling and internationalization system using Fluent, custom ErrorMeta macro, and singleton localization service
- **[Tauri Desktop](01-architecture/tauri-desktop.md)** - Desktop application architecture and native OS integration
- **[Authentication](01-architecture/authentication.md)** - OAuth2 integration, JWT handling, and security implementation

#### Development Standards
- **[API Integration](01-architecture/api-integration.md)** - Frontend-backend integration patterns, "dumb frontend" principles, OAuth callback examples, query hooks, error handling, and best practices summary
- **[Development Conventions](01-architecture/development-conventions.md)** - Coding standards, naming conventions, best practices, verification status, and source code references
- **[UI Design System](01-architecture/ui-design-system.md)** - Design system foundations, component library, and visual consistency

#### Quality Assurance
- **[Testing Strategy](01-architecture/testing-strategy.md)** - High-level testing approach and quality assurance strategy
- **[Frontend Testing](01-architecture/frontend-testing.md)** - Frontend testing patterns, React components, and user interactions
- **[Backend Testing](01-architecture/backend-testing.md)** - Backend testing approaches, database testing, and API integration
- **[Backend Testing Utils](01-architecture/backend-testing-utils.md)** - Test utilities pattern with feature flags for cross-crate test object sharing
- **[Build & Configuration](01-architecture/build-config.md)** - Build systems, configuration management, and deployment patterns

#### Reference Documents
- **[Architecture Summary](01-architecture/ARCHITECTURE_SUMMARY.md)** - High-level architectural overview and key decisions
- **[Testing Guide](01-architecture/TESTING_GUIDE.md)** - Comprehensive testing guidelines and best practices
- **[AI IDE Memories](01-architecture/ai-ide-memories.md)** - Refined and consistent memories for AI IDE interactions with the BodhiApp project

### ⚡ [Features](02-features/) - Application Capabilities
Feature documentation organized by development status and implementation timeline

#### Implemented Features
- **[Chat Interface](02-features/implemented/chat-interface.md)** - Real-time chat interface with streaming responses and model interaction
- **[Model Management](02-features/implemented/model-management.md)** - Model alias system, configuration management, and model lifecycle
- **[Authentication](02-features/implemented/authentication.md)** - User authentication flows, session management, and security features
- **[Navigation](02-features/implemented/navigation.md)** - Application navigation system and routing architecture

#### Active Development Stories
- **[OAuth 2.0 Token Exchange](02-features/20250628-token-exchange.md)** - Cross-client token validation with Keycloak integration for enhanced security and interoperability
  - **[Technical Specification](02-features/20250628-token-exchange-tech-spec.md)** - Detailed technical implementation guide with code patterns and testing strategies
- **[API Tokens](02-features/active-stories/api-tokens.md)** - API token management and programmatic access
- **[App Settings](02-features/active-stories/app-settings.md)** - Application configuration and user preferences
- **[Model Alias Revamp](02-features/active-stories/model-alias-revamp.md)** - Enhanced model alias system and management
- **[Modelfiles Revamp](02-features/active-stories/modelfiles-revamp.md)** - Improved model file handling and configuration
- **[Setup Wizard](02-features/active-stories/setup-wizard.md)** - Initial application setup and onboarding flow
- **[User Roles](02-features/active-stories/story-20250112-user-roles.md)** - Role-based access control implementation

#### Completed Development Stories
- **[API Authorization](02-features/completed-stories/story-20250116-api-authorization.md)** - API authorization system implementation
- **[API Authorization Tests](02-features/completed-stories/story-20250116-api-authorization-tests.md)** - Test coverage for authorization features
- **[API Documentation](02-features/completed-stories/story-20250119-api-docs.md)** - OpenAPI documentation generation and tooling
- **[Download Llama Server](02-features/completed-stories/story-20250126-download-llama-server.md)** - Llama server download and installation
- **[Setup Auth Mode](02-features/completed-stories/story-20250130-setup-auth-mode.md)** - Authentication mode configuration
- **[Setup Bodhi Info](02-features/completed-stories/story-20250130-setup-bodhi-info.md)** - Application information setup
- **[Setup Finish](02-features/completed-stories/story-20250130-setup-finish.md)** - Setup completion and finalization
- **[Setup LLM Engine](02-features/completed-stories/story-20250130-setup-llm-engine.md)** - LLM engine configuration
- **[Setup Model Download](02-features/completed-stories/story-20250130-setup-model-download.md)** - Model download during setup
- **[Setup Resource Admin](02-features/completed-stories/story-20250130-setup-resource-admin.md)** - Resource server admin setup
- **[Login Info Non-Authz](02-features/completed-stories/story-authz-20250111-login-info-non-authz.md)** - Non-authenticated mode login information
- **[Reset to Authz](02-features/completed-stories/story-authz-20250111-reset-to-authz.md)** - Authentication mode reset functionality
- **[App Initialization Refactoring](02-features/completed-stories/20250614-app-initialization-refactoring.md)** - CLI-first architecture refactoring to parse command line arguments before app initialization, enabling command-specific initialization path
- **[NPM Dependency Upgrade](02-features/completed-stories/20250613-npm-dependency-upgrade.md)** - Comprehensive strategy for safely upgrading npm dependencies in the BodhiApp frontend, including risk-based batching, testing procedures, and rollback plan
- **[App Setup Refactoring](02-features/completed-stories/20250610-lib-bodhiserver.md)** - Technical specification for `lib_bodhiserver` library crate to centralize initialization logic, eliminate code duplication between production and test paths, and e
- **[NAPI-RS FFI Implementation Completion](02-features/completed-stories/20250616-napi-ffi-implementation-completion.md)** - Completion documentation for NAPI-RS FFI layer with embedded frontend assets and working Playwright UI testing

#### Planned Features
- **[Enhanced Configuration Management](02-features/planned/20250616-enhanced-configuration-management.md)** - Flexible configuration management for lib_bodhiserver to eliminate hardcoded values and provide comprehensive builder pattern for FFI clients
- **[NAPI-RS FFI API Testing](02-features/completed-stories/20250615-napi-ffi-ui-testing.md)** - Implementation specification for NAPI-RS based FFI layer to expose lib_bodhiserver functionality for TypeScript/JavaScript API testing with programmatic server control (100% Complete)
- **[NAPI-RS FFI Playwright UI Testing](02-features/planned/20250615-napi-ffi-playwright-ui-testing.md)** - Implementation specification for enabling Playwright UI testing through NAPI-RS FFI by resolving static asset serving issues and implementing comprehensive browser-based test scenarios
- **[AppRegInfo JWT Simplification](02-features/planned/appreginfo-jwt-simplification.md)** - Remove unused JWT validation fields from AppRegInfo and implement runtime fetching of JWT validation parameters from Keycloak well-known endpoints
- **[Remove Non-Authenticated Mode](02-features/planned/remove-non-authenticated-mode.md)** - Simplify app by removing non-authenticated installation option, requiring OAuth2 for all installations
- **[User Management](02-features/planned/user-management.md)** - Multi-user support, user administration, and role management
- **[Remote Models](02-features/planned/remote-models.md)** - Remote model integration, cloud sync, and distributed inference

### 🔧 [Crates](03-crates/) - Individual Rust Crate Documentation
Detailed documentation for each Rust crate in the workspace, covering implementation details and APIs

#### Foundation Crates
- **[objs](03-crates/objs.md)** - Domain objects, types, error handling, and validation logic
- **[services](03-crates/services.md)** - Business logic layer and external service integrations
- **[server_core](03-crates/server_core.md)** - HTTP server infrastructure and core functionality
- **[auth_middleware](03-crates/auth_middleware.md)** - Authentication and authorization middleware

#### API Route Crates
- **[routes_oai](03-crates/routes_oai.md)** - OpenAI-compatible API endpoints and handlers
- **[routes_app](03-crates/routes_app.md)** - Application-specific API endpoints and handlers
- **[routes_all](03-crates/routes_all.md)** - Unified route composition and aggregation

#### Application Crates
- **[server_app](03-crates/server_app.md)** - Standalone HTTP server application
- **[bodhi-tauri](03-crates/bodhi-tauri.md)** - Tauri desktop application integration
- **[commands](03-crates/commands.md)** - Command-line interface implementation

#### Utility Crates
- **[llama_server_proc](03-crates/llama_server_proc.md)** - LLM process management and llama.cpp integration
- **[errmeta_derive](03-crates/errmeta_derive.md)** - Procedural macros for error metadata
- **[integration-tests](03-crates/integration-tests.md)** - End-to-end testing framework
- **[xtask](03-crates/xtask.md)** - Build automation and development tasks

### 📖 [Reference](04-reference/) - Technical Reference Materials
Technical reference documentation and external tool integration guides

*Currently being reorganized - content moved to appropriate architecture sections*

### 📢 [Marketing](05-marketing/) - Product Marketing
Marketing materials, community outreach, and promotional content for product positioning

- **[Product Positioning](05-marketing/product-positioning.md)** - Product messaging, unique selling propositions, and target audience analysis
- **[Launch Materials](05-marketing/launch-materials.md)** - Product Hunt campaign content and launch strategy materials
- **[Community Outreach](05-marketing/community-outreach.md)** - Community engagement strategies and outreach plans
- **[Presentations](05-marketing/presentations.md)** - Conference presentations and speaking engagement materials
- **[WhatsApp Intro](05-marketing/whatsapp-intro.md)** - Community introduction templates and messaging
- **[Product Hunt](05-marketing/product-hunt.txt)** - Product Hunt submission content and strategy

### 📚 [Knowledge Transfer](06-knowledge-transfer/) - Learning Resources
Implementation guides, technical knowledge, and learning resources for developers and users

- **[LLM Resource Server](06-knowledge-transfer/llm-resource-server.md)** - OAuth2 resource server architecture, vision, and implementation guide
- **[Chat UI](06-knowledge-transfer/chat-ui.md)** - Chat interface implementation patterns and user experience design
- **[Model Parameters](06-knowledge-transfer/model-parameters.md)** - Model configuration, parameter management, and optimization guides
- **[Setup Processes](06-knowledge-transfer/setup-processes.md)** - Application installation, setup procedures, and configuration workflows
- **[Rust Dependency Management](06-knowledge-transfer/unused-upgrade-dependencies.md)** - Comprehensive guide for Rust workspace dependency management, including unused dependency removal, systematic upgrade strategies, and handling major version blockers
- **[NPM Dependency Upgrades](02-features/completed-stories/20250613-npm-dependency-upgrade.md)** - Strategic approach to upgrading npm dependencies in the frontend, with risk-based batching and testing procedures
- **[FFI UI Testing Research](07-research/20250615-ffi-ui-testing-research.md)** - Comprehensive analysis of FFI approaches for exposing lib_bodhiserver to TypeScript/JavaScript for UI testing, with NAPI-RS recommendation and dependency isolation integration
- **[NAPI-RS FFI Implementation](06-knowledge-transfer/20250615-napi-ffi-ui-testing.md)** - NAPI-RS FFI implementation guide and handoff documentation
- **[NAPI-RS FFI Continuation](06-knowledge-transfer/20250616-napi-ffi-continuation-prompt.md)** - Continuation prompt for NAPI-RS FFI technical debt resolution and production readiness

### 🔬 [Research](07-research/) - Technical Research & Analysis
Technical research documents, dependency analysis, and architectural investigations

- **[AppRegInfo JWT Simplification Analysis](07-research/appreginfo-jwt-simplification-analysis.md)** - Comprehensive analysis of JWT validation fields in AppRegInfo, revealing unused dead code and proposing runtime parameter fetching architecture
- **[OAuth 2.1 Token Exchange Security Research](07-research/token-exchange.md)** - Research on secure token exchange patterns for preventing privilege escalation when third-party clients access our resource server, with scope-limited exchange recommendations

### 📦 [Archive](99-archive/) - Historical Materials
Historical documents, deprecated content, and reference materials preserved for context

- **[Archive README](99-archive/README.md)** - Archive organization, purpose, and content overview
- **[Samples](99-archive/samples/)** - Historical code samples and examples (directory structure preserved)

## Finding Specific Information

### By User Type

#### For Developers
1. **Getting Started**: [System Overview](01-architecture/system-overview.md) → [Frontend Next.js](01-architecture/frontend-react.md) → [Development Conventions](01-architecture/development-conventions.md)
2. **API Integration**: [API Integration](01-architecture/api-integration.md) → [Rust Backend](01-architecture/rust-backend.md)
3. **Architecture Understanding**: [Architectural Decisions](01-architecture/architectural-decisions.md) → [Roadmap](01-architecture/roadmap.md)
4. **Technical Details**: [System Overview](01-architecture/system-overview.md) → [Crates Documentation](03-crates/)
5. **Current Work**: [Active Stories](02-features/active-stories/) → [Frontend Testing](01-architecture/frontend-testing.md)

#### For Designers
1. **Design System**: [UI Design System](01-architecture/ui-design-system.md) → [Frontend Next.js](01-architecture/frontend-react.md)
2. **User Experience**: [System Overview](01-architecture/system-overview.md) → [Chat UI](06-knowledge-transfer/chat-ui.md)
3. **Implementation**: [Implemented Features](02-features/implemented/) → [Active Stories](02-features/active-stories/)

#### For Product Managers
1. **Product Understanding**: [System Overview](01-architecture/system-overview.md) → [Features Overview](02-features/)
2. **Current Capabilities**: [Implemented Features](02-features/implemented/) → [Completed Stories](02-features/completed-stories/)
3. **Strategic Direction**: [Roadmap](01-architecture/roadmap.md) → [Architectural Decisions](01-architecture/architectural-decisions.md)
4. **Feature Planning**: [Planned Features](02-features/planned/) → [Active Stories](02-features/active-stories/)
5. **Marketing**: [Product Positioning](05-marketing/product-positioning.md) → [Marketing Materials](05-marketing/)

#### For Users
1. **Setup**: [Setup Processes](06-knowledge-transfer/setup-processes.md) → [Setup Wizard](02-features/active-stories/setup-wizard.md)
2. **Usage**: [Chat UI](06-knowledge-transfer/chat-ui.md) → [Model Parameters](06-knowledge-transfer/model-parameters.md)
3. **Configuration**: [App Settings](02-features/active-stories/app-settings.md) → [Authentication](02-features/implemented/authentication.md)

### By Topic

#### Architecture & Strategic Planning
- **System Design**: [System Overview](01-architecture/system-overview.md) → [Architectural Decisions](01-architecture/architectural-decisions.md)
- **Strategic Direction**: [Roadmap](01-architecture/roadmap.md) - Future improvements and evolution plans
- **Key Features**: [System Overview](01-architecture/system-overview.md) - OpenAI/Ollama compatibility, token system, model aliases
- **Technology Choices**: [Architectural Decisions](01-architecture/architectural-decisions.md) - Rationale for embedded web server, SSE over WebSockets, multi-crate architecture

#### Authentication & Security
- **Architecture**: [Authentication](01-architecture/authentication.md)
- **Implementation**: [Authentication Features](02-features/implemented/authentication.md)
- **OAuth2 Details**: [LLM Resource Server](06-knowledge-transfer/llm-resource-server.md)
- **API Access**: [API Tokens](02-features/active-stories/api-tokens.md)

#### Model Management
- **Current System**: [Model Management](02-features/implemented/model-management.md)
- **Configuration**: [Model Parameters](06-knowledge-transfer/model-parameters.md)
- **Improvements**: [Model Alias Revamp](02-features/active-stories/model-alias-revamp.md)
- **File Handling**: [Modelfiles Revamp](02-features/active-stories/modelfiles-revamp.md)

#### API Integration & Data Management
- **Frontend Patterns**: [API Integration](01-architecture/api-integration.md)
- **Backend Services**: [Rust Backend](01-architecture/rust-backend.md)
- **Authentication**: [Authentication](01-architecture/authentication.md)
- **Development Standards**: [Development Conventions](01-architecture/development-conventions.md)

#### Chat Interface
- **Implementation**: [Chat Interface](02-features/implemented/chat-interface.md)
- **User Experience**: [Chat UI](06-knowledge-transfer/chat-ui.md)
- **Technical Details**: [Frontend Next.js](01-architecture/frontend-react.md)

#### Setup & Configuration
- **Setup Process**: [Setup Processes](06-knowledge-transfer/setup-processes.md)
- **Setup Wizard**: [Setup Wizard](02-features/active-stories/setup-wizard.md)
- **App Settings**: [App Settings](02-features/active-stories/app-settings.md)
- **Completed Setup Stories**: [Completed Stories](02-features/completed-stories/)

#### Development & Testing
- **Architecture**: [System Overview](01-architecture/system-overview.md) → [Testing Strategy](01-architecture/testing-strategy.md)
- **API Integration**: [API Integration](01-architecture/api-integration.md) → [Rust Backend](01-architecture/rust-backend.md)
- **Conventions**: [Development Conventions](01-architecture/development-conventions.md)
- **Testing**: [Testing Strategy](01-architecture/testing-strategy.md) → [Frontend Testing](01-architecture/frontend-testing.md) → [Backend Testing](01-architecture/backend-testing.md)
- **Test Utilities**: [Backend Testing Utils](01-architecture/backend-testing-utils.md) - Cross-crate test object sharing with feature flags
- **Build & Deploy**: [Build & Configuration](01-architecture/build-config.md)
- **Crate Details**: [Individual Crate Docs](03-crates/)

## Navigation Tips

### Quick Reference
- **Search within documents**: Use Ctrl/Cmd+F to find specific topics
- **Cross-references**: Follow links between documents for comprehensive understanding
- **Section READMEs**: Each main section has a README with detailed navigation
- **Multiple sections**: Topics may span categories, check related sections

### Documentation Conventions
- **File naming**: kebab-case for consistency (e.g., `model-management.md`)
- **Cross-references**: Internal links use relative paths
- **Status tracking**: Development stories track implementation progress
- **Hierarchical organization**: Logical grouping by purpose and audience

## Contributing to Documentation

When adding or updating documentation:

1. **Choose the appropriate section** based on content type and target audience
2. **Follow naming conventions** (kebab-case for files, descriptive names)
3. **Update this index** when adding new documents or changing structure
4. **Include cross-references** to related documents for comprehensive coverage
5. **Consider consolidation** opportunities to avoid duplication

### Content Guidelines
- **Architecture**: Technical implementation details, system design, development standards
- **Features**: User-facing capabilities, development stories, implementation status
- **Crates**: Individual Rust crate documentation and API details
- **Reference**: Technical reference materials and tool integration guides
- **Marketing**: Product positioning, community outreach, promotional materials
- **Knowledge Transfer**: Learning resources, guides, and implementation patterns
- **Research**: Technical research documents, dependency analysis, and architectural investigations
- **Archive**: Historical materials and deprecated content

### Documentation Update Rules
- **Feature Specifications (`ai-docs/02-features/`)**: Historical documents that reflect what was true at the time of creation. **Do not update these after creation** - they serve as historical records of requirements and implementation plans.
- **Architecture Documentation (`ai-docs/01-architecture/`)**: Living documents that reflect the current truth for the git commit. **Update these documents** when the architecture, patterns, or implementation details change to maintain accuracy.

## Support and Feedback

### Getting Help
- **Section READMEs**: Each main section contains detailed navigation and context
- **Cross-references**: Follow document links for comprehensive understanding
- **GitHub Issues**: Report documentation issues or suggest improvements
- **Community**: Engage with the development community for questions and discussions

### Maintenance
This index is maintained to reflect the current state of the documentation system. When the structure changes, this navigation guide is updated to ensure accurate and helpful guidance for all users.

---

*This documentation index provides comprehensive navigation for the Bodhi App documentation system, organized into logical sections for efficient information discovery and access.*
