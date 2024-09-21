use crate::{error::ObjError, repo::Repo};
use derive_new::new;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::Serialize;
use std::fmt;
use std::{fs, path::PathBuf};

pub static REGEX_HF_REPO_FILE: Lazy<Regex> = Lazy::new(|| {
  Regex::new(r"^(?P<hf_cache>.+)/models--(?P<username>[^/]+)--(?P<repo_name>[^/]+)/snapshots/(?P<snapshot>[^/]+)/(?P<filename>.*)$").unwrap()
});

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Serialize, new)]
#[cfg_attr(any(test, feature = "test-utils"), derive(derive_builder::Builder))]
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
  type Error = ObjError;

  fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
    let path = value.display().to_string();
    let caps = REGEX_HF_REPO_FILE
        .captures(&path)
        .ok_or_else(|| ObjError::Conversion {
            from: "PathBuf".to_string(),
            to: "HubFile".to_string(),
            error: format!(
                "The path '{}' does not match the expected Hugging Face hub cache filepath pattern. \
                 Expected format: '<hf_cache>/models--<username>--<repo_name>/snapshots/<snapshot>/<filename>'",
                path
            ),
        })?;
    let size = match fs::metadata(&value) {
      Ok(metadata) => Some(metadata.len()),
      Err(_) => None,
    };
    let repo = Repo::try_from(format!("{}/{}", &caps["username"], &caps["repo_name"]))?;
    Ok(HubFile {
      hf_cache: PathBuf::from(caps["hf_cache"].to_string()),
      repo,
      filename: caps["filename"].to_string(),
      snapshot: caps["snapshot"].to_string(),
      size,
    })
  }
}

impl fmt::Display for HubFile {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "HubFile {{ repo: {}, filename: {}, snapshot: {} }}",
      self.repo, self.filename, self.snapshot
    )
  }
}

#[cfg(test)]
mod test {
  use super::{HubFile, Repo};
  use crate::test_utils::hf_cache;
  use rstest::rstest;
  use std::path::PathBuf;
  use tempfile::TempDir;

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
      Repo::try_from("MyFactory/testalias-gguf")?,
      "testalias.Q8_0.gguf".to_string(),
      "5007652f7a641fe7170e0bad4f63839419bd9213".to_string(),
      Some(21),
    );
    assert_eq!(expected, local_model);
    Ok(())
  }

  #[test]
  fn test_hub_file_display() {
    let hub_file = HubFile {
      hf_cache: PathBuf::from("/tmp"),
      repo: Repo::try_from("test/repo").unwrap(),
      filename: "test.gguf".to_string(),
      snapshot: "abc123".to_string(),
      size: Some(1000),
    };
    assert_eq!(
      format!("{}", hub_file),
      "HubFile { repo: test/repo, filename: test.gguf, snapshot: abc123 }"
    );
  }
}
