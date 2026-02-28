use super::*;
use tokio::time::{timeout, Duration};

#[tokio::test]
async fn test_enqueue_dequeue() {
  let queue = InMemoryQueue::new();
  let now = Utc::now();

  let task = RefreshTask::RefreshAll { created_at: now };

  queue.enqueue(task.clone()).await.unwrap();

  let dequeued = timeout(Duration::from_millis(100), queue.dequeue())
    .await
    .unwrap()
    .unwrap();

  match dequeued {
    RefreshTask::RefreshAll { created_at } => {
      assert_eq!(created_at.timestamp(), now.timestamp());
    }
    _ => panic!("Expected RefreshAll task"),
  }
}

#[tokio::test]
async fn test_queue_length() {
  let queue = InMemoryQueue::new();
  let now = Utc::now();

  assert_eq!(queue.queue_length().await, 0);

  queue
    .enqueue(RefreshTask::RefreshAll { created_at: now })
    .await
    .unwrap();
  queue
    .enqueue(RefreshTask::RefreshSingle {
      alias: "test".to_string(),
      created_at: now,
    })
    .await
    .unwrap();

  assert_eq!(queue.queue_length().await, 2);
}

#[tokio::test]
async fn test_shutdown_returns_none() {
  let queue = InMemoryQueue::new();

  queue.shutdown();

  let result = timeout(Duration::from_millis(100), queue.dequeue()).await;
  assert!(result.is_ok());
  assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn test_fifo_order() {
  let queue = InMemoryQueue::new();
  let now = Utc::now();

  queue
    .enqueue(RefreshTask::RefreshSingle {
      alias: "first".to_string(),
      created_at: now,
    })
    .await
    .unwrap();
  queue
    .enqueue(RefreshTask::RefreshSingle {
      alias: "second".to_string(),
      created_at: now,
    })
    .await
    .unwrap();

  let first = queue.dequeue().await.unwrap();
  let second = queue.dequeue().await.unwrap();

  match first {
    RefreshTask::RefreshSingle { alias, .. } => assert_eq!(alias, "first"),
    _ => panic!("Expected RefreshSingle"),
  }

  match second {
    RefreshTask::RefreshSingle { alias, .. } => assert_eq!(alias, "second"),
    _ => panic!("Expected RefreshSingle"),
  }
}
