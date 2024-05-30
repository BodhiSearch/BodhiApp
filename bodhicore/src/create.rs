use crate::{
  cli::GptContextParams,
  error::{AppError, Result},
  objs::{default_features, Alias, ChatTemplate, OAIRequestParams, Repo, TOKENIZER_CONFIG_JSON},
  service::AppServiceFn,
  Command,
};

#[derive(Debug, Clone, PartialEq)]
pub struct CreateCommand {
  alias: String,
  repo: Repo,
  filename: String,
  chat_template: ChatTemplate,
  family: Option<String>,
  force: bool,
  oai_request_params: OAIRequestParams,
  context_params: GptContextParams,
}

impl TryFrom<Command> for CreateCommand {
  type Error = AppError;

  fn try_from(value: Command) -> std::result::Result<Self, Self::Error> {
    match value {
      Command::Create {
        alias,
        repo,
        filename,
        chat_template,
        tokenizer_config,
        family,
        force,
        oai_request_params,
        context_params,
      } => {
        let chat_template = match chat_template {
          Some(chat_template) => ChatTemplate::Id(chat_template),
          None => match tokenizer_config {
            Some(tokenizer_config) => ChatTemplate::Repo(Repo::try_new(tokenizer_config)?),
            None => {
              return Err(AppError::BadRequest(format!(
                "cannot initialize create command with invalid state. chat_template: '{chat_template:?}', tokenizer_config: '{tokenizer_config:?}'"
              )))
            }
          },
        };
        let result = CreateCommand {
          alias,
          repo: Repo::try_new(repo)?,
          filename,
          chat_template,
          family,
          force,
          oai_request_params,
          context_params,
        };
        Ok(result)
      }
      cmd => Err(AppError::ConvertCommand(cmd, "create".to_string())),
    }
  }
}

impl From<CreateCommand> for Alias {
  fn from(value: CreateCommand) -> Self {
    let CreateCommand {
      alias,
      repo,
      filename,
      chat_template,
      family,
      force: _force,
      oai_request_params,
      context_params,
    } = value;
    Alias::new(
      alias,
      family,
      repo,
      filename,
      None,
      default_features(),
      chat_template,
      // oai_request_params,
    )
  }
}

impl CreateCommand {
  #[allow(clippy::result_large_err)]
  pub fn execute(self, service: &dyn AppServiceFn) -> Result<()> {
    if !self.force && service.find_alias(&self.alias).is_some() {
      return Err(AppError::AliasExists(self.alias.clone()));
    }
    let local_model_file = service.download(&self.repo, &self.filename, self.force)?;
    if let ChatTemplate::Repo(repo) = &self.chat_template {
      service.download(repo.as_ref(), TOKENIZER_CONFIG_JSON, true)?;
    }
    let mut alias: Alias = self.into();
    alias.snapshot = Some(local_model_file.snapshot.clone());
    service.save_alias(alias)?;
    Ok(())
  }
}

#[cfg(test)]
mod test {
  use super::CreateCommand;
  use crate::{
    cli::{Command, GptContextParams},
    objs::{Alias, ChatTemplate, ChatTemplateId, LocalModelFile, OAIRequestParams, Repo},
    test_utils::{mock_app_service, MockAppServiceFn, SNAPSHOT},
  };
  use anyhow_trace::anyhow_trace;
  use mockall::predicate::eq;
  use rstest::rstest;
  use std::path::PathBuf;

  #[rstest]
  #[case(
  Command::Create {
    alias: "testalias:instruct".to_string(),
    repo: "MyFactory/testalias-gguf".to_string(),
    filename: "testalias.Q8_0.gguf".to_string(),
    chat_template: Some(ChatTemplateId::Llama3),
    tokenizer_config: None,
    family: Some("testalias".to_string()),
    force: false,
    oai_request_params: OAIRequestParams::default(),
    context_params: GptContextParams::default(),
  },
  CreateCommand {
    alias: "testalias:instruct".to_string(),
    repo: Repo::try_new("MyFactory/testalias-gguf".to_string())?,
    filename: "testalias.Q8_0.gguf".to_string(),
    chat_template: ChatTemplate::Id(ChatTemplateId::Llama3),
    family: Some("testalias".to_string()),
    force: false,
    oai_request_params: OAIRequestParams::default(),
    context_params: GptContextParams::default(),
  })]
  fn test_create_try_from_valid(
    #[case] input: Command,
    #[case] expected: CreateCommand,
  ) -> anyhow::Result<()> {
    let command = CreateCommand::try_from(input)?;
    assert_eq!(expected, command);
    Ok(())
  }

  #[rstest]
  #[case(Command::App {}, "Command 'app' cannot be converted into command 'create'")]
  #[anyhow_trace]
  fn test_create_try_from_invalid(
    #[case] input: Command,
    #[case] message: String,
  ) -> anyhow::Result<()> {
    let actual = CreateCommand::try_from(input);
    assert!(actual.is_err());
    assert_eq!(message, actual.unwrap_err().to_string());
    Ok(())
  }

  #[rstest]
  fn test_create_execute_fails_if_exists_force_false(
    mock_app_service: MockAppServiceFn,
  ) -> anyhow::Result<()> {
    let mut mock = mock_app_service;
    let create = CreateCommand {
      alias: "testalias:instruct".to_string(),
      repo: Repo::try_new("MyFactory/testalias-gguf".to_string())?,
      filename: "testalias.Q8_0.gguf".to_string(),
      chat_template: ChatTemplate::Id(ChatTemplateId::Llama3),
      family: None,
      force: false,
      oai_request_params: OAIRequestParams::default(),
      context_params: GptContextParams::default(),
    };
    mock
      .data_service
      .expect_find_alias()
      .with(eq("testalias:instruct"))
      .return_once(|_| {
        let alias = Alias {
          alias: "testalias:instruct".to_string(),
          ..Alias::default()
        };
        Some(alias)
      });
    let result = create.execute(&mock);
    assert!(result.is_err());
    assert_eq!(
      "alias 'testalias:instruct' already exists. Use --force to overwrite the alias config",
      result.unwrap_err().to_string()
    );
    Ok(())
  }

  #[rstest]
  fn test_create_execute_downloads_model_saves_alias(
    mock_app_service: MockAppServiceFn,
  ) -> anyhow::Result<()> {
    let mut mock = mock_app_service;
    let create = CreateCommand {
      alias: "testalias:instruct".to_string(),
      repo: Repo::try_new("MyFactory/testalias-gguf".to_string())?,
      filename: "testalias.Q8_0.gguf".to_string(),
      chat_template: ChatTemplate::Id(ChatTemplateId::Llama3),
      family: None,
      force: false,
      oai_request_params: OAIRequestParams::default(),
      context_params: GptContextParams::default(),
    };
    mock
      .data_service
      .expect_find_alias()
      .with(eq("testalias:instruct"))
      .return_once(|_| None);
    mock
      .hub_service
      .expect_download()
      .with(
        eq("MyFactory/testalias-gguf"),
        eq("testalias.Q8_0.gguf"),
        eq(false),
      )
      .return_once(|_, _, _| Ok(LocalModelFile::testalias()));
    let alias = Alias::new(
      "testalias:instruct".to_string(),
      None,
      Repo::try_new("MyFactory/testalias-gguf".to_string())?,
      "testalias.Q8_0.gguf".to_string(),
      Some(SNAPSHOT.to_string()),
      vec!["chat".to_string()],
      ChatTemplate::Id(ChatTemplateId::Llama3),
    );
    mock
      .data_service
      .expect_save_alias()
      .with(eq(alias))
      .return_once(|_| Ok(PathBuf::from(".")));
    create.execute(&mock)?;
    Ok(())
  }

  #[rstest]
  fn test_create_execute_with_tokenizer_config_downloads_tokenizer_saves_alias(
    mock_app_service: MockAppServiceFn,
  ) -> anyhow::Result<()> {
    let mut mock = mock_app_service;
    let create = CreateCommand {
      alias: "testalias:instruct".to_string(),
      repo: Repo::try_new("MyFactory/testalias-gguf".to_string())?,
      filename: "testalias.Q8_0.gguf".to_string(),
      chat_template: ChatTemplate::Repo(Repo::try_new("MyFactory/testalias".to_string())?),
      family: None,
      force: false,
      oai_request_params: OAIRequestParams::default(),
      context_params: GptContextParams::default(),
    };
    mock
      .data_service
      .expect_find_alias()
      .with(eq("testalias:instruct"))
      .return_once(|_| None);
    mock
      .hub_service
      .expect_download()
      .with(
        eq("MyFactory/testalias-gguf"),
        eq("testalias.Q8_0.gguf"),
        eq(false),
      )
      .return_once(|_, _, _| {
        let local_model_file = LocalModelFile::new(
          PathBuf::from("/tmp/huggingface/hub/"),
          Repo::try_new("MyFactory/testalias-gguf".to_string()).unwrap(),
          "testalias.Q8_0.gguf".to_string(),
          SNAPSHOT.to_string(),
          Some(22),
        );
        Ok(local_model_file)
      });
    mock
      .hub_service
      .expect_download()
      .with(
        eq("MyFactory/testalias"),
        eq("tokenizer_config.json"),
        eq(true),
      )
      .return_once(|_, _, _| Ok(LocalModelFile::never_download()));
    let alias = Alias::new(
      "testalias:instruct".to_string(),
      None,
      Repo::try_new("MyFactory/testalias-gguf".to_string())?,
      "testalias.Q8_0.gguf".to_string(),
      Some(SNAPSHOT.to_string()),
      vec!["chat".to_string()],
      ChatTemplate::Repo(Repo::try_new("MyFactory/testalias".to_string())?),
    );
    mock
      .data_service
      .expect_save_alias()
      .with(eq(alias))
      .return_once(|_| Ok(PathBuf::from("ignored")));
    create.execute(&mock)?;
    Ok(())
  }
}
