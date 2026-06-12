# OAuth2 Token Exchange Auth Service Update - Feature Specification

**Status**: Completed  
**Implementation Date**: January 2025  
**Related Stories**: [20250628-token-exchange.md](../20250628-token-exchange.md)

## Overview

This feature specification documents the migration from Keycloak v23 to v26 and the implementation of standard OAuth2 Token Exchange v2, replacing the preview token exchange implementation with the standardized approach that includes dynamic audience management.

## Problem Statement

### Migration Requirements
- Keycloak upgrade from v23 to v26 required updating token exchange implementation
- v23 used preview token exchange with `audience` parameter
- v26 implements standard OAuth2 Token Exchange v2 specification
- Need for dynamic audience management to support marketplace-style multi-client scenarios

### Key Changes Required
- Update from preview token exchange format to standard v2 specification
- Replace direct audience parameter with dynamic consent flow
- Implement resource client request-access mechanism
- Update client creation endpoints from `/clients` to `/apps` and `/resources`
- Add new `/request-access` endpoint for dynamic audience management

## Implementation Details

### 1. Token Exchange Format Migration

**From v23 Preview Format**:
```
grant_type=urn:ietf:params:oauth:grant-type:token-exchange
&subject_token={app_user_token}
&client_id={app_client_id}
&audience={resource_client_id}
&scope=openid email profile roles {user_scope}
```

**To v26 Standard Format**:
```
subject_token={app_user_token}
&scope=openid email profile roles scope_user_user
&grant_type=urn:ietf:params:oauth:grant-type:token-exchange
&subject_token_type=urn:ietf:params:oauth:token-type:access_token
&requested_token_type=urn:ietf:params:oauth:token-type:access_token
```

### 2. Dynamic Audience Management Flow

**New Consent-Based Approach**:
1. **Resource Client Creation**: Use `/bodhi/resources` endpoint instead of `/clients`
2. **App Client Creation**: Use `/bodhi/apps` endpoint for public clients
3. **Dynamic Access Request**: Resource client requests audience access via `/resources/request-access`
4. **User Consent**: App user authorizes with resource-specific scope in consent flow
5. **Token Exchange**: Standard v2 format without explicit audience parameter

### 3. Endpoint Changes

**Client Creation Endpoints**:
- **Old**: `/realms/{realm}/bodhi/clients` (generic)
- **New**: 
  - `/realms/{realm}/bodhi/apps` (public clients)
  - `/realms/{realm}/bodhi/resources` (confidential clients)

**New Endpoints**:
- `/realms/{realm}/bodhi/resources/request-access` - Dynamic audience management

### 4. Authentication Flow Updates

**v23 Flow (Deprecated)**:
1. Create resource client via Bodhi API
2. Create app client via Admin CLI
3. Get resource service account token
4. Make user resource admin
5. Add user to groups
6. Get app user token with OAuth2 flow
7. Token exchange with explicit audience parameter

**v26 Flow (Current)**:
1. Get dev console user token (OAuth2 authorization code flow)
2. Create app client via `/bodhi/apps`
3. Create resource client via `/bodhi/resources`
4. Get resource client service account token
5. Resource client requests audience access for app client
6. App user authorization with resource-specific consent
7. Standard token exchange v2 (no explicit audience)

### 5. Scope and Audience Handling

**Dynamic Resource Scopes**:
- Resource clients automatically get scope: `scope_resource-{client_id}`
- Resource scope is returned from `/request-access` endpoint
- App user must consent to resource scope during authorization

**Audience Management**:
- v23: Explicit `audience` parameter in token exchange
- v26: Audience automatically included in app user token via consent flow
- Token exchange validates audience from subject token, not request parameter

### 6. Integration Test Updates

**AuthServerTestClient Enhancements**:
- Updated `setup_dynamic_clients()` to use new endpoint structure
- Added support for v26 token exchange format
- Implemented dynamic audience request flow
- Updated client creation methods for apps vs resources distinction

**Test Environment Configuration**:
- Environment variables remain the same
- Test flows updated to match v26 patterns
- Maintains compatibility with live Keycloak test environment

## Technical Architecture

### Client Type Distinction

**App Clients (Public)**:
- Created via `/bodhi/apps` endpoint
- No client secrets
- Used for frontend applications
- Support OAuth2 authorization code flow with PKCE

**Resource Clients (Confidential)**:
- Created via `/bodhi/resources` endpoint
- Include client secrets
- Used for backend services
- Support client credentials and token exchange flows

### Token Exchange Validation

**Standard v2 Compliance**:
- Uses standard OAuth2 Token Exchange grant type
- Includes proper token type specifications
- Validates audience from subject token rather than request
- Returns `issued_token_type` in response

**Security Enhancements**:
- Dynamic consent ensures user explicitly approves resource access
- Resource-specific scopes provide granular permissions
- Audience validation prevents token misuse across resources

## Benefits and Impact

### Standards Compliance
- **OAuth2 Specification**: Full compliance with Token Exchange v2 standard
- **Keycloak Compatibility**: Works with latest Keycloak versions
- **Future-Proof**: Based on standardized specifications rather than preview features

### Dynamic Architecture
- **Marketplace Support**: Enables dynamic client-to-client relationships
- **User Consent**: Clear user understanding of resource access permissions
- **Scalable**: No pre-configuration required for client relationships

### Developer Experience
- **Clearer Endpoints**: Distinct endpoints for different client types
- **Simplified Flow**: Standard token exchange without custom parameters
- **Better Testing**: Reliable integration tests with live environment

## Migration Impact

### Breaking Changes
- Token exchange requests must remove `audience` parameter
- Client creation must use appropriate endpoint (`/apps` vs `/resources`)
- Resource clients must request audience access before token exchange

### Compatibility
- Existing tokens remain valid during transition
- Test environment supports both flows during migration period
- No changes to user experience for end-users

## Conclusion

The migration to Keycloak v26 and OAuth2 Token Exchange v2 provides a more robust, standards-compliant foundation for multi-client authentication scenarios. The dynamic audience management approach eliminates the need for pre-configured client relationships while maintaining security through explicit user consent flows.

This implementation supports marketplace-style applications where multiple resource clients can dynamically request access to app client tokens, providing a scalable solution for complex authentication architectures. 