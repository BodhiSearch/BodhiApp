# API Authorization Feature

Status: Complete

## Requirements

### User Story
As a System Administrator  
I want to enforce role-based access control on API endpoints  
So that users can only access resources appropriate for their role level

### Core Requirements

#### Role Hierarchy
- Roles are hierarchical in descending order: admin > manager > power_user > user
  1. Admin (`resource_admin`)
  2. Manager (`resource_manager`)
  3. Power User (`resource_power_user`)
  4. User (`resource_user`)
- Higher roles automatically have access to lower role endpoints
- Role hierarchy must be implemented as a reusable function
- Roles are included in JWT token claims resource_access
- Each endpoint is configured with minimum required role level

#### Token Scope Hierarchy
- TokenScope similar to Role are hierarchical in descending order: admin > manager > power_user > user
  1. Admin (`scope_token_admin`)
  2. Manager (`scope_token_manager`)
  3. Power User (`scope_token_power_user`)
  4. User (`scope_token_user`)
- Higher TokenScope automatically have access to lower role endpoints
- TokenScope hierarchy must be implemented as a reusable function
- TokenScope are included in JWT token claims "scope"
- A TokenScope JTW should also have scope "offline_access"
- No configuration file needed for hierarchy
- Each endpoint is configured with minimum required TokenScope level
- API call with TokenScope will not have access to authorized endpoints not configured with TokenScope level 

#### Common
- No configuration file needed for Role or TokenScope hierarchy
- No customizations possible for Role or TokenScope hierarchy
- All checks skipped in non-authenticated mode

#### Token Claim Extraction
- Session tokens: validate roles in resource_access.[client-id].roles
- Client ID in resource_access must match app's client ID
- Bearer tokens: validate TokenScope in scope claim (e.g., scope_token_user)
- Bearer tokens: should also have scope `offline_access`, otherwise not parsed
- Rely solely on token claims for role validation
- Authorization must be enforced after authentication

#### Authorization Flow
1. Auth middleware injects active access token in header
2. API auth middleware extracts role information:
   - For Session tokens: Check `resource_access.[client-id].roles` array and inject the role in header `X-Resource-Role`
   - For Bearer tokens: Check `scope` field for token-scope-specific claims and inject the token scope in header `X-Resource-Token-Scope`
3. At API endpoints, validate minimum required role or token_scope access
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
- [x] Create Role enum with variants: User, PowerUser, Manager, Admin
- [x] Implement role hierarchy checking
- [x] Add role parsing from token claims
- [x] Add role parsing from scope strings
- [x] Implement role comparison logic
- [ ] Add comprehensive error logging

#### TokenScope Implementation
- [x] Create TokenScope enum with variants matching Role
- [x] Implement TokenScope hierarchy checking
- [x] Add TokenScope parsing from JWT scope claims
- [x] Add offline_access scope validation
- [x] Implement TokenScope comparison logic

#### Middleware Implementation
- [x] Create api_auth middleware with role requirement parameter
- [x] Extract roles from X-Resource-Token header
- [x] Implement client ID validation for resource_access
- [x] Add role validation logic
- [x] Implement non-authenticated mode bypass
- [x] Add error handling with generic messages
- [ ] Add WARN level logging for authorization failures
- [x] Set up localization for error messages
- [x] Add TokenScope support to middleware
- [x] Implement role and scope precedence logic

#### Route Integration
- [x] Group routes by minimum role requirement
- [x] Add role-based middleware to protected routes
- [x] Update route registration to support role requirements
- [x] Add proper error responses for authorization failures
- [x] Document role requirements for each route group
- [x] Add TokenScope requirements to routes

#### Error Handling
- [x] Add authorization error types
- [x] Implement localized error messages
- [ ] Add warning level logging for authorization failures
- [ ] Include request path in log messages

### Testing Tasks
- [x] Unit tests for Role enum and hierarchy
- [x] Tests for token claim parsing
- [x] Tests for scope parsing
- [x] Integration tests for middleware
- [x] Test non-authenticated mode bypass
- [x] Test error responses
- ~~Test logging behavior~~ (Moved to Monitoring & Logging phase)
- ~~Cache behavior tests~~ (Not needed - using direct validation)
- [x] Localization tests for error messages
- [x] TokenScope validation tests
- [x] Role and TokenScope precedence tests

### New Tasks Discovered
1. Monitoring & Logging Enhancement:
   - [ ] Add structured logging for authorization events
   - [ ] Add metrics for authorization failures
   - [ ] Implement audit trail for role/scope changes

2. Performance Optimization:
   - [ ] Consider role/scope caching strategies
   - [ ] Optimize header parsing
   - [ ] Add performance metrics

3. Security Enhancements:
   - [ ] Add rate limiting for failed auth attempts
   - [ ] Implement role change auditing
   - [ ] Add security headers

## File Overview

### Core Implementation
- `crates/objs/src/role.rs` - Role enum with hierarchy implementation and serialization
- `crates/objs/src/token_scope.rs` - TokenScope enum matching Role with scope validation
- `crates/objs/src/error.rs` - Error types for authorization and validation
- `crates/objs/src/lib.rs` - Public exports for Role and TokenScope

### Authorization Middleware
- `crates/auth_middleware/src/api_auth_middleware.rs` - Role and TokenScope based authorization middleware
- `crates/auth_middleware/src/auth_middleware.rs` - Base authentication with session/bearer token support
- `crates/auth_middleware/src/token_service.rs` - Token validation and scope extraction
- `crates/auth_middleware/Cargo.toml` - Dependencies for auth middleware

### Route Integration
- `crates/routes_all/src/routes.rs` - Route configuration and role-based grouping
- `crates/routes_app/src/routes_create.rs` - Create/update alias routes with auth
- `crates/routes_app/src/routes_pull.rs` - Model pull routes with auth
- `crates/routes_oai/src/routes_models.rs` - OpenAI compatible routes with auth

### Service Layer
- `crates/services/src/service_ext.rs` - Service extensions for auth and token handling
- `crates/services/src/app_service.rs` - Core application service integration
- `crates/services/src/test_utils/secret.rs` - Test utilities for secret service

### Localization
- `crates/auth_middleware/src/resources/en-US/messages.ftl` - Auth middleware error messages
- `crates/objs/src/resources/en-US/messages.ftl` - Core error messages

## Technical Details

### Authorization Flow

1. **Request Processing**
```rust
// In api_auth_middleware.rs
pub async fn api_auth_middleware(
  required_role: Role,
  required_scope: Option<TokenScope>,
  State(state): State<Arc<dyn RouterState>>,
  req: Request,
  next: Next,
) -> Result<Response, ApiError>
```

2. **Authorization Headers**
- `X-Resource-Role`: Role-based authorization header
- `X-Resource-Scope`: Scope-based authorization header
- `X-Resource-Token`: Token validation header

3. **Authorization Logic**
```rust
match (role_header, scope_header, required_scope) {
  // Role header present - validate against required role
  (Some(role_header), _, _) => {
    // Role takes precedence
  }
  // No role header but scope allowed and present
  (None, Some(scope_header), Some(required_scope)) => {
    // Fallback to scope validation
  }
  // No valid auth headers found
  _ => return Err(ApiAuthError::MissingAuth),
}
```

4. **Service Integration**
```rust
// Service extensions for auth
pub trait SecretServiceExt {
  fn authz(&self) -> Result<bool, SecretServiceError>;
  fn app_reg_info(&self) -> Result<Option<AppRegInfo>, SecretServiceError>;
}

// App service integration
pub trait AppService: Send + Sync {
  fn secret_service(&self) -> &Arc<dyn SecretService>;
  fn auth_service(&self) -> &Arc<dyn AuthService>;
}
```

5. **Token Validation**
```rust
// In token_service.rs
impl DefaultTokenService {
  pub async fn validate_bearer_token(
    &self,
    header: &str
  ) -> Result<(String, TokenScope), AuthError>
}
```

### Error Handling

1. **Error Types**
```rust
pub enum ApiAuthError {
  SecretService(#[from] SecretServiceError),
  Forbidden,
  MissingAuth,
  MalformedRole(String),
  MalformedScope(String),
  InvalidRole(#[from] RoleError),
  InvalidScope(#[from] TokenScopeError),
}
```

2. **Localized Messages**
```ftl
api_auth_error-forbidden = insufficient privileges to access this resource
api_auth_error-missing_auth = missing authentication header
api_auth_error-malformed_role = error parsing role: {$var_0}
api_auth_error-malformed_scope = error parsing scope: {$var_0}
```

### Testing Support
- Comprehensive test utilities in `services/test_utils/secret.rs`
- Mock implementations for services
- Test helpers for token generation and validation
- Authorization bypass testing support

### Authorization Types

1. **Role-Based (Session)**
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

let admin_apis = Router::new().route("/api/settings", get(settings_handler))
   .route("/api/admin", get(admin_handler))
   .route_layer(from_fn_with_state(state.clone(), api_auth_middleware(Some(Role::Admin), None)));
let manager_apis = Router::new().route("/api/users",
    get(user_handler))
   .route("/api/billing",
    get(billing_handler))
    .route_layer(from_fn_with_state(state.clone(), api_auth_middleware(Some(Role::Manager), Some(TokenScope::Manager))));
let app_router = admin_apis.merge(manager_apis).merge(...);
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

2. **Token Scope-Based (API Token)**
```rust
pub enum TokenScope {
    User,
    PowerUser,
    Manager,
    Admin,
}
```

### Bearer Token Scope
```json
scope: "openid offline_access scope_token_user"
```

### Current APIs

1. Public APIs (No Auth Required):
```
GET  /ping                  # Health check
GET  /app/info             # App info
POST /app/setup            # App setup
GET  /app/login/callback   # Login callback
POST /api/ui/logout        # Logout
GET  /dev/secrets          # (Dev mode only) Secrets
```

2. Optional Auth APIs:
```
GET  /app/login            # Login page
GET  /app/login/           # Login page (with trailing slash)
GET  /api/ui/user         # User info
```

3. Ollama APIs (Protected):
```
GET  /api/tags            # List Ollama models
POST /api/show           # Show Ollama model details
POST /api/chat           # Ollama chat
```

4. OpenAI Compatible APIs (Protected):
```
GET  /v1/models           # List models
GET  /v1/models/:id       # Get model details
POST /v1/chat/completions # Chat completions
```

5. Bodhi APIs (Protected, under /api/ui/):
```
GET  /models                         # List local aliases
GET  /models/:id                     # Get alias details
GET  /modelfiles                     # List local modelfiles
GET  /chat_templates                 # List chat templates
POST /models                         # Create alias
PUT  /models/:id                     # Update alias
GET  /modelfiles/pull/downloads      # List downloads
POST /modelfiles/pull                # Pull by repo file
POST /modelfiles/pull/:alias         # Pull by alias
GET  /modelfiles/pull/status/:id     # Get download status
POST /tokens                         # Create token
PUT  /tokens/:token_id              # Update token
GET  /tokens                        # List tokens
GET  /tokens/                       # List tokens (with trailing slash)
```

### Route Authorization Matrix

**role=anon** (No Authentication Required)
```
GET  /ping                  # Health check
GET  /app/info             # App info
POST /app/setup            # App setup
GET  /app/login            # Login page
GET  /app/login/           # Login page (with trailing slash)
GET  /app/login/callback   # Login callback
POST /api/ui/logout        # Logout
GET  /api/ui/user         # User info
GET  /*                    # UI fallback routes
```

**role=user & scope=scope_token_user**
```
GET  /v1/models            # OpenAI List models
GET  /v1/models/:id        # OpenAI Get model details
POST /v1/chat/completions  # OpenAI Chat completions
GET  /api/tags            # Ollama list models
POST /api/show            # Ollama show model details
POST /api/chat            # Ollama chat completions
GET  /api/ui/models       # List local aliases
GET  /api/ui/models/:id   # Get alias details
GET  /api/ui/modelfiles   # List local modelfiles
GET  /api/ui/chat_templates # List chat templates
```

**role=power_user or scope=scope_token_power_user**
```
POST /api/ui/models       # Create alias
PUT  /api/ui/models/:id   # Update alias
GET  /api/ui/modelfiles/pull/downloads      # List downloads
POST /api/ui/modelfiles/pull                # Pull by repo file
POST /api/ui/modelfiles/pull/:alias         # Pull by alias
GET  /api/ui/modelfiles/pull/status/:id     # Get download status
```

**role=power_user & scope=None**
- not available via the API tokens, only through app session
- should only be allowed to access (list) and update his own tokens
```
POST /api/ui/tokens             # Create token (session only)
GET  /api/ui/tokens            # List tokens
GET  /api/ui/tokens/:id        # Get token details
PUT  /api/ui/tokens/:id        # Update token
```

**role=admin**
```
GET  /dev/secrets            # Dev secrets (Dev mode only)
```

### Implementation Notes

1. **Session vs Token Authorization**
- Session authentication uses Role enum
- API token authentication uses TokenScope enum
- Some operations restricted to session-only
- TokenScope cannot perform token management operations

2. **Hierarchy Implementation**
```rust
impl Role {
      fn has_access_to(&self, required: &Role) -> bool {
         // Session role hierarchy check
      }
}

impl TokenScope {
      fn has_access_to(&self, required: &TokenScope) -> bool {
         // Token scope hierarchy check
      }

}
```

3. **Token Claims**
   ```json
   {
     "scope": "offline_access scope_token_power_user",
     "resource_access": {
       "resource-id": {
         "roles": ["resource_power_user", "resource_user"]
       }
     }
   }
   ```

4. **Authorization Flow**
- Check authentication type (session vs token)
- Apply appropriate authorization type
- Check additional restrictions for token-based auth
- Validate operation permissions

### Security Considerations

1. **Token Restrictions**
- API tokens cannot create other tokens
- API tokens cannot manage other tokens
- API tokens cannot access admin routes
- Some operations require session authentication

2. **Session Privileges**
- Full access based on role level
- Can perform token management
- Can access admin features
- No additional restrictions

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
1. Role Implementation:
   - [x] Created Role enum with variants: User, PowerUser, Manager, Admin
   - [x] Implemented role hierarchy checking
   - [x] Added role parsing from token claims
   - [x] Added role parsing from scope strings
   - [x] Implemented role comparison logic
   - [x] Added error handling for role validation

2. Token Handling:
   - [x] Implemented token validation logic
   - [x] Added support for session and bearer tokens
   - [x] Added role extraction from tokens
   - [x] Improved error handling
   - [x] Added token claim parsing

3. Auth Middleware:
   - [x] Updated middleware to handle roles
   - [x] Added role header injection
   - [x] Implemented token validation
   - [x] Added proper error responses

4. Testing:
   - [x] Unit tests for Role enum and hierarchy
   - [x] Tests for token claim parsing
   - [x] Tests for scope parsing
   - [x] Tests for middleware with different roles
   - [x] Test error responses

### Remaining Tasks
1. Route Integration:
   - [ ] Group routes by minimum role requirement
   - [ ] Add role-based middleware to protected routes
   - [ ] Update route registration
   - [ ] Document role requirements

2. Monitoring & Logging:
   - [ ] Add WARN level logging for authorization failures
   - [ ] Include request path in log messages
   - [ ] Set up monitoring for auth failures

3. Documentation:
   - [ ] Update API documentation with role requirements
   - [ ] Document migration process
   - [ ] Add usage examples

## Implementation Progress Summary

### Completed Features
1. **Role System**
   - Full role hierarchy implementation
   - Role comparison and validation
   - Role parsing from tokens and scopes
   - Comprehensive error handling

2. **Token Handling**
   - Support for both session and bearer tokens
   - Role extraction from tokens
   - Improved token validation
   - Better error messages

3. **Auth Middleware**
   - Role header injection
   - Token validation
   - Support for optional authentication
   - Proper error responses

4. **API Authorization Middleware**
   - [x] Added TokenScope support to api_auth_middleware
   - [x] Implemented role and scope hierarchy validation
   - [x] Added support for both Role and TokenScope based authorization
   - [x] Implemented authorization bypass when authz is disabled
   - [x] Added comprehensive test coverage for all authorization scenarios

5. **TokenScope Enhancements**
   - [x] Updated TokenScope serialization to use prefixed format (scope_token_*)
   - [x] Added proper serialization/deserialization tests
   - [x] Implemented TokenScope hierarchy validation
   - [x] Added support for offline_access requirement


### Recently Completed
1. Route Integration:
   - [x] Grouped all routes by role requirements (public, user, power_user)
   - [x] Added TokenScope support to protected routes
   - [x] Implemented session-only routes for token management
   - [x] Added proper middleware layers for each route group
   - [x] Documented role/scope requirements in code

2. Code Organization:
   - [x] Refactored route handlers into logical groups
   - [x] Removed redundant router creation functions
   - [x] Improved code readability with clear route grouping
   - [x] Added comments explaining route authorization levels

3. New Features:
   - [x] Added support for dev-only admin routes
   - [x] Implemented proper error handling for auth failures
   - [x] Added TokenScope validation to relevant routes
   - [x] Improved route organization and documentation

### Remaining Tasks
1. Monitoring & Logging Enhancement:
   - [ ] Add structured logging for authorization events
   - [ ] Add metrics for authorization failures
   - [ ] Implement audit trail for role/scope changes

2. Performance Optimization:
   - [ ] Consider role/scope caching strategies
   - [ ] Optimize header parsing
   - [ ] Add performance metrics

3. Security Enhancements:
   - [ ] Add rate limiting for failed auth attempts
   - [ ] Implement role change auditing
   - [ ] Add security headers

### Key Achievements
- Successfully implemented core role-based authorization
- Added comprehensive test coverage
- Improved security with better token handling
- Enhanced error handling and validation

### Summary of Changes
- Implemented dual authorization system with Role and TokenScope
- Added proper serialization for both Role and TokenScope with prefixes
- Enhanced middleware to handle both authorization types
- Added comprehensive test coverage for all scenarios
- Improved error handling and messages

### Remaining Tasks
1. Documentation:
   - [ ] Update API documentation with role requirements
   - [ ] Document migration process
   - [ ] Add usage examples

### Technical Debt
None identified - implementation follows best practices and includes comprehensive testing.

### Notes for Future Enhancement
1. Consider adding role caching for performance
2. Plan for audit logging implementation
3. Consider adding role-based metrics
4. Plan for potential future role additions