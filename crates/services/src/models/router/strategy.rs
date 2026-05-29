use super::error::ModelRouterError;
use crate::db::DbService;
use crate::models::{Alias, ModelRouterAlias, RouterTarget, RoutingStrategyConfig};
use crate::{AiApiClientFactory, DataService};
use async_trait::async_trait;
use axum::http::{HeaderName, HeaderValue, Method};
use axum::response::Response;
use serde_json::Value;
use std::sync::Arc;

const HEADER_ROUTED_ALIAS: &str = "x-bodhi-routed-alias";
const HEADER_ROUTED_MODEL: &str = "x-bodhi-routed-model";
const HEADER_ROUTER_STRATEGY: &str = "x-bodhi-router-strategy";
const HEADER_ROUTER_ATTEMPTS: &str = "x-bodhi-router-attempts";

/// Shared primitives every routing strategy calls. Phase 1 holds only what
/// pass-through forwarding needs; the health registry / selection state seams
/// (Phase 2/3) attach here without changing strategy signatures.
pub struct RouterContext {
  pub tenant_id: String,
  pub user_id: String,
  pub request: Value,
  pub query_params: Option<Vec<(String, String)>>,
  pub data_service: Arc<dyn DataService>,
  pub db_service: Arc<dyn DbService>,
  pub ai_api: Arc<dyn AiApiClientFactory>,
}

impl RouterContext {
  /// Resolve a target's referenced alias, pin the model, and forward the chat
  /// request to its upstream. Returns the upstream `Response` verbatim (any status).
  /// Returns a structural error if the referenced alias is missing, a nested router,
  /// or a format without a chat-completions surface.
  pub async fn forward_one(&self, target: &RouterTarget) -> Result<Response, ModelRouterError> {
    // Resolve the target by its identity (alias_name): name for user/model/router,
    // id for api. This differs from `find_alias`, which resolves a model string via
    // prefix matching and would not find an api alias by its id.
    let inner = match self
      .data_service
      .list_aliases(&self.tenant_id, &self.user_id)
      .await
      .ok()
      .and_then(|aliases| aliases.into_iter().find(|a| a.alias_name() == target.alias))
    {
      Some(a) if a.is_model_router() => {
        return Err(ModelRouterError::NestedRouterNotAllowed {
          alias: target.alias.clone(),
        })
      }
      Some(a) => a,
      None => {
        return Err(ModelRouterError::ReferencedAliasNotFound {
          alias: target.alias.clone(),
        })
      }
    };

    if let Alias::Api(ref api) = inner {
      if !api.api_format.supports_chat_completions() {
        return Err(ModelRouterError::TargetFormatUnsupported {
          alias: target.alias.clone(),
          api_format: api.api_format.to_string(),
        });
      }
    }

    let api_key = match &inner {
      Alias::Api(api) => {
        self
          .db_service
          .get_api_key_for_alias(&self.tenant_id, &self.user_id, &api.id)
          .await?
      }
      _ => None,
    };

    let mut req = self.request.clone();
    req["model"] = Value::String(target.model.clone());

    let client = self.ai_api.for_alias(&inner, api_key)?;
    let resp = client
      .forward_request_with_method(
        &Method::POST,
        "/chat/completions",
        Some(req),
        self.query_params.clone(),
        None,
      )
      .await?;
    Ok(resp)
  }
}

/// A routing strategy owns the control flow for one request, calling shared
/// primitives on `RouterContext`. Adding a strategy = a new variant + a new impl.
#[async_trait]
pub trait RoutingStrategy: Send + Sync {
  async fn execute(
    &self,
    targets: &[RouterTarget],
    ctx: &RouterContext,
  ) -> Result<Response, ModelRouterError>;
}

impl RoutingStrategyConfig {
  /// Map persisted config → behavior. One arm per strategy.
  pub fn behavior(&self) -> &dyn RoutingStrategy {
    match self {
      RoutingStrategyConfig::Fallback(c) => c,
    }
  }
}

/// Entry point for the chat handler: route one chat-completion request through
/// the router's targets using its configured strategy.
pub async fn route_chat_completion(
  router: &ModelRouterAlias,
  ctx: &RouterContext,
) -> Result<Response, ModelRouterError> {
  router
    .strategy
    .behavior()
    .execute(&router.targets, ctx)
    .await
}

/// Attach observability headers identifying the target that served the response,
/// the strategy, and the attempt count.
pub fn with_obs_headers(
  mut resp: Response,
  target: &RouterTarget,
  strategy: &str,
  attempts: u32,
) -> Response {
  let headers = resp.headers_mut();
  insert_header(headers, HEADER_ROUTED_ALIAS, &target.alias);
  insert_header(headers, HEADER_ROUTED_MODEL, &target.model);
  insert_header(headers, HEADER_ROUTER_STRATEGY, strategy);
  insert_header(headers, HEADER_ROUTER_ATTEMPTS, &attempts.to_string());
  resp
}

fn insert_header(headers: &mut axum::http::HeaderMap, name: &'static str, value: &str) {
  if let Ok(v) = HeaderValue::from_str(value) {
    headers.insert(HeaderName::from_static(name), v);
  }
}
