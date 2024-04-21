use axum::body::Body;
use axum::extract::Path as UrlPath;
use axum::http::Response;
use axum::response::IntoResponse;
use axum::response::Json;
use chrono::serde::ts_milliseconds;
use chrono::Utc;
use regex::Regex;
use serde::Deserialize;
use serde::Serialize;
use std::fs;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;
use thiserror::Error;

use super::utils::BODHI_HOME;

#[derive(Error, Debug)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum ChatError {
  #[error("Failed to get user home directory")]
  HomeDirErr,
  #[error("Failed to create app home directory")]
  HomeDirCreateErr,
  #[error("Failed to get chats directory")]
  ChatDirErr,
  #[error("Chats directory is not a directory")]
  ChatNotDirErr,
  #[error("Failed to read directory")]
  ReadDirErr,
  #[error("Failed to find the chat with given id")]
  ChatNotFoundErr,
  #[error(transparent)]
  IOError(#[from] io::Error),
  #[error(transparent)]
  JsonError(#[from] serde_json::Error),
}

impl PartialEq for ChatError {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (ChatError::HomeDirErr, ChatError::HomeDirErr) => true,
      (ChatError::HomeDirCreateErr, ChatError::HomeDirCreateErr) => true,
      (ChatError::ChatDirErr, ChatError::ChatDirErr) => true,
      (ChatError::ReadDirErr, ChatError::ReadDirErr) => true,
      (ChatError::ChatNotFoundErr, ChatError::ChatNotFoundErr) => true,
      (ChatError::IOError(e1), ChatError::IOError(e2)) => e1.kind() == e2.kind(),
      (ChatError::JsonError(e1), ChatError::JsonError(e2)) => {
        format!("{}", e1) == format!("{}", e2)
      }
      _ => false,
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
struct ApiError {
  error: String,
}

impl IntoResponse for ChatError {
  fn into_response(self) -> Response<Body> {
    Json(ApiError {
      error: format!("{}", self),
    })
    .into_response()
  }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub(crate) struct ChatPreview {
  id: String,
  title: String,
  #[serde(rename = "createdAt", with = "ts_milliseconds")]
  created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub(crate) struct Message {
  role: String,
  content: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub(crate) struct Chat {
  id: String,
  title: String,
  messages: Vec<Message>,
  #[serde(rename = "createdAt", with = "ts_milliseconds")]
  created_at: chrono::DateTime<Utc>,
}

pub(crate) async fn ui_chats_handler() -> Result<Json<Vec<ChatPreview>>, ChatError> {
  let chats = _ui_chats_handler()?;
  Ok(Json(chats))
}

pub(crate) async fn ui_chats_delete_handler() -> Result<Json<()>, ChatError> {
  let chats = _ui_chats_delete_handler()?;
  Ok(Json(chats))
}

pub(crate) async fn ui_chat_handler(UrlPath(id): UrlPath<String>) -> Result<Json<Chat>, ChatError> {
  let chat = _ui_chat_handler(&id)?;
  Ok(Json(chat))
}

pub(crate) async fn ui_chat_delete_handler(
  UrlPath(id): UrlPath<String>,
) -> Result<Json<()>, ChatError> {
  let chat = _ui_chat_delete_handler(&id)?;
  Ok(Json(chat))
}

fn get_chats_dir() -> Result<PathBuf, ChatError> {
  if let Ok(bodhi_home) = std::env::var(BODHI_HOME) {
    let chats_dir = PathBuf::from(bodhi_home).join("chats");
    if chats_dir.exists() {
      if chats_dir.is_dir() {
        Ok(chats_dir)
      } else {
        Err(ChatError::ChatNotDirErr)
      }
    } else {
      Err(ChatError::ChatDirErr)
    }
  } else {
    let home_dir = dirs::home_dir().ok_or(ChatError::HomeDirErr)?;
    let chats_dir = Path::new(&home_dir).join(".bodhi/chats");
    if !chats_dir.exists() {
      fs::create_dir_all(&chats_dir).map_err(|_| ChatError::HomeDirCreateErr)?;
    }
    Ok(chats_dir)
  }
}

fn _ui_chats_handler() -> Result<Vec<ChatPreview>, ChatError> {
  let chats_dir = get_chats_dir()?;
  let mut files: Vec<_> = fs::read_dir(chats_dir)
    .map_err(|_| ChatError::ReadDirErr)?
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
  let chats = files.into_iter().fold(chats, |mut chats, file| {
    if let Ok(content) = fs::read_to_string(file) {
      match serde_json::from_str::<ChatPreview>(&content) {
        Ok(obj) => {
          chats.push(obj);
        }
        Err(e) => {
          eprintln!("error parsing:{e}");
        }
      }
    }
    chats
  });
  Ok(chats)
}

fn _ui_chats_delete_handler() -> Result<(), ChatError> {
  let chats_dir = get_chats_dir()?;
  remove_dir_contents(&chats_dir)?;
  Ok(())
}

fn _ui_chat_handler(id: &str) -> Result<Chat, ChatError> {
  let chats_dir = get_chats_dir()?;
  let file = find_file_by_id(&chats_dir, id).ok_or(ChatError::ChatNotFoundErr)?;
  let file = File::open(file)?;
  let chat: Chat = serde_json::from_reader(BufReader::new(file))?;
  Ok(chat)
}

fn _ui_chat_delete_handler(id: &str) -> Result<(), ChatError> {
  let chats_dir = get_chats_dir()?;
  let file = find_file_by_id(&chats_dir, id).ok_or(ChatError::ChatNotFoundErr)?;
  if file.exists() {
    fs::remove_file(&file)?;
    Ok(())
  } else {
    Err(ChatError::ChatNotFoundErr)
  }
}

fn find_file_by_id(directory: &Path, id: &str) -> Option<PathBuf> {
  let pattern = format!(r"\d{{17}}_({})\.json", regex::escape(id));
  let re = Regex::new(&pattern).expect("Invalid regex");
  if !directory.is_dir() {
    return None;
  }
  if let Ok(entries) = fs::read_dir(directory) {
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
  use chrono::Utc;
  use rstest::{fixture, rstest};
  use tempfile::TempDir;

  use super::_ui_chats_handler;
  use crate::server::{
    routes_ui::{
      Chat, ChatError, ChatPreview, Message, _ui_chat_delete_handler, _ui_chat_handler,
      _ui_chats_delete_handler,
    },
    utils::BODHI_HOME,
  };
  use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
  };

  #[rstest]
  pub fn test_get_chats(bodhi_home: anyhow::Result<TempDir>) -> anyhow::Result<()> {
    let _bodhi_home = bodhi_home?;
    let chats = _ui_chats_handler()?;
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
    Ok(())
  }

  #[rstest]
  fn test_get_chat(bodhi_home: anyhow::Result<TempDir>) -> anyhow::Result<()> {
    let _bodhi_home = bodhi_home?;
    let id = "2sglRnL";
    let chat = _ui_chat_handler(id)?;
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
    Ok(())
  }

  #[rstest]
  fn test_delete_chat(bodhi_home: anyhow::Result<TempDir>) -> anyhow::Result<()> {
    let _bodhi_home = bodhi_home?;
    let id = "2sglRnL";
    _ui_chat_delete_handler(id)?;
    Ok(())
  }

  #[rstest]
  fn test_delete_chat_file_missing(bodhi_home: anyhow::Result<TempDir>) -> anyhow::Result<()> {
    let _bodhi_home = bodhi_home?;
    let id = "undefined";
    let result = _ui_chat_delete_handler(id);
    assert!(result.is_err());
    assert_eq!(ChatError::ChatNotFoundErr, result.unwrap_err());
    Ok(())
  }

  #[rstest]
  fn test_delete_chats(bodhi_home: anyhow::Result<TempDir>) -> anyhow::Result<()> {
    let _bodhi_home = bodhi_home?;
    _ui_chats_delete_handler()?;
    let chat_file = PathBuf::from(_bodhi_home.path())
      .join("chats")
      .join("20240420105119639_UE6qd0b.json");
    assert_eq!(false, chat_file.exists());
    Ok(())
  }

  #[fixture]
  fn bodhi_home() -> anyhow::Result<TempDir> {
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
