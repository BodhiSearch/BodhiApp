# Implementation Guides & Context Documents

This directory contains practical implementation guides documents designed specifically for AI coding assistants. The guide is supposed to be distributed independently of the source code, and allow 3rd projects to integrate with Bodhi App.

## Purpose

These documents provide comprehensive context about specific areas of the codebase, including:
- Current state and conventions
- Architectural patterns and decisions
- Best practices and preferences
- Implementation peculiarities and gotchas

## Current Guides

### [BodhiApp API Integration Guide](bodhiapp-ai-integration-guide.md)
Comprehensive guide for integrating with BodhiApp APIs, covering authentication, endpoints, and usage patterns. Useful for 3rd party projects that need to interact with BodhiApp APIs.

### [NAPI Bindings Integration Guide](app-bindings-guide.md)
Complete guide for embedding BodhiApp server functionality using the `@bodhiapp/app-bindings` NAPI library. Covers installation, configuration, server lifecycle management, and integration testing patterns for 3rd party applications that need programmatic server control.

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
2. Include current state and not go detailing of legacy 
3. Document peculiarities and edge cases that might not be obvious
4. Update this README with appropriate categorization
5. Reference from the main ai-docs README.md

## Categories for Future Guides

Potential areas for future context documents:
- **Native Integration**: Integrating Bodhi App with natively running 3rd party apps
- **Web AI**: Integrating Bodhi App APIs via companion chrome extension