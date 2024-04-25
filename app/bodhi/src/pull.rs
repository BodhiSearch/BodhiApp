use hf_hub::{Cache, Repo};
use tokio::runtime::Builder;

#[derive(Debug, PartialEq)]
pub struct Pull {
  pub repo: String,
  pub file: String,
  pub force: bool,
}

impl Pull {
  pub fn download(self) -> anyhow::Result<()> {
    let from_cache = Cache::default()
      .repo(Repo::model(self.repo.clone()))
      .get(&self.file);
    if let Some(file) = from_cache {
      if !self.force {
        println!("model file already exists in cache: '{}'", file.display());
        println!("use '--force' to force download it again");
        return Ok(());
      }
    }
    let runtime = Builder::new_multi_thread().enable_all().build();
    match runtime {
      Ok(runtime) => {
        runtime.block_on(async move { self.download_async().await })?;
      }
      Err(_) => {
        self.download_sync()?;
      }
    }
    Ok(())
  }

  async fn download_async(self) -> anyhow::Result<()> {
    use hf_hub::api::tokio::Api;

    let Pull { repo, file, .. } = self;
    let api = Api::new()?;
    println!("Downloading from repo {repo}, model file {file}:");
    api.model(repo).download(&file).await?;
    Ok(())
  }

  fn download_sync(self) -> anyhow::Result<()> {
    use hf_hub::api::sync::Api;

    let Pull { repo, file, .. } = self;
    let api = Api::new()?;
    println!("Downloading from repo {repo}, model file {file}:");
    api.model(repo).download(&file)?;
    Ok(())
  }
}
