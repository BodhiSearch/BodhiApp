# TECHDEBT.md - GitHub Actions / CI

## PostgreSQL Session Test Integration

**Location**: `.github/workflows/build.yml`, PostgreSQL service block

**Issue**: The build workflow provisions a PostgreSQL service (`bodhi_test` / `bodhi_sessions`) for session integration tests, but there is no CI step that actually runs tests against this service. The `INTEG_TEST_SESSION_PG_URL` environment variable is not set in the workflow, so the parameterized tests in `crates/services/src/test_session_service.rs` only exercise the SQLite backend in CI.

**Required steps to enable PG tests in CI**:
1. Add `INTEG_TEST_SESSION_PG_URL` to the `cargo test` step:
   ```yaml
   env:
     INTEG_TEST_SESSION_PG_URL: postgres://bodhi_test:bodhi_test@localhost:54320/bodhi_sessions
   ```
2. Align the health check: use `pg_isready -U bodhi_test -d bodhi_sessions` (consistent with local docker-compose).
3. Ensure the PG service is started before the test step (add `needs` or ordering).

**Deferred because**: Setting up PostgreSQL tests in CI requires confirming the service connectivity and ensuring the test suite is stable before enabling in the critical path.
