use std::path::PathBuf;

use crate::list::find_remote_model;
use anyhow::{anyhow, bail};
use hf_hub::{
  api::{sync::ApiError as SyncApiError, tokio::ApiError},
  Cache, Repo,
};
use tokio::runtime::Builder;

#[derive(Debug, PartialEq)]
pub struct Pull {
  pub id: Option<String>,
  pub repo: Option<String>,
  pub file: Option<String>,
  pub force: bool,
}

impl Pull {
  pub fn new(id: Option<String>, repo: Option<String>, file: Option<String>, force: bool) -> Self {
    Pull {
      id,
      repo,
      file,
      force,
    }
  }

  pub fn execute(self) -> anyhow::Result<()> {
    match &self.id {
      Some(_) => {
        self.download_with_id()?;
      }
      None => {
        self.download_with_repo_file()?;
      }
    }
    Ok(())
  }

  fn download_with_repo_file(self) -> anyhow::Result<()> {
    let Pull {
      repo, file, force, ..
    } = self;
    let repo = repo.ok_or_else(|| anyhow!("repo is missing"))?;
    let file = file.ok_or_else(|| anyhow!("file is missing"))?;
    download(repo, file, force)?;
    Ok(())
  }

  fn download_with_id(self) -> anyhow::Result<()> {
    let Pull { id, force, .. } = self;
    let Some(id) = id else {
      bail!("model id is required");
    };
    let model = find_remote_model(&id);
    let Some(model) = model else {
      bail!(
        "model with id {} not found in pre-configured remote models",
        id
      );
    };
    download(model.repo_id(), model.default, force)?;
    Ok(())
  }
}

pub(crate) fn download(repo: String, file: String, force: bool) -> anyhow::Result<PathBuf> {
  let from_cache = Cache::default().repo(Repo::model(repo.clone())).get(&file);
  if let Some(file) = from_cache {
    if !force {
      println!("model file already exists in cache: '{}'", file.display());
      println!("use '--force' to force download it again");
      bail!("");
    }
  }
  let runtime = Builder::new_multi_thread().enable_all().build();
  let path = match runtime {
    Ok(runtime) => {
      let path = runtime.block_on(async move { download_async(repo, file).await })?;
      path
    }
    Err(_) => download_sync(repo, file)?,
  };
  Ok(path)
}

async fn download_async(repo: String, file: String) -> anyhow::Result<PathBuf> {
  use hf_hub::api::tokio::Api;

  let api = Api::new()?;
  println!("Downloading from repo {repo}, model file {file}:");
  let path = match api.model(repo.clone()).download(&file).await {
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

fn download_sync(repo: String, file: String) -> anyhow::Result<PathBuf> {
  use hf_hub::api::sync::Api;

  let api = Api::new()?;
  println!("Downloading from repo {repo}, model file {file}:");
  let path = match api.model(repo.clone()).download(&file) {
    Ok(path) => path,
    Err(err) => {
      if let SyncApiError::RequestError(_) = err {
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
