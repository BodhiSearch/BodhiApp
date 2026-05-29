use super::error::ModelRouterError;
use super::strategy::{with_obs_headers, RouterContext, RoutingStrategy};
use crate::models::{FallbackConfig, RouterTarget};
use async_trait::async_trait;
use axum::response::Response;

const STRATEGY_NAME: &str = "fallback";

#[async_trait]
impl RoutingStrategy for FallbackConfig {
  /// Phase 1: forward to the first enabled target and return its response verbatim.
  /// No fall-through on failure yet (Phase 2). Disabled targets are skipped.
  async fn execute(
    &self,
    targets: &[RouterTarget],
    ctx: &RouterContext,
  ) -> Result<Response, ModelRouterError> {
    let target = targets
      .iter()
      .find(|t| t.enabled)
      .ok_or(ModelRouterError::EmptyChain)?;
    let resp = ctx.forward_one(target).await?;
    Ok(with_obs_headers(resp, target, STRATEGY_NAME, 1))
  }
}
