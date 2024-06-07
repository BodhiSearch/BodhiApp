use chrono::{DateTime, Timelike, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
#[allow(unused_imports)]
use crate::objs::BuilderError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
#[cfg_attr(test, derive(derive_builder::Builder))]
#[cfg_attr(test,
  builder(
    default,
    setter(into, strip_option),
    build_fn(error = BuilderError)))]
pub struct Conversation {
  pub id: String,
  pub title: String,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
  pub messages: Vec<Message>
}

impl Default for Conversation {
    fn default() -> Self {
      let now = chrono::Utc::now();
      let now = now.with_nanosecond(0).unwrap_or(now);
      Self {
          id: Uuid::new_v4().to_string(),
          title: Default::default(),
          created_at: now,
          updated_at: now,
          messages: vec![]
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
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
  pub id: String,
  pub conversation_id: String,
  pub role: String,
  pub name: Option<String>,
  pub content: Option<String>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

impl Default for Message {
  fn default() -> Self {
    let now = chrono::Utc::now();
    let now = now.with_nanosecond(0).unwrap_or(now);
    Self {
      id: Uuid::new_v4().to_string(),
      conversation_id: Default::default(),
      role: Default::default(),
      name: Default::default(),
      content: Default::default(),
      created_at: now,
      updated_at: now
    }
  }
}