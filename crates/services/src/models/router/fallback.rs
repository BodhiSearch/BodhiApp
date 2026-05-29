use super::classify::{classify_status, Disposition};
use super::error::ModelRouterError;
use super::strategy::{with_obs_headers, RouterContext, RoutingStrategy};
use crate::models::{FallbackConfig, RouterTarget};
use async_trait::async_trait;
use axum::response::Response;

const STRATEGY_NAME: &str = "fallback";

#[async_trait]
impl RoutingStrategy for FallbackConfig {
  /// Phase 2: try enabled targets in declared order. A 2xx (success) or
  /// 400/422 (terminal) response is returned verbatim and routing stops. A
  /// retryable status, a transport error, or a structural problem (dangling
  /// alias, nested router, unsupported format) falls through to the next
  /// target. On exhaustion the last upstream response is returned verbatim; if
  /// no target ever produced a response, the last typed error surfaces.
  /// `max_attempts` (0 = whole chain) caps how many targets are tried.
  async fn execute(
    &self,
    targets: &[RouterTarget],
    ctx: &RouterContext,
  ) -> Result<Response, ModelRouterError> {
    let enabled: Vec<&RouterTarget> = targets.iter().filter(|t| t.enabled).collect();
    if enabled.is_empty() {
      return Err(ModelRouterError::EmptyChain);
    }

    let cap = if self.max_attempts == 0 {
      enabled.len()
    } else {
      (self.max_attempts as usize).min(enabled.len())
    };

    let mut attempts: u32 = 0;
    let mut last_resp: Option<(&RouterTarget, Response)> = None;
    let mut last_err: Option<ModelRouterError> = None;

    for target in enabled.into_iter().take(cap) {
      attempts += 1;
      match ctx.forward_one(target).await {
        Ok(resp) => match classify_status(resp.status()) {
          Disposition::Success | Disposition::Terminal => {
            return Ok(with_obs_headers(resp, target, STRATEGY_NAME, attempts));
          }
          // Hold the retryable response (body not consumed) in case the chain
          // is exhausted and we must return the last one verbatim.
          Disposition::Retryable => last_resp = Some((target, resp)),
        },
        // Transport error or structural skip — fall through to the next target.
        Err(e) => last_err = Some(e),
      }
    }

    match (last_resp, last_err) {
      (Some((target, resp)), _) => Ok(with_obs_headers(resp, target, STRATEGY_NAME, attempts)),
      (None, Some(e)) => Err(e),
      (None, None) => Err(ModelRouterError::EmptyChain),
    }
  }
}
