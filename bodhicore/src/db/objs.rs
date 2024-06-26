use crate::objs::is_default;
#[allow(unused_imports)]
use crate::objs::BuilderError;
use chrono::{serde::ts_milliseconds, DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, FromRow)]
#[cfg_attr(test, derive(derive_builder::Builder))]
#[cfg_attr(test,
  builder(
    default,
    setter(into, strip_option),
    build_fn(error = BuilderError)))]
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

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, FromRow)]
#[cfg_attr(test, derive(derive_builder::Builder))]
#[cfg_attr(
  test,
  builder(
    default,
    setter(into, strip_option),
    build_fn(error = BuilderError)
  )
)]
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

#[cfg(test)]
mod test {
  use super::{Conversation, Message, ConversationBuilder, MessageBuilder};
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
  fn test_db_objs_skip_serialize_if_default(#[case] obj: Conversation, #[case] expected: String) -> anyhow::Result<()> {
    let content = serde_json::to_string(&obj).unwrap();
    assert_eq!(expected, content);
    Ok(())
  }
}
