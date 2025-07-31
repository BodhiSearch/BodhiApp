# Context Documentation

This directory contains contextual documentation that provides background, implementation details, and architectural context for specific features and technical decisions.

## Purpose

Context documents serve to:
- Capture implementation decisions and rationale
- Provide technical background for complex features
- Document integration patterns and dependencies
- Support knowledge transfer and onboarding

## Current Context Documents

### Authentication & OAuth
- **[OAuth 2.0 Token Exchange - Auth Service](oauth2-token-exchange-auth-service-context.md)** - Context for OAuth 2.0 token exchange implementation changes, dynamic audience management, and incremental development approach

### Integration & Testing
- **[Testcontainers Keycloak Integration](testcontainers-keycloak-integration-context.md)** - Context for integrating Testcontainers with Keycloak for comprehensive authentication testing, including Docker setup, test isolation, and CI/CD considerations

### CI/CD & DevOps
- **[GitHub Workflows Context](github-workflows-context.md)** - Complete context about the GitHub Actions CI/CD system, including workflow architecture, reusable actions, platform-specific considerations, and maintenance patterns

## Document Guidelines

### When to Create Context Documents
- Complex feature implementations requiring background knowledge
- Integration patterns with external systems
- Technical decisions with long-term implications
- Migration or refactoring processes

### Content Structure
- **Overview**: Brief description and purpose
- **Implementation Details**: Technical specifics and patterns
- **Design Decisions**: Rationale and alternatives considered
- **Dependencies**: Upstream and downstream relationships
- **Testing Strategy**: Approach and considerations
- **Migration Notes**: Breaking changes and compatibility

### Maintenance
- Update documents when implementation changes
- Archive outdated context to prevent confusion
- Reference from related documentation for discoverability 