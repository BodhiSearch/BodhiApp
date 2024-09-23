use crate::db::{Conversation, DownloadRequest, Message, NoOpDbService};
use chrono::{DateTime, Timelike, Utc};
use derive_new::new;
use sqlx::{migrate::MigrateError, SqlitePool};
use std::{str::FromStr, sync::Arc};
use uuid::Uuid;

use super::DownloadStatus;

pub static CONVERSATIONS: &str = "conversations";
pub static MESSAGES: &str = "messages";

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait TimeService: std::fmt::Debug + Send + Sync {
  fn utc_now(&self) -> DateTime<Utc>;
}

#[derive(Debug, Clone, Default)]
pub struct DefaultTimeService;

impl TimeService for DefaultTimeService {
  fn utc_now(&self) -> DateTime<Utc> {
    let now = chrono::Utc::now();
    now.with_nanosecond(0).unwrap_or(now)
  }
}

#[derive(Debug, thiserror::Error)]
pub enum DbError {
  #[error("sqlx_query: {source}\ntable: {table}")]
  Sqlx {
    #[source]
    source: sqlx::Error,
    table: String,
  },
  #[error("sqlx_connect: {source}\nurl: {url}")]
  SqlxConnect {
    #[source]
    source: sqlx::Error,
    url: String,
  },
  #[error("sqlx_migrate: {0}")]
  Migrate(#[from] MigrateError),
}

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait DbService: std::fmt::Debug + Send + Sync {
  async fn migrate(&self) -> Result<(), DbError>;

  async fn save_conversation(&self, conversation: &mut Conversation) -> Result<(), DbError>;

  async fn save_message(&self, message: &mut Message) -> Result<(), DbError>;

  async fn list_conversations(&self) -> Result<Vec<Conversation>, DbError>;

  async fn delete_conversations(&self, id: &str) -> Result<(), DbError>;

  async fn delete_all_conversations(&self) -> Result<(), DbError>;

  async fn get_conversation_with_messages(&self, id: &str) -> Result<Conversation, DbError>;

  async fn create_download_request(&self, request: &DownloadRequest) -> Result<(), DbError>;

  async fn get_download_request(&self, id: &str) -> Result<Option<DownloadRequest>, DbError>;

  async fn update_download_request(&self, request: &DownloadRequest) -> Result<(), DbError>;

  async fn list_pending_downloads(&self) -> Result<Vec<DownloadRequest>, DbError>;
}

#[derive(Debug, Clone, new)]
pub struct SqliteDbService {
  pool: SqlitePool,
  time_service: Arc<dyn TimeService>,
}

impl SqliteDbService {
  pub fn no_op() -> impl DbService {
    NoOpDbService::new()
  }
}

#[async_trait::async_trait]
impl DbService for SqliteDbService {
  async fn migrate(&self) -> Result<(), DbError> {
    sqlx::migrate!("./migrations").run(&self.pool).await?;
    Ok(())
  }

  async fn save_conversation(&self, conversation: &mut Conversation) -> Result<(), DbError> {
    if conversation.id.is_empty() {
      conversation.id = Uuid::new_v4().to_string()
    } else {
      self.delete_conversations(&conversation.id).await?;
    }
    conversation.updated_at = self.time_service.utc_now();
    sqlx::query(
      "INSERT INTO conversations
        (
          id,
          title,
          created_at,
          updated_at
        )
        VALUES (?, ?, ?, ?)
        ON CONFLICT(id) DO UPDATE SET title = ?, updated_at = ?",
    )
    .bind(&conversation.id)
    .bind(&conversation.title)
    .bind(conversation.created_at.timestamp())
    .bind(conversation.updated_at.timestamp())
    .bind(&conversation.title)
    .bind(conversation.updated_at.timestamp())
    .execute(&self.pool)
    .await
    .map_err(|source| DbError::Sqlx {
      source,
      table: CONVERSATIONS.to_string(),
    })?;
    for message in &mut conversation.messages {
      if message.conversation_id.is_empty() {
        message.conversation_id.clone_from(&conversation.id);
      }
      self.save_message(message).await?;
    }
    Ok(())
  }

  async fn save_message(&self, message: &mut Message) -> Result<(), DbError> {
    if message.id.is_empty() {
      message.id = Uuid::new_v4().to_string();
    }
    sqlx::query(
      "INSERT INTO messages
        (
          id,
          conversation_id,
          role,
          name,
          content,
          created_at
        )
        VALUES (?, ?, ?, ?, ?, ?)
        ON CONFLICT(id) DO UPDATE SET conversation_id = ?, role = ?, name = ?, content = ?, created_at = ?",
    )
    .bind(&message.id)
    .bind(&message.conversation_id)
    .bind(&message.role)
    .bind(&message.name)
    .bind(&message.content)
    .bind(message.created_at.timestamp())
    .bind(&message.conversation_id)
    .bind(&message.role)
    .bind(&message.name)
    .bind(&message.content)
    .bind(message.created_at.timestamp())
    .execute(&self.pool)
    .await
    .map_err(|source| DbError::Sqlx {
      source,
      table: MESSAGES.to_string(),
    })?;
    Ok(())
  }

  async fn list_conversations(&self) -> Result<Vec<Conversation>, DbError> {
    let conversations = sqlx::query_as::<_, (String, String, i64, i64)>(
      "SELECT id, title, created_at, updated_at FROM conversations ORDER BY created_at DESC",
    )
    .fetch_all(&self.pool)
    .await
    .map_err(|source| DbError::Sqlx {
      source,
      table: CONVERSATIONS.to_string(),
    })?;

    let mut result = Vec::new();
    for (id, title, created_at, updated_at) in conversations {
      result.push(Conversation {
        id,
        title,
        created_at: chrono::DateTime::<Utc>::from_timestamp(created_at, 0).unwrap_or_default(),
        updated_at: chrono::DateTime::<Utc>::from_timestamp(updated_at, 0).unwrap_or_default(),
        messages: Vec::new(),
      });
    }

    Ok(result)
  }

  async fn get_conversation_with_messages(&self, id: &str) -> Result<Conversation, DbError> {
    let messages = sqlx::query_as::<_, Message>(
      "SELECT id, conversation_id, role, name, content, created_at FROM messages WHERE conversation_id = ?"
    )
    .bind(id)
    .fetch_all(&self.pool)
    .await.map_err(|source| DbError::Sqlx { source, table: MESSAGES.to_string() })?;

    let row = sqlx::query_as::<_, (String, String, i64, i64)>(
      "SELECT id, title, created_at, updated_at FROM conversations WHERE id = ?",
    )
    .bind(id)
    .fetch_one(&self.pool)
    .await
    .map_err(|source| DbError::Sqlx {
      source,
      table: CONVERSATIONS.to_string(),
    })?;

    let conversation = Conversation {
      id: row.0.clone(),
      title: row.1,
      created_at: chrono::DateTime::<Utc>::from_timestamp(row.2, 0).unwrap_or_default(),
      updated_at: chrono::DateTime::<Utc>::from_timestamp(row.3, 0).unwrap_or_default(),
      messages,
    };

    Ok(conversation)
  }

  async fn delete_conversations(&self, id: &str) -> Result<(), DbError> {
    sqlx::query("DELETE FROM messages where conversation_id=?")
      .bind(id)
      .execute(&self.pool)
      .await
      .map_err(|source| DbError::Sqlx {
        source,
        table: MESSAGES.to_string(),
      })?;
    sqlx::query("DELETE FROM conversations where id=?")
      .bind(id)
      .execute(&self.pool)
      .await
      .map_err(|source| DbError::Sqlx {
        source,
        table: CONVERSATIONS.to_string(),
      })?;
    Ok(())
  }

  async fn delete_all_conversations(&self) -> Result<(), DbError> {
    sqlx::query("DELETE FROM messages")
      .execute(&self.pool)
      .await
      .map_err(|source| DbError::Sqlx {
        source,
        table: MESSAGES.to_string(),
      })?;
    sqlx::query("DELETE FROM conversations")
      .execute(&self.pool)
      .await
      .map_err(|source| DbError::Sqlx {
        source,
        table: CONVERSATIONS.to_string(),
      })?;
    Ok(())
  }

  async fn create_download_request(&self, request: &DownloadRequest) -> Result<(), DbError> {
    sqlx::query(
      "INSERT INTO download_requests (id, repo, filename, status, created_at, updated_at)
       VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&request.id)
    .bind(&request.repo)
    .bind(&request.filename)
    .bind(request.status.to_string())
    .bind(request.created_at.timestamp())
    .bind(request.updated_at.timestamp())
    .execute(&self.pool)
    .await
    .map_err(|source| DbError::Sqlx {
      source,
      table: "download_requests".to_string(),
    })?;
    Ok(())
  }

  async fn get_download_request(&self, id: &str) -> Result<Option<DownloadRequest>, DbError> {
    let result = sqlx::query_as::<_, (String, String, String, String, i64, i64)>(
      "SELECT id, repo, filename, status, created_at, updated_at FROM download_requests WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&self.pool)
    .await
    .map_err(|source| DbError::Sqlx {
      source,
      table: "download_requests".to_string(),
    })?;

    match result {
      Some((id, repo, filename, status, created_at, updated_at)) => {
        let Ok(status) = DownloadStatus::from_str(&status) else {
          tracing::warn!("unknown download status: {status} for id: {id}");
          return Ok(None);
        };

        Ok(Some(DownloadRequest {
          id,
          repo,
          filename,
          status,
          created_at: DateTime::<Utc>::from_timestamp(created_at, 0).unwrap_or_default(),
          updated_at: DateTime::<Utc>::from_timestamp(updated_at, 0).unwrap_or_default(),
        }))
      }
      None => Ok(None),
    }
  }

  async fn update_download_request(&self, request: &DownloadRequest) -> Result<(), DbError> {
    sqlx::query("UPDATE download_requests SET status = ?, updated_at = ? WHERE id = ?")
      .bind(request.status.to_string())
      .bind(request.updated_at.timestamp())
      .bind(&request.id)
      .execute(&self.pool)
      .await
      .map_err(|source| DbError::Sqlx {
        source,
        table: "download_requests".to_string(),
      })?;
    Ok(())
  }

  async fn list_pending_downloads(&self) -> Result<Vec<DownloadRequest>, DbError> {
    let results = sqlx::query_as::<_, (String, String, String, String, i64, i64)>(
      "SELECT id, repo, filename, status, created_at, updated_at FROM download_requests WHERE status = ?",
    )
    .bind(DownloadStatus::Pending.to_string())
    .fetch_all(&self.pool)
    .await
    .map_err(|source| DbError::Sqlx {
      source,
      table: "download_requests".to_string(),
    })?;

    let results = results
      .into_iter()
      .filter_map(|(id, repo, filename, status, created_at, updated_at)| {
        let status = DownloadStatus::from_str(&status).ok()?;
        Some(DownloadRequest {
          id,
          repo,
          filename,
          status,
          created_at: DateTime::<Utc>::from_timestamp(created_at, 0).unwrap_or_default(),
          updated_at: DateTime::<Utc>::from_timestamp(updated_at, 0).unwrap_or_default(),
        })
      })
      .collect::<Vec<DownloadRequest>>();
    Ok(results)
  }
}

#[cfg(test)]
mod test {
  use crate::{
    db::{
      ConversationBuilder, DbService, DefaultTimeService, DownloadRequest, DownloadStatus,
      MessageBuilder, SqliteDbService, TimeService,
    },
    test_utils::db_service,
  };
  use chrono::{DateTime, Days, Timelike, Utc};
  use rstest::rstest;
  use tempfile::TempDir;
  use uuid::Uuid;

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_db_service_conversations_create(
    #[future] db_service: (TempDir, DateTime<Utc>, SqliteDbService),
  ) -> anyhow::Result<()> {
    let (_tempdir, now, service) = db_service;
    let created = chrono::Utc::now()
      .checked_sub_days(Days::new(1))
      .unwrap()
      .with_nanosecond(0)
      .unwrap();
    let mut conversation = ConversationBuilder::default()
      .id(Uuid::new_v4().to_string())
      .title("test chat")
      .created_at(created)
      .updated_at(created)
      .build()?;
    service.save_conversation(&mut conversation.clone()).await?;
    let convos = service.list_conversations().await?;
    assert_eq!(1, convos.len());
    conversation.updated_at = now;
    assert_eq!(&conversation, convos.first().unwrap());
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_db_service_conversations_update(
    #[future] db_service: (TempDir, DateTime<Utc>, SqliteDbService),
  ) -> anyhow::Result<()> {
    let (_tempdir, _now, service) = db_service;
    let created = chrono::Utc::now()
      .checked_sub_days(Days::new(1))
      .unwrap()
      .with_nanosecond(0)
      .unwrap();
    let mut conversation = ConversationBuilder::default()
      .id(Uuid::new_v4().to_string())
      .title("test chat")
      .created_at(created)
      .updated_at(created)
      .build()?;
    service.save_conversation(&mut conversation).await?;
    conversation.title = "new test chat".to_string();
    service.save_conversation(&mut conversation).await?;

    let convos = service.list_conversations().await?;
    assert_eq!(1, convos.len());
    assert_eq!(&conversation, convos.first().unwrap());
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_db_service_list_conversation(
    #[future] db_service: (TempDir, DateTime<Utc>, SqliteDbService),
  ) -> anyhow::Result<()> {
    let (_tempdir, _now, service) = db_service;
    service
      .save_conversation(&mut ConversationBuilder::default().build().unwrap())
      .await?;
    service
      .save_conversation(&mut ConversationBuilder::default().build().unwrap())
      .await?;
    let convos = service.list_conversations().await?;
    assert_eq!(2, convos.len());
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_db_service_save_message(
    #[future] db_service: (TempDir, DateTime<Utc>, SqliteDbService),
  ) -> anyhow::Result<()> {
    let (_tempdir, _now, service) = db_service;
    let mut conversation = ConversationBuilder::default()
      .title("test title")
      .build()
      .unwrap();
    service.save_conversation(&mut conversation).await?;
    let mut message = MessageBuilder::default()
      .id(Uuid::new_v4().to_string())
      .conversation_id(conversation.id.clone())
      .role("user")
      .content("test message")
      .build()
      .unwrap();
    service.save_message(&mut message).await?;
    let convos = service
      .get_conversation_with_messages(&conversation.id)
      .await?;
    assert_eq!(&message, convos.messages.first().unwrap());
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_db_service_delete_conversation(
    #[future] db_service: (TempDir, DateTime<Utc>, SqliteDbService),
  ) -> anyhow::Result<()> {
    let (_tempdir, _now, service) = db_service;
    let mut conversation = ConversationBuilder::default()
      .title("test title")
      .build()
      .unwrap();
    service.save_conversation(&mut conversation).await?;
    let mut message = MessageBuilder::default()
      .id(Uuid::new_v4().to_string())
      .conversation_id(conversation.id.clone())
      .role("user")
      .content("test message")
      .build()
      .unwrap();
    service.save_message(&mut message).await?;
    service.delete_conversations(&conversation.id).await?;
    let convos = service
      .get_conversation_with_messages(&conversation.id)
      .await;
    assert!(convos.is_err());
    assert_eq!(
      "sqlx_query: no rows returned by a query that expected to return at least one row\ntable: conversations",
      convos.unwrap_err().to_string()
    );
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_db_service_delete_all_conversation(
    #[future] db_service: (TempDir, DateTime<Utc>, SqliteDbService),
  ) -> anyhow::Result<()> {
    let (_tempdir, _now, service) = db_service;
    let mut conversation = ConversationBuilder::default().build().unwrap();
    service.save_conversation(&mut conversation).await?;
    let mut message = MessageBuilder::default()
      .id(Uuid::new_v4().to_string())
      .conversation_id(conversation.id.clone())
      .build()
      .unwrap();
    service.save_message(&mut message).await?;
    service.delete_all_conversations().await?;
    let convos = service.list_conversations().await?;
    assert!(convos.is_empty());
    Ok(())
  }

  #[test]
  fn test_time_service_utc_now() -> anyhow::Result<()> {
    let now = DefaultTimeService.utc_now();
    let now_chrono = chrono::Utc::now();
    assert!(now.timestamp() - now_chrono.timestamp() < 1);
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_db_service_create_download_request(
    #[future] db_service: (TempDir, DateTime<Utc>, SqliteDbService),
  ) -> anyhow::Result<()> {
    let (_tempdir, now, service) = db_service;
    let request = DownloadRequest {
      id: Uuid::new_v4().to_string(),
      repo: "test/repo".to_string(),
      filename: "test_file.gguf".to_string(),
      status: DownloadStatus::Pending,
      created_at: now,
      updated_at: now,
    };
    service.create_download_request(&request).await?;

    let fetched = service.get_download_request(&request.id).await?;
    assert!(fetched.is_some());
    assert_eq!(request, fetched.unwrap());
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_db_service_update_download_request(
    #[future] db_service: (TempDir, DateTime<Utc>, SqliteDbService),
  ) -> anyhow::Result<()> {
    let (_tempdir, now, service) = db_service;
    let mut request = DownloadRequest {
      id: Uuid::new_v4().to_string(),
      repo: "test/repo".to_string(),
      filename: "test_file.gguf".to_string(),
      status: DownloadStatus::Pending,
      created_at: now,
      updated_at: now,
    };
    service.create_download_request(&request).await?;

    request.status = DownloadStatus::Completed;
    request.updated_at = now + chrono::Duration::hours(1);
    service.update_download_request(&request).await?;

    let fetched = service.get_download_request(&request.id).await?.unwrap();
    assert_eq!(request, fetched);
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_db_service_list_pending_downloads(
    #[future] db_service: (TempDir, DateTime<Utc>, SqliteDbService),
  ) -> anyhow::Result<()> {
    let (_tempdir, now, service) = db_service;
    let pending_request = DownloadRequest {
      id: Uuid::new_v4().to_string(),
      repo: "test/repo1".to_string(),
      filename: "test_file1.gguf".to_string(),
      status: DownloadStatus::Pending,
      created_at: now,
      updated_at: now,
    };
    let completed_request = DownloadRequest {
      id: Uuid::new_v4().to_string(),
      repo: "test/repo2".to_string(),
      filename: "test_file2.gguf".to_string(),
      status: DownloadStatus::Completed,
      created_at: now,
      updated_at: now,
    };

    service.create_download_request(&pending_request).await?;
    service.create_download_request(&completed_request).await?;

    let pending_downloads = service.list_pending_downloads().await?;
    assert_eq!(1, pending_downloads.len());
    assert_eq!(pending_request, pending_downloads[0]);
    Ok(())
  }
}
