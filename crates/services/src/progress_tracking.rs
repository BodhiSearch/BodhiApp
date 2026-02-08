use crate::db::DbService;
use hf_hub::api::tokio::Progress as HfProgress;
use std::sync::{
  atomic::{AtomicU64, Ordering},
  Arc,
};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{error, info};

// Progress wrapper to work around trait object limitations
// Since hf-hub Progress trait uses impl Future, it cannot be used as dyn trait object
#[derive(Debug, Clone)]
pub enum Progress {
  Database(DatabaseProgress),
}

impl HfProgress for Progress {
  async fn init(&mut self, size: usize, filename: &str) {
    match self {
      Progress::Database(db_progress) => db_progress.init(size, filename).await,
    }
  }

  async fn update(&mut self, size: usize) {
    match self {
      Progress::Database(db_progress) => db_progress.update(size).await,
    }
  }

  async fn finish(&mut self) {
    match self {
      Progress::Database(db_progress) => db_progress.finish().await,
    }
  }
}

const SYNC_INTERVAL: u64 = 3000;
/// Production progress tracker that updates download request in database
/// Implements hf-hub's Progress trait for seamless integration
/// Uses lock-free atomics for high performance
#[derive(Debug)]
pub struct DatabaseProgress {
  db_service: Arc<dyn DbService>,
  request_id: String,
  downloaded_bytes: Arc<AtomicU64>,
  total_bytes: Arc<AtomicU64>,
  last_sync_time: Arc<AtomicU64>, // Last database sync time in milliseconds
}

fn current_time_millis() -> u64 {
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap_or_default()
    .as_millis() as u64
}

impl Clone for DatabaseProgress {
  fn clone(&self) -> Self {
    Self {
      db_service: self.db_service.clone(),
      request_id: self.request_id.clone(),
      downloaded_bytes: self.downloaded_bytes.clone(),
      total_bytes: self.total_bytes.clone(),
      last_sync_time: self.last_sync_time.clone(),
    }
  }
}

impl DatabaseProgress {
  /// Create a new database progress tracker
  pub fn new(db_service: Arc<dyn DbService>, request_id: String) -> Self {
    Self {
      db_service,
      request_id,
      downloaded_bytes: Arc::new(AtomicU64::new(0)),
      total_bytes: Arc::new(AtomicU64::new(0)),
      last_sync_time: Arc::new(AtomicU64::new(0)),
    }
  }

  /// Sync progress to database without spawning
  async fn sync_to_database(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let downloaded = self.downloaded_bytes.load(Ordering::Relaxed);
    let total = self.total_bytes.load(Ordering::Relaxed);

    if total == 0 {
      return Ok(()); // No data to sync yet
    }

    update_download_progress(
      self.db_service.clone(),
      self.request_id.clone(),
      downloaded,
      total,
    )
    .await
  }
}

impl HfProgress for DatabaseProgress {
  async fn init(&mut self, size: usize, filename: &str) {
    info!("Progress init: filename={}, size={}", filename, size);
    self.total_bytes.store(size as u64, Ordering::Relaxed);
    self.downloaded_bytes.store(0, Ordering::Relaxed);
    self
      .last_sync_time
      .store(current_time_millis(), Ordering::Relaxed);

    // Sync initial state
    if let Err(e) = self.sync_to_database().await {
      error!("Failed to initialize download progress: {}", e);
    }
  }

  async fn update(&mut self, size: usize) {
    // Simple atomic accumulation
    self
      .downloaded_bytes
      .fetch_add(size as u64, Ordering::Relaxed);

    // Time-based database sync every 3 seconds
    let now = current_time_millis();
    let last_sync = self.last_sync_time.load(Ordering::Relaxed);
    if now - last_sync >= SYNC_INTERVAL {
      // Try to update the sync time atomically to prevent multiple concurrent syncs
      if self
        .last_sync_time
        .compare_exchange(last_sync, now, Ordering::Relaxed, Ordering::Relaxed)
        .is_ok()
      {
        if let Err(e) = self.sync_to_database().await {
          error!("Failed to sync progress to database: {}", e);
        }
      }
    }
  }

  async fn finish(&mut self) {
    let downloaded = self.downloaded_bytes.load(Ordering::Relaxed);
    let total = self.total_bytes.load(Ordering::Relaxed);
    info!(
      "Progress finish: downloaded={}, total={}",
      downloaded, total
    );

    // Final sync to database
    if let Err(e) = self.sync_to_database().await {
      error!("Failed to finish download progress: {}", e);
    } else {
      info!("Progress finish: database sync completed");
    }
  }
}

/// Updates download request progress in database
async fn update_download_progress(
  db_service: Arc<dyn DbService>,
  request_id: String,
  downloaded_bytes: u64,
  total_bytes: u64,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let mut download_request = db_service
    .get_download_request(&request_id)
    .await?
    .ok_or_else(|| format!("Download request {} not found", request_id))?;

  download_request.downloaded_bytes = downloaded_bytes;
  download_request.total_bytes = Some(total_bytes);
  download_request.updated_at = db_service.now();

  if download_request.started_at.is_none() {
    download_request.started_at = Some(db_service.now());
  }

  db_service
    .update_download_request(&download_request)
    .await?;

  Ok(())
}

#[cfg(test)]
mod tests {
  //! Progress tracking tests using hf-hub Progress trait
  //!
  //! Tests focus on:
  //! - DatabaseProgress integration with real database updates
  //! - Speed/ETA calculation logic
  //! - Hub service integration with DatabaseProgress
  use super::DatabaseProgress;
  use crate::Progress;
  use crate::{
    db::{DownloadRequest, ModelRepository},
    hub_service::HubService,
    test_utils::{test_db_service, test_hf_service, TestDbService, TestHfService},
  };
  use anyhow_trace::anyhow_trace;
  use hf_hub::api::tokio::Progress as HfProgress;
  use objs::{HubFile, Repo};
  use pretty_assertions::assert_eq;
  use rstest::rstest;
  use std::{sync::Arc, time::Duration};

  macro_rules! wait_for_event {
    ($rx:expr, $event_name:expr, $timeout:expr) => {{
      loop {
        tokio::select! {
            event = $rx.recv() => {
                match event {
                    Ok(e) if e == $event_name => break true,
                    _ => continue
                }
            }
            _ = tokio::time::sleep($timeout) => break false
        }
      }
    }};
  }

  /// Integration test: DatabaseProgress with real database updates using subscribe/notify
  #[rstest]
  #[awt]
  #[tokio::test]
  #[anyhow_trace]
  async fn test_database_progress_integration(
    #[future]
    #[from(test_db_service)]
    db_service: TestDbService,
  ) -> anyhow::Result<()> {
    let now = db_service.now();
    let mut rx = db_service.subscribe();
    let db_service = Arc::new(db_service);

    // Create a download request
    let request = DownloadRequest::new_pending("test/repo", "test.gguf", now);
    db_service.create_download_request(&request).await?;

    // Create DatabaseProgress
    let mut progress = Progress::Database(DatabaseProgress::new(
      db_service.clone(),
      request.id.clone(),
    ));

    // Test init() - simulates hf-hub calling init with file size
    progress.init(4096, "test.gguf").await;

    // Wait for init update
    let event_received = wait_for_event!(rx, "update_download_request", Duration::from_millis(100));
    assert!(event_received, "Timed out waiting for init update");

    // Test update() calls - simulates hf-hub calling update with incremental bytes
    // Note: Individual updates are batched every 3 seconds for performance
    progress.update(1024).await; // First chunk
    progress.update(1024).await; // Second chunk
    progress.update(2048).await; // Final chunk

    // Updates are batched, so no immediate database events expected

    // Test finish() - simulates hf-hub calling finish when download completes
    progress.finish().await;

    // Wait for finish update (finish() always syncs immediately)
    let event_received = wait_for_event!(rx, "update_download_request", Duration::from_millis(100));
    assert!(event_received, "Timed out waiting for finish update");

    // Verify final database state
    let retrieved = db_service.get_download_request(&request.id).await?;
    assert!(retrieved.is_some());

    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.downloaded_bytes, 4096); // Total downloaded
    assert_eq!(retrieved.total_bytes, Some(4096));
    assert!(retrieved.started_at.is_some());
    assert_eq!(retrieved.updated_at, db_service.now()); // Uses frozen time

    Ok(())
  }

  /// Unit test: HubService download with DatabaseProgress
  #[rstest]
  #[awt]
  #[tokio::test]
  #[anyhow_trace]
  async fn test_hub_service_with_database_progress(
    #[future]
    #[from(test_db_service)]
    db_service: TestDbService,
    #[from(test_hf_service)] mut test_hf_service: TestHfService,
  ) -> anyhow::Result<()> {
    let now = db_service.now();
    let db_service = Arc::new(db_service);

    // Create a download request
    let request = DownloadRequest::new_pending("test/repo", "test.gguf", now);
    db_service.create_download_request(&request).await?;

    // Setup mock HubService to accept progress parameter
    test_hf_service
      .inner_mock
      .expect_download()
      .times(1)
      .returning(move |_, _, _, _| Ok(HubFile::testalias()));

    // Test with DatabaseProgress
    let progress = Progress::Database(DatabaseProgress::new(
      db_service.clone(),
      request.id.clone(),
    ));
    let result = test_hf_service
      .download(&Repo::testalias(), "test.gguf", None, Some(progress))
      .await;

    assert!(result.is_ok());
    Ok(())
  }
}
