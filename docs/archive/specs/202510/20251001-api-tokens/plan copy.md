# API Token Implementation Plan

**Date:** 2025-09-02
**Author:** Gemini

## 1. Overview & Goal

The primary goal of this plan is to re-architect the API token system for BodhiApp. We will move away from the current implementation, which relies on exchanging user session tokens for Keycloak-issued `offline_access` JWTs, and transition to a self-contained, database-backed system.

This change is motivated by the need to simplify the architecture, remove external dependencies on Keycloak's specific token logic, and enhance the security and maintainability of the API token feature. All legacy logic related to the old system will be removed.

## 2. Token Design & Format

### 2.1. Token Structure

New API tokens will follow a structured, prefixed format inspired by industry best practices (e.g., OpenAI, Stripe).

- **Format:** `bodhiapp_<random_string>`
- **Prefix (`bodhiapp_`):** This makes the token instantly identifiable as a BodhiApp key, which is crucial for security scanning, debugging, and preventing accidental cross-service key usage.
- **Random String:** A cryptographically secure random string (e.g., 40 characters, Base64-encoded for a good character set) will form the secret part of the token.

The full token will be shown to the user only once upon creation.

### 2.2. Storage and Verification Strategy

To ensure tokens are stored and verified securely, we will use a prefix-and-hash system.

- **`token_prefix`:** A short, non-secret, unique portion of the token (e.g., `bodhiapp_` plus the first 8 characters of the random string) will be stored in the database. This column will be indexed for fast lookups.
- **`token_hash`:** The full, complete token (`bodhiapp_<random_string>`) will be hashed using SHA-256. The resulting hash will be stored in the database for verification.

### 2.3. Rationale for Prefix vs. Direct Hash Lookup

While using the `token_hash` directly for lookups is possible, the prefix-based approach provides a stronger security posture against **timing attacks**.

1.  **Find (Lookup):** We use the public `token_prefix` to quickly find the potential matching record in the database. This query is fast and does not involve the secret token itself.
2.  **Verify (Comparison):** Once the record is retrieved, we hash the full token provided by the user in our application code and perform a **constant-time comparison** against the `token_hash` stored in the database.

This two-step process is a cryptographic best practice because it isolates the security-critical comparison from the database, preventing attackers from using response time variations to gain information about the stored hashes.

## 3. Implementation Plan

### Phase 1: Database and Core Object Updates

1.  **Update Migration File (`crates/services/migrations/0003_create_api_tokens.up.sql`):**
    - The existing migration file will be modified directly, as no production data needs to be preserved.
    - The `token_id` column will be renamed to `token_prefix`.
    - The index will be updated to be on `token_prefix`.
    - A new column, `scopes TEXT NOT NULL`, will be added to store the token's permission level as a string (e.g., `"scope_token_user"`).

2.  **Update `ApiToken` Struct (`crates/services/src/db/objs.rs`):**
    - The Rust struct will be updated to reflect the new database schema: `token_id` field renamed to `token_prefix`, and a `scopes: String` field added.

3.  **Update `DbService` (`services/src/db/service.rs`):**
    - The `DbService` trait and its implementation will be modified to support the new schema. Methods will be updated to look up by `token_prefix` and to handle the `scopes` field during token creation and retrieval.

### Phase 2: Token Generation and Validation Logic

1.  **Implement `create_token_handler` (in `crates/routes_app/src/routes_api_token.rs`):**
    - The endpoint will be re-enabled and its logic fully implemented.
    - **Scope Enforcement:** It will inspect the session token of the user making the request to determine their role. The new API token's scope will be capped at the user's current privilege level (e.g., a user with the `PowerUser` role can create `User` or `PowerUser` tokens, but not `Manager` or `Admin` tokens).
    - **Token Generation:** It will generate the `bodhiapp_<random_string>`, derive the `token_prefix` for lookup, and compute the SHA-256 `token_hash` for storage.
    - **Database Storage:** It will call the updated `DbService` to persist the new token record.
    - **User Response:** It will return the full, unhashed token in the API response.

2.  **Refactor `token_service.rs`:**
    - **Legacy Code Removal:** All code related to Keycloak offline token exchange and JWT validation for API tokens will be completely removed to simplify the service.
    - **New Validation in `validate_bearer_token`:**
        - The function will be streamlined to check for the `bodhiapp_` prefix.
        - It will use the token's derived prefix to query the database.
        - If a record is found, it will perform the constant-time hash comparison.
        - If verification succeeds, it will construct the `ResourceScope` for the request using the `scopes` value from the database record, which will then be used by the `api_auth_middleware` for authorization.
