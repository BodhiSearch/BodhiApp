# Architecture Documentation

This section contains modular technical architecture documentation for the Bodhi App, organized by technology stack and development concerns for efficient discovery and focused guidance.

## Contents

### Core System Architecture
- **[System Overview](system-overview.md)** - High-level application architecture, crate organization, and data flows
- **[App Status & Lifecycle](app-status.md)** - Application state machine, status transitions, and lifecycle management
- **[Architectural Decisions](architectural-decisions.md)** - Key architectural decisions, rationale, and design patterns
- **[Roadmap](roadmap.md)** - Strategic direction, planned improvements, and future architecture evolution

### Technology Stack Guides
- **[Frontend Next.js](frontend-react.md)** - Next.js v14 development patterns, "dumb frontend" architecture, component structure, TypeScript conventions, and backend-driven validation
- **[Rust Backend](rust-backend.md)** - Rust backend development, service patterns, and database integration
- **[Tauri Desktop](tauri-desktop.md)** - Desktop application architecture and native OS integration
- **[Authentication](authentication.md)** - OAuth2 integration, JWT handling, and security implementation
- **[Backend Error & L10n](01-architecture/backend-error-l10n.md)** - Error handling and internationalization system using Fluent, custom ErrorMeta macro, and singleton localization service

### Development Standards
- **[API Integration](api-integration.md)** - Frontend-backend integration patterns, "dumb frontend" principles, OAuth callback examples, query hooks, and error handling
- **[Development Conventions](development-conventions.md)** - Coding standards, naming conventions, and best practices
- **[UI Design System](ui-design-system.md)** - Design system foundations, component library, and visual consistency

### Quality Assurance
- **[Testing Strategy](testing-strategy.md)** - High-level testing approach and quality assurance strategy
- **[Frontend Testing](frontend-testing.md)** - Frontend testing patterns, React components, and user interactions
- **[Backend Testing](backend-testing.md)** - Backend testing approaches, database testing, and API integration
- **[Build & Configuration](build-config.md)** - Build systems, configuration management, and deployment patterns

## Architecture Principles

### Modular Design
- **Technology-Focused Modules** - Each guide focuses on specific technology stack concerns
- **Separation of Concerns** - Clear boundaries between system design, implementation, and standards
- **Documentation-Driven Development** - Architecture guides reference detailed implementation docs
- **Focused Guidance** - Targeted information for specific development contexts

### System Design Philosophy
- **Rust-First Backend** - Type-safe, performant backend services with clear service boundaries
- **"Dumb Frontend" Architecture** - Frontend focuses on presentation, backend handles all business logic and validation
- **Next.js+TypeScript Frontend** - Modern, full-stack React framework with strong typing and SSG capabilities
- **Desktop-Native Integration** - Tauri-based desktop app with native OS capabilities
- **Security by Design** - OAuth2, JWT, and role-based access control throughout
- **Developer Experience** - Clear patterns, consistent conventions, and comprehensive tooling

## Technology Stack Overview

### Frontend Technologies
- **React 18+ with TypeScript** - Component-based UI with strong typing
- **Next.js v14.2.6** - Full-stack React framework with SSG capabilities
- **Tailwind CSS + Shadcn/ui** - Utility-first styling with component library
- **React Query v3.39.3** - Data fetching, caching, and synchronization
- **Next.js App Router** - File-based routing and navigation

### Backend Technologies
- **Rust** - Systems programming language for performance and safety
- **SQLx** - Async SQL toolkit with compile-time checked queries
- **Tokio** - Async runtime for concurrent operations
- **Axum** - Web framework for HTTP APIs
- **SQLite** - Embedded database for local data storage

### Desktop Integration
- **Tauri** - Rust-based desktop application framework
- **Native OS APIs** - File system, notifications, and system integration
- **WebView** - Modern web technologies in native desktop shell

### Development & Quality Tools
- **Vitest** - Fast unit testing framework
- **ESLint + Prettier** - Code quality and formatting
- **MSW** - API mocking for reliable tests
- **Cargo** - Rust package manager and build system
- **PWA Support** - Progressive Web App capabilities with @ducanh2912/next-pwa

## Quick Reference Guide

### For Frontend Development
1. **Start with**: [Frontend Next.js](frontend-react.md) - "Dumb frontend" architecture, Next.js patterns, and TypeScript conventions
2. **API Integration**: [API Integration](api-integration.md) - Backend-driven patterns, OAuth examples, query hooks, and error handling
3. **UI Components**: [UI Design System](ui-design-system.md) - Design tokens and component usage
4. **Testing**: [Testing Strategy](testing-strategy.md) - Frontend testing patterns

### For Backend Development
1. **Start with**: [Rust Backend](rust-backend.md) - Service patterns and database integration
2. **Authentication**: [Authentication](authentication.md) - OAuth2 and security implementation
3. **API Design**: [API Integration](api-integration.md) - Endpoint patterns and error handling
4. **Testing**: [Testing Strategy](testing-strategy.md) - Backend testing approaches

### For Desktop Development
1. **Start with**: [Tauri Desktop](tauri-desktop.md) - Native integration patterns
2. **System Integration**: [System Overview](system-overview.md) - Architecture and data flows
3. **Build Process**: [Build & Configuration](build-config.md) - Desktop build and packaging

### For System Understanding
1. **Architecture**: [System Overview](system-overview.md) - High-level system design
2. **Application Lifecycle**: [App Status & Lifecycle](app-status.md) - State management
3. **Standards**: [Development Conventions](development-conventions.md) - Coding standards
4. **Error**: [Backend Error & L10n](backend-error-l10n.md) - Error handling and internationalization system

## Documentation Organization

### Modular Architecture Approach
Each architecture document focuses on a specific technology stack or development concern:

- **Technology-Specific Guides** - Focused guidance for React, Rust, Tauri development
- **Integration Patterns** - How different parts of the system work together
- **Development Standards** - Consistent conventions and best practices
- **Quality Assurance** - Testing, security, and performance considerations

### Cross-References
Architecture documents reference each other and point to detailed implementation guides in other sections:
- **[Features](../02-features/)** - Feature-specific implementation details
- **[Crates](../03-crates/)** - Individual Rust crate documentation
- **[Knowledge Transfer](../06-knowledge-transfer/)** - Implementation guides and tutorials

## Contributing to Architecture Documentation

### Adding New Architecture Documents
1. **Choose appropriate focus** - Technology stack, integration pattern, or development standard
2. **Follow modular approach** - Keep documents focused and cross-reference related content
3. **Update this README** - Add new documents to appropriate sections
4. **Reference implementation details** - Link to specific crates, features, or guides

### Content Guidelines
- **Focus on "why" and "what"** rather than detailed "how" (save that for implementation docs)
- **Include architectural decisions** and rationale behind technology choices
- **Cross-reference related documents** for comprehensive understanding
- **Keep examples high-level** - detailed code examples belong in implementation docs

### Maintenance
Architecture documentation should be updated when:
- Technology stack changes or major versions are adopted
- Architectural patterns or design decisions change
- New integration patterns are established
- Development standards or conventions evolve

---

*This modular architecture documentation provides focused, technology-specific guidance for understanding and developing the Bodhi App system.*
