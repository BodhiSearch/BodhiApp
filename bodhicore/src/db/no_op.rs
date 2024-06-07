use super::{
  objs::{Conversation, Message},
  service::CONVERSATIONS,
  DbError, DbServiceFn,
};

#[derive(Debug, PartialEq)]
pub(super) struct NoOpDbService {}

impl NoOpDbService {
  pub(super) fn new() -> Self {
    NoOpDbService {}
  }
}

#[async_trait::async_trait]
impl DbServiceFn for NoOpDbService {
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
}

#[cfg(test)]
mod test {
  use super::{
    super::{
      objs::{Conversation, Message},
      DbServiceFn,
    },
    NoOpDbService,
  };

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
