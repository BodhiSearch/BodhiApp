use serde::{Deserialize, Serialize};
use std::{fs, io, path::PathBuf};
use thiserror::Error;
pub static DEFAULT_PORT: u16 = 7735;
// TODO: see if can use lazy_static to not duplicate port
pub static DEFAULT_PORT_STR: &str = "7735";
pub static DEFAULT_HOST: &str = "127.0.0.1";
pub static BODHI_HOME: &str = "BODHI_HOME";

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ApiError {
  pub(crate) error: String,
}

pub fn port_from_env_vars(port: Result<String, std::env::VarError>) -> u16 {
  match port {
    Ok(port) => match port.parse::<u16>() {
      Ok(port) => port,
      Err(err) => {
        tracing::debug!(
          err = ?err,
          port = port,
          default_port = DEFAULT_PORT,
          "error parsing port set in environment variable, using default port",
        );
        DEFAULT_PORT
      }
    },
    Err(err) => {
      tracing::debug!(
        err = ?err,
        default_port = DEFAULT_PORT,
        "error reading port from environment variable, using default port",
      );
      DEFAULT_PORT
    }
  }
}

#[derive(Debug, Error)]
pub(crate) enum HomeDirError {
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
  #[error(transparent)]
  IOError(#[from] io::Error),
}

impl PartialEq for HomeDirError {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (HomeDirError::HomeDirErr, HomeDirError::HomeDirErr) => true,
      (HomeDirError::HomeDirCreateErr, HomeDirError::HomeDirCreateErr) => true,
      (HomeDirError::ChatDirErr, HomeDirError::ChatDirErr) => true,
      (HomeDirError::ReadDirErr, HomeDirError::ReadDirErr) => true,
      (HomeDirError::IOError(e1), HomeDirError::IOError(e2)) => e1.kind() == e2.kind(),
      _ => false,
    }
  }
}

pub(crate) fn get_bodhi_home_dir() -> Result<PathBuf, HomeDirError> {
  if let Ok(bodhi_home) = std::env::var(BODHI_HOME) {
    let home_dir = PathBuf::from(bodhi_home);
    if home_dir.exists() {
      if home_dir.is_dir() {
        Ok(home_dir)
      } else {
        Err(HomeDirError::ChatNotDirErr)
      }
    } else {
      Err(HomeDirError::ChatDirErr)
    }
  } else {
    let home_dir = dirs::home_dir().ok_or(HomeDirError::HomeDirErr)?;
    if !home_dir.exists() {
      fs::create_dir_all(&home_dir).map_err(|_| HomeDirError::HomeDirCreateErr)?;
    }
    Ok(home_dir)
  }
}

pub(crate) fn get_chats_dir() -> Result<PathBuf, HomeDirError> {
  let bodhi_home = get_bodhi_home_dir()?;
  let chats_dir = bodhi_home.join("chats");
  if chats_dir.exists() {
    if chats_dir.is_dir() {
      Ok(chats_dir)
    } else {
      Err(HomeDirError::ChatNotDirErr)
    }
  } else {
    fs::create_dir_all(&chats_dir)?;
    Ok(chats_dir)
  }
}

#[cfg(test)]
mod tests {
  use super::{port_from_env_vars, DEFAULT_PORT};
  use rstest::rstest;

  #[test]
  pub fn test_port_from_env_vars_not_present() {
    let port = port_from_env_vars(Err(std::env::VarError::NotPresent));
    assert_eq!(port, DEFAULT_PORT);
  }

  #[test]
  pub fn test_port_from_env_vars_valid() {
    let port = port_from_env_vars(Ok("8055".to_string()));
    assert_eq!(port, 8055);
  }

  #[rstest]
  #[case("notu16")]
  #[case("65536")]
  #[case("-1")]
  pub fn test_port_from_env_vars_malformed(#[case] input: &str) {
    let port = port_from_env_vars(Ok(input.to_string()));
    assert_eq!(port, DEFAULT_PORT);
  }
}
