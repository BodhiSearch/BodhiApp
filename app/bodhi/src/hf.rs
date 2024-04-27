use chrono::{DateTime, Utc};
use hf_hub::Cache;
use regex::Regex;
use std::{borrow::Borrow, fs, path::Path, time::SystemTime};
use walkdir::WalkDir;

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct ModelItem {
  pub path: String,
  pub name: String,
  pub owner: String,
  pub repo: String,
  pub sha: String,
  pub size: Option<u64>,
  pub updated: Option<DateTime<Utc>>,
}
impl ModelItem {
  pub fn model_id(&self) -> String {
    format!("{}/{}:{}", self.owner, self.repo, self.name)
  }
}

pub fn list_models() -> Vec<ModelItem> {
  return _list_models(Cache::default().path());
}

pub(super) fn _list_models(cache_dir: &Path) -> Vec<ModelItem> {
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

// TODO: cache the response and load every 5 mins
pub fn find_model(model_id: &str) -> Option<ModelItem> {
  _find_model(Cache::default().path(), model_id)
}

pub(super) fn _find_model(cache_dir: &Path, model_id: &str) -> Option<ModelItem> {
  let models = _list_models(cache_dir);
  models.into_iter().find(|item| {
    let current_id = item.model_id();
    current_id.eq(model_id)
  })
}

#[cfg(test)]
mod test {
  use super::{ModelItem, _find_model, _list_models};
  use rstest::{fixture, rstest};
  use std::fs::{self, File};
  use std::io::Write;
  use tempdir::TempDir;

  #[fixture]
  fn cache_dir() -> (TempDir, String, String) {
    _cache_dir().unwrap()
  }

  fn _cache_dir() -> anyhow::Result<(TempDir, String, String)> {
    let cache_dir = tempdir::TempDir::new("hf_hub")?;
    let cache_path = cache_dir.path().to_path_buf();
    let model_dir = cache_path.join("models--User1--repo-coder/snapshots/9e221e6b41cb/");
    fs::create_dir_all(&model_dir)?;
    let model_file = model_dir.join("coder-6.7b-instruct.Q8_0.gguf");
    writeln!(File::create_new(model_file.clone())?, "sample model file")?;

    let model_dir = cache_path.join("models--TheYoung--AndRestless/snapshots/046744d93031/");
    fs::create_dir_all(&model_dir)?;
    let model_file2 = model_dir.join("bigbag-14.2b-theory.Q1_0.gguf");
    writeln!(File::create_new(model_file2.clone())?, "sample model file")?;

    Ok((
      cache_dir,
      model_file.to_string_lossy().into_owned(),
      model_file2.to_string_lossy().into_owned(),
    ))
  }

  #[rstest]
  fn test_list_models(cache_dir: (TempDir, String, String)) -> anyhow::Result<()> {
    let (cache_dir, model_file1, _) = cache_dir;
    let mut models = _list_models(cache_dir.path());
    models.sort_by(|a, b| b.cmp(a));
    assert_eq!(2, models.len());
    let modified = fs::metadata(model_file1)?.modified()?;
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

  #[rstest]
  fn test_find_model(cache_dir: (TempDir, String, String)) -> anyhow::Result<()> {
    let (cache_dir, _, model_file2) = cache_dir;
    let model = _find_model(
      cache_dir.path(),
      "TheYoung/AndRestless:bigbag-14.2b-theory.Q1_0.gguf",
    );
    assert!(model.is_some());
    let model = model.unwrap();
    let modified = fs::metadata(model_file2)?.modified()?;
    assert_eq!(
      ModelItem {
        path: "models--TheYoung--AndRestless/snapshots/046744d93031/bigbag-14.2b-theory.Q1_0.gguf"
          .to_string(),
        name: "bigbag-14.2b-theory.Q1_0.gguf".to_string(),
        owner: "TheYoung".to_string(),
        repo: "AndRestless".to_string(),
        sha: "046744d93031".to_string(),
        size: Some(18),
        updated: Some(modified.into()),
      },
      model
    );
    Ok(())
  }
}
