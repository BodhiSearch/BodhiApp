use crate::{
  error::{BodhiError, Common},
  server::BODHI_HOME,
};
use std::{env, fs, path::PathBuf};

pub fn user_home() -> Result<PathBuf, BodhiError> {
  dirs::home_dir().ok_or_else(|| BodhiError::HomeDirectory)
}

pub fn bodhi_home() -> Result<PathBuf, BodhiError> {
  let bodhi_home = match env::var(BODHI_HOME) {
    Ok(bodhi_home) => PathBuf::from(bodhi_home),
    Err(_) => user_home()?.join(".cache").join("bodhi"),
  };
  if !bodhi_home.exists() {
    fs::create_dir_all(&bodhi_home).map_err(|source| Common::IoDir {
      source,
      path: bodhi_home.display().to_string(),
    })?;
  }
  Ok(bodhi_home)
}

pub fn logs_dir() -> Result<PathBuf, BodhiError> {
  let bodhi_home = bodhi_home()?;
  let logs_dir = PathBuf::from(format!("{}/logs", bodhi_home.display()));
  if !logs_dir.exists() {
    std::fs::create_dir_all(&logs_dir).map_err(|source| Common::IoDir {
      source,
      path: logs_dir.display().to_string(),
    })?;
  }
  Ok(logs_dir)
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_user_home_is_resolvable() -> anyhow::Result<()> {
    let user_home = user_home()?;
    assert_ne!("", user_home.as_os_str().to_string_lossy().into_owned());
    Ok(())
  }

  #[test]
  fn test_bodhi_home_is_resolvable() -> anyhow::Result<()> {
    let bodhi_home = bodhi_home()?;
    assert_ne!("", bodhi_home.as_os_str().to_string_lossy().into_owned());
    Ok(())
  }

  #[test]
  fn test_logs_dir_is_resolvable() -> anyhow::Result<()> {
    let logs_dir = logs_dir()?;
    assert_ne!("", logs_dir.as_os_str().to_string_lossy().into_owned());
    Ok(())
  }
}
