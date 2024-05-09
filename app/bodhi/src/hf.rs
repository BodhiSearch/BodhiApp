use chrono::{DateTime, Utc};
use hf_hub::{Cache, Repo};
use regex::Regex;
use std::{
  borrow::Borrow,
  fs,
  path::{Path, PathBuf},
  time::SystemTime,
};
use walkdir::WalkDir;

#[allow(unused)]
pub(crate) static HF_HOME: &str = "HF_HOME";
pub(crate) static HF_API_PROGRESS: &str = "HF_API_PROGRESS";
pub(crate) static HF_TOKEN: &str = "HF_TOKEN";

pub(crate) fn hf_cache() -> Cache {
  Cache::default()
}

pub(crate) fn model_file(repo: &str, filename: &str) -> Option<PathBuf> {
  hf_cache().repo(Repo::model(repo.to_string())).get(filename)
}

pub(crate) fn download_url(url: &str, destination: &Path) -> anyhow::Result<PathBuf> {
  tracing::info!(url, "downloading file");
  let response = ureq::get(url).call()?;
  let mut buffer = Vec::new();
  response.into_reader().read_to_end(&mut buffer)?;
  std::fs::write(destination, buffer)?;
  Ok(destination.to_path_buf())
}

pub(crate) fn download_file(repo: &str, filename: &str) -> anyhow::Result<PathBuf> {
  tracing::info!(repo, filename, "downloading file");
  let hf_cache = hf_cache();
  let hf_repo = hf_cache.repo(Repo::model(repo.to_string()));
  let from_cache = hf_repo.get(filename);
  match from_cache {
    Some(path) => Ok(path),
    None => {
      let path = download_sync(repo, filename)?;
      Ok(path)
    }
  }
}

pub(crate) async fn download_async(repo: &str, file: &str) -> anyhow::Result<PathBuf> {
  use hf_hub::api::tokio::{ApiBuilder, ApiError};

  let progress_bar = std::env::var(HF_API_PROGRESS)
    .unwrap_or_else(|_| "true".to_string())
    .parse::<bool>()?;
  let api = ApiBuilder::new().with_progress(progress_bar).build()?;
  println!("Downloading from repo {repo}, model file {file}:");
  let path = match api.model(repo.to_string()).download(&file).await {
    Err(err) => {
      if let ApiError::RequestError(_) = err {
        err_msg(&repo);
      }
      return Err(err.into());
    }
    Ok(path) => path,
  };
  Ok(path)
}

pub(crate) fn download_sync(repo: &str, file: &str) -> anyhow::Result<PathBuf> {
  use hf_hub::api::sync::{ApiBuilder, ApiError};
  let mut api_builder = ApiBuilder::new();
  if let Some(progress_bar) = std::env::var_os(HF_API_PROGRESS) {
    api_builder = api_builder.with_progress(
      progress_bar
        .to_string_lossy()
        .into_owned()
        .parse::<bool>()
        .unwrap_or(false),
    );
  }
  if let Some(token) = std::env::var_os(HF_TOKEN) {
    let token = token.to_string_lossy().into_owned();
    api_builder = if token.is_empty() {
      api_builder.with_token(None)
    } else {
      api_builder.with_token(Some(token))
    }
  }
  let api = api_builder.build()?;
  tracing::info!("Downloading from repo {repo}, file {file}:");
  let path = match api.model(repo.to_string()).download(&file) {
    Ok(path) => path,
    Err(err) => {
      if let ApiError::RequestError(_) = err {
        err_msg(&repo);
      }
      return Err(err.into());
    }
  };
  Ok(path)
}

fn err_msg(repo: &str) {
  eprintln!("Download failed");
  eprintln!("1. You need to be logged in to huggingface using CLI - `huggingface_cli login`");
  eprintln!(
    "2. Accept the T&C of model on its homepage - https://huggingface.co/{}",
    repo
  );
  eprintln!("before you can download and use the model.")
}

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct ModelItem {
  pub name: String,
  pub repo: String,
  pub path: String,
  pub sha: String,
  pub size: Option<u64>,
  pub updated: Option<DateTime<Utc>>,
}

impl ModelItem {
  pub fn model_id(&self) -> String {
    format!("{}:{}", self.repo, self.name)
  }

  pub fn model_path(&self) -> String {
    hf_cache()
      .path()
      .join(&self.path)
      .to_string_lossy()
      .into_owned()
  }
}

pub fn list_models() -> Vec<ModelItem> {
  return _list_models(hf_cache().path());
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
        repo: format!("{}/{}", &caps["username"], &caps["repo_name"]),
        sha: String::from(&caps["commit"]),
        size,
        updated,
      })
    })
    .collect::<Vec<_>>()
}

// TODO: cache the response and load every 5 mins
pub fn find_model(model_id: &str) -> Option<ModelItem> {
  _find_model(hf_cache().path(), model_id)
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
  use tempfile::{Builder, TempDir};

  #[fixture]
  fn cache_dir() -> (TempDir, String, String) {
    _cache_dir().unwrap()
  }

  fn _cache_dir() -> anyhow::Result<(TempDir, String, String)> {
    let cache_dir = Builder::new().prefix("hf_hub").tempdir()?;
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
      repo: "User1/repo-coder".to_string(),
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
        repo: "TheYoung/AndRestless".to_string(),
        sha: "046744d93031".to_string(),
        size: Some(18),
        updated: Some(modified.into()),
      },
      model
    );
    Ok(())
  }
}
