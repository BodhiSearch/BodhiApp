mod classify;
mod error;
mod fallback;
mod service;
mod strategy;

pub use error::ModelRouterError;
pub use service::{DefaultModelRouterService, ModelRouterService};
pub use strategy::{route_chat_completion, with_obs_headers, RouterContext, RoutingStrategy};

#[cfg(any(test, feature = "test-utils"))]
pub use service::MockModelRouterService;

#[cfg(test)]
#[path = "test_fallback_strategy.rs"]
mod test_fallback_strategy;
