use super::Repo;
use crate::{service::DataServiceError, tokenizer_config::TokenizerConfig};
use derive_new::new;
use once_cell::sync::Lazy;
use prettytable::{Cell, Row};
use regex::Regex;
use serde::Serialize;
use std::{fs, path::PathBuf};

pub static REGEX_HF_REPO_FILE: Lazy<Regex> = Lazy::new(|| {
  Regex::new(r"^(?P<hf_cache>.+)/models--(?P<username>[^/]+)--(?P<repo_name>[^/]+)/snapshots/(?P<snapshot>[^/]+)/(?P<filename>.*)$").unwrap()
});

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Serialize, new)]
#[cfg_attr(test, derive(derive_builder::Builder))]
pub struct HubFile {
  pub hf_cache: PathBuf,
  pub repo: Repo,
  pub filename: String,
  pub snapshot: String,
  pub size: Option<u64>,
}

impl HubFile {
  pub fn path(&self) -> PathBuf {
    let mut path = self.hf_cache.clone();
    path.push(self.repo.path());
    path.push("snapshots");
    path.push(&self.snapshot);
    path.push(&self.filename);
    path
  }
}

impl TryFrom<PathBuf> for HubFile {
  type Error = DataServiceError;

  fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
    let path = value.display().to_string();
    let caps = REGEX_HF_REPO_FILE.captures(&path).ok_or(anyhow::anyhow!(
      "'{path}' does not match huggingface hub cache filepath pattern"
    ))?;
    let size = match fs::metadata(&value) {
      Ok(metadata) => Some(metadata.len()),
      Err(_) => None,
    };
    let repo = Repo::try_new(format!("{}/{}", &caps["username"], &caps["repo_name"]))?;
    Ok(HubFile {
      hf_cache: PathBuf::from(caps["hf_cache"].to_string()),
      repo,
      filename: caps["filename"].to_string(),
      snapshot: caps["snapshot"].to_string(),
      size,
    })
  }
}

impl From<HubFile> for Row {
  fn from(model: HubFile) -> Self {
    let HubFile {
      repo,
      filename,
      snapshot,
      size,
      ..
    } = model;
    let human_size = size
      .map(|size| format!("{:.2} GB", size as f64 / 2_f64.powf(30.0)))
      .unwrap_or_else(|| String::from("Unknown"));
    Row::from(vec![
      Cell::new(&filename),
      Cell::new(&repo),
      Cell::new(&snapshot[..8]),
      Cell::new(&human_size),
    ])
  }
}

impl TryFrom<HubFile> for TokenizerConfig {
  type Error = DataServiceError;

  fn try_from(value: HubFile) -> Result<Self, Self::Error> {
    let path = value.path();
    let content = std::fs::read_to_string(path.clone())
      .map_err(move |source| DataServiceError::IoWithDetail { source, path })?;
    let tokenizer_config: TokenizerConfig = serde_json::from_str(&content)?;
    Ok(tokenizer_config)
  }
}

#[cfg(test)]
mod test {
  use super::{HubFile, Repo};
  use crate::test_utils::hf_cache;
  use prettytable::{Cell, Row};
  use rstest::rstest;
  use std::path::PathBuf;
  use tempfile::TempDir;

  #[test]
  fn test_local_model_to_row() -> anyhow::Result<()> {
    let model = HubFile::new(
      PathBuf::from("."),
      Repo::try_new("QuantFactory/Meta-Llama-3-8B-Instruct-GGUF".to_string())?,
      "Meta-Llama-3-8B-Instruct.Q8_0.gguf".to_string(),
      "1234567890".to_string(),
      Some(1024 * 1024 * 1024 * 10),
    );
    let row = model.into();
    let expected = Row::from(vec![
      Cell::new("Meta-Llama-3-8B-Instruct.Q8_0.gguf"),
      Cell::new("QuantFactory/Meta-Llama-3-8B-Instruct-GGUF"),
      Cell::new("12345678"),
      Cell::new("10.00 GB"),
    ]);
    assert_eq!(expected, row);
    Ok(())
  }

  #[rstest]
  fn test_local_model_file_from_pathbuf(hf_cache: (TempDir, PathBuf)) -> anyhow::Result<()> {
    let (_temp, hf_cache) = hf_cache;
    let filepath = hf_cache
      .clone()
      .join("models--MyFactory--testalias-gguf")
      .join("snapshots")
      .join("5007652f7a641fe7170e0bad4f63839419bd9213")
      .join("testalias.Q8_0.gguf");
    let local_model = HubFile::try_from(filepath)?;
    let expected = HubFile::new(
      hf_cache,
      Repo::try_new("MyFactory/testalias-gguf".to_string())?,
      "testalias.Q8_0.gguf".to_string(),
      "5007652f7a641fe7170e0bad4f63839419bd9213".to_string(),
      Some(21),
    );
    assert_eq!(expected, local_model);
    Ok(())
  }
}
