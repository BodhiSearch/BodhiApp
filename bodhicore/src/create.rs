use crate::{
  cli::{GptContextParams, OAIRequestParams},
  error::{AppError, Result},
  objs::{default_features, Alias, ChatTemplate, Repo, TOKENIZER_CONFIG_JSON},
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
                Some(tokenizer_config) => {
                  ChatTemplate::Repo(Repo::new(tokenizer_config))
                },
                None => return Err(AppError::BadRequest("one of chat_template or tokenizer_config is required".to_string())),
            },
        };
        let result = CreateCommand {alias, repo: Repo::new(repo), filename, chat_template, family, force, oai_request_params, context_params };
        Ok(result)
      }
      _ => Err(AppError::BadRequest(format!(
        "{value:?} cannot be converted into CreateCommand, only `Command::Create` variant supported."
      ))),
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
      force,
      oai_request_params,
      context_params,
    } = value;
    Alias::new(
      alias,
      family,
      Some(repo),
      Some(filename),
      default_features(),
      chat_template,
    )
  }
}

impl CreateCommand {
  pub fn execute(self, service: &dyn AppServiceFn) -> Result<()> {
    if !self.force && service.find_alias(&self.alias).is_some() {
      return Err(AppError::AliasExists(self.alias.clone()));
    }
    service.download(&self.repo, &self.filename, self.force)?;
    if let ChatTemplate::Repo(repo) = &self.chat_template {
      service.download(repo.value.as_ref(), TOKENIZER_CONFIG_JSON, true)?;
    }
    let alias: Alias = self.into();
    service.save_alias(alias)?;
    Ok(())
  }
}

#[cfg(test)]
mod test {
  use super::CreateCommand;
  use crate::{
    cli::{ChatTemplateId, Cli, GptContextParams, OAIRequestParams},
    objs::{Alias, ChatTemplate, Repo},
    test_utils::{mock_app_service, MockAppServiceFn},
  };
  use anyhow_trace::anyhow_trace;
  use clap::Parser;
  use mockall::predicate::eq;
  use rstest::rstest;
  use std::path::PathBuf;

  #[rstest]
  #[case(vec![
    "bodhi", "create",
    "testalias:instruct",
    "--repo", "MyFactory/testalias-gguf",
    "--filename", "testalias.Q8_0.gguf",
    "--family", "testalias",
    "--chat-template", "llama3",
  ], ChatTemplate::Id(ChatTemplateId::Llama3))]
  #[case(vec![
    "bodhi", "create",
    "testalias:instruct",
    "--repo", "MyFactory/testalias-gguf",
    "--filename", "testalias.Q8_0.gguf",
    "--family", "testalias",
    "--tokenizer-config", "MyFactory/testalias",
  ], ChatTemplate::Repo(Repo::new("MyFactory/testalias".to_string())))]
  fn test_create_try_from_valid(
    #[case] args: Vec<&str>,
    #[case] chat_template: ChatTemplate,
  ) -> anyhow::Result<()> {
    let command = Cli::try_parse_from(args)?.command;
    let actual: CreateCommand = command.try_into()?;
    let expected = CreateCommand {
      alias: "testalias:instruct".to_string(),
      repo: Repo::new("MyFactory/testalias-gguf".to_string()),
      filename: "testalias.Q8_0.gguf".to_string(),
      chat_template,
      family: Some("testalias".to_string()),
      force: false,
      oai_request_params: OAIRequestParams::default(),
      context_params: GptContextParams::default(),
    };
    assert_eq!(expected, actual);
    Ok(())
  }

  #[rstest]
  #[case(vec!["bodhi", "pull",
  "--repo", "MyFactory/testalias-gguf",
  "--filename", "testalias.Q8_0.gguf",
  ], "Pull { alias: None, repo: Some(\"MyFactory/testalias-gguf\"), filename: Some(\"testalias.Q8_0.gguf\"), force: false } cannot be converted into CreateCommand, only `Command::Create` variant supported.")]
  #[anyhow_trace]
  fn test_create_try_from_invalid(
    #[case] args: Vec<&str>,
    #[case] message: String,
  ) -> anyhow::Result<()> {
    let command = Cli::try_parse_from(args)?.command;
    let actual = CreateCommand::try_from(command);
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
      repo: Repo::new("MyFactory/testalias-gguf".to_string()),
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
  fn test_create_execute(mock_app_service: MockAppServiceFn) -> anyhow::Result<()> {
    let mut mock = mock_app_service;
    let create = CreateCommand {
      alias: "testalias:instruct".to_string(),
      repo: Repo::new("MyFactory/testalias-gguf".to_string()),
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
      .return_once(|_, _, _| Ok(PathBuf::from(".")));
    let alias = Alias::new(
      "testalias:instruct".to_string(),
      None,
      Some(Repo::new("MyFactory/testalias-gguf".to_string())),
      Some("testalias.Q8_0.gguf".to_string()),
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
  fn test_create_execute_with_tokenizer_config(
    mock_app_service: MockAppServiceFn,
  ) -> anyhow::Result<()> {
    let mut mock = mock_app_service;
    let create = CreateCommand {
      alias: "testalias:instruct".to_string(),
      repo: Repo::new("MyFactory/testalias-gguf".to_string()),
      filename: "testalias.Q8_0.gguf".to_string(),
      chat_template: ChatTemplate::Repo(Repo::new("MyFactory/testalias".to_string())),
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
      .return_once(|_, _, _| Ok(PathBuf::from(".")));
    mock
      .hub_service
      .expect_download()
      .with(
        eq("MyFactory/testalias"),
        eq("tokenizer_config.json"),
        eq(true),
      )
      .return_once(|_, _, _| Ok(PathBuf::from(".")));
    let alias = Alias::new(
      "testalias:instruct".to_string(),
      None,
      Some(Repo::new("MyFactory/testalias-gguf".to_string())),
      Some("testalias.Q8_0.gguf".to_string()),
      vec!["chat".to_string()],
      ChatTemplate::Repo(Repo::new("MyFactory/testalias".to_string())),
    );
    mock
      .data_service
      .expect_save_alias()
      .with(eq(alias))
      .return_once(|_| Ok(PathBuf::from(".")));
    create.execute(&mock)?;
    Ok(())
  }
}
