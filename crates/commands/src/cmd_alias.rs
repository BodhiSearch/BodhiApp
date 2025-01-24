use crate::StdoutWriter;
use objs::{impl_error_from, AppError, IoError, SerdeYamlError};
use services::{AliasNotFoundError, AppService, DataServiceError};
use std::{env, sync::Arc};

#[derive(Debug, PartialEq)]
pub enum ManageAliasCommand {
  Show { alias: String },
  Copy { alias: String, new_alias: String },
  Edit { alias: String },
  Delete { alias: String },
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum AliasCommandError {
  #[error(transparent)]
  AliasNotExists(#[from] AliasNotFoundError),
  #[error(transparent)]
  SerdeYamlError(#[from] SerdeYamlError),
  #[error(transparent)]
  DataService(#[from] DataServiceError),
  #[error(transparent)]
  Io(#[from] IoError),
}

impl_error_from!(::std::io::Error, AliasCommandError::Io, ::objs::IoError);
impl_error_from!(
  ::serde_yaml::Error,
  AliasCommandError::SerdeYamlError,
  ::objs::SerdeYamlError
);

type Result<T> = std::result::Result<T, AliasCommandError>;

impl ManageAliasCommand {
  pub fn execute(&self, service: Arc<dyn AppService>, stdout: &mut dyn StdoutWriter) -> Result<()> {
    match self {
      ManageAliasCommand::Show { alias } => {
        self.show(alias, service, stdout)?;
      }
      ManageAliasCommand::Copy { alias, new_alias } => {
        self.copy(alias, new_alias, service, stdout)?;
      }
      ManageAliasCommand::Edit { alias } => {
        self.edit(alias, service, stdout)?;
      }
      ManageAliasCommand::Delete { alias } => {
        self.delete(alias, service, stdout)?;
      }
    };
    Ok(())
  }

  fn show(
    &self,
    alias: &str,
    service: Arc<dyn AppService>,
    stdout: &mut dyn StdoutWriter,
  ) -> Result<()> {
    let Some(alias) = service.data_service().find_alias(alias) else {
      return Err(AliasNotFoundError(alias.to_string()).into());
    };
    let result = serde_yaml::to_string(&alias)?;
    stdout.write(&result)?;
    Ok(())
  }

  fn delete(
    &self,
    alias: &str,
    service: Arc<dyn AppService>,
    stdout: &mut dyn StdoutWriter,
  ) -> Result<()> {
    service.data_service().delete_alias(alias)?;
    stdout.write(&format!("alias '{alias}' deleted.\n"))?;
    Ok(())
  }

  fn copy(
    &self,
    alias: &str,
    new_alias: &str,
    service: Arc<dyn AppService>,
    stdout: &mut dyn StdoutWriter,
  ) -> Result<()> {
    service.data_service().copy_alias(alias, new_alias)?;
    stdout.write(&format!(
      "created new alias '{new_alias}' from '{alias}'.\n"
    ))?;
    Ok(())
  }

  fn edit(
    &self,
    alias: &str,
    service: Arc<dyn AppService>,
    stdout: &mut dyn StdoutWriter,
  ) -> Result<()> {
    let filename = service.data_service().alias_filename(alias)?;
    match env::var("EDITOR") {
      Ok(editor) => {
        stdout.write(&format!(
          "opening file '{}' in external EDITOR '{}'.\n",
          filename.display(),
          editor
        ))?;
        std::process::Command::new(&editor)
          .arg(filename.display().to_string())
          .spawn()?
          .wait()?;
      }
      Err(_) => {
        stdout.write(&format!(
          "opening file '{}' in using system 'open'.\n",
          filename.display(),
        ))?;
        std::process::Command::new("open")
          .arg(filename.display().to_string())
          .spawn()?
          .wait()?;
      }
    };
    Ok(())
  }
}

#[cfg(test)]
mod test {
  use crate::{ManageAliasCommand, MockStdoutWriter};
  use anyhow_trace::anyhow_trace;
  use mockall::predicate::eq;
  use rstest::{fixture, rstest};
  use services::{test_utils::AppServiceStubBuilder, AppService};
  use std::sync::Arc;

  #[fixture]
  fn service() -> Arc<dyn AppService> {
    Arc::new(
      AppServiceStubBuilder::default()
        .with_hub_service()
        .with_data_service()
        .build()
        .unwrap(),
    )
  }

  #[anyhow_trace]
  #[rstest]
  fn test_manage_alias_show(service: Arc<dyn AppService>) -> anyhow::Result<()> {
    let show = ManageAliasCommand::Show {
      alias: "tinyllama:instruct".to_string(),
    };
    let mut mock = MockStdoutWriter::default();
    mock
      .expect_write()
      .with(eq(
        r#"alias: tinyllama:instruct
repo: TheBloke/TinyLlama-1.1B-Chat-v0.3-GGUF
filename: tinyllama-1.1b-chat-v0.3.Q2_K.gguf
snapshot: b32046744d93031a26c8e925de2c8932c305f7b9
chat_template: TinyLlama/TinyLlama-1.1B-Chat-v1.0
"#,
      ))
      .times(1)
      .return_once(|input| Ok(input.len()));
    show.execute(service, &mut mock)?;
    Ok(())
  }

  #[anyhow_trace]
  #[rstest]
  fn test_manage_alias_delete(service: Arc<dyn AppService>) -> anyhow::Result<()> {
    let delete = ManageAliasCommand::Delete {
      alias: "tinyllama:instruct".to_string(),
    };
    let mut mock = MockStdoutWriter::default();
    mock
      .expect_write()
      .with(eq("alias 'tinyllama:instruct' deleted.\n"))
      .return_once(|input| Ok(input.len()));
    delete.execute(service, &mut mock)?;
    Ok(())
  }

  #[anyhow_trace]
  #[rstest]
  fn test_manage_alias_copy(service: Arc<dyn AppService>) -> anyhow::Result<()> {
    let copy = ManageAliasCommand::Copy {
      alias: "tinyllama:instruct".to_string(),
      new_alias: "tinyllama:myconfig".to_string(),
    };
    let mut mock = MockStdoutWriter::default();
    mock
      .expect_write()
      .with(eq(
        "created new alias 'tinyllama:myconfig' from 'tinyllama:instruct'.\n",
      ))
      .times(1)
      .return_once(|input| Ok(input.len()));
    copy.execute(service.clone(), &mut mock)?;
    assert!(service
      .data_service()
      .find_alias("tinyllama:myconfig")
      .is_some());
    Ok(())
  }
}
