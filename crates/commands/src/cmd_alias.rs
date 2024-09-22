use crate::{CmdIntoError, Command, StdoutWriter};
use services::{AppService, DataServiceError};
use std::{env, io, sync::Arc};

#[derive(Debug, PartialEq)]
pub enum ManageAliasCommand {
  Show { alias: String },
  Copy { alias: String, new_alias: String },
  Edit { alias: String },
  Delete { alias: String },
}

impl TryFrom<Command> for ManageAliasCommand {
  type Error = CmdIntoError;

  fn try_from(value: Command) -> std::result::Result<Self, Self::Error> {
    match value {
      Command::Show { alias } => Ok(ManageAliasCommand::Show { alias }),
      Command::Cp { alias, new_alias } => Ok(ManageAliasCommand::Copy { alias, new_alias }),
      Command::Edit { alias } => Ok(ManageAliasCommand::Edit { alias }),
      Command::Rm { alias } => Ok(ManageAliasCommand::Delete { alias }),
      cmd => Err(CmdIntoError::Convert {
        input: cmd.to_string(),
        output: "alias".to_string(),
      }),
    }
  }
}

#[derive(Debug, thiserror::Error)]
pub enum AliasCommandError {
  #[error("alias '{0}' not found")]
  AliasNotFound(String),
  #[error("io_file: {source}\npath='{path}'")]
  IoFile {
    #[source]
    source: io::Error,
    path: String,
  },
  #[error(transparent)]
  SerdeYamlDeserialize(#[from] serde_yaml::Error),
  #[error(transparent)]
  DataService(#[from] DataServiceError),
  #[error("io: {0}")]
  Io(#[from] std::io::Error),
}

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
      return Err(AliasCommandError::AliasNotFound(alias.to_string()));
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
  use crate::{CmdIntoError, Command, ManageAliasCommand, MockStdoutWriter};
  use mockall::predicate::eq;
  use rstest::rstest;
  use services::test_utils::AppServiceStubBuilder;
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

  #[rstest]
  #[case::show(
      Command::Show { alias: "test_alias".to_string() },
      ManageAliasCommand::Show { alias: "test_alias".to_string() }
  )]
  #[case::copy(
      Command::Cp {
          alias: "old_alias".to_string(), 
          new_alias: "new_alias".to_string() 
      },
      ManageAliasCommand::Copy {
          alias: "old_alias".to_string(), 
          new_alias: "new_alias".to_string() 
      }
  )]
  #[case::edit(
      Command::Edit { alias: "edit_alias".to_string() },
      ManageAliasCommand::Edit { alias: "edit_alias".to_string() }
  )]
  #[case::delete(
      Command::Rm { alias: "delete_alias".to_string() },
      ManageAliasCommand::Delete { alias: "delete_alias".to_string() }
  )]
  fn test_manage_alias_command_try_from_valid(
    #[case] input: Command,
    #[case] expected: ManageAliasCommand,
  ) {
    let result = ManageAliasCommand::try_from(input);
    assert_eq!(Ok(expected), result);
  }

  #[test]
  fn test_manage_alias_command_try_from_invalid() {
    let invalid_cmd = Command::List {
      remote: false,
      models: false,
    };
    let result = ManageAliasCommand::try_from(invalid_cmd);
    assert_eq!(
      result,
      Err(CmdIntoError::Convert {
        input: "list".to_string(),
        output: "alias".to_string()
      })
    );
  }
}
