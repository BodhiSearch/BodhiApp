use crate::{error::Common, service::AppServiceFn, CliError, Command, StdoutWriter};
use std::{env, sync::Arc};

pub enum ManageAliasCommand {
  Show { alias: String },
  Copy { alias: String, new_alias: String },
  Edit { alias: String },
  Delete { alias: String },
}

impl TryFrom<Command> for ManageAliasCommand {
  type Error = CliError;

  fn try_from(value: Command) -> Result<Self, Self::Error> {
    match value {
      Command::Show { alias } => Ok(ManageAliasCommand::Show { alias }),
      Command::Cp { alias, new_alias } => Ok(ManageAliasCommand::Copy { alias, new_alias }),
      Command::Edit { alias } => Ok(ManageAliasCommand::Edit { alias }),
      Command::Rm { alias } => Ok(ManageAliasCommand::Delete { alias }),
      cmd => Err(CliError::ConvertCommand(
        cmd.to_string(),
        "show".to_string(),
      )),
    }
  }
}

impl ManageAliasCommand {
  pub fn execute(
    &self,
    service: Arc<dyn AppServiceFn>,
    stdout: &mut dyn StdoutWriter,
  ) -> crate::error::Result<()> {
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
    service: Arc<dyn AppServiceFn>,
    stdout: &mut dyn StdoutWriter,
  ) -> crate::error::Result<()> {
    let Some(alias) = service.data_service().find_alias(alias) else {
      return Err(crate::BodhiError::AliasNotFound(alias.to_string()));
    };
    let result = serde_yaml::to_string(&alias).map_err(Common::from)?;
    stdout.write(&result).map_err(Common::from)?;
    Ok(())
  }

  fn delete(
    &self,
    alias: &str,
    service: Arc<dyn AppServiceFn>,
    stdout: &mut dyn StdoutWriter,
  ) -> crate::error::Result<()> {
    service.data_service().delete_alias(alias)?;
    stdout
      .write(&format!("alias '{alias}' deleted.\n"))
      .map_err(Common::from)?;
    Ok(())
  }

  fn copy(
    &self,
    alias: &str,
    new_alias: &str,
    service: Arc<dyn AppServiceFn>,
    stdout: &mut dyn StdoutWriter,
  ) -> crate::error::Result<()> {
    service.data_service().copy_alias(alias, new_alias)?;
    stdout
      .write(&format!(
        "created new alias '{new_alias}' from '{alias}'.\n"
      ))
      .map_err(Common::from)?;
    Ok(())
  }

  fn edit(
    &self,
    alias: &str,
    service: Arc<dyn AppServiceFn>,
    stdout: &mut dyn StdoutWriter,
  ) -> crate::error::Result<()> {
    let filename = service.data_service().alias_filename(alias)?;
    match env::var("EDITOR") {
      Ok(editor) => {
        stdout
          .write(&format!(
            "opening file '{}' in external EDITOR '{}'.\n",
            filename.display(),
            editor
          ))
          .map_err(Common::from)?;
        std::process::Command::new(&editor)
          .arg(filename.display().to_string())
          .spawn()
          .map_err(Common::from)?
          .wait()
          .map_err(Common::from)?;
      }
      Err(_) => {
        stdout
          .write(&format!(
            "opening file '{}' in using system 'open'.\n",
            filename.display(),
          ))
          .map_err(Common::from)?;
        std::process::Command::new("open")
          .arg(filename.display().to_string())
          .spawn()
          .map_err(Common::from)?
          .wait()
          .map_err(Common::from)?;
      }
    };
    Ok(())
  }
}

#[cfg(test)]
mod test {
  use crate::{test_utils::AppServiceStubBuilder, Command, ManageAliasCommand, MockStdoutWriter};
  use mockall::predicate::eq;
  use rstest::rstest;
  use std::sync::Arc;

  #[rstest]
  fn test_manage_alias_show() -> anyhow::Result<()> {
    let service = AppServiceStubBuilder::default()
      .with_data_service()
      .build()?;
    let show = ManageAliasCommand::try_from(Command::Show {
      alias: "tinyllama:instruct".to_string(),
    })?;
    let mut mock = MockStdoutWriter::default();
    mock
      .expect_write()
      .with(eq(
        r#"alias: tinyllama:instruct
repo: TheBloke/TinyLlama-1.1B-Chat-v0.3-GGUF
filename: tinyllama-1.1b-chat-v0.3.Q2_K.gguf
snapshot: b32046744d93031a26c8e925de2c8932c305f7b9
features:
- chat
chat_template: TinyLlama/TinyLlama-1.1B-Chat-v1.0
"#,
      ))
      .return_once(|input| Ok(input.len()));
    show.execute(Arc::new(service), &mut mock)?;
    Ok(())
  }

  #[rstest]
  fn test_manage_alias_delete() -> anyhow::Result<()> {
    let service = AppServiceStubBuilder::default()
      .with_data_service()
      .build()?;
    let delete = ManageAliasCommand::try_from(Command::Rm {
      alias: "tinyllama:instruct".to_string(),
    })?;
    let mut mock = MockStdoutWriter::default();
    mock
      .expect_write()
      .with(eq("alias 'tinyllama:instruct' deleted.\n"))
      .return_once(|input| Ok(input.len()));
    delete.execute(Arc::new(service), &mut mock)?;
    Ok(())
  }

  #[rstest]
  fn test_manage_alias_copy() -> anyhow::Result<()> {
    let service = AppServiceStubBuilder::default()
      .with_data_service()
      .build()?;
    let service = Arc::new(service);
    let copy = ManageAliasCommand::try_from(Command::Cp {
      alias: "tinyllama:instruct".to_string(),
      new_alias: "tinyllama:myconfig".to_string(),
    })?;
    let mut mock = MockStdoutWriter::default();
    mock
      .expect_write()
      .with(eq(
        "created new alias 'tinyllama:myconfig' from 'tinyllama:instruct'.\n",
      ))
      .return_once(|input| Ok(input.len()));
    copy.execute(service.clone(), &mut mock)?;
    assert!(service
      .bodhi_home()
      .join("aliases")
      .join("tinyllama--myconfig.yaml")
      .exists());
    Ok(())
  }
}
