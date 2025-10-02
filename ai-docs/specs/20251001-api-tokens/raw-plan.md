# API Token Feature - Raw Implementation Analysis

**Date**: 2025-10-01
**Purpose**: Complete documentation of ApiToken feature implementation extracted from git diff analysis
**Source**: Analysis of local changes in working branch

## Executive Summary

The ApiToken feature transitions from Keycloak offline token exchange to a self-contained, database-backed token system. The implementation includes:

- Database schema with `token_prefix` and `token_hash` for secure storage
- Token format: `bodhiapp_<random_string>` (32-byte cryptographically secure random)
- SHA-256 hashing with constant-time comparison for security
- Scope-based authorization with role enforcement
- Simplified token service removing Keycloak dependencies
- Complete CRUD API with OpenAPI documentation

## 1. Database Layer Changes

### 1.1 Migration File Updates

**File**: `crates/services/migrations/0003_create_api_tokens.up.sql`

**Changes**:
- Renamed `token_id` → `token_prefix` (stores `bodhiapp_` + first 8 chars for fast lookup)
- Added `scopes TEXT NOT NULL` field for permission storage
- Updated index from `idx_api_tokens_token_id` → `idx_api_tokens_token_prefix`

**Schema**:
```sql
CREATE TABLE api_tokens (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    name TEXT DEFAULT '',
    token_prefix TEXT NOT NULL UNIQUE,
    token_hash TEXT NOT NULL,
    scopes TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('active', 'inactive')),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_api_tokens_token_prefix ON api_tokens(token_prefix);
```

### 1.2 Domain Object Updates

**File**: `crates/services/src/db/objs.rs`

**Changes to `ApiToken` struct**:
```rust
pub struct ApiToken {
  pub id: String,
  pub user_id: String,
  pub name: String,
  pub token_prefix: String,  // Changed from token_id
  pub token_hash: String,
  pub scopes: String,        // Added field
  pub status: TokenStatus,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}
```

### 1.3 Database Service Updates

**File**: `crates/services/src/db/service.rs`

**Method Changes**:
- **Removed**: `create_api_token_from(name: &str, token: &str)` - No longer needed for offline token exchange
- **Updated**: `get_api_token_by_token_id()` → `get_api_token_by_prefix(prefix: &str)`
- **Updated**: `create_api_token()` to include `scopes` field in INSERT

**Key Implementation Details**:
- SELECT query now includes `scopes` field
- INSERT query binds `token_prefix`, `token_hash`, and `scopes`
- Prefix-based lookup for performance with indexed column

## 2. Token Generation and Validation

### 2.1 Token Format and Generation

**Format**: `bodhiapp_<random_string>`

**Generation Process** (from `crates/routes_app/src/routes_api_token.rs`):
```rust
// 1. Generate 32 random bytes
let mut random_bytes = [0u8; 32];
rand::rng().fill_bytes(&mut random_bytes);

// 2. Base64 encode (URL-safe, no padding)
let random_string = general_purpose::URL_SAFE_NO_PAD.encode(random_bytes);

// 3. Create token with prefix
let token_str = format!("bodhiapp_{}", random_string);

// 4. Extract prefix for lookup (first 8 chars after "bodhiapp_")
let token_prefix = &token_str[.."bodhiapp_".len() + 8];

// 5. Hash full token for storage
let mut hasher = Sha256::new();
hasher.update(token_str.as_bytes());
let token_hash = format!("{:x}", hasher.finalize());
```

**Security Properties**:
- **Prefix for lookup**: Public, indexed, fast database queries
- **Hash for validation**: Secure, constant-time comparison prevents timing attacks
- **Token shown once**: Full token only returned at creation time

### 2.2 Token Validation Logic

**File**: `crates/auth_middleware/src/token_service.rs`

**Process**:
```rust
// 1. Check token prefix
if bearer_token.starts_with(BODHIAPP_TOKEN_PREFIX) {
    // 2. Extract prefix for lookup
    let token_prefix = &bearer_token[..BODHIAPP_TOKEN_PREFIX.len() + 8];

    // 3. Query database by prefix
    if let Some(api_token) = db_service.get_api_token_by_prefix(token_prefix).await? {
        // 4. Check token status
        if api_token.status == TokenStatus::Inactive {
            return Err(AuthError::TokenInactive);
        }

        // 5. Hash provided token
        let mut hasher = Sha256::new();
        hasher.update(bearer_token.as_bytes());
        let hash = format!("{:x}", hasher.finalize());

        // 6. Constant-time comparison
        if constant_time_eq::constant_time_eq(hash.as_bytes(), api_token.token_hash.as_bytes()) {
            let scope = TokenScope::from_str(&api_token.scopes)?;
            return Ok((bearer_token.to_string(), ResourceScope::Token(scope)));
        } else {
            return Err(AuthError::TokenNotFound);
        }
    }
}
```

**Key Security Features**:
- Constant-time comparison prevents timing attacks
- Prefix-based lookup separates public lookup from secret comparison
- Token status check supports revocation
- Scope extracted from database for authorization

### 2.3 Legacy Code Removal

**Removed from `token_service.rs`**:
- All Keycloak offline token validation logic
- JWT signature verification for API tokens (now database-backed)
- Token caching for offline tokens (cache still used for external client tokens)
- `validate_token_claims()` method for offline tokens
- `create_token_digest()` function for old token format

**Retained**:
- External client token validation and exchange (for OAuth2 flows)
- Session token handling
- Cache service for external token exchange results

## 3. Authentication Middleware Integration

### 3.1 Middleware Updates

**File**: `crates/auth_middleware/src/auth_middleware.rs`

**Changes**:
- Added `DbError` to `AuthError` enum for database operation errors
- Updated test infrastructure for database-backed token testing
- Added comprehensive test: `test_auth_middleware_bodhiapp_token_success`

**Test Flow**:
```rust
// 1. Create random token
let mut random_bytes = [0u8; 32];
rand::rng().fill_bytes(&mut random_bytes);
let token_str = format!("bodhiapp_{}", random_string);

// 2. Insert into database
let api_token = ApiToken {
    token_prefix: /* first 8 chars */,
    token_hash: /* SHA-256 hash */,
    scopes: "scope_token_user".to_string(),
    status: TokenStatus::Active,
    /* ... */
};
db_service.create_api_token(&mut api_token).await?;

// 3. Make authenticated request
let req = Request::get("/with_auth")
    .header("Authorization", format!("Bearer {}", token_str));

// 4. Verify middleware injects scope header
assert_eq!(response.x_resource_scope, "scope_token_user");
```

### 3.2 Authorization Flow

**Headers Injected by Middleware**:
- `X-Resource-Token`: The validated access token (for database tokens, this is the API token itself)
- `X-Resource-Scope`: The token's scope from database (e.g., "scope_token_user")
- `X-Resource-Role`: User's role (for session-based auth, not applicable to API tokens)

**Precedence**:
- Bearer token authentication takes precedence over session-based authentication
- Database tokens are validated before attempting external client token exchange

## 4. API Routes Implementation

### 4.1 Create Token Endpoint

**File**: `crates/routes_app/src/routes_api_token.rs`

**Endpoint**: `POST /api/tokens`

**Implementation**:
```rust
pub async fn create_token_handler(
    headers: HeaderMap,
    State(state): State<Arc<dyn RouterState>>,
    WithRejection(Json(payload), _): WithRejection<Json<CreateApiTokenRequest>, ApiError>,
) -> Result<(StatusCode, Json<ApiTokenResponse>), ApiError>
```

**Process**:
1. Extract `X-Resource-Token` header to get user session token
2. Extract user ID from JWT claims
3. Extract `X-Resource-Role` header to determine user's role
4. Map user role to token scope (Admin→Admin, PowerUser→PowerUser, etc.)
5. Generate new `bodhiapp_` token with random bytes
6. Hash token and extract prefix
7. Create `ApiToken` struct with scope from user's role
8. Insert into database via `db_service.create_api_token()`
9. Return full token in response (only time user sees it)

**Authorization**:
- User can create tokens at their own role level or below
- For pilot: only User and PowerUser level tokens are allowed (enforced by role restrictions)

**Request**:
```json
{
  "name": "optional-token-name"
}
```

**Response**:
```json
{
  "offline_token": "bodhiapp_<random_string>"
}
```

### 4.2 Other Token Endpoints

**List Tokens**: `GET /api/tokens` (with pagination)
**Get Token**: `GET /api/tokens/{id}`
**Update Token**: `PUT /api/tokens/{id}` (name and status only)
**Delete Token**: `DELETE /api/tokens/{id}`

All endpoints use database service methods with proper authorization checks.

## 5. Scope and Role System

### 5.1 Token Scopes

**File**: `crates/objs/src/token_scope.rs` (inferred from usage)

**Scopes**:
- `scope_token_admin` - Full administrative access
- `scope_token_manager` - Manager level access
- `scope_token_power_user` - PowerUser level access
- `scope_token_user` - Standard user access

**Storage**: Stored as string in `api_tokens.scopes` field

### 5.2 Role Mapping

**Role → TokenScope mapping**:
```rust
let token_scope = match user_role {
    Role::Admin => TokenScope::Admin,
    Role::Manager => TokenScope::Manager,
    Role::PowerUser => TokenScope::PowerUser,
    Role::User => TokenScope::User,
};
```

**Authorization**:
- Token scope determines what API operations are allowed
- Enforced by `api_auth_middleware` using `ResourceScope::Token(scope)`
- Hierarchical: Admin > Manager > PowerUser > User

## 6. Testing Infrastructure Updates

### 6.1 Test Utilities

**File**: `crates/services/src/test_utils/db.rs`

**Changes**:
- Removed `create_api_token_from()` wrapper
- Updated `get_api_token_by_token_id()` → `get_api_token_by_prefix()`
- All test database methods now support new schema

### 6.2 Test Data Updates

**Files**: Multiple test files updated with new token format

**Example** (from `routes_api_token.rs` tests):
```rust
ApiToken {
    id: Uuid::new_v4().to_string(),
    user_id: user_id.to_string(),
    name: format!("Test Token {}", i),
    token_prefix: format!("bodhiapp_test{:02}", i),  // Updated
    token_hash: "token_hash".to_string(),
    scopes: "scope_token_user".to_string(),          // Added
    status: TokenStatus::Active,
    created_at: app_service.time_service().utc_now(),
    updated_at: app_service.time_service().utc_now(),
}
```

### 6.3 Integration Test

**New Test**: `test_auth_middleware_bodhiapp_token_success`

Tests complete flow:
1. Generate token with random bytes
2. Store in database with proper prefix and hash
3. Make HTTP request with Bearer token
4. Verify middleware validates and injects proper headers

## 7. Import Organization Changes

**Files affected**:
- `crates/objs/src/lib.rs`
- `crates/objs/src/alias.rs`
- `crates/objs/src/user_alias.rs`
- `crates/commands/src/cmd_create.rs`
- `crates/services/src/hub_service.rs`
- `crates/services/src/data_service.rs`

**Changes**:
- Reordered imports to have `Alias` before `UserAlias` (alphabetical)
- Removed unused imports related to offline token handling
- Added new imports for token generation: `rand`, `sha2`, `base64`, `uuid`, `chrono::Utc`

## 8. Dependencies Added

**Cargo.toml changes**:

**auth_middleware**:
- `base64` - For token encoding
- `rand` - For cryptographically secure random bytes
- `sha2` - For SHA-256 hashing
- `constant_time_eq` - For secure token comparison

**routes_app**:
- `base64` - For token encoding
- `rand` - For cryptographically secure random bytes
- `sha2` - For SHA-256 hashing
- `uuid` - For generating token IDs
- `chrono` - For timestamps

## 9. Error Handling Updates

### 9.1 New Error Types

**AuthError additions**:
```rust
#[error(transparent)]
DbError(#[from] DbError),
```

**ApiTokenError** (existing, but now used):
- `AccessTokenMissing` - When X-Resource-Token header is missing
- `TokenInactive` - When token status is inactive in database
- `TokenNotFound` - When token doesn't exist or hash doesn't match

### 9.2 Error Flow

1. Database errors propagate as `DbError`
2. Converted to `AuthError::DbError` in middleware
3. Translated to appropriate HTTP status codes by error handling layer
4. Returned as OpenAI-compatible error responses

## 10. Security Considerations

### 10.1 Implemented Security Features

1. **Constant-Time Comparison**: Prevents timing attacks on token validation
2. **Hash Storage**: Never store raw tokens in database
3. **Secure Random Generation**: Uses cryptographically secure RNG
4. **Prefix-Based Lookup**: Separates public lookup from secret comparison
5. **Token Revocation**: Status field allows deactivating tokens
6. **Single Display**: Token shown only once at creation time

### 10.2 Token Lifecycle Security

- **Creation**: Only authenticated users can create tokens
- **Storage**: Tokens hashed with SHA-256 before database storage
- **Validation**: Constant-time comparison prevents side-channel attacks
- **Revocation**: Status field enables token deactivation without deletion
- **Audit**: Created_at and updated_at timestamps for tracking

## 11. Key Design Decisions

### 11.1 Why Database-Backed?

**Advantages over Offline Token Exchange**:
- Simpler architecture (no Keycloak offline token dependency)
- Better control over token lifecycle (revocation, status management)
- Reduced complexity (no JWT validation for API tokens)
- Better performance (single database lookup vs JWT validation + exchange)

### 11.2 Why Prefix + Hash?

**Rationale**:
- **Prefix**: Fast indexed lookup without exposing secrets
- **Hash**: Secure validation with constant-time comparison
- **Separation**: Defense-in-depth against timing attacks

### 11.3 Why Show Token Once?

**Security Best Practice**:
- Forces users to store tokens securely
- Reduces exposure window
- Standard practice (GitHub, Stripe, OpenAI, etc.)
- Hash-only storage means no token recovery possible

## 12. Migration from Old System

### 12.1 Breaking Changes

**Database Schema**:
- `token_id` field renamed to `token_prefix`
- `scopes` field added (required)
- Index changed to use `token_prefix`

**API Changes**:
- Token format changed from JTI to `bodhiapp_` prefix
- Token creation no longer accepts external JWT tokens
- Token validation logic completely replaced

### 12.2 Migration Strategy

Since no production data exists:
- Directly modify migration file `0003_create_api_tokens.up.sql`
- No need for data migration or backward compatibility
- Fresh start with new token format

## 13. Feature Completeness

### 13.1 Implemented Features

✅ Database schema with secure storage
✅ Token generation with `bodhiapp_` prefix
✅ SHA-256 hashing with constant-time comparison
✅ Scope-based authorization
✅ Create token API endpoint
✅ Token validation in auth middleware
✅ Integration with database service
✅ Comprehensive testing infrastructure
✅ Role-based token creation enforcement

### 13.2 Not Yet Implemented

⏸️ Token expiration (future enhancement)
⏸️ Token usage tracking (future enhancement)
⏸️ Token rate limiting (future enhancement)
⏸️ Manager and Admin level tokens (restricted for pilot)

## 14. Code Quality

### 14.1 Testing Coverage

- Unit tests for token generation
- Integration tests for database operations
- Middleware integration tests
- API endpoint tests with mocked services
- Test utilities updated for new schema

### 14.2 Error Handling

- Comprehensive error types for all failure modes
- Proper error propagation through middleware
- OpenAI-compatible error responses
- Localized error messages

### 14.3 Code Organization

- Clear separation of concerns (database, service, middleware, routes)
- Consistent naming conventions
- Proper use of Rust traits and dependency injection
- Well-documented with inline comments for complex logic

## 15. Performance Considerations

### 15.1 Database Optimization

- Indexed `token_prefix` for fast lookups
- Single query for token validation
- No joins required for validation flow

### 15.2 Validation Performance

- Prefix-based lookup O(log n) with index
- Constant-time hash comparison
- No external service calls for validation

### 15.3 Caching Strategy

- External client tokens still cached
- Database tokens not cached (single fast query)
- Status check prevents inactive token usage

## Conclusion

The ApiToken feature has been successfully implemented with a secure, database-backed approach. The implementation removes Keycloak offline token dependencies, simplifies the token lifecycle, and provides a foundation for future enhancements like token expiration, usage tracking, and rate limiting.

The feature is ready for re-implementation from scratch following the patterns and security practices documented in this analysis.