use super::classify::{classify_status, Disposition};
use super::error::ModelRouterError;
use super::health::{cooldown_for, order_by_health, target_key};
use super::strategy::{with_obs_headers, RouterContext, RoutingStrategy};
use crate::models::{FallbackConfig, RouterTarget};
use async_trait::async_trait;
use axum::response::Response;

const STRATEGY_NAME: &str = "fallback";

#[async_trait]
impl RoutingStrategy for FallbackConfig {
  /// Try enabled targets, ordered by health: not-cooled targets first (in
  /// declared order), then cooled ones by soonest recovery. A 2xx (success) or
  /// 400/422 (terminal) response is returned verbatim and routing stops; a
  /// success also clears the target's health (so a recovered primary returns to
  /// the front next request). A retryable status or genuine transport failure
  /// cools the target (`cooldown_secs`, extended to `Retry-After`) and falls
  /// through. A structural problem (dangling alias, nested router, unsupported
  /// format) is skipped but NOT cooled — it isn't transient. On exhaustion the
  /// last upstream response is returned verbatim; if no target ever produced a
  /// response, the last typed error surfaces. `max_attempts` (0 = whole chain)
  /// caps how many targets are tried.
  async fn execute(
    &self,
    targets: &[RouterTarget],
    ctx: &RouterContext,
  ) -> Result<Response, ModelRouterError> {
    let enabled: Vec<&RouterTarget> = targets.iter().filter(|t| t.enabled).collect();
    if enabled.is_empty() {
      return Err(ModelRouterError::EmptyChain);
    }

    let now = ctx.time_service.utc_now();
    // Skip cooled targets to the back (never-starve: all-cooled still yields all).
    let ordered = order_by_health(&enabled, ctx.health.as_ref(), &ctx.tenant_id, now);

    let cap = if self.max_attempts == 0 {
      ordered.len()
    } else {
      (self.max_attempts as usize).min(ordered.len())
    };

    let mut attempts: u32 = 0;
    let mut last_resp: Option<(&RouterTarget, Response)> = None;
    let mut last_err: Option<ModelRouterError> = None;

    for target in ordered.into_iter().take(cap) {
      attempts += 1;
      let key = target_key(&ctx.tenant_id, &target.alias, &target.model);
      match ctx.forward_one(target).await {
        Ok(resp) => match classify_status(resp.status()) {
          Disposition::Success => {
            // Half-open trial (or normal hit) succeeded — recover the target.
            ctx.health.record_success(&key);
            return Ok(with_obs_headers(resp, target, STRATEGY_NAME, attempts));
          }
          // The request itself is the problem; leave health untouched.
          Disposition::Terminal => {
            return Ok(with_obs_headers(resp, target, STRATEGY_NAME, attempts));
          }
          // Retryable upstream error — cool the target and hold the response
          // (body not consumed) in case the chain is exhausted.
          Disposition::Retryable => {
            ctx
              .health
              .cooldown(&key, cooldown_for(resp.headers(), self, now));
            last_resp = Some((target, resp));
          }
        },
        Err(e) => {
          // Genuine transport failures are transient → cool; structural skips
          // (dangling/nested/unsupported) are not → leave eligibility unchanged.
          if e.is_transport_failure() {
            ctx
              .health
              .cooldown(&key, cooldown_for(&Default::default(), self, now));
          }
          last_err = Some(e);
        }
      }
    }

    match (last_resp, last_err) {
      (Some((target, resp)), _) => Ok(with_obs_headers(resp, target, STRATEGY_NAME, attempts)),
      (None, Some(e)) => Err(e),
      (None, None) => Err(ModelRouterError::EmptyChain),
    }
  }
}
