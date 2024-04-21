use axum::body::Body;
use axum::extract::Path as UrlPath;
use axum::http::Response;
use axum::response::IntoResponse;
use axum::response::Json;
use chrono::serde::ts_milliseconds;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use std::fs;
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
  #[error("Failed to read directory")]
  ReadDirErr,
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

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Message {
  id: String,
  role: String,
  content: String,
  #[serde(rename = "createdAt", with = "ts_milliseconds")]
  created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
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

fn _ui_chats_handler() -> Result<Vec<ChatPreview>, ChatError> {
  let chats_dir = if let Ok(bodhi_home) = std::env::var(BODHI_HOME) {
    PathBuf::from(bodhi_home).join("chats")
  } else {
    let home_dir = dirs::home_dir().ok_or(ChatError::HomeDirErr)?;
    let bodhi_home = Path::new(&home_dir).join(".bodhi/chats");
    if !bodhi_home.exists() {
      fs::create_dir_all(&bodhi_home).map_err(|_| ChatError::HomeDirCreateErr)?;
    }
    bodhi_home
  };
  if !chats_dir.exists() {
    return Err(ChatError::ChatDirErr);
  }
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

#[cfg(test)]
mod test {
  use chrono::Utc;
  use rstest::{fixture, rstest};
  use tempfile::TempDir;

  use super::_ui_chats_handler;
  use crate::server::{routes_ui::ChatPreview, utils::BODHI_HOME};
  use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
  };

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
    create_temp_file(content, chats_dir, "20240401122734_UE6qd0b.json")?;
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
    create_temp_file(content, chats_dir, "20240402123456_2sglRnL.json")?;
    std::env::set_var(BODHI_HOME, format!("{}", home_dir.path().display()));
    Ok(home_dir)
  }

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

  fn create_temp_file(content: &str, temp_dir: &Path, filename: &str) -> anyhow::Result<()> {
    let mut tmp_file = tempfile::NamedTempFile::new_in(temp_dir)?;
    writeln!(tmp_file, "{}", content)?;
    let new_file_path = PathBuf::from(temp_dir).join(filename);
    std::fs::rename(tmp_file.into_temp_path(), new_file_path)?;
    Ok(())
  }
}
