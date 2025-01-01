use crate::{ObjValidationError, Repo};
use serde::Serialize;
use std::fmt;
use std::{fs, path::PathBuf};

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Serialize, derive_new::new)]
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
  type Error = ObjValidationError;

  fn try_from(mut value: PathBuf) -> Result<Self, Self::Error> {
    let path_str = value.display().to_string();
    let size = fs::metadata(&value).ok().map(|metadata| metadata.len());
    // Get filename
    let filename = value
      .file_name()
      .and_then(|f| f.to_str())
      .ok_or_else(|| ObjValidationError::FilePatternMismatch(path_str.clone()))?
      .to_string();

    // Get snapshot hash
    value.pop(); // move to parent
    let snapshot = value
      .file_name()
      .and_then(|f| f.to_str())
      .ok_or_else(|| ObjValidationError::FilePatternMismatch(path_str.clone()))?
      .to_string();

    // Verify "snapshots" directory
    value.pop();
    if value
      .file_name()
      .and_then(|f| f.to_str())
      .map_or(true, |name| name != "snapshots")
    {
      return Err(ObjValidationError::FilePatternMismatch(path_str));
    }
    value.pop();

    // Extract repo info from models--username--repo_name format
    let repo_dir = value
      .file_name()
      .and_then(|f| f.to_str())
      .ok_or_else(|| ObjValidationError::FilePatternMismatch(path_str.clone()))?
      .to_string();

    // Store repo parts before moving value
    let repo_parts: Vec<&str> = repo_dir.split("--").collect();
    if repo_parts.len() != 3 || repo_parts[0] != "models" {
      return Err(ObjValidationError::FilePatternMismatch(path_str));
    }

    // Get hf_cache (parent directory of the repo directory)
    value.pop();
    let hf_cache = value;

    // Construct repo from username/repo_name
    let repo = Repo::try_from(format!("{}/{}", repo_parts[1], repo_parts[2]))?;

    Ok(HubFile {
      hf_cache,
      repo,
      filename,
      snapshot,
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
  use crate::{test_utils::temp_hf_home, HubFile, Repo};
  use rstest::rstest;
  use std::path::PathBuf;
  use tempfile::TempDir;

  #[rstest]
  fn test_local_model_file_from_pathbuf(temp_hf_home: TempDir) -> anyhow::Result<()> {
    let hf_cache = temp_hf_home.path().join("huggingface").join("hub");
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
      Some(96),
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
      "HubFile { repo: test/repo, filename: test.gguf, snapshot: abc123 }",
      format!("{}", hub_file)
    );
  }
}
