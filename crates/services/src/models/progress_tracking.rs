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

  download_request.downloaded_bytes = downloaded_bytes as i64;
  download_request.total_bytes = Some(total_bytes as i64);
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
#[path = "test_progress_tracking.rs"]
mod test_progress_tracking;
