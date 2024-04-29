use anyhow::{anyhow, bail};
use hf_hub::{Cache, Repo};

use crate::{interactive::launch_interactive, list::find_remote_model, pull::download};

pub struct Run {
  pub id: Option<String>,
  pub repo: Option<String>,
  pub file: Option<String>,
}

impl Run {
  pub fn new(id: Option<String>, repo: Option<String>, file: Option<String>) -> Self {
    Self { id, repo, file }
  }

  pub fn execute(self) -> anyhow::Result<()> {
    let Run { id, repo, file } = self;
    let (repo, file) = match id {
      Some(id) => {
        let Some(model) = find_remote_model(&id) else {
          bail!(
            "model with id {} not found in pre-configured remote models",
            id
          );
        };
        (model.repo_id(), model.default)
      }
      None => {
        let repo = repo.ok_or_else(|| anyhow!("required param repo is missing"))?;
        let file = file.ok_or_else(|| anyhow!("required param file is missing"))?;
        (repo, file)
      }
    };
    let model_file = match Cache::default().repo(Repo::model(repo.clone())).get(&file) {
      None => {
        download(repo, file, true)?
      }
      Some(path) => path,
    };
    launch_interactive(&model_file)?;
    Ok(())
  }
}
