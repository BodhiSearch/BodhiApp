use crate::SharedContext;
use axum::body::Body;
use axum::response::Response;
use serde_json::Value;
use services::{
  inference::{InferenceError, InferenceService, LlmEndpoint},
  AiApiService, Alias, ApiAlias,
};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::task::JoinHandle;
use tracing::{debug, info, warn};

#[derive(Debug)]
pub struct StandaloneInferenceService {
  ctx: Arc<dyn SharedContext>,
  ai_api_service: Arc<dyn AiApiService>,
  keep_alive_secs: RwLock<i64>,
  timer_handle: RwLock<Option<JoinHandle<()>>>,
}

impl StandaloneInferenceService {
  pub fn new(
    ctx: Arc<dyn SharedContext>,
    ai_api_service: Arc<dyn AiApiService>,
    keep_alive_secs: i64,
  ) -> Self {
    Self {
      ctx,
      ai_api_service,
      keep_alive_secs: RwLock::new(keep_alive_secs),
      timer_handle: RwLock::new(None),
    }
  }

  fn start_timer(&self) {
    let keep_alive = *self.keep_alive_secs.read().unwrap();
    if keep_alive < 0 {
      debug!("Keep alive is < 0, cancelling the timer");
      self.cancel_timer();
      return;
    }
    if keep_alive == 0 {
      debug!("Keep alive is 0, cancelling the timer and stopping the server");
      self.cancel_timer();
      let ctx = self.ctx.clone();
      tokio::spawn(async move {
        if ctx.is_loaded().await {
          if let Err(err) = ctx.stop().await {
            warn!(?err, "Error stopping server in keep-alive timer");
          };
        }
      });
      return;
    }

    debug!("Starting keep-alive timer for {} seconds", keep_alive);
    let ctx = self.ctx.clone();
    let handle = tokio::spawn(async move {
      tokio::time::sleep(Duration::from_secs(keep_alive as u64)).await;
      if ctx.is_loaded().await {
        info!("Stopping server in keep-alive timer");
        if let Err(e) = ctx.stop().await {
          warn!(?e, "Error stopping server in keep-alive timer");
        }
      } else {
        info!("Server is not loaded, skipping stop");
      }
    });

    let mut timer_handle = self.timer_handle.write().unwrap();
    if let Some(old_handle) = timer_handle.take() {
      old_handle.abort();
    }
    *timer_handle = Some(handle);
  }

  fn cancel_timer(&self) {
    let mut timer_handle = self.timer_handle.write().unwrap();
    if let Some(handle) = timer_handle.take() {
      debug!("Cancelling keep-alive timer");
      handle.abort();
    }
  }

  /// Called after a successful forward_local to reset the keep-alive timer.
  fn on_request_completed(&self) {
    let keep_alive = *self.keep_alive_secs.read().unwrap();
    match keep_alive {
      -1 => {} // Never stop
      0 => {
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
          if let Err(err) = ctx.stop().await {
            debug!(?err, "Error stopping server after request completion");
          }
        });
      }
      _ => {
        // Reset timer
        info!("Resetting keep-alive timer after request");
        self.start_timer();
      }
    }
  }
}

#[async_trait::async_trait]
impl InferenceService for StandaloneInferenceService {
  async fn forward_local(
    &self,
    endpoint: LlmEndpoint,
    request: Value,
    alias: Alias,
  ) -> Result<Response, InferenceError> {
    let reqwest_response = self
      .ctx
      .forward_request(endpoint, request, alias)
      .await
      .map_err(|e| InferenceError::Internal(e.to_string()))?;

    let result = convert_reqwest_to_axum(reqwest_response);
    self.on_request_completed();
    result
  }

  async fn forward_remote(
    &self,
    endpoint: LlmEndpoint,
    request: Value,
    api_alias: &ApiAlias,
    api_key: Option<String>,
  ) -> Result<Response, InferenceError> {
    proxy_to_remote(
      &self.ai_api_service,
      endpoint,
      request,
      api_alias,
      api_key,
      None,
    )
    .await
  }

  async fn forward_remote_with_params(
    &self,
    endpoint: LlmEndpoint,
    request: Value,
    api_alias: &ApiAlias,
    api_key: Option<String>,
    query_params: Option<Vec<(String, String)>>,
  ) -> Result<Response, InferenceError> {
    proxy_to_remote(
      &self.ai_api_service,
      endpoint,
      request,
      api_alias,
      api_key,
      query_params,
    )
    .await
  }

  async fn stop(&self) -> Result<(), InferenceError> {
    self.cancel_timer();
    self
      .ctx
      .stop()
      .await
      .map_err(|e| InferenceError::Internal(e.to_string()))
  }

  async fn set_variant(&self, variant: &str) -> Result<(), InferenceError> {
    self
      .ctx
      .set_exec_variant(variant)
      .await
      .map_err(|e| InferenceError::Internal(e.to_string()))
  }

  async fn set_keep_alive(&self, secs: i64) {
    debug!("Updating keep-alive to {} seconds", secs);
    *self.keep_alive_secs.write().unwrap() = secs;
    self.start_timer();
  }

  async fn is_loaded(&self) -> bool {
    self.ctx.is_loaded().await
  }
}

pub(crate) async fn proxy_to_remote(
  ai_api_service: &Arc<dyn AiApiService>,
  endpoint: LlmEndpoint,
  request: Value,
  api_alias: &ApiAlias,
  api_key: Option<String>,
  query_params: Option<Vec<(String, String)>>,
) -> Result<Response, InferenceError> {
  let method = endpoint.http_method();
  let api_path = endpoint.api_path();
  let body = if *method == axum::http::Method::POST {
    Some(request)
  } else {
    None
  };
  ai_api_service
    .forward_request_with_method(method, &api_path, api_alias, api_key, body, query_params)
    .await
    .map_err(InferenceError::from)
}

pub(crate) fn convert_reqwest_to_axum(
  reqwest_response: reqwest::Response,
) -> Result<Response, InferenceError> {
  let status = reqwest_response.status();
  let headers = reqwest_response.headers().clone();

  let mut builder = Response::builder().status(status.as_u16());
  for (key, value) in &headers {
    if let Ok(value_str) = value.to_str() {
      builder = builder.header(key.as_str(), value_str);
    }
  }

  let body_stream = reqwest_response.bytes_stream();
  let body = Body::from_stream(body_stream);

  builder
    .body(body)
    .map_err(|e| InferenceError::Internal(format!("Failed to build axum response: {}", e)))
}
