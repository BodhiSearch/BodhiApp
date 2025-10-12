# Token Refresh Race Condition Fix - Specification

**Status:** Draft
**Created:** 2025-10-11
**Last Updated:** 2025-10-12
**Owner:** Architecture Team
**Priority:** P0 - Critical

## Executive Summary

This specification addresses a critical race condition in BodhiApp's OAuth2 token refresh implementation that causes authentication failures under concurrent load. The issue manifests as "Token is not active" errors when multiple requests simultaneously attempt to refresh the same single-use refresh token.

### The Problem
On 2025-10-11 at 23:56:34 UTC, BodhiApp experienced authentication failures where two concurrent API requests detected an expired access token simultaneously and both attempted to refresh using the same OAuth2 refresh token. Since Keycloak implements single-use refresh tokens per OAuth 2.0 Security Best Current Practice, the first request succeeded while the second failed with "invalid_grant: Token is not active" error.

### The Solution
Implement a `ConcurrencyControlService` abstraction that provides distributed locking capabilities with two implementation variants:
1. **LocalConcurrencyControlService** - In-memory mutex for single-instance deployments (Tauri desktop, standalone server, single Docker container)
2. **RedisConcurrencyControlService** - Redis-based distributed locks for clustered deployments (Kubernetes, horizontal scaling, multi-region)

### Impact
- **Severity:** Critical - Causes user-facing authentication failures
- **Frequency:** Rare but increasing with concurrent usage patterns
- **Scope:** Affects all deployment modes (desktop app, Docker, standalone server)
- **User Experience:** Users encounter unexpected "authentication failed" errors requiring re-login

## Quick Navigation

### Understanding the Problem
- [01-problem-analysis.md](./01-problem-analysis.md) - Root cause analysis with log evidence
- [logs/application.log](./logs/application.log) - BodhiApp application logs
- [logs/keycloak.log](./logs/keycloak.log) - Keycloak server logs

### Architecture and Design
- [02-architecture-design.md](./02-architecture-design.md) - ConcurrencyControlService design for both local and distributed scenarios
- [diagrams/architecture.md](./diagrams/architecture.md) - Visual architecture diagrams

### Implementation
- [03-implementation-guide.md](./03-implementation-guide.md) - Step-by-step implementation instructions
- [06-api-reference.md](./06-api-reference.md) - Complete API reference for service traits
- [code-samples/](./code-samples/) - Reference implementation code

### Testing and Rollout
- [04-testing-strategy.md](./04-testing-strategy.md) - Comprehensive testing approach
- [05-migration-plan.md](./05-migration-plan.md) - Rollout strategy and backwards compatibility

## Key Design Decisions

### 1. Service-Based Abstraction
Create a new `ConcurrencyControlService` trait in the `services` crate rather than direct mutex usage in `auth_middleware`. This provides:
- Clean separation of concerns
- Testability through mocking
- Deployment flexibility
- Future extensibility

### 2. Per-User Locking Granularity
Implement per-user locks (`user:{user_id}:token_refresh`) rather than global locks to:
- Maximize concurrency (different users don't block each other)
- Minimize lock contention
- Reduce latency for concurrent users

### 3. Double-Check Pattern After Lock Acquisition
After acquiring lock, re-validate token expiration to handle cases where another request already completed the refresh:
```rust
acquire_lock(user_id) -> guard
  current_token = session.get_access_token()
  if !is_expired(current_token):
    return current_token  // Another request already refreshed
  perform_refresh()
```

### 4. Feature-Gated Distributed Implementation
Use Cargo features to gate distributed implementations:
- `default` = Local in-memory implementation
- `distributed-redis` = Redis-based distributed locks
- `distributed-db` = Database advisory locks (fallback)

This allows:
- Zero additional dependencies for desktop/standalone deployments
- Optional Redis dependency only when clustering needed
- Future extensibility for other backends (etcd, Consul, etc.)

### 5. Automatic Lock Cleanup with TTL
All locks have Time-To-Live (TTL) guarantees to prevent deadlocks:
- Local: In-memory tracking with cleanup task
- Redis: Native EXPIRE with automatic removal
- Database: Timestamp-based expiration with background cleanup

## Implementation Phases

### Phase 1: Foundation (Immediate - Sprint 1)
**Goal:** Fix the critical race condition for all current deployment scenarios

1. Create `ConcurrencyControlService` trait in `services` crate
2. Implement `LocalConcurrencyControlService` with in-memory mutexes
3. Integrate with `DefaultTokenService` in `auth_middleware`
4. Add comprehensive unit and integration tests
5. Update documentation and deployment guides

**Deliverables:**
- Zero "Token is not active" errors under concurrent load
- Single token refresh call to Keycloak per expiration event
- Backwards compatible with existing deployments

**Timeline:** 3-5 days
**Dependencies:** None
**Risk:** Low - Isolated change with comprehensive testing

### Phase 2: Distributed Support (Future - Sprint 3+)
**Goal:** Enable horizontal scaling for high-availability deployments

1. Add `distributed-redis` Cargo feature flag
2. Implement `RedisConcurrencyControlService`
3. Add configuration for Redis connection
4. Create distributed deployment guides
5. Performance benchmarking and optimization

**Deliverables:**
- Redis-based distributed locks with sub-50ms acquisition
- Kubernetes deployment examples
- Multi-region clustering support

**Timeline:** 5-7 days
**Dependencies:** Redis infrastructure availability
**Risk:** Medium - Requires new infrastructure component

## Success Metrics

### Correctness Metrics
- ✅ Zero "Token is not active" errors in production logs
- ✅ Single refresh token call per expiration event (Keycloak audit logs)
- ✅ No authentication failures under concurrent load testing (1000+ concurrent users)

### Performance Metrics
- ✅ Lock acquisition latency: < 10ms (p99) for local, < 50ms (p99) for distributed
- ✅ Token refresh latency increase: < 5ms overhead
- ✅ Memory overhead: < 1KB per active user session

### Operational Metrics
- ✅ Automatic lock cleanup: 100% cleanup within TTL expiration
- ✅ Service restart recovery: Zero orphaned locks after restart
- ✅ Monitoring integration: Lock acquisition metrics in Prometheus/OpenTelemetry

## Risk Assessment

### High Priority Risks
1. **Lock Cleanup Failure**
   - *Risk:* Orphaned locks preventing future authentication
   - *Mitigation:* TTL-based automatic expiration, background cleanup tasks
   - *Detection:* Monitoring for lock age and cleanup metrics

2. **Deadlock Scenarios**
   - *Risk:* Multiple services deadlocking on lock acquisition
   - *Mitigation:* Timeout-based lock acquisition, deadlock detection
   - *Detection:* Lock timeout alerts and distributed tracing

### Medium Priority Risks
3. **Distributed Lock Consistency**
   - *Risk:* Redis failure causing lock unavailability
   - *Mitigation:* Fallback to database-based locks, Redis clustering
   - *Detection:* Redis health checks and failover monitoring

4. **Performance Degradation**
   - *Risk:* Lock acquisition adding unacceptable latency
   - *Mitigation:* Fast path optimization, lock-free cache checks
   - *Detection:* P99 latency monitoring and alerting

### Low Priority Risks
5. **Migration Complexity**
   - *Risk:* Breaking existing deployments during rollout
   - *Mitigation:* Backwards compatibility, feature flags, gradual rollout
   - *Detection:* Comprehensive integration testing before deployment

## Dependencies

### Required
- ✅ Rust 1.70+ (already satisfied)
- ✅ Tokio 1.x (already satisfied)
- ✅ SQLite/PostgreSQL (already satisfied)

### Optional (Phase 2)
- ⏳ Redis 6.0+ (for distributed deployment)
- ⏳ Redis client library (`redis-rs`)
- ⏳ Kubernetes infrastructure (for clustered deployment)

## Backwards Compatibility

This change is **fully backwards compatible** with existing deployments:

1. **No API Changes:** Public API of `DefaultTokenService` remains unchanged
2. **No Configuration Changes:** Existing configurations continue to work
3. **No Database Migration:** Zero schema changes required
4. **Transparent Integration:** Lock acquisition is internal implementation detail

Existing deployments will automatically benefit from the fix without any action required.

## Related Work

### Similar Patterns in BodhiApp
- `KeyringService` - Platform-specific abstractions with local/cloud implementations
- `CacheService` - In-memory vs Redis caching abstraction
- `DbService` - SQLite vs PostgreSQL abstraction

### Industry Precedents
- **Auth0:** Refresh token rotation with reuse detection
- **Okta:** Client-side request coalescing for concurrent refreshes
- **AWS Cognito:** Exponential backoff with jitter for concurrent requests
- **Google OAuth2:** Token refresh queuing with result sharing

## Open Questions

1. **Q:** Should we implement heartbeat mechanism for long-running lock holders?
   **A:** Defer to Phase 2 - current TTL approach sufficient for token refresh (< 5s operations)

2. **Q:** Should we cache "lock acquisition in progress" to avoid lock contention?
   **A:** Yes - implement fast-path check before lock acquisition for high-concurrency scenarios

3. **Q:** Should distributed lock require consensus (majority voting)?
   **A:** No - single Redis instance sufficient for token refresh use case; future work for critical operations

4. **Q:** Should we implement lock priority for different request types?
   **A:** No - all token refresh requests have equal priority; FIFO ordering sufficient

## Document Changelog

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2025-10-11 | 0.1 | Initial draft with problem analysis | Architecture Team |
| 2025-10-12 | 0.2 | Added architecture design and API reference | Architecture Team |
| TBD | 1.0 | Final review and approval | Engineering Leadership |

## References

- [RFC 6749 - OAuth 2.0 Authorization Framework](https://datatracker.ietf.org/doc/html/rfc6749)
- [RFC 8693 - OAuth 2.0 Token Exchange](https://datatracker.ietf.org/doc/html/rfc8693)
- [OAuth 2.0 Security Best Current Practice](https://datatracker.ietf.org/doc/html/draft-ietf-oauth-security-topics)
- [Keycloak Security Documentation](https://www.keycloak.org/docs/latest/securing_apps/)
- [Redis SETNX Pattern for Distributed Locks](https://redis.io/docs/manual/patterns/distributed-locks/)
- [PostgreSQL Advisory Locks](https://www.postgresql.org/docs/current/explicit-locking.html#ADVISORY-LOCKS)
