use crate::{queue_service::RefreshTask, QueueProducer};
use async_trait::async_trait;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// No-op queue producer for testing that doesn't track expectations.
/// Use `MockQueueProducer` when you need to verify enqueue calls.
#[derive(Debug, Default)]
pub struct StubQueue;

#[async_trait]
impl QueueProducer for StubQueue {
  async fn enqueue(&self, _task: RefreshTask) -> Result<()> {
    Ok(())
  }

  async fn queue_length(&self) -> usize {
    0
  }

  fn queue_status(&self) -> String {
    "idle".to_string()
  }
}
