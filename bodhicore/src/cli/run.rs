use super::CliError;
#[cfg(not(test))]
use crate::interactive::InteractiveRuntime;
#[cfg(test)]
use crate::test_utils::MockInteractiveRuntime as InteractiveRuntime;
use crate::{error::BodhiError, service::AppServiceFn, Command, PullCommand};
use std::sync::Arc;
pub enum RunCommand {
  WithAlias { alias: String },
}

impl TryFrom<Command> for RunCommand {
  type Error = CliError;

  fn try_from(value: Command) -> std::result::Result<Self, Self::Error> {
    match value {
      Command::Run { alias } => Ok(RunCommand::WithAlias { alias }),
      cmd => Err(CliError::ConvertCommand(cmd.to_string(), "run".to_string())),
    }
  }
}

impl RunCommand {
  #[allow(clippy::result_large_err)]
  pub fn execute(self, service: Arc<dyn AppServiceFn>) -> crate::error::Result<()> {
    match self {
      RunCommand::WithAlias { alias } => {
        let alias = match service.find_alias(&alias) {
          Some(alias_obj) => alias_obj,
          None => match service.find_remote_model(&alias)? {
            Some(remote_model) => {
              let command = PullCommand::ByAlias {
                alias: remote_model.alias.clone(),
                force: false,
              };
              command.execute(service.clone())?;
              match service.find_alias(&alias) {
                Some(alias_obj) => alias_obj,
                None => return Err(BodhiError::AliasNotFound(alias)),
              }
            }
            None => return Err(BodhiError::AliasNotFound(alias)),
          },
        };
        InteractiveRuntime::new().execute(alias, service)?;
        Ok(())
      }
    }
  }
}

#[cfg(test)]
mod test {
  use crate::{
    objs::{Alias, HubFile, RemoteModel},
    test_utils::{MockAppService, MockInteractiveRuntime},
    Repo, RunCommand,
  };
  use mockall::predicate::{always, eq};
  use rstest::rstest;
  use std::{path::PathBuf, sync::Arc};

  #[rstest]
  fn test_run_with_alias_return_error_if_alias_not_found() -> anyhow::Result<()> {
    let run_command = RunCommand::WithAlias {
      alias: "testalias:instruct".to_string(),
    };
    let mut mock = MockAppService::default();
    mock
      .expect_find_alias()
      .with(eq("testalias:instruct"))
      .return_once(|_| None);
    mock
      .expect_find_remote_model()
      .with(eq("testalias:instruct"))
      .return_once(|_| Ok(None));
    let result = run_command.execute(Arc::new(mock));
    assert!(result.is_err());
    assert_eq!(
      r#"model alias 'testalias:instruct' not found in pre-configured model aliases.
Run `bodhi list -r` to see list of pre-configured model aliases
"#,
      result.unwrap_err().to_string()
    );
    Ok(())
  }

  #[rstest]
  fn test_run_with_alias_downloads_a_known_alias_if_not_configured() -> anyhow::Result<()> {
    let run_command = RunCommand::WithAlias {
      alias: "testalias:instruct".to_string(),
    };
    let mut mock = MockAppService::default();
    mock
      .expect_find_alias()
      .with(eq("testalias:instruct"))
      .times(2)
      .returning(|_| None);
    mock
      .expect_find_remote_model()
      .with(eq("testalias:instruct"))
      .times(2)
      .returning(|_| Ok(Some(RemoteModel::testalias())));
    mock
      .expect_download()
      .with(
        eq(Repo::try_from("MyFactory/testalias-gguf")?),
        eq("testalias.Q8_0.gguf"),
        eq(false),
      )
      .return_once(|_, _, _| Ok(HubFile::testalias()));
    mock
      .expect_save_alias()
      .with(eq(Alias::testalias()))
      .return_once(|_| Ok(PathBuf::from("ignore")));
    mock
      .expect_find_alias()
      .with(eq("testalias:instruct"))
      .return_once(|_| Some(Alias::testalias()));
    let mut mock_interactive = MockInteractiveRuntime::default();
    mock_interactive
      .expect_execute()
      .with(eq(Alias::testalias()), always())
      .return_once(|_, _| Ok(()));
    mock
      .expect_fmt()
      .with(always())
      .returning(|f| f.debug_struct("MockAppService").finish());
    let ctx = MockInteractiveRuntime::new_context();
    ctx.expect().return_once(move || mock_interactive);
    run_command.execute(Arc::new(mock))?;
    Ok(())
  }
}