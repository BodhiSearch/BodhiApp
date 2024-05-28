use crate::{hf::model_file, interactive::launch_interactive, list::find_remote_model};
use anyhow::bail;

pub enum Run {
  WithId { id: String },
  WithRepo { repo: String, filename: String },
}

impl Run {
  pub fn execute(self) -> anyhow::Result<()> {
    let (repo, filename) = match self {
      Run::WithId { id } => {
        let Some(model) = find_remote_model(&id) else {
          bail!(
            "model with id {} not found in pre-configured remote models",
            id
          );
        };
        (model.repo, model.filename)
      }
      Run::WithRepo { repo, filename } => (repo, filename),
    };
    let model_file = match model_file(&repo, &filename) {
      None => {
        // download(&repo, &filename, true)?
        todo!()
      }
      Some(path) => path,
    };
    launch_interactive(&repo, &model_file)?;
    Ok(())
  }
}
