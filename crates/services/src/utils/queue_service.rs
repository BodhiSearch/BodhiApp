use crate::models::{
  gguf::{extract_metadata, get_chat_template},
  Alias, AliasSource,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::{
  collections::VecDeque,
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
  },
};
use tokio::sync::{Mutex, Notify};

use crate::{db::DbService, models::ModelMetadataEntity, DataService, HubService};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

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
pub async fn extract_and_store_metadata(
  alias: &Alias,
  hub_service: &dyn HubService,
  db_service: &dyn DbService,
) -> std::result::Result<ModelMetadataEntity, MetadataExtractionError> {
  use crate::models::Repo;
  use std::str::FromStr;

  // Metadata is always stored with source='model' since both UserAlias and ModelAlias
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
    Alias::Api(_) | Alias::ModelRouter(_) => {
      return Err(MetadataExtractionError::ApiAliasNotSupported)
    }
  };

  let repo_obj = Repo::from_str(&repo_str)
    .map_err(|e| MetadataExtractionError::FileNotFound(format!("Invalid repo format: {}", e)))?;

  let hub_file = hub_service
    .find_local_file(&repo_obj, &filename, Some(snapshot.clone()))
    .map_err(|e| {
      MetadataExtractionError::FileNotFound(format!("File not found in cache: {}", e))
    })?;

  let file_path = hub_file.path();

  let gguf_metadata = crate::models::gguf::GGUFMetadata::new(&file_path)
    .map_err(|e| MetadataExtractionError::ParseError(format!("Failed to parse GGUF: {}", e)))?;

  let model_metadata = extract_metadata(&gguf_metadata, &filename);

  let chat_template = get_chat_template(&gguf_metadata);

  // Always AliasSource::Model since this represents the physical GGUF file.
  let now = db_service.now();
  let metadata_row = ModelMetadataEntity {
    id: String::new(),
    tenant_id: String::new(),
    source: AliasSource::Model,
    repo: Some(repo_str.clone()),
    filename: Some(filename.clone()),
    snapshot: Some(snapshot.clone()),
    api_model_id: None,
    capabilities: Some(model_metadata.capabilities),
    context: Some(model_metadata.context),
    architecture: Some(model_metadata.architecture),
    additional_metadata: None,
    chat_template,
    extracted_at: now,
    created_at: now,
    updated_at: now,
  };

  tracing::debug!(
    "Upserting metadata: source='{}', repo={:?}, filename={:?}, snapshot={:?}",
    metadata_row.source,
    metadata_row.repo,
    metadata_row.filename,
    metadata_row.snapshot
  );

  db_service.upsert_model_metadata(&metadata_row).await?;

  tracing::info!(
    "Metadata extracted and stored for: {}/{}/{}",
    repo_str,
    filename,
    snapshot
  );

  Ok(metadata_row)
}

#[derive(Debug, Clone)]
pub enum RefreshTask {
  RefreshAll {
    created_at: DateTime<Utc>,
  },
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

#[async_trait]
#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait QueueProducer: Send + Sync + std::fmt::Debug {
  async fn enqueue(&self, task: RefreshTask) -> Result<()>;

  async fn queue_length(&self) -> usize;

  /// Returns "idle" or "processing".
  fn queue_status(&self) -> String;
}

#[async_trait]
pub trait QueueConsumer: Send + Sync {
  /// Blocks until a task is available or shutdown is signalled.
  async fn dequeue(&self) -> Option<RefreshTask>;

  fn shutdown(&self);
}

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

    let aliases = self.data_service.list_aliases("", "").await?;
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
      .find_alias("", "", alias_name)
      .await
      .ok_or_else(|| format!("Alias not found: {}", alias_name))?;

    match &alias {
      Alias::User(_) | Alias::Model(_) => {
        self.extract_and_store(&alias).await?;
        tracing::info!("Metadata refresh complete for: {}", alias_name);
        Ok(())
      }
      Alias::Api(_) | Alias::ModelRouter(_) => Err(
        format!(
          "Cannot refresh metadata for non-local alias: {}",
          alias_name
        )
        .into(),
      ),
    }
  }

  async fn extract_and_store(&self, alias: &Alias) -> Result<bool> {
    // API aliases and model-routers have no physical GGUF file.
    if matches!(alias, Alias::Api(_) | Alias::ModelRouter(_)) {
      return Ok(false);
    }

    let (repo_str, filename, snapshot) = match alias {
      Alias::User(ua) => (ua.repo.to_string(), &ua.filename, &ua.snapshot),
      Alias::Model(ma) => (ma.repo.to_string(), &ma.filename, &ma.snapshot),
      Alias::Api(_) | Alias::ModelRouter(_) => unreachable!(),
    };

    // Skip re-extraction when the stored snapshot already matches (async-queue optimization).
    if let Some(existing) = self
      .db_service
      .get_model_metadata_by_file("", &repo_str, filename, snapshot)
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

    extract_and_store_metadata(alias, self.hub_service.as_ref(), self.db_service.as_ref())
      .await
      .map_err(|e| format!("Failed to extract metadata: {}", e))?;

    Ok(true)
  }
}

#[cfg(test)]
#[path = "test_queue_service.rs"]
mod test_queue_service;
