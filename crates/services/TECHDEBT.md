# TECHDEBT.md - services Crate

## Cookie Security Configuration

**Location**: `crates/services/src/session_service/session_service.rs`, `session_layer()` method

**Issue**: `with_secure(false)` is hardcoded in the `SessionManagerLayer` builder. This disables the `Secure` cookie attribute, which means session cookies are transmitted over plain HTTP.

**Impact**: In production or cluster deployments with HTTPS (e.g., Docker with a reverse proxy, RunPod), the `Secure` flag should be set to `true` to prevent session cookies from being sent over unencrypted connections.

**Proposed fix**: Read from `BODHI_SCHEME` setting (or equivalent deployment context flag) and set `with_secure(true)` when the scheme is `https`.

**Deferred because**: The setting is not currently exposed via `SettingService`, and the deployment context detection needs design. Tracked for when HTTPS support is formalized.
