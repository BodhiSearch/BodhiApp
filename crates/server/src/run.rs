#[cfg(not(test))]
use crate::interactive::InteractiveRuntime;
#[cfg(test)]
use crate::test_utils::MockInteractiveRuntime as InteractiveRuntime;
use crate::InteractiveError;
use commands::{PullCommand, PullCommandError};
use services::{AppService, DataServiceError};
use std::sync::Arc;

pub enum RunCommand {
  WithAlias { alias: String },
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
  use objs::{
    test_utils::SNAPSHOT, Alias, ChatTemplate, ChatTemplateId, GptContextParams, HubFile,
    OAIRequestParams, Repo, TOKENIZER_CONFIG_JSON,
  };
  use rstest::rstest;
  use services::{test_utils::AppServiceStubBuilder, AppService, MockHubService};
  use std::sync::Arc;

  #[rstest]
  #[tokio::test]
  async fn test_run_with_alias_return_error_if_alias_not_found() -> anyhow::Result<()> {
    let run_command = RunCommand::WithAlias {
      alias: "testalias:notexists".to_string(),
    };
    let service = AppServiceStubBuilder::default()
      .with_data_service()
      .build()?;
    let result = run_command.aexecute(Arc::new(service)).await;
    assert!(result.is_err());
    assert_eq!(
      r#"model alias 'testalias:notexists' not found in pre-configured model aliases.
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
    let mut mock_hub_service = MockHubService::new();
    mock_hub_service
      .expect_find_local_file()
      .with(eq(Repo::testalias()), eq("testalias.Q8_0.gguf"), eq(None))
      .return_once(|_, _, _| Ok(None));
    mock_hub_service
      .expect_download()
      .with(eq(Repo::testalias()), eq("testalias.Q8_0.gguf"), eq(None))
      .return_once(|_, _, _| Ok(HubFile::testalias()));
    mock_hub_service
      .expect_find_local_file()
      .with(eq(Repo::llama3()), eq(TOKENIZER_CONFIG_JSON), eq(None))
      .return_once(|_, _, _| Ok(None));
    mock_hub_service
      .expect_download()
      .with(eq(Repo::llama3()), eq(TOKENIZER_CONFIG_JSON), eq(None))
      .return_once(|_, _, _| Ok(HubFile::llama3_tokenizer()));
    let mut mock_interactive = MockInteractiveRuntime::default();
    mock_interactive
      .expect_execute()
      .with(eq(Alias::testalias()), always())
      .return_once(|_, _| Ok(()));
    let service = Arc::new(
      AppServiceStubBuilder::default()
        .with_data_service()
        .hub_service(Arc::new(mock_hub_service))
        .build()?,
    );
    let ctx = MockInteractiveRuntime::new_context();
    ctx.expect().return_once(move || mock_interactive);
    run_command.aexecute(service.clone()).await?;
    let created = service
      .data_service()
      .find_alias("testalias:instruct")
      .unwrap();
    assert_eq!(
      Alias {
        alias: "testalias:instruct".to_string(),
        family: Some("testalias".to_string()),
        repo: Repo::testalias(),
        filename: "testalias.Q8_0.gguf".to_string(),
        snapshot: SNAPSHOT.to_string(),
        features: vec!["chat".to_string()],
        chat_template: ChatTemplate::Id(ChatTemplateId::Llama3),
        request_params: OAIRequestParams::default(),
        context_params: GptContextParams::default(),
      },
      created
    );
    Ok(())
  }
}
