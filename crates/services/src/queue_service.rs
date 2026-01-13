use async_trait::async_trait;
use chrono::{DateTime, Utc};
use objs::{
  gguf::{extract_metadata, get_chat_template},
  Alias, AliasSource,
};
use std::{
  collections::VecDeque,
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
  },
};
use tokio::sync::{Mutex, Notify};

use crate::{
  db::{DbService, ModelMetadataRow},
  DataService, HubService,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Error type for metadata extraction operations
#[derive(Debug, thiserror::Error)]
pub enum MetadataExtractionError {
  #[error("Cannot extract metadata for API alias")]
  ApiAliasNotSupported,
  #[error("GGUF file not found: {0}")]
  FileNotFound(String),
  #[error("Failed to parse GGUF: {0}")]
  ParseError(String),
  #[error("Database error: {0}")]
  DbError(#[from] crate::db::DbError),
  #[error("{0}")]
  Other(String),
}

/// Extracts GGUF metadata and stores in database. Returns the metadata row on success.
///
/// This function:
/// 1. Extracts repo/filename/snapshot from Alias (returns error for API aliases)
/// 2. Locates GGUF file via hub_service.find_local_file()
/// 3. Parses GGUF metadata via objs::gguf::GGUFMetadata::new()
/// 4. Extracts capabilities via objs::gguf::extract_metadata()
/// 5. Builds and upserts ModelMetadataRow
/// 6. Returns the row for caller
pub async fn extract_and_store_metadata(
  alias: &Alias,
  hub_service: &dyn HubService,
  db_service: &dyn DbService,
) -> std::result::Result<ModelMetadataRow, MetadataExtractionError> {
  use objs::Repo;
  use std::str::FromStr;

  // Extract alias information
  // Note: Metadata is always stored with source='model' since both UserAlias and ModelAlias
  // reference the same physical GGUF file. UserAlias is just a user configuration pointing
  // to a physical model file represented by ModelAlias.
  let (repo_str, filename, snapshot) = match alias {
    Alias::User(ua) => (
      ua.repo.to_string(),
      ua.filename.clone(),
      ua.snapshot.clone(),
    ),
    Alias::Model(ma) => (
      ma.repo.to_string(),
      ma.filename.clone(),
      ma.snapshot.clone(),
    ),
    Alias::Api(_) => return Err(MetadataExtractionError::ApiAliasNotSupported),
  };

  // Locate GGUF file
  let repo_obj = Repo::from_str(&repo_str)
    .map_err(|e| MetadataExtractionError::FileNotFound(format!("Invalid repo format: {}", e)))?;

  let hub_file = hub_service
    .find_local_file(&repo_obj, &filename, Some(snapshot.clone()))
    .map_err(|e| {
      MetadataExtractionError::FileNotFound(format!("File not found in cache: {}", e))
    })?;

  let file_path = hub_file.path();

  // Parse GGUF metadata
  let gguf_metadata = objs::gguf::GGUFMetadata::new(&file_path)
    .map_err(|e| MetadataExtractionError::ParseError(format!("Failed to parse GGUF: {}", e)))?;

  // Extract capabilities and metadata
  let model_metadata = extract_metadata(&gguf_metadata, &filename);

  // Extract chat template directly from GGUF metadata
  let chat_template = get_chat_template(&gguf_metadata);

  // Helper to convert Option<bool> to Option<i64> (SQLite boolean representation)
  let bool_to_i64 = |b: Option<bool>| b.map(|v| if v { 1 } else { 0 });

  // Convert to database row
  // Always use AliasSource::Model since this represents the physical GGUF file
  let now = db_service.now();
  let metadata_row = ModelMetadataRow {
    id: 0, // Will be set by database
    source: AliasSource::Model.to_string(),
    repo: Some(repo_str.clone()),
    filename: Some(filename.clone()),
    snapshot: Some(snapshot.clone()),
    api_model_id: None,
    capabilities_vision: bool_to_i64(model_metadata.capabilities.vision),
    capabilities_audio: bool_to_i64(model_metadata.capabilities.audio),
    capabilities_thinking: bool_to_i64(model_metadata.capabilities.thinking),
    capabilities_function_calling: bool_to_i64(model_metadata.capabilities.tools.function_calling),
    capabilities_structured_output: bool_to_i64(
      model_metadata.capabilities.tools.structured_output,
    ),
    context_max_input_tokens: model_metadata.context.max_input_tokens.map(|v| v as i64),
    context_max_output_tokens: model_metadata.context.max_output_tokens.map(|v| v as i64),
    architecture: serde_json::to_string(&model_metadata.architecture).ok(),
    additional_metadata: None,
    chat_template,
    extracted_at: now,
    created_at: now,
    updated_at: now,
  };

  // Debug logging before upsert
  tracing::debug!(
    "Upserting metadata: source='{}', repo={:?}, filename={:?}, snapshot={:?}",
    metadata_row.source,
    metadata_row.repo,
    metadata_row.filename,
    metadata_row.snapshot
  );

  // Upsert to database
  db_service.upsert_model_metadata(&metadata_row).await?;

  tracing::info!(
    "Metadata extracted and stored for: {}/{}/{}",
    repo_str,
    filename,
    snapshot
  );

  Ok(metadata_row)
}

/// Task types for metadata refresh operations
#[derive(Debug, Clone)]
pub enum RefreshTask {
  /// Refresh metadata for all local GGUF models
  RefreshAll { created_at: DateTime<Utc> },
  /// Refresh metadata for a single model by alias
  RefreshSingle {
    alias: String,
    created_at: DateTime<Utc>,
  },
}

impl RefreshTask {
  pub fn created_at(&self) -> DateTime<Utc> {
    match self {
      RefreshTask::RefreshAll { created_at } => *created_at,
      RefreshTask::RefreshSingle { created_at, .. } => *created_at,
    }
  }
}

/// Producer interface for enqueuing tasks
#[async_trait]
#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait QueueProducer: Send + Sync + std::fmt::Debug {
  /// Enqueue a refresh task (non-blocking)
  async fn enqueue(&self, task: RefreshTask) -> Result<()>;

  /// Get current queue length (for monitoring)
  async fn queue_length(&self) -> usize;

  /// Get queue status: "idle" or "processing"
  fn queue_status(&self) -> String;
}

/// Consumer interface for dequeuing tasks
#[async_trait]
pub trait QueueConsumer: Send + Sync {
  /// Dequeue next task (blocking until available or shutdown)
  async fn dequeue(&self) -> Option<RefreshTask>;

  /// Signal worker shutdown
  fn shutdown(&self);
}

/// In-memory queue implementation using VecDeque
#[derive(Debug)]
pub struct InMemoryQueue {
  queue: Arc<Mutex<VecDeque<RefreshTask>>>,
  notify: Arc<Notify>,
  shutdown: Arc<AtomicBool>,
  is_processing: Arc<AtomicBool>,
}

impl InMemoryQueue {
  pub fn new() -> Self {
    Self {
      queue: Arc::new(Mutex::new(VecDeque::new())),
      notify: Arc::new(Notify::new()),
      shutdown: Arc::new(AtomicBool::new(false)),
      is_processing: Arc::new(AtomicBool::new(false)),
    }
  }

  pub fn get_is_processing(&self) -> Arc<AtomicBool> {
    Arc::clone(&self.is_processing)
  }
}

impl Default for InMemoryQueue {
  fn default() -> Self {
    Self::new()
  }
}

#[async_trait]
impl QueueProducer for InMemoryQueue {
  async fn enqueue(&self, task: RefreshTask) -> Result<()> {
    self.queue.lock().await.push_back(task);
    self.notify.notify_one();
    Ok(())
  }

  async fn queue_length(&self) -> usize {
    self.queue.lock().await.len()
  }

  fn queue_status(&self) -> String {
    if self.is_processing.load(Ordering::Relaxed) {
      "processing".to_string()
    } else {
      let queue_len = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async { self.queue.lock().await.len() })
      });
      if queue_len > 0 {
        "processing".to_string()
      } else {
        "idle".to_string()
      }
    }
  }
}

#[async_trait]
impl QueueConsumer for InMemoryQueue {
  async fn dequeue(&self) -> Option<RefreshTask> {
    loop {
      if self.shutdown.load(Ordering::Relaxed) {
        return None;
      }

      let mut queue = self.queue.lock().await;
      if let Some(task) = queue.pop_front() {
        return Some(task);
      }
      drop(queue);

      self.notify.notified().await;
    }
  }

  fn shutdown(&self) {
    self.shutdown.store(true, Ordering::Relaxed);
    self.notify.notify_one();
  }
}

/// Worker that processes refresh tasks from the queue
pub struct RefreshWorker {
  consumer: Arc<dyn QueueConsumer>,
  hub_service: Arc<dyn HubService>,
  data_service: Arc<dyn DataService>,
  db_service: Arc<dyn DbService>,
  is_processing: Arc<AtomicBool>,
}

impl RefreshWorker {
  pub fn new(
    consumer: Arc<dyn QueueConsumer>,
    hub_service: Arc<dyn HubService>,
    data_service: Arc<dyn DataService>,
    db_service: Arc<dyn DbService>,
    is_processing: Arc<AtomicBool>,
  ) -> Self {
    Self {
      consumer,
      hub_service,
      data_service,
      db_service,
      is_processing,
    }
  }

  /// Run the worker loop (blocks until shutdown)
  pub async fn run(&self) {
    tracing::info!("Model metadata refresh worker started");

    while let Some(task) = self.consumer.dequeue().await {
      self.is_processing.store(true, Ordering::Relaxed);
      if let Err(e) = self.process_task(task).await {
        tracing::error!("Task processing failed: {}", e);
      }
      self.is_processing.store(false, Ordering::Relaxed);
    }

    tracing::info!("Model metadata refresh worker shutting down");
  }

  async fn process_task(&self, task: RefreshTask) -> Result<()> {
    match task {
      RefreshTask::RefreshAll { .. } => self.refresh_all().await,
      RefreshTask::RefreshSingle { alias, .. } => self.refresh_single(&alias).await,
    }
  }

  async fn refresh_all(&self) -> Result<()> {
    tracing::info!("Refreshing metadata for all local GGUF models");

    let aliases = self.data_service.list_aliases().await?;
    let local_aliases: Vec<&Alias> = aliases
      .iter()
      .filter(|a| matches!(a, Alias::User(_) | Alias::Model(_)))
      .collect();

    tracing::info!("Found {} local models to process", local_aliases.len());

    let mut success_count = 0;
    let mut error_count = 0;

    for alias in local_aliases {
      match self.extract_and_store(alias).await {
        Ok(updated) => {
          if updated {
            success_count += 1;
          }
        }
        Err(e) => {
          tracing::warn!(
            "Failed to extract metadata for {}: {}",
            alias.alias_name(),
            e
          );
          error_count += 1;
        }
      }
    }

    tracing::info!(
      "Metadata refresh complete: {} updated, {} errors",
      success_count,
      error_count
    );

    Ok(())
  }

  async fn refresh_single(&self, alias_name: &str) -> Result<()> {
    tracing::info!("Refreshing metadata for model: {}", alias_name);

    let alias = self
      .data_service
      .find_alias(alias_name)
      .await
      .ok_or_else(|| format!("Alias not found: {}", alias_name))?;

    match &alias {
      Alias::User(_) | Alias::Model(_) => {
        self.extract_and_store(&alias).await?;
        tracing::info!("Metadata refresh complete for: {}", alias_name);
        Ok(())
      }
      Alias::Api(_) => Err(format!("Cannot refresh metadata for API alias: {}", alias_name).into()),
    }
  }

  async fn extract_and_store(&self, alias: &Alias) -> Result<bool> {
    // For API aliases, skip
    if matches!(alias, Alias::Api(_)) {
      return Ok(false);
    }

    // Extract repo/filename/snapshot for snapshot check
    let (repo_str, filename, snapshot) = match alias {
      Alias::User(ua) => (ua.repo.to_string(), &ua.filename, &ua.snapshot),
      Alias::Model(ma) => (ma.repo.to_string(), &ma.filename, &ma.snapshot),
      Alias::Api(_) => unreachable!(),
    };

    // Check if metadata exists and snapshot matches (optimization for async queue)
    if let Some(existing) = self
      .db_service
      .get_model_metadata_by_file(&repo_str, filename, snapshot)
      .await?
    {
      if existing.snapshot.as_deref() == Some(snapshot.as_str()) {
        tracing::debug!(
          "Metadata for {}/{}/{} is up-to-date",
          repo_str,
          filename,
          snapshot
        );
        return Ok(false);
      }
    }

    // Use shared extraction function
    extract_and_store_metadata(alias, self.hub_service.as_ref(), self.db_service.as_ref())
      .await
      .map_err(|e| format!("Failed to extract metadata: {}", e))?;

    Ok(true)
  }
}

#[cfg(test)]
mod tests {
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
}
