# OAuth 2.0 Token Exchange for Cross-Client Token Validation

## Overview

This feature implements OAuth 2.0 Token Exchange (RFC 8693) to enable cross-client token validation in our Rust/Axum backend with Keycloak integration. The implementation allows tokens issued by the same Keycloak issuer but for different clients to be validated and exchanged for tokens valid for our client.

## Problem Statement

### Current Limitation
Our application currently only accepts offline tokens issued by the same OAuth client. Tokens are validated by checking existence in our local database. This creates a limitation where tokens issued by the same Keycloak issuer but for different clients are rejected.

### Business Need
We need to expand support to accept tokens issued by the same Keycloak issuer but for different clients. This enables:
- **Cross-service authentication**: Services can authenticate users with tokens from other trusted clients
- **Microservice architecture**: Better support for distributed authentication
- **Third-party integration**: Accept tokens from partner applications using the same identity provider
- **Enhanced security**: Maintain zero-trust validation while expanding token acceptance

## Functional Requirements

### FR1: Cross-Client Token Acceptance
**Requirement**: Accept tokens issued by same Keycloak issuer for different clients

**Current State**: Application only accepts tokens where `azp` (authorized party) claim matches our client_id

**Desired State**: Accept tokens from any client within the same Keycloak realm/issuer

**Acceptance Criteria**:
- Tokens from different clients in same Keycloak realm are accepted
- Issuer validation ensures tokens are from trusted Keycloak instance
- Existing single-client token validation continues to work unchanged
- Invalid issuers are rejected with clear error messages

### FR2: Token Exchange Implementation
**Requirement**: Exchange third-party client tokens for our client tokens

**Flow**: subject_token (from third-party client) → requested_token (valid for our client)

**Acceptance Criteria**:
- Use existing `AuthService::exchange_token` method for RFC 8693 compliance
- Exchange only occurs for tokens not found in local database
- Exchanged tokens have appropriate scopes for our application
- Exchange failures are handled gracefully with fallback to rejection

### FR3: Secure Token Caching
**Requirement**: Cache exchanged tokens to avoid repeated exchanges

**Security Requirements**:
- Use token ID and cryptographic digest to prevent cache poisoning
- Cache keys must be tamper-resistant and collision-resistant
- Cached tokens must respect expiration times
- Cache must be invalidated when tokens expire

**Acceptance Criteria**:
- Cache hit ratio >80% for repeated cross-client tokens
- Cache keys include token JTI and hash prefix for security
- Expired tokens are automatically removed from cache
- Cache integrity is validated on each access

### FR4: Token Expiry Validation and Error Response
**Requirement**: Validate token expiration and provide appropriate error responses

**Acceptance Criteria**:
- Expired tokens return HTTP 401 with specific expiry error
- Error response includes information for client token refresh
- Expiration checking includes safety buffer (e.g., 1 minute)
- Both cached and fresh tokens are checked for expiration

**Error Response Format**:
```json
{
  "error": "token_expired",
  "error_description": "The provided token has expired",
  "expires_at": "2024-01-15T10:30:00Z"
}
```

### FR5: Issuer Validation
**Requirement**: Verify token issuer matches configured Keycloak server

**Acceptance Criteria**:
- Compare `iss` claim against expected issuer URL from configuration
- Reject tokens from unauthorized issuers immediately
- Support configurable issuer validation (enable/disable)
- Log issuer validation failures for security monitoring

## Non-Functional Requirements

### NFR1: Security
- **Zero Trust**: All tokens must be validated against Keycloak
- **Tamper Prevention**: Use cryptographic hashes for cache keys
- **Audit Trail**: Log all token exchange operations
- **Rate Limiting**: Prevent abuse of token exchange endpoint

### NFR2: Performance
- **Cache Efficiency**: Minimize repeated token exchanges (target >80% hit ratio)
- **Response Time**: Token validation <100ms (95th percentile)
- **Memory Usage**: Cache overhead <50MB under normal load
- **Async Operations**: Non-blocking token exchange calls

### NFR3: Reliability
- **Error Handling**: Graceful degradation on exchange failures
- **Timeout Management**: Reasonable timeouts for Keycloak calls (5-10 seconds)
- **Fallback Strategy**: Clear error messages for debugging
- **Backward Compatibility**: Existing token validation unchanged

## User Stories

### Story 1: Cross-Service Authentication
**As a** microservice developer
**I want** to accept tokens from other services in the same Keycloak realm
**So that** users don't need separate authentication for each service

**Acceptance Criteria**:
- Service A can validate tokens issued to Service B by the same Keycloak
- Token validation maintains security standards
- Performance impact is minimal

### Story 2: Third-Party Integration
**As a** system administrator
**I want** to allow partner applications to authenticate users
**So that** we can provide seamless integration experiences

**Acceptance Criteria**:
- Partner tokens from same Keycloak realm are accepted
- Partner tokens are exchanged for our application tokens
- All security validations are maintained

### Story 3: Token Expiry Handling
**As a** client application
**I want** to receive clear expiry information when tokens expire
**So that** I can refresh tokens and retry requests

**Acceptance Criteria**:
- HTTP 401 response for expired tokens
- Error response includes expiry timestamp
- Client can use this information to refresh tokens

## Implementation Approach

### High-Level Flow
1. **Token Reception**: Extract token from Authorization header
2. **Database Check**: Try existing validation first (backward compatibility)
3. **Cross-Client Validation**: If not found, validate issuer and exchange
4. **Caching**: Store exchanged tokens for performance
5. **Response**: Return validated token or appropriate error

### Security Validation Steps
1. **Issuer Verification**: Ensure token is from trusted Keycloak instance
2. **Expiration Check**: Validate token hasn't expired (with safety buffer)
3. **Token Exchange**: Use OAuth 2.0 Token Exchange (RFC 8693)
4. **Cache Security**: Use cryptographic hashes for cache keys
5. **Audit Logging**: Record all validation attempts for security monitoring

### Error Handling Strategy
- **Invalid Issuer**: Immediate rejection with security logging
- **Expired Token**: HTTP 401 with expiry information for client refresh
- **Exchange Failure**: Fallback to rejection with clear error message
- **Cache Errors**: Graceful degradation without affecting functionality

## Configuration Requirements

### Environment Variables
- `TOKEN_EXCHANGE_ENABLED`: Enable/disable cross-client token exchange (default: true)
- `TOKEN_EXCHANGE_CACHE_TTL`: Cache time-to-live in seconds (default: 3600)
- `TOKEN_EXCHANGE_RATE_LIMIT`: Maximum exchanges per minute per client (default: 100)

### Keycloak Configuration
- Token exchange must be enabled in Keycloak realm
- Client permissions must allow token exchange operations
- Issuer URL must be properly configured in application settings

## Success Metrics

### Performance Metrics
- **Cache Hit Ratio**: >80% for repeated cross-client tokens
- **Token Validation Latency**: <100ms (95th percentile)
- **Memory Usage**: <50MB increase under normal load
- **Exchange Success Rate**: >95% for valid tokens

### Security Metrics
- **Zero Token Leakage**: No full tokens in logs or error messages
- **Audit Coverage**: 100% of token exchange attempts logged
- **Rate Limiting Effectiveness**: Prevent >100 exchanges/minute per client
- **Issuer Validation**: 100% rejection of unauthorized issuers

## Testing Requirements and Patterns

### Application-Specific Testing Philosophy

This application employs a sophisticated testing approach for authentication logic that emphasizes realistic token handling, comprehensive mock patterns, and security-focused validation. Understanding these patterns is crucial for implementing cross-client token exchange testing that aligns with existing conventions.

#### Token Generation and Validation Testing Approach

**Realistic JWT Testing**: The application uses actual RSA public/private key pairs for JWT token generation and validation in tests, rather than simple string tokens or basic mocks. This approach ensures that token parsing, signature validation, and claims extraction are tested with realistic JWT structures.

**Test Certificate Infrastructure**: Dedicated test certificates and keys are maintained in the test data directory, providing consistent cryptographic material for token generation across all authentication tests. This allows for testing of actual JWT signing and verification processes.

**Claims-Based Testing**: Tests focus on JWT claims validation (issuer, expiration, audience, authorized party) rather than just token presence, ensuring that business logic around token content is properly validated.

#### Mock Patterns and Service Testing

**Service Layer Mocking**: The application uses `mockall` for creating sophisticated mocks of authentication services, allowing for precise control over service behavior in unit tests. Mock expectations are set up to validate specific parameters and return realistic responses.

**Stub Services for Configuration**: Configuration-related services use stub implementations that can be pre-configured with test data, providing predictable behavior for authentication configuration during testing.

**Layered Mock Strategy**: Different levels of mocking are employed - from individual service methods to complete service implementations - allowing tests to focus on specific layers of the authentication stack.

#### JWT Token Testing Patterns

**Multi-Scenario Token Generation**: Test utilities support generation of various token types (valid, expired, malformed, different issuers, different clients) to comprehensively test token validation logic across all scenarios.

**Claims Customization**: Token generation utilities allow for fine-grained control over JWT claims, enabling testing of specific business logic around token content and validation rules.

**Signature Validation Testing**: Tests verify both valid signatures (using correct keys) and invalid signatures (using wrong keys) to ensure proper cryptographic validation.

#### Authentication Middleware Testing Principles

**Request Flow Testing**: Middleware tests simulate complete HTTP request flows, including header extraction, token validation, and response generation, ensuring end-to-end authentication behavior.

**Error Response Validation**: Tests verify not just that errors occur, but that the correct HTTP status codes, error messages, and response structures are returned for different failure scenarios.

**State Management Testing**: Tests validate that authentication state is properly managed across request processing, including session handling and token caching behavior.

#### Token Service Testing Conventions

**Database Integration Testing**: Token service tests use real database connections with test databases, ensuring that token storage, retrieval, and validation work correctly with actual database operations.

**Cache Behavior Testing**: Tests validate cache hit/miss scenarios, cache invalidation, and cache key generation to ensure performance optimizations work correctly.

**Concurrent Access Testing**: Some tests verify that token services handle concurrent access correctly, particularly important for caching and rate limiting functionality.

#### Integration Testing Structure

**Live Service Integration**: Integration tests can run against actual Keycloak instances when configured, providing end-to-end validation of OAuth flows and token exchange operations.

**Environment-Based Testing**: Tests adapt to different environments (unit test with mocks, integration test with live services) based on configuration, allowing for comprehensive testing across deployment scenarios.

**OAuth Flow Testing**: Integration tests simulate complete OAuth authentication flows, including token acquisition, validation, and refresh, ensuring that the entire authentication pipeline works correctly.

#### Security Testing Approach

**Attack Simulation**: Tests simulate various attack scenarios (token replay, signature tampering, issuer spoofing) to ensure that security measures are effective.

**Boundary Condition Testing**: Tests focus on edge cases and boundary conditions (expired tokens, malformed tokens, rate limiting thresholds) to ensure robust security behavior.

**Audit Trail Validation**: Security-related tests verify that appropriate audit logs are generated for authentication events, ensuring compliance and monitoring requirements are met.

### Cross-Client Token Exchange Testing Requirements

Building on these established patterns, cross-client token exchange testing should incorporate:

#### Unit Testing Focus Areas
- **Issuer Validation Logic**: Test validation of token issuers against configured Keycloak instances
- **Cache Key Security**: Validate cryptographic integrity of cache key generation
- **Token Exchange Flow**: Test the complete token exchange process with various scenarios
- **Error Handling**: Comprehensive testing of all failure modes with proper error responses
- **Rate Limiting**: Validate rate limiting behavior under various load conditions

#### Integration Testing Scenarios
- **Multi-Client Flows**: Test token exchange between different OAuth clients in the same realm
- **Cache Performance**: Validate that token exchange caching provides expected performance benefits
- **Security Boundaries**: Test that tokens from unauthorized issuers are properly rejected
- **Expiry Handling**: Verify that expired tokens generate appropriate 401 responses with refresh guidance

#### Security Testing Priorities
- **Cross-Client Isolation**: Ensure that token exchange doesn't compromise client isolation
- **Cache Poisoning Prevention**: Test that cache keys cannot be manipulated to access unauthorized tokens
- **Rate Limiting Effectiveness**: Verify that rate limiting prevents abuse of token exchange endpoints
- **Audit Completeness**: Ensure all token exchange operations are properly logged for security monitoring



## Error Handling Requirements

### Error Categories
- **Invalid Issuer**: Token from unauthorized Keycloak instance
- **Token Expired**: Token has passed expiration time
- **Exchange Failed**: Keycloak token exchange operation failed
- **Cache Errors**: Issues with token caching operations
- **Rate Limiting**: Too many exchange requests from client
- **Configuration**: Missing or invalid configuration

### Error Response Requirements
- **HTTP 401**: For authentication failures (expired, invalid issuer)
- **HTTP 429**: For rate limiting violations
- **HTTP 500**: For internal server errors
- **Clear Messages**: Human-readable error descriptions
- **Security**: No sensitive information in error responses



## Security Considerations

### Token Validation Security
1. **Issuer Verification**: Strict validation against configured Keycloak instance
2. **Cache Key Security**: Cryptographic hash prevents cache poisoning
3. **Token Expiration**: Respect token expiration times with safety buffers
4. **Audit Logging**: Log all cross-client token validation attempts

### Attack Prevention
1. **Token Replay**: Cache keys include token hash to prevent replay
2. **Privilege Escalation**: Exchange only for equivalent or lesser scopes
3. **Rate Limiting**: Implement rate limiting on token exchange endpoint
4. **Monitoring**: Alert on unusual token exchange patterns

### Data Protection
1. **Sensitive Data**: Never log full tokens, only token IDs
2. **Cache Encryption**: Consider encrypting cached token data
3. **Memory Safety**: Clear sensitive data from memory promptly
4. **Transport Security**: Ensure HTTPS for all Keycloak communication



## Implementation Phases

### Phase 1: Core Infrastructure (Week 1-2)
- Enhance error types for cross-client validation scenarios
- Implement issuer validation logic
- Add secure cache key generation
- Create comprehensive unit tests

### Phase 2: Token Exchange Integration (Week 3-4)
- Implement cross-client token validation flow
- Add secure caching mechanism for exchanged tokens
- Integrate with existing AuthService for token exchange
- Add integration tests with Keycloak

### Phase 3: Security Hardening (Week 5)
- Implement rate limiting for token exchange operations
- Add comprehensive audit logging
- Security testing and penetration testing
- Performance optimization

### Phase 4: Configuration and Deployment (Week 6)
- Configuration management and validation
- Middleware integration with feature flags
- Documentation updates
- Monitoring and alerting setup





## Dependencies and Risks

### External Dependencies
- **Keycloak Server**: Must support OAuth 2.0 Token Exchange (RFC 8693)
- **Network Connectivity**: Reliable connection to Keycloak for token exchange
- **Configuration Management**: Environment variables or config files

### Internal Dependencies
- **Existing AuthService**: Token exchange method already implemented
- **Cache Service**: MokaCacheService for token caching
- **Database Service**: For token validation and storage
- **Settings Service**: For configuration management

### Risk Mitigation

#### Technical Risks
1. **Keycloak Compatibility**: Verify token exchange support in target Keycloak version
2. **Performance Impact**: Monitor token exchange latency and cache efficiency
3. **Cache Memory Usage**: Implement cache size limits and cleanup

#### Security Risks
1. **Token Leakage**: Ensure tokens are never logged in full
2. **Cache Poisoning**: Validate cache key integrity
3. **Rate Limiting Bypass**: Implement multiple rate limiting strategies

#### Operational Risks
1. **Configuration Errors**: Provide clear configuration validation
2. **Monitoring Gaps**: Ensure comprehensive metrics and alerting
3. **Rollback Strategy**: Support disabling feature via configuration

## Monitoring and Observability

### Metrics
- Token exchange success/failure rates
- Cache hit/miss ratios for exchanged tokens
- Token validation latency
- Cross-client token usage patterns

### Logging
- All token exchange attempts (success/failure)
- Invalid issuer detection
- Cache operations for exchanged tokens
- Performance metrics for token validation

### Alerts
- High token exchange failure rates
- Unusual cross-client token patterns
- Cache performance degradation
- Keycloak connectivity issues

## Deployment Considerations

### Backward Compatibility
- **Zero Breaking Changes**: Existing token validation continues to work
- **Gradual Rollout**: Feature can be enabled/disabled via configuration
- **Fallback Strategy**: Falls back to existing validation if exchange fails

### Performance Impact
- **Cache Efficiency**: Reduces repeated token exchanges
- **Network Overhead**: Additional Keycloak calls for new token types
- **Memory Usage**: Minimal increase for cache storage

### Security Deployment
- **Rate Limiting**: Configure appropriate limits for production
- **Monitoring**: Set up alerts for unusual token exchange patterns
- **Audit Logging**: Ensure compliance with security requirements

## Success Criteria

### Functional Success
- [ ] Cross-client tokens validated successfully
- [ ] Token exchange caching reduces Keycloak calls by >80%
- [ ] Rate limiting prevents abuse scenarios
- [ ] Error handling provides clear feedback

### Performance Success
- [ ] Token validation latency < 100ms (95th percentile)
- [ ] Cache hit ratio > 80% for repeated tokens
- [ ] Memory usage increase < 50MB under normal load
- [ ] No degradation in existing token validation performance

### Security Success
- [ ] All security tests pass
- [ ] Audit logging captures all relevant events
- [ ] Rate limiting prevents DoS scenarios
- [ ] No token leakage in logs or error messages

## Related Documentation

- **[Authentication Architecture](../01-architecture/authentication.md)** - Current authentication system
- **[API Integration](../01-architecture/api-integration.md)** - Backend integration patterns
- **[Auth Middleware](../03-crates/auth_middleware.md)** - Middleware implementation details
- **[Services Crate](../03-crates/services.md)** - Service layer architecture
- **[OAuth 2.0 Token Exchange RFC 8693](https://datatracker.ietf.org/doc/html/rfc8693)** - Official specification
- **[Keycloak Token Exchange Documentation](https://www.keycloak.org/securing-apps/token-exchange)** - Keycloak implementation details
- **[Technical Specification](20250628-token-exchange-tech-spec.md)** - Detailed technical implementation guide
