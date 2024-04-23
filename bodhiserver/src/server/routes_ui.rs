use super::utils;
use super::utils::get_chats_dir;
use super::utils::ApiError;
use super::utils::HomeDirError;
use super::utils::ROLE_ASSISTANT;
use super::utils::ROLE_USER;
use async_openai::types::ChatCompletionRequestMessage;
use async_openai::types::CreateChatCompletionRequest;
use axum::body::Body;
use axum::extract::Query;
use axum::http::Response;
use axum::response::IntoResponse;
use axum::response::Json;
use chrono::serde::ts_milliseconds;
use chrono::Utc;
use regex::Regex;
use serde::Deserialize;
use serde::Serialize;
use std::cmp::min;
use std::fs;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;
use thiserror::Error;
use tokio::sync::mpsc::UnboundedReceiver;

#[derive(Error, Debug)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum ChatError {
  #[error(transparent)]
  HomeDirError(#[from] utils::HomeDirError),
  #[error("Failed to find the chat with given id: '{0}'")]
  ChatNotFoundErr(String),
  #[error(transparent)]
  IOError(#[from] io::Error),
  #[error(transparent)]
  JsonError(#[from] serde_json::Error),
  #[error("Last chat by user not found in messages")]
  LastChatNotFoundErr,
}

impl PartialEq for ChatError {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (ChatError::HomeDirError(e1), ChatError::HomeDirError(e2)) => e1.eq(e2),
      (ChatError::ChatNotFoundErr(id1), ChatError::ChatNotFoundErr(id2)) => id1.eq(id2),
      (ChatError::IOError(e1), ChatError::IOError(e2)) => e1.kind() == e2.kind(),
      (ChatError::JsonError(e1), ChatError::JsonError(e2)) => {
        format!("{}", e1) == format!("{}", e2)
      }
      _ => false,
    }
  }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ChatPreview {
  pub id: String,
  pub title: String,
  #[serde(rename = "createdAt", with = "ts_milliseconds")]
  pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Message {
  pub role: String,
  pub content: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Chat {
  pub id: String,
  pub title: String,
  pub messages: Vec<Message>,
  #[serde(rename = "createdAt", with = "ts_milliseconds")]
  pub created_at: chrono::DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct WithId {
  id: Option<String>,
}

pub(crate) async fn ui_chats_handler(
  Query(WithId { id }): Query<WithId>,
) -> Result<Response<Body>, ApiError> {
  match id {
    Some(id) => {
      let chat = _ui_chat_handler(get_chats_dir()?, &id)?;
      Ok(Json(chat).into_response())
    }
    None => {
      let chats = _ui_chats_handler(get_chats_dir()?)?;
      Ok(Json(chats).into_response())
    }
  }
}

pub(crate) async fn ui_chats_delete_handler(
  Query(WithId { id }): Query<WithId>,
) -> Result<Response<Body>, ApiError> {
  match id {
    Some(id) => {
      let chat = _ui_chat_handler(get_chats_dir()?, &id)?;
      Ok(Json(chat).into_response())
    }
    None => {
      _ui_chats_delete_handler(get_chats_dir()?)?;
      Ok(Json(()).into_response())
    }
  }
}

fn _ui_chats_handler(chats_dir: PathBuf) -> Result<Vec<ChatPreview>, ChatError> {
  let mut files: Vec<_> = fs::read_dir(chats_dir)
    .map_err(|_| HomeDirError::ReadDirErr)?
    .filter_map(|entry| {
      let entry = entry.ok()?;
      let path = entry.path();
      if path.is_file() && path.extension().unwrap_or_default() == "json" {
        Some(path)
      } else {
        None
      }
    })
    .collect();
  files.sort_by(|a, b| b.cmp(a));
  let chats = Vec::with_capacity(files.len());
  let chats = files
    .into_iter()
    .filter_map(|file| {
      let content = fs::read_to_string(file).ok()?;
      serde_json::from_str::<ChatPreview>(&content).ok()
    })
    .fold(chats, |mut chats, chat| {
      chats.push(chat);
      chats
    });
  Ok(chats)
}

fn _ui_chats_delete_handler(chats_dir: PathBuf) -> Result<(), ChatError> {
  remove_dir_contents(&chats_dir)?;
  Ok(())
}

fn _ui_chat_handler(chats_dir: PathBuf, id: &str) -> Result<Chat, ChatError> {
  let file =
    find_file_by_id(&chats_dir, id).ok_or_else(|| ChatError::ChatNotFoundErr(id.to_string()))?;
  let file = File::open(file)?;
  let chat: Chat = serde_json::from_reader(BufReader::new(file))?;
  Ok(chat)
}

fn _ui_chat_delete_handler(chats_dir: PathBuf, id: &str) -> Result<(), ChatError> {
  let file =
    find_file_by_id(&chats_dir, id).ok_or_else(|| ChatError::ChatNotFoundErr(id.to_string()))?;
  if file.exists() {
    fs::remove_file(&file)?;
    Ok(())
  } else {
    Err(ChatError::ChatNotFoundErr(id.to_string()))
  }
}

pub(crate) async fn save_stream(
  id: Option<String>,
  request: CreateChatCompletionRequest,
  mut rx: UnboundedReceiver<String>,
) -> Result<(), ChatError> {
  if id.is_none() {
    return Ok(());
  }
  let id = id.unwrap();
  let mut deltas = String::new();
  while let Some(message) = rx.recv().await {
    deltas.push_str(&message);
  }
  let chats_dir = get_chats_dir()?;
  save_chat(&chats_dir, id, request, deltas)?;
  Ok(())
}

fn save_chat(
  chats_dir: &Path,
  id: String,
  request: CreateChatCompletionRequest,
  response: String,
) -> Result<(), ChatError> {
  let Some(message) = request.messages.last() else {
    return Err(ChatError::LastChatNotFoundErr);
  };
  let user_chat = match message {
    // TODO bug in serde probably
    ChatCompletionRequestMessage::System(message) => {
      let content = message.content.to_owned();
      Message {
        role: ROLE_USER.to_string(),
        content,
      }
    }
    _ => return Err(ChatError::LastChatNotFoundErr),
  };
  let assistant_chat = Message {
    role: ROLE_ASSISTANT.to_string(),
    content: response,
  };

  let file = find_file_by_id(chats_dir, &id);
  match file {
    Some(file) => {
      // existing chat
      let file_handle = File::open(file.clone())?;
      let mut chat: Chat = serde_json::from_reader(BufReader::new(file_handle))?;
      chat.messages.push(user_chat);
      chat.messages.push(assistant_chat);
      let file_content = serde_json::to_string(&chat)?;
      fs::write(file, file_content)?;
    }
    None => {
      let epoch_millis = Utc::now().format("%Y%m%d%H%M%S%3f").to_string();
      let filename = format!("{}_{}.json", epoch_millis, id);
      let file_path = chats_dir.join(filename);
      let chat = Chat {
        id,
        title: user_chat.content[0..min(user_chat.content.len() - 1, 100)].to_string(),
        messages: vec![user_chat, assistant_chat],
        created_at: Utc::now(),
      };
      let file_content = serde_json::to_string(&chat)?;
      fs::write(file_path, file_content)?;
    }
  };
  Ok(())
}

fn find_file_by_id(chats_dir: &Path, id: &str) -> Option<PathBuf> {
  let pattern = format!(r"\d{{17}}_({})\.json", regex::escape(id));
  let re = Regex::new(&pattern).expect("Invalid regex");
  if !chats_dir.is_dir() {
    return None;
  }
  if let Ok(entries) = fs::read_dir(chats_dir) {
    for entry in entries.flatten() {
      let path = entry.path();
      if path.is_file() {
        if let Some(file_name) = path.file_name().and_then(|name| name.to_str()) {
          if re.is_match(file_name) {
            return Some(path);
          }
        }
      }
    }
  }
  None
}

fn remove_dir_contents(dir: &Path) -> Result<(), ChatError> {
  for entry in fs::read_dir(dir)? {
    let entry = entry?;
    let path = entry.path();
    if path.is_dir() {
      fs::remove_dir_all(&path)?;
    } else {
      fs::remove_file(&path)?;
    }
  }
  Ok(())
}

#[cfg(test)]
mod test {
  use async_openai::types::CreateChatCompletionRequest;
  use chrono::Utc;
  use rstest::{fixture, rstest};
  use serde_json::json;
  use tempfile::TempDir;

  use super::_ui_chats_handler;
  use crate::server::{
    routes_ui::{
      Chat, ChatError, ChatPreview, Message, _ui_chat_delete_handler, _ui_chat_handler,
      _ui_chats_delete_handler, save_chat,
    },
    utils::BODHI_HOME,
  };
  use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
  };

  #[rstest]
  pub fn test_get_chats(bodhi_home: TempDir) -> anyhow::Result<()> {
    let chats_dir = bodhi_home.path().join("chats");
    let chats = _ui_chats_handler(chats_dir)?;
    assert_eq!(2, chats.len());
    assert_eq!(
      &ChatPreview {
        id: "2sglRnL".to_string(),
        title: "what day comes after Monday?".to_string(),
        created_at: chrono::DateTime::<Utc>::from_timestamp_millis(1713582468174).unwrap()
      },
      chats.first().unwrap()
    );
    assert_eq!(
      &ChatPreview {
        id: "UE6qd0b".to_string(),
        title: "list down months in a year".to_string(),
        created_at: chrono::DateTime::<Utc>::from_timestamp_millis(1713590479639).unwrap()
      },
      chats.get(1).unwrap()
    );
    drop(bodhi_home);
    Ok(())
  }

  #[rstest]
  fn test_get_chat(bodhi_home: TempDir) -> anyhow::Result<()> {
    let chats_dir = bodhi_home.path().join("chats");
    let id = "2sglRnL";
    let chat = _ui_chat_handler(chats_dir, id)?;
    let expected = Chat {
      id: "2sglRnL".to_string(),
      title: "what day comes after Monday?".to_string(),
      created_at: chrono::DateTime::<Utc>::from_timestamp_millis(1713582468174).unwrap(),
      messages: vec![
        Message {
          role: "user".to_string(),
          content: "what day comes after Monday?".to_string(),
        },
        Message {
          role: "assistant".to_string(),
          content: "The day that comes after Monday is Tuesday.".to_string(),
        },
      ],
    };
    assert_eq!(expected, chat);
    drop(bodhi_home);
    Ok(())
  }

  #[rstest]
  fn test_delete_chat(bodhi_home: TempDir) -> anyhow::Result<()> {
    let chats_dir = bodhi_home.path().join("chats");
    let id = "2sglRnL";
    _ui_chat_delete_handler(chats_dir, id)?;
    drop(bodhi_home);
    Ok(())
  }

  #[rstest]
  fn test_delete_chat_file_missing(bodhi_home: TempDir) -> anyhow::Result<()> {
    let chats_dir = bodhi_home.path().join("chats");
    let id = "undefined";
    let result = _ui_chat_delete_handler(chats_dir, id);
    assert!(result.is_err());
    assert_eq!(
      ChatError::ChatNotFoundErr(id.to_string()),
      result.unwrap_err()
    );
    drop(bodhi_home);
    Ok(())
  }

  #[rstest]
  fn test_delete_chats(bodhi_home: TempDir) -> anyhow::Result<()> {
    let chats_dir = bodhi_home.path().join("chats");
    _ui_chats_delete_handler(chats_dir)?;
    let chat_file = PathBuf::from(bodhi_home.path())
      .join("chats")
      .join("20240420105119639_UE6qd0b.json");
    assert!(!chat_file.exists());
    drop(bodhi_home);
    Ok(())
  }

  #[rstest]
  fn test_update_chat(bodhi_home: TempDir) -> anyhow::Result<()> {
    let chats_dir = bodhi_home.path().join("chats");
    let id = "UE6qd0b".to_string();
    let request = json! {{"model": "tinyllama", "messages": [{"role": "user", "content": "What day comes after Monday?"}]}};
    let request: CreateChatCompletionRequest = serde_json::from_value(request)?;
    save_chat(
      &chats_dir,
      id.clone(),
      request,
      "The day that comes after Monday is Tuesday.".to_string(),
    )?;
    let chat = _ui_chat_handler(chats_dir, &id)?;
    assert_eq!(4, chat.messages.len());
    assert_eq!(
      "What day comes after Monday?",
      chat.messages.get(2).unwrap().content
    );
    assert_eq!(
      "The day that comes after Monday is Tuesday.",
      chat.messages.get(3).unwrap().content
    );
    drop(bodhi_home);
    Ok(())
  }

  #[rstest]
  fn test_create_chat(bodhi_home: TempDir) -> anyhow::Result<()> {
    let chats_dir = bodhi_home.path().join("chats");
    let id = "NEWID07".to_string();
    let request = json! {{"model": "tinyllama", "messages": [{"role": "user", "content": "What day comes after Monday?"}]}};
    let request: CreateChatCompletionRequest = serde_json::from_value(request)?;
    save_chat(
      &chats_dir,
      id.clone(),
      request,
      "The day that comes after Monday is Tuesday.".to_string(),
    )?;
    let chat = _ui_chat_handler(chats_dir, &id)?;
    assert_eq!(2, chat.messages.len());
    let first = chat.messages.first().unwrap();
    assert_eq!("What day comes after Monday?", first.content);
    assert_eq!("user", first.role);
    let assistant = chat.messages.get(1).unwrap();
    assert_eq!(
      "The day that comes after Monday is Tuesday.",
      assistant.content
    );
    assert_eq!("assistant", assistant.role);
    drop(bodhi_home);
    Ok(())
  }

  #[fixture]
  pub fn bodhi_home() -> TempDir {
    _bodhi_home().unwrap()
  }

  fn _bodhi_home() -> anyhow::Result<TempDir> {
    let home_dir = tempfile::tempdir()?;
    let temp_chats_dir = tempfile::tempdir_in(&home_dir)?;
    let chats_dir = PathBuf::from(home_dir.path()).join("chats");
    let chats_dir = chats_dir.as_path();
    fs::rename(temp_chats_dir, chats_dir)?;
    let content = r#"{
      "title": "list down months in a year",
      "createdAt": 1713590479639,
      "id": "UE6qd0b",
      "messages": [
        {
          "role": "user",
          "content": "list down months in a year"
        },
        {
          "content": "1. January\n2. February\n3. March\n4. April\n5. May\n6. June\n7. July\n8. August\n9. September\n10. October\n11. November\n12. December",
          "role": "assistant"
        }
      ]
    }"#;
    create_temp_file(content, chats_dir, "20240420105119639_UE6qd0b.json")?;
    let content = r#"{
      "title": "what day comes after Monday?",
      "createdAt": 1713582468174,
      "id": "2sglRnL",
      "messages": [
        {
          "role": "user",
          "content": "what day comes after Monday?"
        },
        {
          "role": "assistant",
          "content": "The day that comes after Monday is Tuesday."
        }
      ]
    }"#;
    create_temp_file(content, chats_dir, "20241011083748174_2sglRnL.json")?;
    std::env::set_var(BODHI_HOME, format!("{}", home_dir.path().display()));
    Ok(home_dir)
  }

  fn create_temp_file(content: &str, temp_dir: &Path, filename: &str) -> anyhow::Result<()> {
    let mut tmp_file = tempfile::NamedTempFile::new_in(temp_dir)?;
    writeln!(tmp_file, "{}", content)?;
    let new_file_path = PathBuf::from(temp_dir).join(filename);
    std::fs::rename(tmp_file.into_temp_path(), new_file_path)?;
    Ok(())
  }
}
