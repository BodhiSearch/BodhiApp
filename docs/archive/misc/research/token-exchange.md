# OAuth 2.1 Token Exchange Security Research

## Executive Summary

This research investigates secure OAuth 2.1 token exchange patterns for preventing privilege escalation when third-party clients exchange tokens for access to our resource server. The key finding is that **scope-limited token exchange with explicit consent mechanisms** is the standard approach, implemented through authorization server policies rather than built-in consent screens.

**Implementation Status**: A working implementation has been completed for Keycloak v23 using a two-step token exchange process with user scope-based authorization. This serves as a temporary solution while preparing for Keycloak v26 migration.

## Problem Statement

**Current Architecture:**
- Our application acts as an OAuth resource server with its own client ID in Keycloak
- Third-party client applications have separate client IDs in Keycloak  
- Users have unified accounts in Keycloak and can authenticate to both our resource server and third-party clients
- **Security Risk**: When users authenticate to third-party clients, those clients inherit the user's full privilege level on our resource server without explicit consent

**Example Scenario:**
If an admin user logs into a third-party app, that app gains admin-level access to our APIs without the user explicitly approving those specific privileges.

## Research Findings

### 1. OAuth 2.1 Token Exchange Standard (RFC 8693)

**Key Specifications:**
- **RFC 8693**: "OAuth 2.0 Token Exchange" - The foundational standard
- **Scope Parameter**: Allows clients to specify desired scope of the requested security token
- **Audience Parameter**: Logical name of target service where token will be used
- **Resource Parameter**: URI indicating target service or resource

**Security Considerations from RFC 8693:**
- Authorization servers MUST perform appropriate validation procedures
- Token exchange should not impact validity of original tokens
- Scope limitations are explicitly supported through the `scope` parameter
- Authorization servers can apply policy based on target audience/resource

### 2. Standard Security Patterns

#### A. Scope-Limited Token Exchange

**Scenario 1: Third-party Client to Resource Server**
**WHO**: Third-party client (`client-abc`) makes the call
```http
POST /realms/bodhi/protocol/openid-connect/token
Content-Type: application/x-www-form-urlencoded

grant_type=urn:ietf:params:oauth:grant-type:token-exchange
&client_id=client-abc
&subject_token=eyJhbGciOiJSUzI1...
&requested_token_type=urn:ietf:params:oauth:token-type:access_token
&audience=resource-xyz
&scope=scope_user_user
```

**Scenario 2: Resource Server to Upstream Resource Server**
**WHO**: Our resource server (`resource-xyz`) makes the call
```http
POST /realms/bodhi/protocol/openid-connect/token
Authorization: Basic cmVzb3VyY2UteHl6OnNlY3JldA==
Content-Type: application/x-www-form-urlencoded

grant_type=urn:ietf:params:oauth:grant-type:token-exchange
&client_id=resource-xyz
&client_secret=resource-xyz-secret
&subject_token=eyJhbGciOiJSUzI1...
&requested_token_type=urn:ietf:params:oauth:token-type:access_token
&audience=resource-lmn
&scope=scope_user_power
```

**Benefits:**
- Limits exchanged token to specific scopes
- Prevents full privilege inheritance
- Standard OAuth 2.1 compliant approach
- Maps directly to existing BodhiApp role system

### 3. Keycloak Standard Token Exchange Capabilities

#### Standard Token Exchange V2 Features
- **RFC 8693 Compliance**: Full implementation of OAuth 2.0 Token Exchange specification
- **Scope Parameter Support**: Allows clients to specify desired scope of requested security token
- **Audience Parameter Support**: Restricts token usage to specific target services
- **Client Scope Integration**: Works with Keycloak's client scope system for consent and scope management

#### Simple Authorization Approach
- **Claims-based Authorization**: Use claims in exchanged tokens that map to roles in OAuth resource server
- **Resource Server Handles Authorization**: OAuth resource server performs its own authorization verification
- **Low Complexity**: Avoids Keycloak Authorization Services for simpler implementation
- **Direct Role Mapping**: Scopes directly map to existing Role/TokenScope enums

**Reference:** [Keycloak Token Exchange Documentation](https://www.keycloak.org/securing-apps/token-exchange)

### 4. Consent Mechanism for BodhiApp

**Key Finding:** Keycloak provides consent screens during the initial authorization flow when `consentRequired=true` is enabled for third-party clients. This allows users to explicitly approve what scopes/access levels third-party clients can request.

**BodhiApp Approach:**
1. **Pre-authorization consent**: Users consent to specific scopes during initial third-party client authentication via Keycloak's built-in consent screen
2. **Scope-limited token exchange**: Token exchange maintains the scope limitations from the original consent
3. **Claims-based authorization**: Resource server handles authorization using claims from exchanged tokens

## Recommended Implementation Strategy

### Phase 1: Scope-Limited Token Exchange

**Implementation Steps:**
1. **Configure Keycloak client permissions** for third-party clients
2. **Define scope mappings** between third-party client permissions and our resource server roles
3. **Implement token exchange endpoint** with scope limitations
4. **Apply audience restrictions** to limit token usage to our resource server

**Example Configuration:**
```json
{
  "client_id": "client-abc",
  "allowed_scopes": ["scope_user_user", "scope_user_power"],
  "resource_server_mappings": {
    "scope_user_user": "resource_user",
    "scope_user_power": "resource_power_user"
  },
  "max_scope_elevation": false
}
```

### Phase 2: Enhanced Security Controls

**Additional Measures:**
1. **Token exchange audit logging** for security monitoring
2. **Rate limiting** on token exchange requests
3. **Time-limited tokens** with shorter expiration for exchanged tokens
4. **Revocation mechanisms** for compromised tokens

### Phase 3: Custom Consent (Optional)

**If explicit consent is required:**
1. **Pre-exchange consent flow**: Redirect users to consent screen before token exchange
2. **Consent storage**: Track user consent decisions for future exchanges
3. **Consent revocation**: Allow users to revoke previously granted permissions

## Security Trade-offs Analysis

| Approach | Security Level | Implementation Complexity | User Experience |
|----------|---------------|---------------------------|-----------------|
| Scope-Limited Exchange | High | Medium | Transparent |
| Custom Consent | Highest | Very High | Additional friction |
| Claims-based Authorization | Medium-High | Low | Transparent |

## Keycloak Configuration for BodhiApp Architecture

### System Configuration Details

**Keycloak Realm**: `bodhi`
**Resource Server Client**: `resource-xyz` (confidential, with client secret)
**Third-party Client**: `client-abc` (non-confidential, PKCE-enabled)
**Upstream Resource Server**: `resource-lmn` (confidential, with client secret)

### Scope Configuration Strategy

Based on the existing BodhiApp role and token scope system, here's the recommended approach:

#### 1. Scope Definition Strategy

**Recommendation**: Use **role-equivalent scopes** that directly map to existing roles and token scopes.

**Rationale**:
- Simplifies integration with existing authorization middleware
- Maintains consistency with current `Role` and `TokenScope` enums
- Reduces complexity in scope-to-role mapping logic

**Proposed Scopes**:
```
scope_user_user       -> Role::User + TokenScope::User
scope_user_power      -> Role::PowerUser + TokenScope::PowerUser
scope_user_manager    -> Role::Manager + TokenScope::Manager
scope_user_admin      -> Role::Admin + TokenScope::Admin
```

#### 2. Scope Level Configuration

**Recommendation**: **Realm-level client scopes** with client-specific assignments.

**Implementation**:
1. **Create realm-level client scopes** for each role level
2. **Assign scopes to clients** based on their intended privilege levels
3. **Enable consent** for third-party clients to show users what access they're granting

**Keycloak Configuration Steps**:

```bash
# 1. Create realm-level client scopes
kcadm.sh create client-scopes -r bodhi \
  -s name=scope_user_user \
  -s description="Basic user access to resource server" \
  -s protocol=openid-connect \
  -s attributes.consent.screen.text="Basic user access"

kcadm.sh create client-scopes -r bodhi \
  -s name=scope_user_power \
  -s description="Power user access to resource server" \
  -s protocol=openid-connect \
  -s attributes.consent.screen.text="Power user access"

kcadm.sh create client-scopes -r bodhi \
  -s name=scope_user_manager \
  -s description="Manager access to resource server" \
  -s protocol=openid-connect \
  -s attributes.consent.screen.text="Manager access"

kcadm.sh create client-scopes -r bodhi \
  -s name=scope_user_admin \
  -s description="Admin access to resource server" \
  -s protocol=openid-connect \
  -s attributes.consent.screen.text="Admin access"
```

#### 3. Client Scope Assignments

**Third-party Client (`client-abc`)**:
```bash
# Assign optional scopes to third-party client
kcadm.sh update clients/client-abc -r bodhi \
  -s 'optionalClientScopes=["scope_user_user","scope_user_power"]'

# Enable consent for third-party client
kcadm.sh update clients/client-abc -r bodhi \
  -s consentRequired=true
```

**Resource Server Client (`resource-xyz`)**:
```bash
# Assign all scopes as default for resource server
kcadm.sh update clients/resource-xyz -r bodhi \
  -s 'defaultClientScopes=["scope_user_user","scope_user_power","scope_user_manager","scope_user_admin"]'

# Enable token exchange for resource server
kcadm.sh update clients/resource-xyz -r bodhi \
  -s attributes.oauth2.device.authorization.grant.enabled=false \
  -s attributes.token.exchange.standard.enabled=true
```

#### 4. Token Exchange Implementation

**Dual Token Exchange Scenarios**:

**Scenario A: Third-party to Resource Server**
**WHO**: Third-party client (`client-abc`) - non-confidential client using PKCE
```http
POST /realms/bodhi/protocol/openid-connect/token
Content-Type: application/x-www-form-urlencoded

grant_type=urn:ietf:params:oauth:grant-type:token-exchange
&client_id=client-abc
&subject_token=eyJhbGciOiJSUzI1NiIs...
&subject_token_type=urn:ietf:params:oauth:token-type:access_token
&audience=resource-xyz
&scope=scope_user_user
```

**Scenario B: Resource Server to Upstream**
**WHO**: Our resource server (`resource-xyz`) - confidential client with client secret
```http
POST /realms/bodhi/protocol/openid-connect/token
Authorization: Basic cmVzb3VyY2UteHl6OnNlY3JldA==
Content-Type: application/x-www-form-urlencoded

grant_type=urn:ietf:params:oauth:grant-type:token-exchange
&client_id=resource-xyz
&client_secret=resource-xyz-secret
&subject_token=eyJhbGciOiJSUzI1NiIs...
&subject_token_type=urn:ietf:params:oauth:token-type:access_token
&audience=resource-lmn
&scope=scope_user_power
```

### Integration with Existing Authorization System

#### 1. Scope-to-Role Mapping

**Proposed UserScope Enum** (similar to TokenScope):
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum UserScope {
  #[strum(serialize = "scope_user_user")]
  User,
  #[strum(serialize = "scope_user_power")]
  PowerUser,
  #[strum(serialize = "scope_user_manager")]
  Manager,
  #[strum(serialize = "scope_user_admin")]
  Admin,
}

impl UserScope {
  pub fn to_role(&self) -> Role {
    match self {
      UserScope::User => Role::User,
      UserScope::PowerUser => Role::PowerUser,
      UserScope::Manager => Role::Manager,
      UserScope::Admin => Role::Admin,
    }
  }

  pub fn to_token_scope(&self) -> TokenScope {
    match self {
      UserScope::User => TokenScope::User,
      UserScope::PowerUser => TokenScope::PowerUser,
      UserScope::Manager => TokenScope::Manager,
      UserScope::Admin => TokenScope::Admin,
    }
  }
}
```

#### 2. Middleware Integration

**Enhanced Authorization Middleware**:
```rust
// In api_auth_middleware.rs
pub async fn api_auth_middleware(
  required_role: Role,
  required_scope: Option<TokenScope>,
  State(state): State<Arc<dyn RouterState>>,
  req: Request,
  next: Next,
) -> Result<Response, ApiError> {
  // Check for exchanged token scope header
  let user_scope_header = req.headers().get("X-User-Scope");

  match (role_header, scope_header, user_scope_header, required_scope) {
    // Existing role-based auth
    (Some(role_header), _, _, _) => {
      // ... existing role validation
    }

    // Existing token scope auth
    (None, Some(scope_header), None, Some(required_scope)) => {
      // ... existing token scope validation
    }

    // NEW: User scope from token exchange
    (None, None, Some(user_scope_header), _) => {
      let user_scope = user_scope_header
        .to_str()
        .map_err(|e| ApiAuthError::MalformedScope(e.to_string()))?
        .parse::<UserScope>()?;

      let user_role = user_scope.to_role();
      if !user_role.has_access_to(&required_role) {
        return Err(ApiAuthError::Forbidden);
      }
    }

    _ => return Err(ApiAuthError::MissingAuth),
  }

  Ok(next.run(req).await)
}
```

### Consent Screen Configuration

#### 1. Enable Consent for Third-party Clients

**Keycloak Configuration**:
```bash
# Enable consent requirement
kcadm.sh update clients/client-abc -r bodhi \
  -s consentRequired=true

# Configure consent screen text for each scope
kcadm.sh update client-scopes/scope_user_user -r bodhi \
  -s 'attributes.consent.screen.text="Basic access to your BodhiApp data"'

kcadm.sh update client-scopes/scope_user_power -r bodhi \
  -s 'attributes.consent.screen.text="Enhanced access to your BodhiApp data"'
```

#### 2. Consent Flow Integration

**User Experience**:
1. User authenticates to third-party client (`client-abc`)
2. Keycloak displays consent screen showing requested scopes
3. User approves specific scope level (e.g., `scope_user_user`)
4. Third-party client receives token with limited scope
5. Token exchange to resource server maintains scope limitation

### Security Considerations

#### 1. Scope Validation

**Token Exchange Validation**:
- Exchanged tokens cannot have higher privileges than original token
- Audience parameter restricts token usage to specific resource servers
- Scope parameter limits privileges within the target resource server

#### 2. Audit Logging

**Enhanced Logging**:
```rust
// Log token exchange events
log::info!(
  "Token exchange: client={} subject_scope={} requested_scope={} audience={}",
  client_id,
  subject_scope,
  requested_scope,
  audience
);
```

## Implementation Recommendations

### Immediate Actions (High Priority)
1. **Implement scope-limited token exchange** using Keycloak's standard token exchange V2
2. **Create realm-level client scopes** mapping to existing Role/TokenScope enums
3. **Configure consent screens** for third-party clients
4. **Enhance authorization middleware** to handle UserScope from exchanged tokens

### Medium-term Enhancements
1. **Implement comprehensive audit logging** for token exchange operations
2. **Add scope validation** in token exchange requests
3. **Create monitoring dashboards** for token exchange patterns

### Long-term Considerations
1. **Evaluate fine-grained permissions** if more complex authorization scenarios emerge
2. **Consider token caching** for performance optimization in high-volume scenarios
3. **Implement automated scope management** for dynamic client registration

## Conclusion

The recommended approach leverages **Keycloak's standard token exchange V2** with **realm-level client scopes** that directly map to BodhiApp's existing role system. This provides:

- **Security**: Scope-limited token exchange prevents privilege escalation
- **Simplicity**: Direct mapping to existing Role/TokenScope enums
- **User Control**: Consent screens for third-party client access
- **Compliance**: Full OAuth 2.1 RFC 8693 compliance
- **Integration**: Seamless integration with existing authorization middleware

The key insight is using **role-equivalent scopes** (`scope_user_admin`) rather than fine-grained scopes (`read:basic`), which aligns perfectly with BodhiApp's existing authorization model and simplifies the implementation.

## Current Implementation (Keycloak v23)

The research findings have been successfully implemented in the BodhiApp backend for Keycloak v23. The implementation includes:

### Key Implementation Details
- **Two-Step Token Exchange**: Uses client credentials grant followed by token exchange for Keycloak v23 compatibility
- **User Scope Mapping**: Direct mapping from `scope_user_*` to internal `UserScope` enum and `Role` system
- **Secure Caching**: Token caching using JWT ID (JTI) for performance without compromising security
- **Comprehensive Validation**: Issuer validation, expiration checking, and scope enforcement

### Implementation Location
- **Main Specification**: `ai-docs/02-features/20250628-token-exchange.md`
- **Auth Middleware**: `crates/auth_middleware/src/auth_middleware.rs`
- **Token Service**: `crates/auth_middleware/src/token_service.rs`
- **Auth Service**: `crates/services/src/auth_service.rs`
- **User Scope Types**: `crates/objs/src/user_scope.rs` and `crates/objs/src/resource_scope.rs`

### Alignment with Research
The implementation successfully incorporates the research recommendations:
- ✅ Scope-limited token exchange prevents privilege escalation
- ✅ Claims-based authorization using scope mappings
- ✅ Issuer validation for security boundaries
- ✅ Integration with existing authorization middleware
- ✅ Comprehensive testing including security scenarios

## References

1. [RFC 8693 - OAuth 2.0 Token Exchange](https://datatracker.ietf.org/doc/html/rfc8693)
2. [Keycloak v23 Token Exchange Documentation](https://www.keycloak.org/docs/23.0.0/securing_apps/index.html#_token-exchange)
3. [Keycloak Server Administration Guide](https://www.keycloak.org/docs/latest/server_admin/index.html)
4. [OAuth 2.0 Security Best Current Practice](https://www.ietf.org/archive/id/draft-ietf-oauth-security-topics-29.html)
5. [BodhiApp Token Exchange Implementation](../02-features/20250628-token-exchange.md) - Current working implementation
