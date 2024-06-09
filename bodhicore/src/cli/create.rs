use super::{CliError, Command};
use crate::{
  error::{BodhiError, Result},
  objs::{
    default_features, Alias, ChatTemplate, GptContextParams, OAIRequestParams, Repo,
    TOKENIZER_CONFIG_JSON,
  },
  service::AppServiceFn,
};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(test, derive(derive_new::new, derive_builder::Builder))]
#[allow(clippy::too_many_arguments)]
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
  type Error = CliError;

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
            Some(tokenizer_config) => ChatTemplate::Repo(Repo::try_from(tokenizer_config)?),
            None => {
              return Err(CliError::BadRequest(format!(
                "cannot initialize create command with invalid state. chat_template: '{chat_template:?}', tokenizer_config: '{tokenizer_config:?}'"
              )))
            }
          },
        };
        let result = CreateCommand {
          alias,
          repo: Repo::try_from(repo)?,
          filename,
          chat_template,
          family,
          force,
          oai_request_params,
          context_params,
        };
        Ok(result)
      }
      cmd => Err(CliError::ConvertCommand(
        cmd.to_string(),
        "create".to_string(),
      )),
    }
  }
}

impl CreateCommand {
  #[allow(clippy::result_large_err)]
  pub fn execute(self, service: Arc<dyn AppServiceFn>) -> Result<()> {
    if !self.force && service.find_alias(&self.alias).is_some() {
      return Err(BodhiError::AliasExists(self.alias.clone()));
    }
    let local_model_file = service.download(&self.repo, &self.filename, self.force)?;
    if let ChatTemplate::Repo(repo) = &self.chat_template {
      service.download(repo, TOKENIZER_CONFIG_JSON, true)?;
    }
    let alias: Alias = Alias::new(
      self.alias,
      self.family,
      self.repo,
      self.filename,
      local_model_file.snapshot.clone(),
      default_features(),
      self.chat_template,
      self.oai_request_params,
      self.context_params,
    );
    service.save_alias(alias)?;
    Ok(())
  }
}

#[cfg(test)]
mod test {
  use super::CreateCommand;
  use crate::{
    cli::Command,
    objs::{
      Alias, ChatTemplate, ChatTemplateId, GptContextParams, HubFile, OAIRequestParams, Repo,
      TOKENIZER_CONFIG_JSON,
    },
    test_utils::MockAppService,
  };
  use anyhow_trace::anyhow_trace;
  use mockall::predicate::eq;
  use rstest::rstest;
  use std::{path::PathBuf, sync::Arc};

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
    repo: Repo::try_from("MyFactory/testalias-gguf".to_string())?,
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
  fn test_create_execute_fails_if_exists_force_false() -> anyhow::Result<()> {
    let create = CreateCommand {
      alias: "testalias:instruct".to_string(),
      repo: Repo::try_from("MyFactory/testalias-gguf".to_string())?,
      filename: "testalias.Q8_0.gguf".to_string(),
      chat_template: ChatTemplate::Id(ChatTemplateId::Llama3),
      family: None,
      force: false,
      oai_request_params: OAIRequestParams::default(),
      context_params: GptContextParams::default(),
    };
    let mut mock = MockAppService::default();
    mock
      .expect_find_alias()
      .with(eq("testalias:instruct"))
      .return_once(|_| {
        let alias = Alias {
          alias: "testalias:instruct".to_string(),
          ..Alias::default()
        };
        Some(alias)
      });
    let result = create.execute(Arc::new(mock));
    assert!(result.is_err());
    assert_eq!(
      "model alias 'testalias:instruct' already exists. Use --force to overwrite the model alias config",
      result.unwrap_err().to_string()
    );
    Ok(())
  }

  #[rstest]
  fn test_create_execute_downloads_model_saves_alias() -> anyhow::Result<()> {
    let create = CreateCommand::testalias();
    let mut mock = MockAppService::default();
    mock
      .expect_find_alias()
      .with(eq(create.alias.clone()))
      .return_once(|_| None);
    mock
      .expect_download()
      .with(
        eq(create.repo.clone()),
        eq(create.filename.clone()),
        eq(false),
      )
      .return_once(|_, _, _| Ok(HubFile::testalias()));
    let alias = Alias::testalias();
    mock
      .expect_save_alias()
      .with(eq(alias))
      .return_once(|_| Ok(PathBuf::from(".")));
    create.execute(Arc::new(mock))?;
    Ok(())
  }

  #[rstest]
  fn test_create_execute_with_tokenizer_config_downloads_tokenizer_saves_alias(
  ) -> anyhow::Result<()> {
    let tokenizer_repo = Repo::try_from("MyFactory/testalias")?;
    let chat_template = ChatTemplate::Repo(tokenizer_repo.clone());
    let create = CreateCommand::testalias_builder()
      .chat_template(chat_template.clone())
      .build()
      .unwrap();
    let mut mock = MockAppService::default();
    mock
      .expect_find_alias()
      .with(eq(create.alias.clone()))
      .return_once(|_| None);
    mock
      .expect_download()
      .with(
        eq(create.repo.clone()),
        eq(create.filename.clone()),
        eq(false),
      )
      .return_once(|_, _, _| Ok(HubFile::testalias()));
    mock
      .expect_download()
      .with(eq(tokenizer_repo), eq(TOKENIZER_CONFIG_JSON), eq(true))
      .return_once(|_, _, _| Ok(HubFile::testalias_tokenizer()));
    let alias = Alias::test_alias_instruct_builder()
      .chat_template(chat_template.clone())
      .build()
      .unwrap();
    mock
      .expect_save_alias()
      .with(eq(alias))
      .return_once(|_| Ok(PathBuf::from("ignored")));
    create.execute(Arc::new(mock))?;
    Ok(())
  }
}
