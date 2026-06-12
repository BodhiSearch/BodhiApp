# OAuth 2.0 Token Exchange Implementation Prompt

## Overview

You are tasked with implementing OAuth 2.0 Token Exchange (RFC 8693) for cross-client token validation in our Rust/Axum backend with Keycloak integration. This is a **security-critical feature** that requires careful attention to token validation, cache security, and proper error handling.

## Primary Specification Document

**IMPORTANT**: Use the functional specification as your primary guidance document:
- **[Functional Specification](02-features/20250628-token-exchange.md)** - Your primary reference for requirements, user stories, and testing patterns

This document contains all the business requirements, acceptance criteria, testing philosophy, and implementation approach you need. Do NOT reference the technical specification - instead, use Test-Driven Development (TDD) to discover the best technical implementation based on the functional requirements.

## Implementation Approach: Test-Driven Development

**Start with Tests First**: Begin by implementing comprehensive unit tests based on the functional requirements, then implement the code to make those tests pass. This approach will help you discover the optimal technical architecture while ensuring robust test coverage.

### Testing Philosophy (Critical)

The application has unique testing patterns that you MUST follow:

1. **Realistic JWT Testing**: Use actual RSA public/private key pairs for token generation, not simple string mocks
2. **Claims-Based Validation**: Test JWT claims (issuer, expiration, audience, authorized party) thoroughly
3. **Comprehensive Mock Patterns**: Use `mockall` for sophisticated service mocking with precise parameter validation
4. **Database Integration**: Use real database connections with test databases for token service testing
5. **Security-Focused Testing**: Simulate attack scenarios and boundary conditions
6. **Multi-Scenario Coverage**: Test valid, expired, malformed, and cross-client tokens

## Phase 1 Implementation Priority: Core Infrastructure

Start with Phase 1 as outlined in the functional specification:

### 1.1 Enhanced Error Types
- Implement new error variants for cross-client validation scenarios
- Focus on **TokenExpired** error with proper 401 HTTP response mapping
- Include expiry timestamp information for client token refresh
- Ensure error responses follow the specified JSON format

### 1.2 Token Expiry Validation (Critical Security Requirement)
**This is a key enhancement from the functional requirements:**
- Implement token expiration validation with safety buffer (e.g., 1 minute)
- Return HTTP 401 with specific expiry error for expired tokens
- Include expiry timestamp in error response for client refresh guidance
- Test both cached and fresh token expiration scenarios

### 1.3 Issuer Validation Logic
- Implement validation of token issuer against configured Keycloak instance
- Reject tokens from unauthorized issuers immediately
- Support configurable issuer validation (enable/disable)
- Log issuer validation failures for security monitoring

### 1.4 Enhanced Token Service Structure
- Extend existing `DefaultTokenService` to support cross-client validation
- Maintain backward compatibility - existing token validation must work unchanged
- Implement the enhanced validation flow as described in functional requirements

## Security Requirements (Non-Negotiable)

1. **Zero Trust Validation**: All tokens must be validated against Keycloak
2. **Proper 401 Responses**: Expired tokens must return HTTP 401 with expiry information
3. **Issuer Security**: Strict validation against configured Keycloak instance
4. **Cache Security**: Use cryptographic hashes for cache keys to prevent tampering
5. **Audit Logging**: Log all cross-client token validation attempts
6. **Rate Limiting**: Prevent abuse of token exchange operations

## Integration Requirements

### Existing Codebase Integration
- **AuthService**: Use existing `exchange_token` method for RFC 8693 compliance
- **Token Validation**: Enhance existing validation flow without breaking changes
- **Middleware**: Integrate seamlessly with current auth middleware
- **Database**: Use existing `api_tokens` table structure (no schema changes required)
- **Cache Service**: Leverage existing `MokaCacheService` for performance

### Backward Compatibility (Critical)
- There is no requirement for Backwards compatability, as we do not have any production release out there
- Feel free to fix any existing implementation that is incorrect, and introduce new feature
- Do not worry about backwards compatability, gradual rollout, fallback strategy etc., implement the spec as given

## Implementation Flow

Follow the high-level flow described in the functional specification:

1. **Token Reception**: Extract token from Authorization header
2. **Database Check**: Try existing validation first
3. **Cross-Client Validation**: If not found, validate issuer and exchange
4. **Caching**: Store exchanged tokens for performance
5. **Response**: Return validated token or appropriate error

## Deliverables

### Code Components
1. **Enhanced Error Types**: New error variants with proper HTTP response mapping
2. **Token Expiry Validation**: Comprehensive expiration checking with 401 responses
3. **Issuer Validator**: Service for validating token issuers
4. **Enhanced Token Service**: Extended `DefaultTokenService` with cross-client support
5. **Cache Security**: Secure caching mechanism for exchanged tokens

### Testing Components
1. **Unit Tests**: Comprehensive test coverage following application testing patterns
2. **Mock Services**: Sophisticated mocks for AuthService and other dependencies
3. **Token Generation Utilities**: Test utilities for various token scenarios
4. **Security Tests**: Attack simulation and boundary condition testing
5. **Integration Tests**: End-to-end validation with realistic scenarios

### Quality Standards
- **Test Coverage**: Aim to improve test coverage on new code
- **Security Testing**: Include attack simulation and edge case testing
- **Error Handling**: Comprehensive error scenarios with proper HTTP responses
- **Documentation**: Clear code documentation and inline comments
- **Performance**: Efficient caching and minimal performance impact
- use rstest case feature to test multiple cases with the same test setup

## Success Criteria

### Functional Success
- [ ] Cross-client tokens validated successfully
- [ ] Existing token validation continues unchanged
- [ ] Proper 401 responses for expired tokens with refresh guidance
- [ ] Rate limiting prevents abuse scenarios
- [ ] Clear error messages for all failure scenarios

### Security Success
- [ ] All security tests pass
- [ ] Issuer validation prevents unauthorized tokens
- [ ] Cache security prevents tampering
- [ ] Audit logging captures all relevant events
- [ ] No token leakage in logs or error messages

### Performance Success
- [ ] Token validation latency <100ms (95th percentile)
- [ ] Cache hit ratio >80% for repeated tokens
- [ ] No degradation in existing token validation performance

## Getting Started

1. **Read the Functional Specification**: Thoroughly review `02-features/20250628-token-exchange.md`
2. **Understand Current Testing Patterns**: Study the testing philosophy section
3. **Start with Tests**: Begin by writing unit tests for token expiry validation
4. **Implement Core Errors**: Create enhanced error types with 401 response mapping
5. **Build Incrementally**: Follow TDD approach to discover optimal architecture

## Key Reminders

- **Security First**: This feature directly impacts application security
- **Test-Driven**: Start with tests, then implement code to pass them
- **Backward Compatible**: Existing functionality must remain unchanged
- **Follow Patterns**: Use established application testing and coding patterns
- **Functional Focus**: Let the functional requirements guide your technical decisions

Remember: The goal is to enable cross-client token validation while maintaining the highest security standards and preserving existing functionality. Use the functional specification as your guide and let TDD help you discover the best technical implementation.
