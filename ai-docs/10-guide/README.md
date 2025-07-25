# Implementation Guides & Context Documents

This directory contains practical implementation guides and context documents designed specifically for AI coding assistants working with the BodhiApp project.

## Purpose

These documents provide comprehensive context about specific areas of the codebase, including:
- Current state and conventions
- Architectural patterns and decisions
- Best practices and preferences
- Implementation peculiarities and gotchas

## Current Guides

### CI/CD & DevOps
- **[GitHub Workflows Context](github-workflows-context.md)** - Complete context about the GitHub Actions CI/CD system, including workflow architecture, reusable actions, platform-specific considerations, and maintenance patterns

## Guide Format

Context documents in this directory follow these principles:
- **Comprehensive Coverage**: Provide complete picture of the current state
- **AI-Focused**: Written specifically for AI coding assistant consumption
- **Implementation-Oriented**: Focus on practical patterns and conventions
- **Current State**: Reflect the actual implementation, not idealized versions
- **Maintenance Context**: Include rationale for architectural decisions

## Adding New Guides

When adding new context documents:
1. Focus on areas with complex conventions or non-obvious patterns
2. Include both current state and historical context where relevant
3. Document peculiarities and edge cases that might not be obvious
4. Update this README with appropriate categorization
5. Reference from the main ai-docs README.md

## Categories for Future Guides

Potential areas for future context documents:
- **Database & Migrations**: Database schema patterns, migration strategies
- **Authentication Integration**: OAuth2 flows, token management patterns
- **Frontend State Management**: React Query patterns, state synchronization
- **Native Integration**: Tauri-specific patterns, OS integration
- **Testing Strategies**: Test organization, mocking patterns, integration appro