#[cfg(not(test))]
use crate::interactive::InteractiveRuntime;
#[cfg(test)]
use crate::test_utils::MockInteractiveRuntime as InteractiveRuntime;
use crate::InteractiveError;
use commands::{CmdIntoError, Command, PullCommand, PullCommandError};
use services::{AppService, DataServiceError};
use std::sync::Arc;

pub enum RunCommand {
  WithAlias { alias: String },
}

impl TryFrom<Command> for RunCommand {
  type Error = CmdIntoError;

  fn try_from(value: Command) -> std::result::Result<Self, Self::Error> {
    match value {
      Command::Run { alias } => Ok(RunCommand::WithAlias { alias }),
      cmd => Err(CmdIntoError::Convert {
        input: cmd.to_string(),
        output: "RunCommand".to_string(),
      }),
    }
  }
}

#[derive(Debug, thiserror::Error)]
pub enum RunCommandError {
  #[error(
    r#"model alias '{0}' not found in pre-configured model aliases.
Run `bodhi list -r` to see list of pre-configured model aliases
"#
  )]
  AliasNotFound(String),
  #[error(transparent)]
  DataServiceError(#[from] DataServiceError),
  #[error(transparent)]
  PullCommandError(#[from] PullCommandError),
  #[error(transparent)]
  InteractiveError(#[from] InteractiveError),
}

type Result<T> = std::result::Result<T, RunCommandError>;

impl RunCommand {
  #[allow(clippy::result_large_err)]
  pub async fn aexecute(self, service: Arc<dyn AppService>) -> Result<()> {
    match self {
      RunCommand::WithAlias { alias } => {
        let alias = match service.data_service().find_alias(&alias) {
          Some(alias_obj) => alias_obj,
          None => match service.data_service().find_remote_model(&alias)? {
            Some(remote_model) => {
              let command = PullCommand::ByAlias {
                alias: remote_model.alias.clone(),
                force: false,
              };
              println!(
                "downloading files to run model alias '{}'",
                remote_model.alias
              );
              command.execute(service.clone())?;
              match service.data_service().find_alias(&alias) {
                Some(alias_obj) => alias_obj,
                None => return Err(RunCommandError::AliasNotFound(alias)),
              }
            }
            None => return Err(RunCommandError::AliasNotFound(alias)),
          },
        };
        InteractiveRuntime::new().execute(alias, service).await?;
        Ok(())
      }
    }
  }
}

#[cfg(test)]
mod test {
  use crate::{test_utils::MockInteractiveRuntime, RunCommand};
  use mockall::predicate::{always, eq};
  use objs::{Alias, HubFile, RemoteModel, Repo, REFS_MAIN, TOKENIZER_CONFIG_JSON};
  use rstest::rstest;
  use services::{test_utils::AppServiceStubMock, MockDataService, MockHubService};
  use std::{path::PathBuf, sync::Arc};

  #[rstest]
  #[tokio::test]
  async fn test_run_with_alias_return_error_if_alias_not_found() -> anyhow::Result<()> {
    let run_command = RunCommand::WithAlias {
      alias: "testalias:instruct".to_string(),
    };
    let mut mock_data_service = MockDataService::new();
    mock_data_service
      .expect_find_alias()
      .with(eq("testalias:instruct"))
      .return_once(|_| None);
    mock_data_service
      .expect_find_remote_model()
      .with(eq("testalias:instruct"))
      .return_once(|_| Ok(None));
    let service = AppServiceStubMock::builder()
      .data_service(mock_data_service)
      .build()?;
    let result = run_command.aexecute(Arc::new(service)).await;
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
  #[tokio::test]
  async fn test_run_with_alias_downloads_a_known_alias_if_not_configured() -> anyhow::Result<()> {
    let run_command = RunCommand::WithAlias {
      alias: "testalias:instruct".to_string(),
    };
    let mut mock_data_service = MockDataService::default();
    mock_data_service
      .expect_find_alias()
      .with(eq("testalias:instruct"))
      .times(2)
      .returning(|_| None);
    mock_data_service
      .expect_find_remote_model()
      .with(eq("testalias:instruct"))
      .times(2)
      .returning(|_| Ok(Some(RemoteModel::testalias())));
    let mut mock_hub_service = MockHubService::new();
    mock_hub_service
      .expect_find_local_file()
      .with(
        eq(Repo::testalias()),
        eq("testalias.Q8_0.gguf"),
        eq(REFS_MAIN),
      )
      .return_once(|_, _, _| Ok(None));
    mock_hub_service
      .expect_download()
      .with(eq(Repo::testalias()), eq("testalias.Q8_0.gguf"), eq(false))
      .return_once(|_, _, _| Ok(HubFile::testalias()));

    mock_hub_service
      .expect_find_local_file()
      .with(eq(Repo::llama3()), eq(TOKENIZER_CONFIG_JSON), eq(REFS_MAIN))
      .return_once(|_, _, _| Ok(None));
    mock_hub_service
      .expect_download()
      .with(eq(Repo::llama3()), eq(TOKENIZER_CONFIG_JSON), eq(false))
      .return_once(|_, _, _| Ok(HubFile::llama3_tokenizer()));
    mock_data_service
      .expect_save_alias()
      .with(eq(Alias::testalias()))
      .return_once(|_| Ok(PathBuf::from("ignore")));
    mock_data_service
      .expect_find_alias()
      .with(eq("testalias:instruct"))
      .return_once(|_| Some(Alias::testalias()));
    let mut mock_interactive = MockInteractiveRuntime::default();
    mock_interactive
      .expect_execute()
      .with(eq(Alias::testalias()), always())
      .return_once(|_, _| Ok(()));
    let service = AppServiceStubMock::builder()
      .hub_service(mock_hub_service)
      .data_service(mock_data_service)
      .build()?;
    let ctx = MockInteractiveRuntime::new_context();
    ctx.expect().return_once(move || mock_interactive);
    run_command.aexecute(Arc::new(service)).await?;
    Ok(())
  }
}
