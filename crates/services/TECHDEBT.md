# TECHDEBT.md - services Crate

## Cookie Security Configuration

**Location**: `crates/services/src/session_service/session_service.rs`, `session_layer()` method

**Issue**: `with_secure(false)` is hardcoded in the `SessionManagerLayer` builder. This disables the `Secure` cookie attribute, which means session cookies are transmitted over plain HTTP.

**Impact**: In production or cluster deployments with HTTPS (e.g., Docker with a reverse proxy, RunPod), the `Secure` flag should be set to `true` to prevent session cookies from being sent over unencrypted connections.

**Proposed fix**: Read from `BODHI_SCHEME` setting (or equivalent deployment context flag) and set `with_secure(true)` when the scheme is `https`.

**Deferred because**: The setting is not currently exposed via `SettingService`, and the deployment context detection needs design. Tracked for when HTTPS support is formalized.

## ConcurrencyService: Cluster-Wide Distributed Locking

**Location**: `crates/services/src/db/concurrency_service.rs`

**Issue**: `ConcurrencyService` uses in-memory `tokio::sync::Mutex` for distributed locking (e.g., token refresh locks keyed by `refresh_token:{session_id}:{client_id}`). This works for single-process deployments but does NOT provide cluster-wide mutual exclusion in multi-deployment scenarios.

**Impact**: In a multi-tenant deployment with multiple backend instances behind a load balancer, two instances could simultaneously attempt to refresh the same token, causing race conditions or token invalidation.

**Proposed fix**: When the database backend is PostgreSQL, use PostgreSQL advisory locks (`pg_advisory_lock` / `pg_try_advisory_lock`) for cluster-wide distributed locking. The lock key can be derived from a hash of the string key. Fall back to in-memory locks for SQLite (single-process only).

**Deferred because**: Single-process deployments are the current target. Multi-instance deployments need broader infrastructure design (session store sharing, load balancer affinity, etc.).

## Session Token Lifecycle Encapsulation

**Location**: `crates/routes_app/src/middleware/token_service/token_service.rs`

**Issue**: Token refresh logic exists in two similar methods: `get_valid_session_token()` for resource tokens and `get_valid_dashboard_token()` for dashboard tokens. Both use distributed locking and auto-refresh, but have separate implementations with duplicated expiry/refresh logic.

**Impact**: Duplicated token lifecycle handling, risk of subtle bugs when one path is updated but not the other.

**Proposed fix**: Encapsulate the token refresh lifecycle into a unified session token manager parameterized by token type/prefix and client credentials.

**Deferred because**: Both methods now use distributed locking and work correctly. Unification requires careful design of the locking strategy and error handling semantics across token types.
