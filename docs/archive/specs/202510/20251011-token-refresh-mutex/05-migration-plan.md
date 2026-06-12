# Migration Plan: Concurrency Control Rollout

## Overview

This document outlines the strategy for safely deploying the token refresh concurrency control fix to production with minimal risk.

## Pre-Deployment Checklist

- [ ] All unit tests passing
- [ ] Integration tests passing with real Keycloak
- [ ] Load tests completed successfully (1000+ concurrent requests)
- [ ] Code review approved by 2+ team members
- [ ] Documentation updated (CLAUDE.md, PACKAGE.md, README.md)
- [ ] Metrics and monitoring dashboards created
- [ ] Rollback plan documented and tested

## Deployment Strategy

### Phase 1: Staging Deployment

**Duration:** 2-3 days
**Goal:** Validate implementation in production-like environment

1. Deploy to staging environment
2. Run automated test suite against staging
3. Manual testing with concurrent requests
4. Monitor for "Token is not active" errors (should be zero)
5. Verify single refresh token call per expiration
6. Performance regression testing

**Success Criteria:**
- Zero authentication errors in staging for 48 hours
- Lock acquisition metrics showing healthy status
- No performance degradation

### Phase 2: Canary Deployment (10%)

**Duration:** 2-3 days
**Goal:** Validate with small percentage of production traffic

1. Deploy to 10% of production instances
2. Monitor error rates, latency, and lock metrics
3. Compare canary metrics vs baseline (remaining 90%)
4. Verify Keycloak audit logs show single refresh per expiration

**Rollback Triggers:**
- Any "Token is not active" errors in canary group
- P99 latency increase > 50ms
- Error rate increase > 0.1%
- Lock timeout rate > 1%

**Success Criteria:**
- Error rate equal to or lower than baseline
- Latency within acceptable range (< 20ms increase)
- Lock acquisition success rate > 99.9%

### Phase 3: Gradual Rollout (50%)

**Duration:** 2-3 days
**Goal:** Expand to half of production traffic

1. Increase deployment to 50% of instances
2. Continue monitoring all metrics
3. Validate under higher concurrency load
4. Check for any edge cases or unexpected behavior

**Success Criteria:**
- Same as Phase 2, sustained over 48 hours
- No user complaints or support tickets related to authentication

### Phase 4: Full Deployment (100%)

**Duration:** 1 day deployment + 1 week monitoring
**Goal:** Complete rollout with full production traffic

1. Deploy to all production instances
2. Intensive monitoring for first 24 hours
3. Daily review of metrics for first week
4. Document any lessons learned

**Success Criteria:**
- Zero "Token is not active" errors
- Lock acquisition success rate > 99.95%
- User authentication experience unchanged or improved
- Support ticket volume unchanged or decreased

## Monitoring Plan

### Key Metrics to Watch

**Error Metrics:**
- `token_refresh.errors{type="token_not_active"}` → Should be ZERO
- `token_refresh.lock.timeouts` → Should be < 0.1% of requests
- `auth.failures.total` → Should not increase

**Performance Metrics:**
- `token_refresh.lock.acquisition.duration_ms` (p50, p95, p99)
- `token_refresh.with_lock.duration_ms` (p50, p95, p99)
- `api.response_time_ms` (no significant change)

**Operational Metrics:**
- `concurrency.locks.active` → Gauge of active locks
- `concurrency.lock.acquisitions.total` → Counter
- `token_refresh.double_check.hits` → Double-check pattern effectiveness

### Alerting Configuration

```yaml
alerts:
  - name: TokenRefreshLockTimeout
    condition: rate(token_refresh.lock.timeouts[5m]) > 0.01
    severity: warning
    message: "Lock acquisition timeouts detected"

  - name: TokenNotActiveError
    condition: increase(token_refresh.errors{type="token_not_active"}[5m]) > 0
    severity: critical
    message: "Token not active error detected - ROLLBACK CANDIDATE"

  - name: LockAcquisitionLatencyHigh
    condition: histogram_quantile(0.99, token_refresh.lock.acquisition.duration_ms) > 100
    severity: warning
    message: "Lock acquisition p99 latency exceeds threshold"
```

### Dashboard Panels

1. **Error Rate Panel:** Token refresh errors by type
2. **Lock Metrics Panel:** Acquisitions, timeouts, active locks
3. **Latency Panel:** Lock acquisition and total refresh duration
4. **Double-Check Effectiveness:** Percentage of refreshes avoided by double-check

## Rollback Procedures

### Immediate Rollback (< 5 minutes)

If critical issues detected:

```bash
# 1. Revert to previous deployment
kubectl rollout undo deployment/bodhi-server

# 2. Verify rollback successful
kubectl rollout status deployment/bodhi-server

# 3. Monitor for error rate decrease
watch 'kubectl logs -l app=bodhi-server --tail=100 | grep "Token is not active"'

# 4. Notify team
slack-notify "#engineering" "Rolled back token refresh concurrency control due to errors"
```

### Partial Rollback (< 15 minutes)

If issues affect specific deployment segment:

```bash
# Roll back canary/50% deployment only
kubectl patch deployment bodhi-server-canary --patch '{"spec":{"template":{"metadata":{"annotations":{"version":"previous"}}}}}'
```

### Feature Flag Rollback (< 1 minute)

If runtime disable needed:

```bash
# Disable via environment variable (requires pod restart)
kubectl set env deployment/bodhi-server ENABLE_TOKEN_REFRESH_LOCKING=false

# Or use ConfigMap update (no pod restart)
kubectl patch configmap bodhi-config --patch '{"data":{"enable_token_refresh_locking":"false"}}'
```

## Post-Deployment Validation

### Day 1: Intensive Monitoring

- [ ] Check error logs every hour
- [ ] Review lock acquisition metrics
- [ ] Monitor Keycloak audit logs for refresh patterns
- [ ] Check support ticket queue for authentication issues
- [ ] Verify no performance regression

### Week 1: Daily Reviews

- [ ] Daily metric review at team standup
- [ ] Weekly summary report
- [ ] Document any issues and resolutions
- [ ] Gather feedback from users (if any)

### Week 2-4: Normal Monitoring

- [ ] Standard operational monitoring
- [ ] Bi-weekly metric reviews
- [ ] Final assessment and sign-off

## Success Declaration

After **4 weeks** in production with:

- ✅ Zero "Token is not active" errors
- ✅ No authentication-related user complaints
- ✅ Lock acquisition success rate > 99.95%
- ✅ Performance within acceptable range
- ✅ Team confident in implementation

Declare Phase 1 **SUCCESS** and close incident TOKEN-REFRESH-001.

## Phase 2 Planning (Distributed)

After Phase 1 success, plan Phase 2 distributed implementation:

1. Evaluate need for distributed deployment
2. Provision Redis infrastructure
3. Follow same staged rollout process
4. Compare local vs distributed performance
5. Document best practices for deployment selection
