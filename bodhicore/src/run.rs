use crate::{
  hf::model_file, interactive::launch_interactive, list::find_remote_model, pull::download,
};
use anyhow::bail;

pub enum Run {
  WithId { id: String },
  WithRepo { repo: String, file: String },
}

impl Run {
  pub fn execute(self) -> anyhow::Result<()> {
    let (repo, file) = match self {
      Run::WithId { id } => {
        let Some(model) = find_remote_model(&id) else {
          bail!(
            "model with id {} not found in pre-configured remote models",
            id
          );
        };
        (model.repo, model.default_variant)
      }
      Run::WithRepo { repo, file } => (repo, file),
    };
    let model_file = match model_file(&repo, &file) {
      None => download(&repo, &file, true)?,
      Some(path) => path,
    };
    launch_interactive(&repo, &model_file)?;
    Ok(())
  }
}
