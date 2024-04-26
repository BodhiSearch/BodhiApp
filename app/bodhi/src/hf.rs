use chrono::{DateTime, Utc};
use regex::Regex;
use std::{borrow::Borrow, fs, path::Path, time::SystemTime};
use walkdir::WalkDir;

#[derive(Debug, PartialEq)]
pub struct ModelItem {
  pub path: String,
  pub name: String,
  pub owner: String,
  pub repo: String,
  pub sha: String,
  pub size: Option<u64>,
  pub updated: Option<DateTime<Utc>>,
}

pub fn list_models(cache_dir: &Path) -> Vec<ModelItem> {
  let mut cache_path = cache_dir.to_string_lossy().into_owned();
  cache_path.push('/');
  let re = Regex::new(r".*/models--(?P<username>[^/]+)--(?P<repo_name>[^/]+)/snapshots/(?P<commit>[^/]+)/(?P<model_name>.*)\.gguf$").unwrap();
  WalkDir::new(cache_dir)
    .follow_links(true)
    .into_iter()
    .filter_map(|e| e.ok())
    .filter(|entry| entry.path().is_file())
    .filter_map(|entry| {
      let path = entry.path();
      let filepath = path.to_string_lossy();
      let filepath = filepath.borrow();
      let caps = re.captures(filepath)?;
      let (size, updated) = match fs::metadata(path) {
        Ok(metadata) => (
          Some(metadata.len()),
          Some(
            metadata
              .modified()
              .unwrap_or_else(|_| SystemTime::now())
              .into(),
          ),
        ),
        Err(_) => (None, None),
      };
      let relative_path = filepath.strip_prefix(&cache_path).unwrap_or("");
      Some(ModelItem {
        path: relative_path.to_string(),
        name: format!("{}.gguf", &caps["model_name"]),
        owner: String::from(&caps["username"]),
        repo: String::from(&caps["repo_name"]),
        sha: String::from(&caps["commit"]),
        size,
        updated,
      })
    })
    .collect::<Vec<_>>()
}

#[cfg(test)]
mod test {
  use std::fs::{self, File};

  use crate::hf::{list_models, ModelItem};
  use std::io::Write;

  #[test]
  fn test_list_models() -> anyhow::Result<()> {
    let cache_dir = tempdir::TempDir::new("hf_hub")?;
    let cache_path = cache_dir.path().to_path_buf();
    let model_dir = cache_path.join("models--User1--repo-coder/snapshots/9e221e6b41cb/");
    fs::create_dir_all(&model_dir)?;
    let model_file = model_dir.join("coder-6.7b-instruct.Q8_0.gguf");
    writeln!(File::create_new(model_file.clone())?, "sample model file")?;

    let models = list_models(&cache_path);
    assert_eq!(1, models.len());
    let modified = fs::metadata(&model_file)?.modified()?;
    let expected = ModelItem {
      path: "models--User1--repo-coder/snapshots/9e221e6b41cb/coder-6.7b-instruct.Q8_0.gguf"
        .to_string(),
      name: "coder-6.7b-instruct.Q8_0.gguf".to_string(),
      owner: "User1".to_string(),
      repo: "repo-coder".to_string(),
      sha: "9e221e6b41cb".to_string(),
      size: Some(18),
      updated: Some(modified.into()),
    };
    assert_eq!(&expected, models.first().unwrap());
    Ok(())
  }
}
