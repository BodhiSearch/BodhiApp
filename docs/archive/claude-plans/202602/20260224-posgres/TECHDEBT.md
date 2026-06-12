# TECHDEBT.md - PostgreSQL Session Support (20260224)

## Dual Pool → Typed Pool Migration

**Location**: `crates/services/src/session_service/session_service.rs`

**Issue**: `DefaultSessionService` currently maintains two separate pool handles for each backend: a typed pool (`SqlitePool` or `PgPool`) used by the tower-sessions store, and an `AnyPool` used for custom `user_id` tracking queries. This dual-pool pattern was chosen as a pragmatic PoC to support both backends without a full architecture commitment.

**Proposed fix**: Once the main application database (`bodhi.sqlite`) supports a dual-backend strategy (SQLite + PostgreSQL), migrate the session service to use a single typed pool approach, eliminating `AnyPool` entirely. Use separate typed queries per backend, or a shared abstraction layer.

**Deferred because**: Requires coordinated changes to the main database service and settings persistence layer.

---

## AppSessionStoreExt Trait Duplication

**Location**: `crates/services/src/session_service/session_service.rs`

**Issue**: `AppSessionStoreExt` is a separate trait from `SessionService` but provides methods that are only implemented on `DefaultSessionService`. This creates an awkward API where callers must use UFCS (`AppSessionStoreExt::clear_sessions_for_user(&service, user_id)`) rather than `service.clear_sessions_for_user(user_id)` directly.

**Proposed options**:
1. Merge `AppSessionStoreExt` methods into `SessionService` trait (requires mock updates).
2. Make `AppSessionStoreExt` a private implementation detail and expose only `SessionService`.
3. Keep as-is but document the intended usage pattern.

**Deferred because**: Requires a decision on the public API shape and impact assessment on all callers, including `routes_app` test utilities.
