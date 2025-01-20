use chrono::{serde::ts_milliseconds, DateTime, Utc};
#[allow(unused_imports)]
use objs::{is_default, BuilderError};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use strum::EnumString;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize, FromRow, derive_builder::Builder,
)]
#[builder(default, setter(into, strip_option), build_fn(error = BuilderError))]
pub struct Conversation {
  #[serde(default)]
  pub id: String,
  pub title: String,
  #[serde(
    rename = "createdAt",
    with = "ts_milliseconds",
    default,
    skip_serializing_if = "is_default"
  )]
  pub created_at: DateTime<Utc>,
  #[serde(
    rename = "updatedAt",
    with = "ts_milliseconds",
    default,
    skip_serializing
  )]
  pub updated_at: DateTime<Utc>,
  pub messages: Vec<Message>,
}

#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize, FromRow, derive_builder::Builder,
)]
#[builder(default, setter(into, strip_option), build_fn(error = BuilderError))]
pub struct Message {
  #[serde(default, skip_serializing)]
  pub id: String,
  #[serde(default, skip_serializing)]
  pub conversation_id: String,
  pub role: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  pub content: Option<String>,
  #[serde(default, skip_serializing)]
  pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, EnumString, strum::Display, PartialEq, ToSchema)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum DownloadStatus {
  Pending,
  Completed,
  Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct DownloadRequest {
  pub id: String,
  pub repo: String,
  pub filename: String,
  pub status: DownloadStatus,
  pub error: Option<String>,
  #[schema(value_type = String, format = "date-time", example = "2024-11-10T04:52:06.786Z")]
  pub created_at: DateTime<Utc>,
  #[schema(value_type = String, format = "date-time", example = "2024-11-10T04:52:06.786Z")]
  pub updated_at: DateTime<Utc>,
}

impl DownloadRequest {
  pub fn new_pending(repo: String, filename: String) -> Self {
    DownloadRequest {
      id: Uuid::new_v4().to_string(),
      repo,
      filename,
      status: DownloadStatus::Pending,
      error: None,
      created_at: Utc::now(),
      updated_at: Utc::now(),
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AccessRequest {
  pub id: i64,
  pub email: String,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
  pub status: RequestStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, EnumString, strum::Display, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum RequestStatus {
  Pending,
  Approved,
  Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, EnumString, strum::Display, PartialEq, ToSchema)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum TokenStatus {
  Active,
  Inactive,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct ApiToken {
  pub id: String,
  pub user_id: String,
  pub name: String,
  pub token_id: String,
  pub token_hash: String,
  pub status: TokenStatus,
  #[schema(value_type = String, format = "date-time", example = "2024-11-10T04:52:06.786Z")]
  pub created_at: DateTime<Utc>,
  #[schema(value_type = String, format = "date-time", example = "2024-11-10T04:52:06.786Z")]
  pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod test {
  use crate::db::{Conversation, ConversationBuilder, Message, MessageBuilder};
  use chrono::{DateTime, Utc};
  use rstest::rstest;

  #[rstest]
  #[case(
    r#"{
  "id": "foobar",
  "title": "test title",
  "createdAt": 1704070800000,
  "messages": []
}"#,
  Conversation {
    id: "foobar".to_string(),
    title: "test title".to_string(),
    created_at: DateTime::<Utc>::from_timestamp_millis(1704070800000).unwrap(),
    updated_at: DateTime::<Utc>::default(),
    messages: vec![],
  })]
  #[case(
    r#"{
  "id": "foobar",
  "title": "test title",
  "createdAt": 1704070800000,
  "updatedAt": 1704070800000,
  "messages": [
    {
      "role": "user",
      "content": "What day comes after Monday?"
    }
  ]
}"#,
  Conversation {
    id: "foobar".to_string(),
    title: "test title".to_string(),
    created_at: DateTime::<Utc>::from_timestamp_millis(1704070800000).unwrap(),
    updated_at: DateTime::<Utc>::from_timestamp_millis(1704070800000).unwrap(),
    messages: vec![
      Message {
        id: "".to_string(), 
        conversation_id: "".to_string(), 
        role: "user".to_string(), 
        name: None,
        content: Some("What day comes after Monday?".to_string()), 
        created_at: DateTime::<Utc>::default(),
      }],
  })]
  fn test_db_objs_serialize(
    #[case] input: String,
    #[case] expected: Conversation,
  ) -> anyhow::Result<()> {
    let result: Conversation = serde_json::from_str(&input)?;
    assert_eq!(expected, result);
    Ok(())
  }

  #[rstest]
  #[case(Conversation::default(), r#"{"id":"","title":"","messages":[]}"#)]
  #[case(ConversationBuilder::default()
    .messages(
      vec![
        MessageBuilder::default()
          .role("user")
          .content("test content")
          .build()
          .unwrap()
      ])
    .build()
    .unwrap(),
    r#"{"id":"","title":"","messages":[{"role":"user","content":"test content"}]}"#)]
  fn test_db_objs_skip_serialize_if_default(
    #[case] obj: Conversation,
    #[case] expected: String,
  ) -> anyhow::Result<()> {
    let content = serde_json::to_string(&obj).unwrap();
    assert_eq!(expected, content);
    Ok(())
  }
}
