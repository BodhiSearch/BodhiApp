# API Authorization Feature

## Requirements

### User Story
As a System Administrator  
I want to enforce role-based access control on API endpoints  
So that users can only access resources appropriate for their role level

### Core Requirements

#### Role Hierarchy
- Roles are hierarchical in descending order: admin > manager > power_user > user
  1. Admin (`resource_admin`, `scope_token_admin`)
  2. Manager (`resource_manager`, `scope_token_manager`)
  3. Power User (`resource_power_user`, `scope_token_power_user`)
  4. User (`resource_user`, `scope_token_user`)
- Higher roles automatically have access to lower role endpoints
- Role hierarchy must be implemented as a reusable function
- Roles are included in JWT token claims
- No configuration file needed for hierarchy
- All role checks skipped in non-authenticated mode
- Each endpoint is configured with minimum required role level

#### Token Validation
- Session tokens: validate roles in resource_access.[client-id].roles
- Bearer tokens: validate roles in scope claim (e.g., scope_token_user)
- Client ID in resource_access must match app's client ID
- Rely solely on token claims for role validation
- Authorization must be enforced after authentication

#### Authorization Flow
1. Auth middleware injects active access token in header
2. API auth middleware extracts role information:
   - Session tokens: Check `resource_access.[client-id].roles` array
   - Bearer tokens: Check `scope` field for role-specific scopes
3. Validate minimum required role access
4. Allow or deny request with appropriate response

#### Security Requirements
- Return 403 Forbidden for insufficient privileges
- Use generic error message without disclosing required role
- Log authorization failures at WARN level
- Include request path in warning logs
- Skip role checks in non-authenticated mode
- Token claims must include valid client ID

#### User Interface Requirements
1. Non-authenticated Mode
   - Show message "App needs to be in authenticated mode for this feature"
   - Skip role validation checks
   - Display clear explanation of non-authenticated mode behavior

2. Error Handling
   - Generic error message for insufficient privileges
   - Localized using FluentLocalizationService
   - No disclosure of required roles in error messages
   - Clear feedback for authorization failures

### Not In Scope
- Custom role definitions
- Role management interface
- Role-based UI components
- Role access tracking/analytics
- Multiple explicit roles per route
- Real-time role updates
- Custom role assignments
- Audit logging of successful access

## Implementation Tasks

### Backend Tasks

#### Role Implementation
- [ ] Create Role enum with variants: User, PowerUser, Manager, Admin
- [ ] Implement role hierarchy checking
- [ ] Add role parsing from token claims
- [ ] Add role parsing from scope strings
- [ ] Implement role comparison logic
- [ ] Add comprehensive error logging

#### Middleware Implementation
- [ ] Create api_auth middleware with role requirement parameter
- [ ] Extract roles from X-Resource-Token header
- [ ] Implement client ID validation for resource_access
- [ ] Add role validation logic
- [ ] Implement non-authenticated mode bypass
- [ ] Add error handling with generic messages
- [ ] Add WARN level logging for authorization failures
- [ ] Set up localization for error messages

#### Route Integration
- [ ] Group routes by minimum role requirement
- [ ] Add role-based middleware to protected routes
- [ ] Update route registration to support role requirements
- [ ] Add proper error responses for authorization failures
- [ ] Document role requirements for each route group

#### Error Handling
- [ ] Add authorization error types
- [ ] Implement localized error messages
- [ ] Add warning level logging for authorization failures
- [ ] Include request path in log messages

### Testing Tasks
- [ ] Unit tests for Role enum and hierarchy
- [ ] Tests for token claim parsing
- [ ] Tests for scope parsing
- [ ] Integration tests for middleware
- [ ] Test non-authenticated mode bypass
- [ ] Test error responses
- [ ] Test logging behavior
- [ ] Cache behavior tests
- [ ] Localization tests for error messages

## File Overview

### Backend (Rust)
- `crates/auth_middleware/src/api_auth.rs`: API authorization middleware
- `crates/auth_middleware/src/role.rs`: Role hierarchy and validation logic
- `crates/routes_all/src/routes.rs`: Route configuration with role groups
- `crates/auth_middleware/src/resources/en-US/messages.ftl`: Error messages for authorization

### Tests
- `crates/auth_middleware/tests/test_api_auth.rs`: Authorization middleware tests
- `crates/auth_middleware/tests/test_role.rs`: Role validation tests
- `crates/auth_middleware/tests/test_cache.rs`: Cache behavior tests

## Technical Details

### Role Hierarchy Implementation
```rust
pub enum Role {
    User,
    PowerUser,
    Manager,
    Admin,
}

impl Role {
    fn has_access_to(&self, required: &Role) -> bool {
        // Implement hierarchy check
    }
}
```

### Token Claims Structure
```json
{
  "resource_access": {
    "resource-71425db3-e706-42d6-b254-81b2e9820346": {
      "roles": [
        "resource_manager",
        "resource_power_user",
        "resource_user",
        "resource_admin"
      ]
    }
  }
}
```

### Bearer Token Scope
```
scope: "openid offline_access scope_token_user"
```

### Route Configuration
```rust
let admin_apis = Router::new().route("/api/settings", 
    get(settings_handler))
.route("/api/admin", 
    get(admin_handler))
    .route_layer(from_fn_with_state(state.clone(), api_auth_middleware(Role::Admin)));
let manager_apis = Router::new().route("/api/users", 
    get(user_handler))
   .route("/api/billing", 
    get(billing_handler))
    .route_layer(from_fn_with_state(state.clone(), api_auth_middleware(Role::Manager)));
let app_router = admin_apis.merge(manager_apis).merge(...);
```

## Migration Plan
1. Deploy core role implementation
   - Implement Role enum and hierarchy
   - Add token validation logic
   - Set up error handling

2. Add middleware layer
   - Create authorization middleware
   - Implement role validation
   - Add logging and monitoring

3. Route integration
   - Group routes by role requirement
   - Apply middleware to route groups
   - Update route documentation

4. Testing phase
   - Run comprehensive test suite
   - Verify error handling
   - Test performance and caching

5. Production rollout
   - Deploy with feature flag
   - Monitor authorization logs
   - Gather feedback and metrics

## Progress Update (2025-01-16)

### Completed Tasks
1. Initial Setup:
   - [ ] Created api_auth.rs file
   - [ ] Created role.rs file
   - [ ] Set up test files
   - [ ] Defined role hierarchy requirements

2. Documentation:
   - [x] Documented token validation approach
   - [x] Created implementation plan
   - [x] Added migration strategy
   - [x] Defined testing requirements

### Next Steps
1. Implement Role enum and hierarchy
2. Create middleware structure
3. Add token claim parsing
4. Begin route integration
5. Set up monitoring and logging

## Recommendations for Implementation
1. Start with core role validation logic before middleware integration
2. Implement comprehensive logging early for debugging
3. Consider adding monitoring for:
   - Authorization failure rates
   - Role validation performance
   - Cache hit/miss ratio
4. Plan for future scaling:
   - Consider caching strategy for role validation
   - Design audit logging structure
   - Plan for role hierarchy expansion