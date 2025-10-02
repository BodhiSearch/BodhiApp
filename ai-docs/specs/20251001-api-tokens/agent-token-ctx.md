# API Token Implementation - Shared Knowledge Base

**Purpose**: Accumulated insights and patterns discovered during ApiToken implementation
**Usage**: Each agent reads this before starting, updates with new insights after completing their phase

---

## How to Use This Context

### Before Starting Your Phase
1. Read this entire file carefully
2. Understand the patterns and anti-patterns
3. Apply relevant patterns to your phase
4. Avoid documented pitfalls

### After Completing Your Phase
1. Review your work for reusable insights
2. Extract patterns that could help future agents
3. Update existing sections if you found better approaches
4. Add new sections for novel discoveries
5. Document any anti-patterns you encountered

### Context Entry Format

```markdown
## [Category/Pattern Name] - Updated [YYYY-MM-DD]

**Last Updated By**: Phase N agent
**Relevance**: [Which phases this applies to]

### Description
[What is this pattern/insight]

### When to Use
[Scenarios where this pattern applies]

### How to Implement
[Step-by-step or code example]

### Why It Works
[Technical explanation]

### Gotchas
[Edge cases, pitfalls, things to watch out for]

### Related Patterns
- [Link to related insight in this file]
```

---

## Initial Context - 2025-10-01

### Source of Knowledge
This context file is initialized with insights extracted from the previous implementation attempt. As agents work through phases, they should validate these insights and add their own discoveries.

---

## Database Patterns

### Pattern: Token Prefix for Fast Lookup

**Relevance**: Phase 1, Phase 2
**Last Updated**: 2025-10-01 (from raw-plan.md analysis)

#### Description
Store a non-secret prefix of the token separately from its hash to enable fast database lookups without exposing the full token.

#### When to Use
When you need to validate tokens that are:
- Used frequently (performance matters)
- Must remain secret (can't store plaintext)
- Need revocation capability (database status check)

#### How to Implement
```rust
// Token format: "bodhiapp_<random_string>"
// Store first 8 chars after prefix as lookup key
let token_prefix = &token_str[.."bodhiapp_".len() + 8];

// Store in database with unique index
CREATE INDEX idx_api_tokens_token_prefix ON api_tokens(token_prefix);

// Lookup by prefix (fast, indexed)
SELECT * FROM api_tokens WHERE token_prefix = ?
```

#### Why It Works
- **Performance**: Indexed prefix enables O(log n) lookup
- **Security**: Prefix alone is not enough to authenticate (needs hash match)
- **Defense in Depth**: Even if prefix is leaked, hash comparison still required

#### Gotchas
- Prefix must be long enough to avoid collisions (8 chars = 2^48 possibilities)
- Prefix must be consistent length for index efficiency
- Don't use prefix for authentication, only for lookup

#### Related Patterns
- See "Constant-Time Hash Comparison" below

---

### Pattern: Scopes as String Storage

**Relevance**: Phase 1, Phase 4
**Last Updated**: 2025-10-01

#### Description
Store token scopes as plain string in database rather than normalized table or enum.

#### When to Use
- When scopes map directly to a well-defined Rust enum
- When scope changes are rare
- When scope parsing is cheap

#### How to Implement
```rust
// In database
scopes: TEXT NOT NULL  // e.g., "scope_token_user"

// In Rust
pub scopes: String,

// Parsing
let token_scope = TokenScope::from_str(&api_token.scopes)?;
```

#### Why It Works
- **Simplicity**: No joins, no normalization complexity
- **Type Safety**: Rust enum provides compile-time guarantees
- **Performance**: String comparison is fast for short strings
- **Flexibility**: Easy to extend with new scopes

#### Gotchas
- Must have `FromStr` implementation for TokenScope
- Must handle parsing errors gracefully
- String format must be documented and consistent

#### Related Patterns
- See "Role to Scope Mapping" in Authorization section

---

## Security Patterns

### Pattern: Constant-Time Hash Comparison

**Relevance**: Phase 2, Phase 3
**Last Updated**: 2025-10-01

#### Description
Use constant-time comparison for token hash validation to prevent timing attacks.

#### When to Use
- Any time you're comparing secrets
- Token validation
- Password verification
- API key validation

#### How to Implement
```rust
use constant_time_eq::constant_time_eq;

// Hash the provided token
let mut hasher = Sha256::new();
hasher.update(bearer_token.as_bytes());
let provided_hash = format!("{:x}", hasher.finalize());

// Compare with stored hash
if constant_time_eq::constant_time_eq(
    provided_hash.as_bytes(),
    stored_hash.as_bytes()
) {
    // Valid token
} else {
    // Invalid token
}
```

#### Why It Works
- **Security**: Prevents attackers from using timing differences to brute-force hashes
- **Defense in Depth**: Even if prefix lookup is somehow compromised, timing attacks won't help
- **Best Practice**: Industry standard for cryptographic comparisons

#### Gotchas
- Must use for EVERY secret comparison, not just some
- Regular `==` comparison is NOT constant-time
- Comparison must be on final hash values, not intermediate steps

#### Related Patterns
- See "SHA-256 Token Hashing" below

---

### Pattern: SHA-256 Token Hashing

**Relevance**: Phase 2, Phase 4
**Last Updated**: 2025-10-01

#### Description
Hash full tokens with SHA-256 before storing in database.

#### When to Use
- Storing any authentication credential
- When you need fast hash computation
- When you need deterministic hashing (no salt needed for tokens)

#### How to Implement
```rust
use sha2::{Digest, Sha256};

let mut hasher = Sha256::new();
hasher.update(token_str.as_bytes());
let token_hash = format!("{:x}", hasher.finalize());

// Store token_hash in database
// Never store token_str in plaintext
```

#### Why It Works
- **Irreversible**: Cannot recover token from hash
- **Fast**: SHA-256 is optimized and fast
- **Deterministic**: Same token always produces same hash (needed for validation)
- **Collision Resistant**: Extremely unlikely for two tokens to have same hash

#### Gotchas
- Must hash the FULL token, not just a portion
- Use hex encoding for storage (format!("{:x}", ...))
- Don't salt token hashes (need deterministic output for validation)
- SHA-256, not SHA-1 (SHA-1 is deprecated)

#### Related Patterns
- See "Cryptographically Secure Random Generation" below

---

### Pattern: Cryptographically Secure Random Generation

**Relevance**: Phase 4
**Last Updated**: 2025-10-01

#### Description
Generate tokens using cryptographically secure random number generator.

#### When to Use
- Generating API tokens
- Generating session IDs
- Any security-sensitive random data

#### How to Implement
```rust
use rand::RngCore;
use base64::{engine::general_purpose, Engine};

// Generate 32 random bytes
let mut random_bytes = [0u8; 32];
rand::rng().fill_bytes(&mut random_bytes);

// Encode as URL-safe base64
let random_string = general_purpose::URL_SAFE_NO_PAD.encode(random_bytes);

// Use in token
let token_str = format!("bodhiapp_{}", random_string);
```

#### Why It Works
- **Unpredictable**: Impossible to guess next token
- **Unique**: 32 bytes = 2^256 possibilities (no collisions)
- **Standard**: Industry best practice
- **URL Safe**: Base64 URL_SAFE encoding works in HTTP headers

#### Gotchas
- Use `rand::rng()`, not `rand::thread_rng()` (simpler API)
- Use `URL_SAFE_NO_PAD`, not standard base64 (no padding chars)
- 32 bytes is minimum for security-critical tokens
- Don't use `rand::random()` (not cryptographically secure)

#### Related Patterns
- See "Token Format with Prefix" below

---

### Pattern: Show Token Once at Creation

**Relevance**: Phase 4
**Last Updated**: 2025-10-01

#### Description
Return full token to user only at creation time; never show again.

#### When to Use
- API token creation
- Password reset tokens
- Any credential generation

#### How to Implement
```rust
// In create_token_handler
let token_str = format!("bodhiapp_{}", random_string);

// Store hash in database
db_service.create_api_token(&mut api_token).await?;

// Return FULL token in response (only time user sees it)
Ok((
    StatusCode::CREATED,
    Json(ApiTokenResponse {
        offline_token: token_str,  // Full token with prefix
    })
))
```

#### Why It Works
- **Security**: Reduces exposure window
- **Forces Secure Storage**: Users must save token immediately
- **Standard Practice**: GitHub, Stripe, OpenAI all do this
- **Impossible to Retrieve**: Only hash stored, so can't show again

#### Gotchas
- User loses token = must create new one
- Must clearly communicate "save this token" in UI
- Response must be over HTTPS to prevent interception
- Consider rate limiting token creation to prevent abuse

#### Related Patterns
- See "Token Revocation via Status" below

---

### Pattern: Token Revocation via Status

**Relevance**: Phase 1, Phase 2
**Last Updated**: 2025-10-01

#### Description
Use status field to enable token revocation without deletion.

#### When to Use
- When you need audit trail (keep record of revoked tokens)
- When deletion might break foreign key constraints
- When you want to potentially reactivate tokens

#### How to Implement
```rust
// In database
status TEXT NOT NULL CHECK (status IN ('active', 'inactive'))

// In validation
if api_token.status == TokenStatus::Inactive {
    return Err(AuthError::TokenInactive);
}

// In revocation API
token.status = TokenStatus::Inactive;
db_service.update_api_token(user_id, &mut token).await?;
```

#### Why It Works
- **Immediate Effect**: Status check in every validation
- **Audit Trail**: Keep history of revoked tokens
- **Reversible**: Can reactivate if needed
- **Performance**: Status check is fast (indexed field)

#### Gotchas
- Status check must be BEFORE hash comparison (fail fast)
- Don't cache token validity (status can change)
- Consider token expiration as complement (not just status)

#### Related Patterns
- See "Database Service Pattern" in Service Patterns section

---

## Token Format Patterns

### Pattern: Token Format with Prefix

**Relevance**: Phase 2, Phase 4
**Last Updated**: 2025-10-01

#### Description
Use identifiable prefix for tokens to enable token scanning and debugging.

#### When to Use
- Public-facing API tokens
- When tokens might be committed to git accidentally
- When you want clear token attribution

#### How to Implement
```rust
// Format: prefix_<random_data>
let token_str = format!("bodhiapp_{}", random_string);

// Validation
if bearer_token.starts_with(BODHIAPP_TOKEN_PREFIX) {
    // Handle as database token
} else {
    // Handle as external client token
}
```

#### Why It Works
- **Debuggability**: Easy to identify BodhiApp tokens in logs
- **Token Scanning**: GitHub can scan for "bodhiapp_" pattern
- **Type Detection**: Different prefixes for different token types
- **Branding**: Makes tokens immediately recognizable

#### Gotchas
- Prefix reduces entropy slightly (but 32 random bytes is still plenty)
- Must be consistent (lowercase, no spaces)
- Document prefix format for users
- Consider prefix in token length limits (if any)

#### Related Patterns
- See "Token Prefix for Fast Lookup" in Database Patterns

---

## Authorization Patterns

### Pattern: Role to Scope Mapping

**Relevance**: Phase 4
**Last Updated**: 2025-10-01

#### Description
Map user's role to token scope when creating tokens.

#### When to Use
- Token creation API
- When token permissions should match user's permissions
- When you want role-based token authorization

#### How to Implement
```rust
let token_scope = match user_role {
    Role::Admin => TokenScope::Admin,
    Role::Manager => TokenScope::Manager,
    Role::PowerUser => TokenScope::PowerUser,
    Role::User => TokenScope::User,
};

let api_token = ApiToken {
    scopes: token_scope.to_string(),
    // ...
};
```

#### Why It Works
- **Security**: User can't create tokens with higher privileges
- **Simplicity**: Direct 1:1 mapping
- **Consistency**: Token permissions match session permissions
- **Flexibility**: Can extend to allow subset of permissions later

#### Gotchas
- Must enforce user can't request higher scope than their role
- Consider pilot restriction (only User and PowerUser)
- Document scope hierarchy in API docs
- Future: may want to allow subset of permissions

#### Related Patterns
- See "Scope-Based Authorization" below

---

### Pattern: Scope-Based Authorization

**Relevance**: Phase 2, Phase 3
**Last Updated**: 2025-10-01

#### Description
Use `ResourceScope` enum to represent different authorization contexts.

#### When to Use
- Authentication middleware
- When you need to support both session and token auth
- When authorization context varies by auth method

#### How to Implement
```rust
// From token validation
ResourceScope::Token(TokenScope::User)

// From session authentication
ResourceScope::User(UserScope::from_role(&role))

// In middleware
match resource_scope {
    ResourceScope::Token(scope) => {
        // Token-based authorization
    }
    ResourceScope::User(scope) => {
        // Session-based authorization
    }
}
```

#### Why It Works
- **Type Safety**: Enum ensures correct context handling
- **Flexibility**: Supports multiple auth mechanisms
- **Clear Intent**: Code explicitly shows auth context
- **Extensibility**: Easy to add new resource types

#### Gotchas
- Must handle both variants in authorization checks
- Token and User scopes have different permission models
- Document which endpoints accept which ResourceScope types

#### Related Patterns
- See "Role to Scope Mapping" above

---

## Service Layer Patterns

### Pattern: Database Service Pattern

**Relevance**: All phases
**Last Updated**: 2025-10-01

#### Description
Access database through trait-based service for testability.

#### When to Use
- All database operations
- When you need to mock database in tests
- When you want implementation flexibility

#### How to Implement
```rust
// Get service from app_service
let db_service = app_service.db_service();

// Use service methods
db_service.create_api_token(&mut token).await?;
let token = db_service.get_api_token_by_prefix(prefix).await?;
```

#### Why It Works
- **Testability**: Can mock database in unit tests
- **Abstraction**: Implementation details hidden
- **Consistency**: Same interface for all database operations
- **Error Handling**: Service errors are typed and localized

#### Gotchas
- Always use service, never direct SQL in routes/middleware
- Service errors must be converted to HTTP errors
- Use `app_service()` to get service registry
- Don't bypass service layer for "just one query"

#### Related Patterns
- See "Error Translation Pattern" below

---

### Pattern: TimeService for Testability

**Relevance**: Phase 1, Phase 4
**Last Updated**: 2025-10-01

#### Description
Use TimeService abstraction instead of `Utc::now()` for testable timestamps.

#### When to Use
- Creating database records
- Generating timestamps in domain objects
- Any time-dependent logic

#### How to Implement
```rust
// In production code
let now = app_service.time_service().utc_now();

let api_token = ApiToken {
    created_at: now,
    updated_at: now,
    // ...
};

// In tests
let time_service = FrozenTimeService::new();
// Can control time for deterministic tests
```

#### Why It Works
- **Testability**: Tests can freeze time for reproducibility
- **Consistency**: All timestamps from same source
- **Nanosecond Handling**: Service removes nanoseconds for SQLite
- **Best Practice**: Never use `Utc::now()` directly per project conventions

#### Gotchas
- Must use `time_service.utc_now()`, not `Utc::now()`
- Constructor should accept time, not create it internally
- Test utilities provide `FrozenTimeService` for tests

#### Related Patterns
- See "Database Service Pattern" above

---

## Error Handling Patterns

### Pattern: Error Translation Pattern

**Relevance**: Phase 3, Phase 4
**Last Updated**: 2025-10-01

#### Description
Convert service errors to HTTP errors at route boundary.

#### When to Use
- In route handlers
- When returning errors to HTTP clients
- When service errors need HTTP status codes

#### How to Implement
```rust
// Service error
return Err(DbError::NotFound);

// Automatically converted to HTTP error via ? operator
// (because DbError implements AppError and From<DbError> for ApiError)

// Result: HTTP 404 with localized error message
```

#### Why It Works
- **Separation of Concerns**: Services don't know about HTTP
- **Automatic Translation**: `?` operator handles conversion
- **Localization**: Error messages in user's language
- **Consistency**: Same error handling across all routes

#### Gotchas
- All service errors must implement `AppError` trait
- Use `impl_error_from!` macro for error conversion
- Don't return service errors directly from routes
- HTTP status codes defined in `ErrorType` enum

#### Related Patterns
- See "Database Service Pattern" above

---

### Pattern: Transparent Error Wrapping

**Relevance**: Phase 2, Phase 3, Phase 4
**Last Updated**: 2025-10-01

#### Description
Use `#[error(transparent)]` to preserve original error context.

#### When to Use
- Wrapping errors from dependencies
- When you want to preserve full error chain
- When adding domain-specific error type

#### How to Implement
```rust
#[derive(Error, Debug)]
pub enum AuthError {
    #[error(transparent)]
    DbError(#[from] DbError),

    #[error(transparent)]
    TokenError(#[from] TokenError),
}
```

#### Why It Works
- **Preserves Context**: Original error message and backtrace
- **Automatic Conversion**: `#[from]` enables `?` operator
- **Error Chain**: Can trace error back to source
- **Type Safety**: Still have typed error variants

#### Gotchas
- Use `#[from]` with `#[error(transparent)]`
- Don't add message when using `transparent`
- Original error must implement `std::error::Error`

---

## Testing Patterns

### Pattern: Test Utilities for API Tokens

**Relevance**: Phase 1, Phase 5
**Last Updated**: 2025-10-01

#### Description
Update test utilities to work with new token schema.

#### When to Use
- Writing tests for any token-related functionality
- Creating test fixtures
- Mocking token operations

#### How to Implement
```rust
// In tests, create test tokens with new schema
ApiToken {
    id: Uuid::new_v4().to_string(),
    user_id: "test-user".to_string(),
    name: "Test Token".to_string(),
    token_prefix: "bodhiapp_test01".to_string(),  // Not token_id
    token_hash: "hash...".to_string(),
    scopes: "scope_token_user".to_string(),       // Required
    status: TokenStatus::Active,
    created_at: Utc::now(),
    updated_at: Utc::now(),
}
```

#### Why It Works
- **Realism**: Tests use same schema as production
- **Maintainability**: Schema changes caught by tests
- **Consistency**: All tests use same test utilities

#### Gotchas
- Must update ALL test utilities when schema changes
- Test token prefixes should be deterministic (e.g., "bodhiapp_test01")
- Don't forget to add `scopes` field (required in database)

---

## Code Organization Patterns

### Pattern: Import Ordering

**Relevance**: Phase 5
**Last Updated**: 2025-10-01

#### Description
Maintain alphabetical import ordering for consistency.

#### When to Use
- All Rust files
- Especially after refactoring
- Before final PR submission

#### How to Implement
```rust
// Alphabetical order
use crate::{Alias, ApiAlias, UserAlias};

// Not this
use crate::{UserAlias, Alias, ApiAlias};
```

#### Why It Works
- **Consistency**: Easy to find imports
- **Merge Conflicts**: Reduces conflicts in imports
- **Readability**: Clean, organized code
- **Project Convention**: Follows Rust best practices

#### Gotchas
- `cargo fmt` doesn't reorder imports (do it manually)
- Group external crates, then internal modules
- Keep `use super::*` prohibition in tests (per project conventions)

---

## Anti-Patterns (Things to Avoid)

### Anti-Pattern: Caching Database Tokens

**Why It's Wrong**: Database lookup is fast enough; caching adds complexity and can miss revocation.

**Exception**: External client tokens ARE cached (different code path).

---

### Anti-Pattern: Storing Plaintext Tokens

**Why It's Wrong**: Security risk if database is compromised.

**Exception**: None. Always hash tokens before storage.

---

### Anti-Pattern: Variable-Length Prefixes

**Why It's Wrong**: Makes index less efficient; complicates lookup.

**Exception**: None. Always use consistent 8-character prefix.

---

### Anti-Pattern: Using Utc::now() Directly

**Why It's Wrong**: Makes tests non-deterministic; violates project conventions.

**Exception**: None. Always use `time_service.utc_now()`.

---

### Anti-Pattern: Non-Constant-Time Comparison

**Why It's Wrong**: Enables timing attacks on token validation.

**Exception**: None. Always use `constant_time_eq::constant_time_eq()`.

---

## Open Questions and Future Enhancements

### Token Expiration
- **Question**: Should tokens have expiration dates?
- **Current**: Tokens don't expire (revoke manually)
- **Future**: Add `expires_at` field for automatic expiration

### Token Usage Tracking
- **Question**: Track when/where tokens are used?
- **Current**: No usage tracking
- **Future**: Add `last_used_at`, `usage_count` fields

### Token Rate Limiting
- **Question**: Rate limit token operations?
- **Current**: No rate limiting (rely on general API limits)
- **Future**: Consider per-token rate limits

### Token Rotation
- **Question**: Support token rotation?
- **Current**: Create new, revoke old manually
- **Future**: Automatic rotation with grace period

---

## Insights to Verify

### From Previous Implementation
These insights were extracted from the previous implementation. Future agents should verify them:

1. **Prefix Length**: Is 8 characters optimal? Could be shorter/longer?
2. **No Caching**: Is database fast enough without caching?
3. **Status Field**: Is active/inactive sufficient, or need more states?
4. **Scope Storage**: Is string storage better than normalized table?

### To Be Discovered
Future agents should document insights on:
- Performance characteristics of prefix-based lookup
- Common errors during token validation
- User experience considerations for token creation
- Integration with frontend applications

---

## Contributing to This Context

### What Makes a Good Insight

**Good**:
- Reusable across phases
- Explains WHY, not just WHAT
- Includes gotchas and edge cases
- Backed by code examples

**Not Useful**:
- Obvious information from documentation
- Phase-specific details (put in log instead)
- Personal preferences without rationale
- Hypothetical scenarios without proof

### When to Update

**Update**:
- Found better approach than documented
- Discovered new pitfall
- Clarified ambiguous guidance
- Validated hypothesis from previous agent

**Don't Update**:
- Just to add your name
- Minor wording preferences
- Duplicating information
- Speculating without testing

---

## Context Maintenance

**Last Full Review**: 2025-10-01 (initial creation)
**Next Review**: After Phase 3 completion (verify security patterns)

Agents should flag outdated or incorrect information rather than silently deleting it. Add "⚠️ VERIFY: ..." notes if unsure.

---

[Agents: Add your insights below, organized into appropriate sections]

