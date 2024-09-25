use crate::db::{
  objs::{Conversation, DownloadRequest, Message},
  service::CONVERSATIONS,
  DbError, DbService,
};

use super::{AccessRequest, RequestStatus};

#[derive(Debug, PartialEq)]
pub struct NoOpDbService {}

impl NoOpDbService {
  pub(super) fn new() -> Self {
    NoOpDbService {}
  }
}

#[async_trait::async_trait]
impl DbService for NoOpDbService {
  async fn migrate(&self) -> Result<(), DbError> {
    Ok(())
  }

  async fn save_conversation(&self, _conversation: &mut Conversation) -> Result<(), DbError> {
    Ok(())
  }

  async fn save_message(&self, _message: &mut Message) -> Result<(), DbError> {
    Ok(())
  }

  async fn list_conversations(&self) -> Result<Vec<Conversation>, DbError> {
    Ok(vec![])
  }

  async fn delete_conversations(&self, _id: &str) -> Result<(), DbError> {
    Err(DbError::Sqlx {
      source: sqlx::Error::RowNotFound,
      table: CONVERSATIONS.to_string(),
    })
  }

  async fn delete_all_conversations(&self) -> Result<(), DbError> {
    Ok(())
  }

  async fn get_conversation_with_messages(&self, _id: &str) -> Result<Conversation, DbError> {
    Err(DbError::Sqlx {
      source: sqlx::Error::RowNotFound,
      table: CONVERSATIONS.to_string(),
    })
  }

  async fn create_download_request(&self, _request: &DownloadRequest) -> Result<(), DbError> {
    Ok(())
  }

  async fn get_download_request(&self, _id: &str) -> Result<Option<DownloadRequest>, DbError> {
    Ok(None)
  }

  async fn update_download_request(&self, _request: &DownloadRequest) -> Result<(), DbError> {
    Ok(())
  }

  async fn list_pending_downloads(&self) -> Result<Vec<DownloadRequest>, DbError> {
    Ok(vec![])
  }

  async fn insert_pending_request(&self, _email: String) -> Result<AccessRequest, DbError> {
    todo!()
  }

  async fn get_pending_request(&self, _email: String) -> Result<Option<AccessRequest>, DbError> {
    Ok(None)
  }

  async fn list_pending_requests(
    &self,
    _page: u32,
    _per_page: u32,
  ) -> Result<Vec<AccessRequest>, DbError> {
    Ok(vec![])
  }

  async fn update_request_status(&self, _id: i64, _status: RequestStatus) -> Result<(), DbError> {
    Ok(())
  }
}

#[cfg(test)]
mod test {
  use crate::db::{Conversation, DbService, Message, NoOpDbService};

  #[tokio::test]
  async fn test_no_op_save() -> anyhow::Result<()> {
    NoOpDbService::new()
      .save_conversation(&mut Conversation::default())
      .await?;
    Ok(())
  }

  #[tokio::test]
  async fn test_no_op_save_message() -> anyhow::Result<()> {
    NoOpDbService::new()
      .save_message(&mut Message::default())
      .await?;
    Ok(())
  }

  #[tokio::test]
  async fn test_no_op_list_convos() -> anyhow::Result<()> {
    let convos = NoOpDbService::new().list_conversations().await?;
    assert!(convos.is_empty());
    Ok(())
  }

  #[tokio::test]
  async fn test_no_op_delete_convos() -> anyhow::Result<()> {
    let result = NoOpDbService::new().delete_conversations("testid").await;
    assert!(result.is_err());
    assert_eq!("sqlx_query: no rows returned by a query that expected to return at least one row\ntable: conversations", result.unwrap_err().to_string());
    Ok(())
  }

  #[tokio::test]
  async fn test_no_op_delete_all() -> anyhow::Result<()> {
    NoOpDbService::new().delete_all_conversations().await?;
    Ok(())
  }

  #[tokio::test]
  async fn test_no_op_get_convo() -> anyhow::Result<()> {
    let result = NoOpDbService::new()
      .get_conversation_with_messages("testid")
      .await;
    assert!(result.is_err());
    assert_eq!("sqlx_query: no rows returned by a query that expected to return at least one row\ntable: conversations", result.unwrap_err().to_string());
    Ok(())
  }
}
