# OAuth2/OIDC: preferred_username vs sub Usage Analysis

## Executive Summary

BodhiApp correctly implements OAuth2/OIDC standards by using `sub` (subject) claim as the immutable unique identifier and `preferred_username` as a mutable display value. This document captures the research and analysis confirming our implementation follows security best practices.

## Standards Compliance and Protocol Alignment

### OIDC Specification
Keycloak's use of `preferred_username` directly aligns with the **OpenID Connect Core 1.0 specification**, which defines `preferred_username` as a standard claim within the `profile` scope. The claim is described as the "End-User's preferred username" and is included alongside other profile claims like name, given_name, family_name, etc.

## Critical Design Principles

### 1. Mutable Nature of preferred_username
The `preferred_username` claim is explicitly designed to be **mutable**, meaning it can change over time. This reflects real-world scenarios where users may change their usernames, email addresses, or display names. 

**Important**: Microsoft's identity platform documentation specifically warns: "Legacy applications sometimes use fields like the email address, phone number, or UPN. All of these fields can change over time, and can also be reused over time."

### 2. Non-Unique Identifier
The OIDC specification explicitly states that claims such as email, phone_number, and preferred_username **"MUST NOT be used as unique identifiers for the End-User"** because they carry no guarantees across different issuers in terms of:
- Stability over time
- Uniqueness across users

Unlike the `sub` (subject) claim which is immutable and unique, `preferred_username` is specifically designed as a display value rather than an identifier.

### 3. User Preference and Display Context
The term "preferred" acknowledges that this is how the user wishes to be addressed or displayed in the application, not necessarily their canonical identity. This aligns with modern identity management practices where users may have:
- Different usernames across systems
- Professional vs. personal identities
- Localized or culturally appropriate display names

## Implementation Best Practices

### Separation of Concerns
By using `preferred_username`, Keycloak and other OIDC providers maintain a clear separation between:
- **Identity** (`sub` claim): Immutable, unique identifier for authorization decisions
- **Display** (`preferred_username`): Mutable, human-readable name for UI purposes

### Federation and Social Login Compatibility
In federated scenarios, especially with social identity providers (Google, Facebook, GitHub), there isn't always a consistent "username" concept across providers. The `preferred_username` claim provides flexibility to map different provider-specific identifiers to a common display field.

## BodhiApp Implementation Analysis

### Current Implementation Status ✅

Based on thorough code analysis, BodhiApp correctly implements these principles:

#### 1. `sub` (Subject) - Used as Unique Identifier:
- ✅ **Database Tables**:
  - `api_tokens` table: `user_id` field stores `sub` claim
  - `access_requests` table: `user_id` field stores `sub` claim
- ✅ **Token Validation**: Uses `sub` for user identification
- ✅ **API Token Creation**: Links tokens to users via `sub`
- ✅ **Authorization Decisions**: Based on `sub` claim
- ✅ **Resource Admin Assignment**: Uses `sub` for user identification

#### 2. `preferred_username` - Used for Display Only:
- ✅ **Database Storage**: `access_requests.username` stores `preferred_username` for display
- ✅ **UI Responses**: Returns `username` (from `preferred_username`) for display
- ✅ **Headers**: `X-BodhiApp-Username` carries `preferred_username` for display
- ✅ **Logging**: Uses `preferred_username` for human-readable logs
- ✅ **Never used for authorization or as a foreign key**

### Key Evidence from Code

1. **Authentication Middleware** (`auth_middleware.rs`):
```rust
// Two separate headers: Username for display, User-Id for identification
req.headers_mut().insert(KEY_RESOURCE_USERNAME, claims.preferred_username.parse().unwrap());
req.headers_mut().insert(KEY_RESOURCE_USER_ID, claims.sub.parse().unwrap());
```

2. **Database Service** (`db/service.rs`):
```rust
let claims = extract_claims::<IdClaims>(token)?;
let api_token = ApiToken {
    user_id: claims.sub,  // Using sub as the identifier
    ...
};
```

3. **Login Handler** (`routes_login.rs`):
```rust
let user_id = claims.sub.clone();  // Using sub for resource admin assignment
```

4. **Database Schema**:
- `access_requests`: Has both `username` (display) and `user_id` (identifier)
- `api_tokens`: Only has `user_id` (the sub claim)

## Security Compliance ✅

The implementation correctly follows OIDC standards:
- **Immutable Identifier**: `sub` is used for all persistence and authorization
- **Mutable Display Name**: `preferred_username` is only used for UI display
- **No Security Decisions on Username**: Authorization uses roles/scopes, not usernames
- **Proper Separation**: Clear distinction between identity (`sub`) and presentation (`preferred_username`)

## Architectural Recommendations

### Current Best Practices (Already Implemented)
1. **Never use `preferred_username` for authorization decisions** - It's mutable and non-unique by design
2. **Use `sub` claim for user identification** - This is the immutable, unique identifier
3. **Treat `preferred_username` as display-only data** - Perfect for UI greeting messages, but not for security decisions
4. **Consider it equivalent to a "display name" or "nickname"** - Not a primary key or foreign key in your data model

### Alternative Approaches (If Needed)
If you need a stable username-like identifier:
- Use the `sub` claim directly
- Create a custom immutable claim in Keycloak
- Map `sub` to an internal username in your application layer
- Use `email_verified` email addresses if they're guaranteed unique in your domain

## Conclusion

BodhiApp's current implementation is **architecturally sound** and follows best practices. The system correctly:
1. Uses `sub` as the primary key/foreign key for user identification
2. Uses `preferred_username` only for display in UI and logs
3. Never makes authorization decisions based on `preferred_username`
4. Maintains proper separation of concerns between identity and display

This aligns perfectly with the OAuth2/OIDC specification's intent that `preferred_username` is mutable and non-unique, while `sub` is the stable identifier for security and data integrity purposes.

## References

- OpenID Connect Core 1.0 Specification
- Microsoft Identity Platform Documentation
- Keycloak Documentation
- OAuth 2.0 Security Best Current Practice

## Implementation Files

Key files demonstrating correct implementation:
- `/crates/auth_middleware/src/auth_middleware.rs` - Header injection
- `/crates/services/src/token.rs` - Claims extraction
- `/crates/services/src/db/service.rs` - Database operations
- `/crates/routes_app/src/routes_login.rs` - Login flow
- `/crates/routes_app/src/routes_access_request.rs` - Access requests
- `/crates/services/migrations/0002_pending-access-requests.up.sql` - Database schema
- `/crates/services/migrations/0003_create_api_tokens.up.sql` - API tokens schema