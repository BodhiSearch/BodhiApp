use crate::SharedContext;
use serde_json::Value;
use services::inference::{LocalLlama, LocalLlamaError};
use services::Alias;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::task::JoinHandle;
use tracing::{debug, info, warn};

#[derive(Debug)]
pub struct LocalLlamaImpl {
  ctx: Arc<dyn SharedContext>,
  keep_alive_secs: RwLock<i64>,
  timer_handle: RwLock<Option<JoinHandle<()>>>,
}

impl LocalLlamaImpl {
  pub fn new(ctx: Arc<dyn SharedContext>, keep_alive_secs: i64) -> Self {
    Self {
      ctx,
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

  fn on_request_completed(&self) {
    let keep_alive = *self.keep_alive_secs.read().unwrap();
    match keep_alive {
      -1 => {}
      0 => {
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
          if let Err(err) = ctx.stop().await {
            debug!(?err, "Error stopping server after request completion");
          }
        });
      }
      _ => {
        info!("Resetting keep-alive timer after request");
        self.start_timer();
      }
    }
  }
}

#[async_trait::async_trait]
impl LocalLlama for LocalLlamaImpl {
  async fn forward_request(
    &self,
    api_path: &str,
    request: Value,
    alias: Alias,
  ) -> Result<reqwest::Response, LocalLlamaError> {
    let result = self
      .ctx
      .forward_request(api_path, request, alias)
      .await
      .map_err(|e| LocalLlamaError::Internal(e.to_string()));
    // Intentional: reset keep-alive on every completion, including errors. A failed
    // forward still touched the model (load attempt or in-flight request); the timer
    // should debounce from the latest user activity, not the latest success.
    self.on_request_completed();
    result
  }

  async fn stop(&self) -> Result<(), LocalLlamaError> {
    self.cancel_timer();
    self
      .ctx
      .stop()
      .await
      .map_err(|e| LocalLlamaError::Internal(e.to_string()))
  }

  async fn set_variant(&self, variant: &str) -> Result<(), LocalLlamaError> {
    self
      .ctx
      .set_exec_variant(variant)
      .await
      .map_err(|e| LocalLlamaError::Internal(e.to_string()))
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
