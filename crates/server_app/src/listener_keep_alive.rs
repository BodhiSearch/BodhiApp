use std::sync::Arc;
use std::sync::RwLock;
use std::time::Duration;

use objs::SettingSource;
use serde_yaml::Value;
use server_core::{ServerState, ServerStateListener, SharedContext};
use services::DEFAULT_KEEP_ALIVE_SECS;
use services::{SettingsChangeListener, BODHI_KEEP_ALIVE_SECS};
use tokio::task::JoinHandle;
use tracing::debug;
use tracing::info;
use tracing::warn;

#[derive(Debug)]
pub struct ServerKeepAlive {
  keep_alive: RwLock<i64>,
  timer_handle: RwLock<Option<JoinHandle<()>>>,
  shared_context: Arc<dyn SharedContext>,
}

impl ServerKeepAlive {
  pub fn new(shared_context: Arc<dyn SharedContext>, keep_alive: i64) -> Self {
    Self {
      keep_alive: RwLock::new(keep_alive),
      timer_handle: RwLock::new(None),
      shared_context,
    }
  }

  fn start_timer(&self) {
    let keep_alive = *self.keep_alive.read().unwrap();
    if keep_alive < 0 {
      debug!("Keep alive is < 0, cancelling the timer");
      self.cancel_timer();
      return;
    }
    if keep_alive == 0 {
      debug!("Keep alive is 0, cancelling the timer and stopping the server");
      self.cancel_timer();
      let ctx = self.shared_context.clone();
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
    let ctx = self.shared_context.clone();
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
}

impl SettingsChangeListener for ServerKeepAlive {
  fn on_change(
    &self,
    key: &str,
    _prev_value: &Option<Value>,
    _prev_source: &SettingSource,
    new_value: &Option<Value>,
    _new_source: &SettingSource,
  ) {
    if key != BODHI_KEEP_ALIVE_SECS {
      return;
    }

    let new_keep_alive = new_value
      .as_ref()
      .and_then(|v| v.as_i64())
      .unwrap_or(DEFAULT_KEEP_ALIVE_SECS);

    debug!("Updating keep-alive to {} seconds", new_keep_alive);
    *self.keep_alive.write().unwrap() = new_keep_alive;

    // If timer is running, restart it with new duration
    self.start_timer();
  }
}

#[async_trait::async_trait]
impl ServerStateListener for ServerKeepAlive {
  async fn on_state_change(&self, state: ServerState) {
    info!("on_state_change: {:?}", state);
    match state {
      ServerState::Start => {
        let keep_alive = *self.keep_alive.read().unwrap();
        if keep_alive >= 0 {
          self.start_timer();
        }
      }
      ServerState::Stop => {
        self.cancel_timer();
      }
      ServerState::ChatCompletions { alias: _ } => {
        let keep_alive = *self.keep_alive.read().unwrap();
        match keep_alive {
          -1 => {} // Never stop
          0 => {
            let ctx = self.shared_context.clone();
            if let Err(err) = ctx.stop().await {
              debug!(?err, "Error stopping server after chat completion");
            }
          }
          _ => {
            // Reset timer
            info!("Resetting keep-alive timer");
            self.start_timer();
          }
        }
      }
      ServerState::Variant { variant: _ } => {} // No action needed
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::ServerKeepAlive;
  use objs::SettingSource;
  use rstest::rstest;
  use serde_yaml::Value;
  use server_core::{MockSharedContext, ServerState, ServerStateListener};
  use services::{SettingsChangeListener, BODHI_KEEP_ALIVE_SECS};
  use std::{sync::Arc, time::Duration};

  #[rstest]
  #[case::never_stop_to_timed_stop(-1, 5)]
  #[case::immediate_stop_to_timed_stop(0, 5)]
  #[tokio::test]
  async fn test_keep_alive_setting_changes_starts_timer(#[case] from: i64, #[case] to: i64) {
    let mut mock_ctx = MockSharedContext::new();
    mock_ctx.expect_stop().never();

    let keep_alive = ServerKeepAlive::new(Arc::new(mock_ctx), from);
    keep_alive.on_state_change(ServerState::Start).await;

    keep_alive.on_change(
      BODHI_KEEP_ALIVE_SECS,
      &Some(Value::Number(from.into())),
      &SettingSource::Database,
      &Some(Value::Number(to.into())),
      &SettingSource::Database,
    );
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(*keep_alive.keep_alive.read().unwrap(), to);
    assert!(keep_alive.timer_handle.read().unwrap().is_some());
  }
  // #[case::immediate_stop_to_timed_stop(0, -1)]
  // #[case::timed_stop_to_never_stop(5, -1)]
  // #[case::change_timer_duration(5, 10)]
  #[rstest]
  #[case::never_stop_to_immediate_stop(-1, 0)]
  #[case::timed_stop_to_immediate_stop(5, 0)]
  #[tokio::test]
  async fn test_keep_alive_setting_changes_stops_timer_for_always_stop(
    #[case] from: i64,
    #[case] to: i64,
  ) {
    let mut mock_ctx = MockSharedContext::new();
    mock_ctx.expect_stop().times(1).return_once(|| Ok(()));
    mock_ctx.expect_is_loaded().times(1).return_once(|| true);

    let keep_alive = ServerKeepAlive::new(Arc::new(mock_ctx), from);
    keep_alive.on_state_change(ServerState::Start).await;

    keep_alive.on_change(
      BODHI_KEEP_ALIVE_SECS,
      &Some(Value::Number(from.into())),
      &SettingSource::Database,
      &Some(Value::Number(to.into())),
      &SettingSource::Database,
    );

    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(*keep_alive.keep_alive.read().unwrap(), to);
    assert!(keep_alive.timer_handle.read().unwrap().is_none());
  }

  #[rstest]
  #[case::never_stop_no_stop_call(-1)]
  #[tokio::test]
  async fn test_chat_completion_never_stop(#[case] secs: i64) {
    let mut mock_ctx = MockSharedContext::new();
    mock_ctx.expect_stop().never();

    let keep_alive = ServerKeepAlive::new(Arc::new(mock_ctx), secs);
    keep_alive.on_state_change(ServerState::Start).await;

    keep_alive
      .on_state_change(ServerState::ChatCompletions {
        alias: "test".to_string(),
      })
      .await;

    tokio::time::sleep(Duration::from_millis(100)).await;
  }

  #[rstest]
  #[case::immediate_stop_calls_stop(0)]
  #[tokio::test]
  async fn test_chat_completion_immediate_stop(#[case] secs: i64) {
    let mut mock_ctx = MockSharedContext::new();
    mock_ctx.expect_stop().times(1).return_once(|| Ok(()));

    let keep_alive = ServerKeepAlive::new(Arc::new(mock_ctx), secs);
    keep_alive.on_state_change(ServerState::Start).await;

    keep_alive
      .on_state_change(ServerState::ChatCompletions {
        alias: "test".to_string(),
      })
      .await;

    tokio::time::sleep(Duration::from_millis(100)).await;
  }

  #[rstest]
  #[case::timed_stop_no_immediate_stop(1)]
  #[tokio::test]
  async fn test_chat_completion_timed_stop(#[case] secs: i64) {
    let mut mock_ctx = MockSharedContext::new();
    mock_ctx.expect_stop().never();

    let keep_alive = ServerKeepAlive::new(Arc::new(mock_ctx), secs);
    keep_alive.on_state_change(ServerState::Start).await;

    keep_alive
      .on_state_change(ServerState::ChatCompletions {
        alias: "test".to_string(),
      })
      .await;
  }

  #[rstest]
  #[case::longer_timeout_reset(2)]
  #[case::short_timeout_reset(1)]
  #[tokio::test]
  async fn test_timer_reset_on_chat_completion(#[case] secs: i64) {
    let mut mock_ctx = MockSharedContext::new();
    mock_ctx.expect_stop().times(1).return_once(|| Ok(()));

    let keep_alive = ServerKeepAlive::new(Arc::new(mock_ctx), secs);
    keep_alive.on_state_change(ServerState::Start).await;

    // Wait a bit then send chat completion to reset timer
    tokio::time::sleep(Duration::from_millis(100)).await;
    keep_alive
      .on_state_change(ServerState::ChatCompletions {
        alias: "test".to_string(),
      })
      .await;

    // Verify timer was reset by waiting full duration
    tokio::time::sleep(Duration::from_secs(secs as u64)).await;
  }

  #[rstest]
  #[case::longer_timeout_cancelled(2)]
  #[case::short_timeout_cancelled(1)]
  #[tokio::test]
  async fn test_timer_cancellation_on_stop(#[case] secs: i64) {
    let mut mock_ctx = MockSharedContext::new();
    mock_ctx.expect_stop().times(0);

    let keep_alive = ServerKeepAlive::new(Arc::new(mock_ctx), secs);
    keep_alive.on_state_change(ServerState::Start).await;

    // Stop should cancel timer
    keep_alive.on_state_change(ServerState::Stop).await;

    // Wait to verify timer was cancelled
    tokio::time::sleep(Duration::from_millis((secs * 1000) as u64 + 100)).await;
  }

  #[rstest]
  #[case::never_stop_no_timer(-1)]
  #[tokio::test]
  async fn test_start_behavior_never_stop(#[case] secs: i64) {
    let mut mock_ctx = MockSharedContext::new();
    mock_ctx.expect_stop().times(0);

    let keep_alive = ServerKeepAlive::new(Arc::new(mock_ctx), secs);
    keep_alive.on_state_change(ServerState::Start).await;

    tokio::time::sleep(Duration::from_millis(1000)).await;
  }

  #[rstest]
  #[case::immediate_stop_no_timer(0)]
  #[tokio::test]
  async fn test_start_behavior_immediate_stop(#[case] secs: i64) {
    let mut mock_ctx = MockSharedContext::new();
    mock_ctx.expect_stop().times(1).return_once(|| Ok(()));
    mock_ctx.expect_is_loaded().times(1).return_once(|| true);

    let keep_alive = ServerKeepAlive::new(Arc::new(mock_ctx), secs);
    keep_alive.on_state_change(ServerState::Start).await;

    tokio::time::sleep(Duration::from_millis(100)).await;
  }
}
