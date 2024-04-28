use anyhow::anyhow;
use hf_hub::{Cache, Repo};
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

  pub fn download(self) -> anyhow::Result<()> {
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
    let from_cache = Cache::default().repo(Repo::model(repo.clone())).get(&file);
    if let Some(file) = from_cache {
      if !force {
        println!("model file already exists in cache: '{}'", file.display());
        println!("use '--force' to force download it again");
        return Ok(());
      }
    }
    let runtime = Builder::new_multi_thread().enable_all().build();
    match runtime {
      Ok(runtime) => {
        runtime.block_on(async move { Pull::download_async(repo, file).await })?;
      }
      Err(_) => {
        Pull::download_sync(repo, file)?;
      }
    }
    Ok(())
  }

  async fn download_async(repo: String, file: String) -> anyhow::Result<()> {
    use hf_hub::api::tokio::Api;

    let api = Api::new()?;
    println!("Downloading from repo {repo}, model file {file}:");
    api.model(repo).download(&file).await?;
    Ok(())
  }

  fn download_sync(repo: String, file: String) -> anyhow::Result<()> {
    use hf_hub::api::sync::Api;

    let api = Api::new()?;
    println!("Downloading from repo {repo}, model file {file}:");
    api.model(repo).download(&file)?;
    Ok(())
  }

  fn download_with_id(self) -> anyhow::Result<()> {
    todo!()
  }
}
