use crate::{
  db::{
    AccessRequest, ApiToken, Conversation, DownloadRequest, DownloadStatus, Message, RequestStatus,
    SqlxError, SqlxMigrateError, TokenStatus,
  },
  decode_access_token,
};
use chrono::{DateTime, Timelike, Utc};
use derive_new::new;
use objs::{impl_error_from, AppError, ErrorType};
use sqlx::{query_as, SqlitePool};
use std::{fs, path::Path, str::FromStr, sync::Arc, time::UNIX_EPOCH};
use uuid::Uuid;

pub static CONVERSATIONS: &str = "conversations";
pub static MESSAGES: &str = "messages";

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait TimeService: std::fmt::Debug + Send + Sync {
  fn utc_now(&self) -> DateTime<Utc>;

  fn created_at(&self, path: &Path) -> u32;
}

#[derive(Debug, Clone, Default)]
pub struct DefaultTimeService;

impl TimeService for DefaultTimeService {
  fn utc_now(&self) -> DateTime<Utc> {
    let now = chrono::Utc::now();
    now.with_nanosecond(0).unwrap_or(now)
  }

  fn created_at(&self, path: &Path) -> u32 {
    fs::metadata(path)
      .map_err(|e| e.to_string())
      .and_then(|m| m.created().map_err(|e| e.to_string()))
      .and_then(|t| t.duration_since(UNIX_EPOCH).map_err(|e| e.to_string()))
      .unwrap_or_default()
      .as_secs() as u32
  }
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum DbError {
  #[error(transparent)]
  SqlxError(#[from] SqlxError),
  #[error(transparent)]
  SqlxMigrateError(#[from] SqlxMigrateError),
  #[error(transparent)]
  #[error_meta(error_type = ErrorType::BadRequest, code="db_error-strum_parse", args_delegate = false)]
  StrumParse(#[from] strum::ParseError),
  #[error("token_validation")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  TokenValidation(String),
}

impl_error_from!(::sqlx::Error, DbError::SqlxError, crate::db::SqlxError);
impl_error_from!(
  ::sqlx::migrate::MigrateError,
  DbError::SqlxMigrateError,
  crate::db::SqlxMigrateError
);

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

  async fn list_all_downloads(&self) -> Result<Vec<DownloadRequest>, DbError>;

  async fn insert_pending_request(&self, email: String) -> Result<AccessRequest, DbError>;

  async fn get_pending_request(&self, email: String) -> Result<Option<AccessRequest>, DbError>;

  async fn list_pending_requests(
    &self,
    page: u32,
    per_page: u32,
  ) -> Result<Vec<AccessRequest>, DbError>;

  async fn update_request_status(&self, id: i64, status: RequestStatus) -> Result<(), DbError>;

  async fn create_api_token(&self, token: &mut ApiToken) -> Result<(), DbError>;

  async fn create_api_token_from(&self, name: &str, token: &str) -> Result<ApiToken, DbError>;

  async fn list_api_tokens(
    &self,
    page: u32,
    per_page: u32,
  ) -> Result<(Vec<ApiToken>, u32), DbError>;

  async fn get_api_token(&self, id: &str) -> Result<Option<ApiToken>, DbError>;

  async fn get_valid_api_token(&self, token: &str) -> Result<Option<ApiToken>, DbError>;

  async fn update_api_token(&self, token: &mut ApiToken) -> Result<(), DbError>;
}

#[derive(Debug, Clone, new)]
pub struct SqliteDbService {
  pool: SqlitePool,
  time_service: Arc<dyn TimeService>,
}

impl SqliteDbService {
  async fn get_by_col(&self, query: &str, id: &str) -> Result<Option<ApiToken>, DbError> {
    let result = query_as::<
      _,
      (
        String,
        String,
        String,
        String,
        String,
        String,
        DateTime<Utc>,
        DateTime<Utc>,
      ),
    >(query)
    .bind(id)
    .fetch_optional(&self.pool)
    .await?;

    match result {
      Some((id, user_id, name, token_id, token_hash, status, created_at, updated_at)) => {
        let Ok(status) = TokenStatus::from_str(&status) else {
          tracing::warn!("unknown token status: {status} for id: {id}");
          return Ok(None);
        };

        let result = ApiToken {
          id,
          user_id,
          name,
          token_id,
          token_hash,
          status,
          created_at,
          updated_at,
        };
        Ok(Some(result))
      }
      None => Ok(None),
    }
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
    .await?;
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
    .await?;
    Ok(())
  }

  async fn list_conversations(&self) -> Result<Vec<Conversation>, DbError> {
    let conversations = query_as::<_, (String, String, i64, i64)>(
      "SELECT id, title, created_at, updated_at FROM conversations ORDER BY created_at DESC",
    )
    .fetch_all(&self.pool)
    .await?;

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
    let messages = query_as::<_, Message>(
      "SELECT id, conversation_id, role, name, content, created_at FROM messages WHERE conversation_id = ?"
    )
    .bind(id)
    .fetch_all(&self.pool)
    .await?;

    let row = query_as::<_, (String, String, i64, i64)>(
      "SELECT id, title, created_at, updated_at FROM conversations WHERE id = ?",
    )
    .bind(id)
    .fetch_one(&self.pool)
    .await?;

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
      .await?;
    sqlx::query("DELETE FROM conversations where id=?")
      .bind(id)
      .execute(&self.pool)
      .await?;
    Ok(())
  }

  async fn delete_all_conversations(&self) -> Result<(), DbError> {
    sqlx::query("DELETE FROM messages")
      .execute(&self.pool)
      .await?;
    sqlx::query("DELETE FROM conversations")
      .execute(&self.pool)
      .await?;
    Ok(())
  }

  async fn create_download_request(&self, request: &DownloadRequest) -> Result<(), DbError> {
    sqlx::query(
      "INSERT INTO download_requests (id, repo, filename, status, error, created_at, updated_at)
       VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&request.id)
    .bind(&request.repo)
    .bind(&request.filename)
    .bind(request.status.to_string())
    .bind(&request.error)
    .bind(request.created_at.timestamp())
    .bind(request.updated_at.timestamp())
    .execute(&self.pool)
    .await?;
    Ok(())
  }

  async fn get_download_request(&self, id: &str) -> Result<Option<DownloadRequest>, DbError> {
    let result = query_as::<_, (String, String, String, String, Option<String>, i64, i64)>(
      "SELECT id, repo, filename, status, error, created_at, updated_at FROM download_requests WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&self.pool)
    .await?;

    match result {
      Some((id, repo, filename, status, error, created_at, updated_at)) => {
        let Ok(status) = DownloadStatus::from_str(&status) else {
          tracing::warn!("unknown download status: {status} for id: {id}");
          return Ok(None);
        };

        Ok(Some(DownloadRequest {
          id,
          repo,
          filename,
          status,
          error,
          created_at: chrono::DateTime::<Utc>::from_timestamp(created_at, 0).unwrap_or_default(),
          updated_at: chrono::DateTime::<Utc>::from_timestamp(updated_at, 0).unwrap_or_default(),
        }))
      }
      None => Ok(None),
    }
  }

  async fn update_download_request(&self, request: &DownloadRequest) -> Result<(), DbError> {
    sqlx::query("UPDATE download_requests SET status = ?, error = ?, updated_at = ? WHERE id = ?")
      .bind(request.status.to_string())
      .bind(&request.error)
      .bind(request.updated_at.timestamp())
      .bind(&request.id)
      .execute(&self.pool)
      .await?;
    Ok(())
  }

  async fn list_all_downloads(&self) -> Result<Vec<DownloadRequest>, DbError> {
    let results = query_as::<_, (String, String, String, String, Option<String>, i64, i64)>(
      "SELECT id, repo, filename, status, error, created_at, updated_at FROM download_requests ORDER BY updated_at DESC",
    )
    .fetch_all(&self.pool)
    .await?;

    let results = results
      .into_iter()
      .filter_map(
        |(id, repo, filename, status, error, created_at, updated_at)| {
          let status = DownloadStatus::from_str(&status).ok()?;
          Some(DownloadRequest {
            id,
            repo,
            filename,
            status,
            error,
            created_at: chrono::DateTime::<Utc>::from_timestamp(created_at, 0).unwrap_or_default(),
            updated_at: chrono::DateTime::<Utc>::from_timestamp(updated_at, 0).unwrap_or_default(),
          })
        },
      )
      .collect::<Vec<DownloadRequest>>();
    Ok(results)
  }

  async fn insert_pending_request(&self, email: String) -> Result<AccessRequest, DbError> {
    let now = self.time_service.utc_now();
    let result = query_as::<_, (i64, String, DateTime<Utc>, DateTime<Utc>, String)>(
      "INSERT INTO access_requests (email, created_at, updated_at, status)
         VALUES (?, ?, ?, ?)
         RETURNING id, email, created_at, updated_at, status",
    )
    .bind(&email)
    .bind(now)
    .bind(now)
    .bind(RequestStatus::Pending.to_string())
    .fetch_one(&self.pool)
    .await?;

    Ok(AccessRequest {
      id: result.0,
      email: result.1,
      created_at: result.2,
      updated_at: result.3,
      status: RequestStatus::from_str(&result.4)?,
    })
  }

  async fn get_pending_request(&self, email: String) -> Result<Option<AccessRequest>, DbError> {
    let result = query_as::<_, (i64, String, DateTime<Utc>, DateTime<Utc>, String)>(
      "SELECT id, email, created_at, updated_at, status
         FROM access_requests
         WHERE email = ? AND status = ?",
    )
    .bind(&email)
    .bind(RequestStatus::Pending.to_string())
    .fetch_optional(&self.pool)
    .await?;

    let result = result
      .map(|(id, email, created_at, updated_at, status)| {
        let Ok(status) = RequestStatus::from_str(&status) else {
          tracing::warn!("unknown request status: {} for id: {}", status, id);
          return None;
        };
        let result = AccessRequest {
          id,
          email,
          created_at,
          updated_at,
          status,
        };
        Some(result)
      })
      .unwrap_or(None);
    Ok(result)
  }

  async fn list_pending_requests(
    &self,
    page: u32,
    per_page: u32,
  ) -> Result<Vec<AccessRequest>, DbError> {
    let offset = (page - 1) * per_page;
    let results = query_as::<_, (i64, String, DateTime<Utc>, DateTime<Utc>, String)>(
      "SELECT id, email, created_at, updated_at, status
         FROM access_requests
         WHERE status = ?
         ORDER BY created_at ASC
         LIMIT ? OFFSET ?",
    )
    .bind(RequestStatus::Pending.to_string())
    .bind(per_page as i64)
    .bind(offset as i64)
    .fetch_all(&self.pool)
    .await?;

    let results = results
      .into_iter()
      .filter_map(|(id, email, created_at, updated_at, status)| {
        let Ok(status) = RequestStatus::from_str(&status) else {
          tracing::warn!("unknown request status: {} for id: {}", status, id);
          return None;
        };
        let result = AccessRequest {
          id,
          email,
          created_at,
          updated_at,
          status,
        };
        Some(result)
      })
      .collect::<Vec<AccessRequest>>();
    Ok(results)
  }

  async fn update_request_status(&self, id: i64, status: RequestStatus) -> Result<(), DbError> {
    let now = self.time_service.utc_now();
    sqlx::query(
      "UPDATE access_requests
         SET status = ?, updated_at = ?
         WHERE id = ?",
    )
    .bind(status.to_string())
    .bind(now)
    .bind(id)
    .execute(&self.pool)
    .await?;
    Ok(())
  }

  async fn create_api_token(&self, token: &mut ApiToken) -> Result<(), DbError> {
    let now = self.time_service.utc_now();
    token.created_at = now;
    token.updated_at = now;

    sqlx::query(
      r#"
      INSERT INTO api_tokens (id, user_id, name, token_id, token_hash, status, created_at, updated_at)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?)
      "#,
    )
    .bind(&token.id)
    .bind(&token.user_id)
    .bind(&token.name)
    .bind(&token.token_id)
    .bind(&token.token_hash)
    .bind(token.status.to_string())
    .bind(token.created_at.timestamp())
    .bind(token.updated_at.timestamp())
    .execute(&self.pool)
    .await?;

    Ok(())
  }

  async fn create_api_token_from(&self, name: &str, token: &str) -> Result<ApiToken, DbError> {
    use crate::MinClaims;
    use sha2::{Digest, Sha256};

    let token_data = decode_access_token::<MinClaims>(token)
      .map_err(|e| DbError::TokenValidation(e.to_string()))?;

    let token_hash = format!("{:x}", Sha256::digest(token.as_bytes()));
    let token_hash = token_hash[..12].to_string();

    let now = self.time_service.utc_now();
    let id = Uuid::new_v4().to_string();

    let api_token = ApiToken {
      id,
      user_id: token_data.claims.sub,
      name: name.to_string(),
      token_id: token_data.claims.jti,
      token_hash,
      status: TokenStatus::Active,
      created_at: now,
      updated_at: now,
    };

    sqlx::query(
      r#"
      INSERT INTO api_tokens (id, user_id, name, token_id, token_hash, status, created_at, updated_at)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?)
      "#,
    )
    .bind(&api_token.id)
    .bind(&api_token.user_id)
    .bind(&api_token.name)
    .bind(&api_token.token_id)
    .bind(&api_token.token_hash)
    .bind(api_token.status.to_string())
    .bind(api_token.created_at.timestamp())
    .bind(api_token.updated_at.timestamp())
    .execute(&self.pool)
    .await?;

    Ok(api_token)
  }

  async fn list_api_tokens(
    &self,
    page: u32,
    per_page: u32,
  ) -> Result<(Vec<ApiToken>, u32), DbError> {
    let offset = (page - 1) * per_page;
    let results = query_as::<
      _,
      (
        String,
        String,
        String,
        String,
        String,
        String,
        DateTime<Utc>,
        DateTime<Utc>,
      ),
    >(
      r#"
      SELECT
        id,
        user_id,
        name,
        token_id,
        token_hash,
        status,
        created_at,
        updated_at
      FROM api_tokens
      ORDER BY created_at DESC
      LIMIT ? OFFSET ?
      "#,
    )
    .bind(per_page as i64)
    .bind(offset as i64)
    .fetch_all(&self.pool)
    .await?;

    let tokens: Vec<_> = results
      .into_iter()
      .filter_map(
        |(id, user_id, name, token_id, token_hash, status, created_at, updated_at)| {
          let Ok(status) = TokenStatus::from_str(&status) else {
            tracing::warn!("unknown token status: {} for id: {}", status, id);
            return None;
          };

          Some(ApiToken {
            id,
            user_id,
            name,
            token_id,
            token_hash,
            status,
            created_at,
            updated_at,
          })
        },
      )
      .collect();

    let total = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM api_tokens")
      .fetch_one(&self.pool)
      .await?;
    Ok((tokens, total as u32))
  }

  async fn get_api_token(&self, id: &str) -> Result<Option<ApiToken>, DbError> {
    let query = r#"
      SELECT
        id,
        user_id,
        name,
        token_id,
        token_hash,
        status,
        created_at,
        updated_at
      FROM api_tokens
      WHERE id = ?
      "#;
    self.get_by_col(query, id).await
  }

  async fn get_valid_api_token(&self, token: &str) -> Result<Option<ApiToken>, DbError> {
    use crate::MinClaims;
    use sha2::{Digest, Sha256};
    let token_data = decode_access_token::<MinClaims>(token)
      .map_err(|e| DbError::TokenValidation(e.to_string()))?;
    let query = r#"
      SELECT
        id,
        user_id,
        name,
        token_id,
        token_hash,
        status,
        created_at,
        updated_at
      FROM api_tokens
      WHERE token_id = ?
      "#;
    let api_token = self.get_by_col(query, &token_data.claims.jti).await?;
    match api_token {
      None => Ok(None),
      Some(api_token) => {
        let token_hash = format!("{:x}", Sha256::digest(token.as_bytes()));
        let token_hash = token_hash[..12].to_string();
        if api_token.token_hash == token_hash {
          Ok(Some(api_token))
        } else {
          Ok(None)
        }
      }
    }
  }

  async fn update_api_token(&self, token: &mut ApiToken) -> Result<(), DbError> {
    token.updated_at = self.time_service.utc_now();
    sqlx::query(
      r#"
      UPDATE api_tokens
      SET name = ?,
          status = ?,
          updated_at = CURRENT_TIMESTAMP
      WHERE id = ?
      "#,
    )
    .bind(&token.name)
    .bind(token.status.to_string())
    .bind(&token.id)
    .execute(&self.pool)
    .await?;

    Ok(())
  }
}

#[cfg(test)]
mod test {
  use crate::{
    db::{
      AccessRequest, ApiToken, ConversationBuilder, DbError, DbService, DownloadRequest,
      DownloadStatus, MessageBuilder, RequestStatus, SqlxError, TokenStatus,
    },
    test_utils::{build_token, test_db_service, TestDbService},
  };
  use chrono::{Days, Timelike, Utc};
  use rstest::rstest;
  use uuid::Uuid;

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_db_service_conversations_create(
    #[future]
    #[from(test_db_service)]
    service: TestDbService,
  ) -> anyhow::Result<()> {
    let now = service.now();
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
    #[future]
    #[from(test_db_service)]
    service: TestDbService,
  ) -> anyhow::Result<()> {
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
    #[future]
    #[from(test_db_service)]
    service: TestDbService,
  ) -> anyhow::Result<()> {
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
    #[future]
    #[from(test_db_service)]
    service: TestDbService,
  ) -> anyhow::Result<()> {
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
    #[future]
    #[from(test_db_service)]
    service: TestDbService,
  ) -> anyhow::Result<()> {
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
    assert!(matches!(
      convos.unwrap_err(),
      DbError::SqlxError(SqlxError { source }) if source.to_string() == sqlx::Error::RowNotFound.to_string()
    ));
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_db_service_delete_all_conversation(
    #[future]
    #[from(test_db_service)]
    service: TestDbService,
  ) -> anyhow::Result<()> {
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

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_db_service_create_download_request(
    #[future]
    #[from(test_db_service)]
    service: TestDbService,
  ) -> anyhow::Result<()> {
    let now = service.now();
    let request = DownloadRequest {
      id: Uuid::new_v4().to_string(),
      repo: "test/repo".to_string(),
      filename: "test_file.gguf".to_string(),
      status: DownloadStatus::Pending,
      error: None,
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
    #[future]
    #[from(test_db_service)]
    service: TestDbService,
  ) -> anyhow::Result<()> {
    let now = service.now();
    let mut request = DownloadRequest {
      id: Uuid::new_v4().to_string(),
      repo: "test/repo".to_string(),
      filename: "test_file.gguf".to_string(),
      status: DownloadStatus::Pending,
      error: None,
      created_at: now,
      updated_at: now,
    };
    service.create_download_request(&request).await?;

    request.status = DownloadStatus::Completed;
    request.updated_at = now + chrono::Duration::hours(1);
    service.update_download_request(&request).await?;

    let fetched = service.get_download_request(&request.id).await?.unwrap();
    assert_eq!(
      DownloadRequest {
        id: request.id,
        repo: "test/repo".to_string(),
        filename: "test_file.gguf".to_string(),
        status: DownloadStatus::Completed,
        error: None,
        created_at: now,
        updated_at: now + chrono::Duration::hours(1),
      },
      fetched
    );
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_db_service_insert_pending_request(
    #[future]
    #[from(test_db_service)]
    service: TestDbService,
  ) -> anyhow::Result<()> {
    let now = service.now();
    let email = "test@example.com".to_string();
    let pending_request = service.insert_pending_request(email.clone()).await?;
    let expected_request = AccessRequest {
      id: pending_request.id, // We don't know this in advance
      email,
      created_at: now,
      updated_at: now,
      status: RequestStatus::Pending,
    };
    assert_eq!(pending_request, expected_request);
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_db_service_get_pending_request(
    #[future]
    #[from(test_db_service)]
    service: TestDbService,
  ) -> anyhow::Result<()> {
    let email = "test@example.com".to_string();
    let inserted_request = service.insert_pending_request(email.clone()).await?;
    let fetched_request = service.get_pending_request(email).await?;
    assert!(fetched_request.is_some());
    assert_eq!(fetched_request.unwrap(), inserted_request);
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_db_service_list_pending_requests(
    #[future]
    #[from(test_db_service)]
    service: TestDbService,
  ) -> anyhow::Result<()> {
    let now = service.now();
    let emails = vec![
      "test1@example.com".to_string(),
      "test2@example.com".to_string(),
      "test3@example.com".to_string(),
    ];
    for email in &emails {
      service.insert_pending_request(email.clone()).await?;
    }
    let page1 = service.list_pending_requests(1, 2).await?;
    assert_eq!(2, page1.len());
    let page2 = service.list_pending_requests(2, 2).await?;
    assert_eq!(1, page2.len());
    for (i, request) in page1.iter().chain(page2.iter()).enumerate() {
      let expected_request = AccessRequest {
        id: request.id,
        email: emails[i].clone(),
        created_at: now,
        updated_at: now,
        status: RequestStatus::Pending,
      };
      assert_eq!(request, &expected_request);
    }
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_db_service_update_request_status(
    #[future]
    #[from(test_db_service)]
    service: TestDbService,
  ) -> anyhow::Result<()> {
    let email = "test@example.com".to_string();
    let inserted_request = service.insert_pending_request(email.clone()).await?;
    service
      .update_request_status(inserted_request.id, RequestStatus::Approved)
      .await?;
    let updated_request = service.get_pending_request(email).await?;
    assert!(updated_request.is_none());
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_create_api_token(
    #[future]
    #[from(test_db_service)]
    service: TestDbService,
  ) -> anyhow::Result<()> {
    let now = service.now();

    // Create token
    let mut token = ApiToken {
      id: Uuid::new_v4().to_string(),
      user_id: Uuid::new_v4().to_string(),
      name: "".to_string(),
      token_id: Uuid::new_v4().to_string(),
      token_hash: "token_hash".to_string(),
      status: TokenStatus::Active,
      created_at: now,
      updated_at: now,
    };

    service.create_api_token(&mut token).await?;

    // List tokens
    let (tokens, _) = service.list_api_tokens(1, 10).await?;
    assert_eq!(1, tokens.len());

    assert_eq!(token, tokens[0]);

    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_update_api_token(
    #[future]
    #[from(test_db_service)]
    service: TestDbService,
  ) -> anyhow::Result<()> {
    // Create initial token
    let mut token = ApiToken {
      id: Uuid::new_v4().to_string(),
      user_id: "test_user".to_string(),
      name: "Initial Name".to_string(),
      token_id: "token123".to_string(),
      token_hash: "token_hash".to_string(),
      status: TokenStatus::Active,
      created_at: Utc::now(),
      updated_at: Utc::now(),
    };
    service.create_api_token(&mut token).await?;

    // Update token
    token.name = "Updated Name".to_string();
    token.status = TokenStatus::Inactive;
    token.updated_at = Utc::now();
    service.update_api_token(&mut token).await?;
    // Verify update
    let updated = service.get_api_token(&token.id).await?.unwrap();
    assert_eq!(updated.name, "Updated Name");
    assert_eq!(updated.status, TokenStatus::Inactive);
    assert_eq!(updated.id, token.id);
    assert_eq!(updated.user_id, token.user_id);
    assert_eq!(updated.token_id, token.token_id);
    assert_eq!(updated.created_at, token.created_at);
    assert!(updated.updated_at >= token.updated_at);

    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_create_api_token_from(
    #[future]
    #[from(test_db_service)]
    db_service: TestDbService,
  ) -> anyhow::Result<()> {
    // Create a test token with known claims
    let test_jti = Uuid::new_v4().to_string();
    let test_sub = Uuid::new_v4().to_string();
    let (token, _) = build_token(serde_json::json!({
      "jti": test_jti,
      "sub": test_sub,
    }))?;

    // Create API token
    let name = "Test Token";
    let api_token = db_service.create_api_token_from(name, &token).await?;

    // Verify the created token
    assert_eq!(api_token.name, name);
    assert_eq!(api_token.token_id, test_jti);
    assert_eq!(api_token.user_id, test_sub);
    assert_eq!(api_token.status, TokenStatus::Active);

    // Verify we can retrieve it
    let retrieved = db_service.get_api_token(&api_token.id).await?;
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.token_id, test_jti);
    assert_eq!(retrieved.user_id, test_sub);
    assert_eq!(retrieved.token_hash, api_token.token_hash);

    Ok(())
  }
}
