use crate::{CmdIntoError, Command};
use objs::{
  default_features, Alias, ChatTemplate, GptContextParams, OAIRequestParams, ObjError, Repo,
  REFS_MAIN, TOKENIZER_CONFIG_JSON,
};
use services::{AppService, DataServiceError, HubServiceError};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, derive_builder::Builder)]
#[allow(clippy::too_many_arguments)]
pub struct CreateCommand {
  pub alias: String,
  pub repo: Repo,
  pub filename: String,
  // TODO(support snapshot): have snapshot as an option
  pub chat_template: ChatTemplate,
  pub family: Option<String>,
  #[builder(default = "true")]
  pub auto_download: bool,
  #[builder(default = "false")]
  pub update: bool,
  pub oai_request_params: OAIRequestParams,
  pub context_params: GptContextParams,
}

#[derive(Debug, thiserror::Error)]
pub enum CreateCommandError {
  #[error("model alias '{0}' already exists")]
  AliasExists(String),
  #[error("model file '{filename}' not found in repo '{repo}'")]
  ModelFileMissing { filename: String, repo: String },
  #[error(transparent)]
  ObjError(#[from] ObjError),
  #[error(transparent)]
  HubServiceError(#[from] HubServiceError),
  #[error(transparent)]
  DataServiceError(#[from] DataServiceError),
}

type Result<T> = std::result::Result<T, CreateCommandError>;

impl TryFrom<Command> for CreateCommand {
  type Error = CmdIntoError;

  fn try_from(value: Command) -> std::result::Result<Self, Self::Error> {
    match value {
      Command::Create {
        alias,
        repo,
        filename,
        chat_template,
        tokenizer_config,
        family,
        update,
        oai_request_params,
        context_params,
      } => {
        let chat_template = match chat_template {
          Some(chat_template) => ChatTemplate::Id(chat_template),
          None => match tokenizer_config {
            Some(tokenizer_config) => match Repo::try_from(tokenizer_config) {
              Ok(repo) => ChatTemplate::Repo(repo),
              Err(err) => Err(CmdIntoError::BadRequest {
                input: "create".to_string(),
                output: "CreateCommand".to_string(),
                error: format!("tokenizer_config repo {err}"),
              })?,
            },
            None => {
              return Err(CmdIntoError::BadRequest {
                input: "create".to_string(),
                output: "CreateCommand".to_string(),
                error: "one of chat_template and tokenizer_config must be provided".to_string(),
              })
            }
          },
        };
        let repo = Repo::try_from(repo).map_err(|err| CmdIntoError::BadRequest {
          input: "create".to_string(),
          output: "CreateCommand".to_string(),
          error: format!("model repo {err}"),
        })?;
        let result = CreateCommand {
          alias,
          repo,
          filename,
          chat_template,
          family,
          auto_download: true,
          update,
          oai_request_params,
          context_params,
        };
        Ok(result)
      }
      cmd => Err(CmdIntoError::Convert {
        input: cmd.to_string(),
        output: "CreateCommand".to_string(),
      }),
    }
  }
}

impl CreateCommand {
  #[allow(clippy::result_large_err)]
  pub fn execute(self, service: Arc<dyn AppService>) -> Result<()> {
    if service.data_service().find_alias(&self.alias).is_some() {
      if !self.update {
        return Err(CreateCommandError::AliasExists(self.alias.clone()));
      }
      println!("Updating existing alias: '{}'", self.alias);
    } else {
      println!("Creating new alias: '{}'", self.alias);
    }
    let local_model_file =
      service
        .hub_service()
        .find_local_file(&self.repo, &self.filename, REFS_MAIN)?;
    let local_model_file = match local_model_file {
      Some(local_model_file) => {
        println!(
          "repo: '{}', filename: '{}' already exists in $HF_HOME",
          &self.repo, &self.filename
        );
        local_model_file
      }
      None => {
        if self.auto_download {
          service
            .hub_service()
            // TODO(support snapshot): have snapshot as an option
            .download(&self.repo, &self.filename, None)?
        } else {
          return Err(CreateCommandError::ModelFileMissing {
            filename: self.filename.clone(),
            repo: self.repo.clone().to_string(),
          });
        }
      }
    };
    let chat_template_repo = Repo::try_from(self.chat_template.clone())?;
    let tokenizer_file = service.hub_service().find_local_file(
      &chat_template_repo,
      TOKENIZER_CONFIG_JSON,
      REFS_MAIN,
    )?;
    match tokenizer_file {
      Some(_) => {
        println!(
          "tokenizer from repo: '{}', filename: '{}' already exists in $HF_HOME",
          &self.repo, &self.filename
        );
      }
      _ => {
        service
          .hub_service()
          .download(&chat_template_repo, TOKENIZER_CONFIG_JSON, None)?;
        println!(
          "tokenizer from repo: '{}', filename: '{}' downloaded into $HF_HOME",
          &self.repo, &self.filename
        );
      }
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
    service.data_service().save_alias(&alias)?;
    println!(
      "model alias: '{}' saved to $BODHI_HOME/aliases",
      alias.alias
    );
    Ok(())
  }
}

#[cfg(test)]
mod test {
  use crate::{Command, CreateCommand};
  use anyhow_trace::anyhow_trace;
  use mockall::predicate::*;
  use objs::{
    Alias, ChatTemplate, ChatTemplateId, GptContextParams, GptContextParamsBuilder, HubFile,
    OAIRequestParams, OAIRequestParamsBuilder, Repo, REFS_MAIN, TOKENIZER_CONFIG_JSON,
  };
  use rstest::rstest;
  use services::{
    test_utils::{AppServiceStubBuilder, AppServiceStubMock, AppServiceStubMockBuilder},
    AppService, MockDataService, MockHubService,
  };
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
    update: true,
    oai_request_params: OAIRequestParams::default(),
    context_params: GptContextParams::default(),
  },
  CreateCommand {
    alias: "testalias:instruct".to_string(),
    repo: Repo::try_from("MyFactory/testalias-gguf".to_string())?,
    filename: "testalias.Q8_0.gguf".to_string(),
    chat_template: ChatTemplate::Id(ChatTemplateId::Llama3),
    family: Some("testalias".to_string()),
    auto_download: true,
    update: true,
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
  #[case(
    Command::App {ui: false},
    "Command 'app' cannot be converted into command 'CreateCommand'"
  )]
  #[case(
    Command::Create {
      alias: "test".to_string(),
      repo: "valid/repo".to_string(),
      filename: "model.gguf".to_string(),
      chat_template: None,
      tokenizer_config: Some("invalid-chat/repo/pattern".to_string()),
      family: None,
      update: false,
      oai_request_params: OAIRequestParams::default(),
      context_params: GptContextParams::default(),
    },
    "Command 'create' cannot be converted into command 'CreateCommand', error: 'tokenizer_config repo Validation failed: value: does not match the huggingface repo pattern 'username/repo''"
  )]
  #[case(
    Command::Create {
      alias: "test".to_string(),
      repo: "invalid-repo".to_string(),
      filename: "model.gguf".to_string(),
      chat_template: Some(ChatTemplateId::Llama3),
      tokenizer_config: None,
      family: None,
      update: false,
      oai_request_params: OAIRequestParams::default(),
      context_params: GptContextParams::default(),
    },
    "Command 'create' cannot be converted into command 'CreateCommand', error: 'model repo Validation failed: value: does not match the huggingface repo pattern 'username/repo''"
  )]
  #[case(
    Command::Create {
      alias: "test".to_string(),
      repo: "invalid-repo".to_string(),
      filename: "model.gguf".to_string(),
      chat_template: None,
      tokenizer_config: None,
      family: None,
      update: false,
      oai_request_params: OAIRequestParams::default(),
      context_params: GptContextParams::default(),
    },
    "Command 'create' cannot be converted into command 'CreateCommand', error: 'one of chat_template and tokenizer_config must be provided'"
  )]
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
  fn test_create_execute_updates_if_exists() -> anyhow::Result<()> {
    let update_alias = CreateCommand {
      alias: "tinyllama:instruct".to_string(),
      repo: Repo::try_from("TheBloke/TinyLlama-1.1B-Chat-v0.3-GGUF".to_string())?,
      filename: "tinyllama-1.1b-chat-v0.3.Q2_K.gguf".to_string(),
      chat_template: ChatTemplate::Repo(Repo::try_from("TinyLlama/TinyLlama-1.1B-Chat-v1.0")?),
      family: Some("tinyllama".to_string()),
      auto_download: false,
      update: true,
      oai_request_params: OAIRequestParamsBuilder::default()
        .frequency_penalty(1.0)
        .max_tokens(2048 as u16)
        .build()
        .unwrap(),
      context_params: GptContextParamsBuilder::default()
        .n_ctx(2048)
        .n_keep(2048)
        .n_parallel(2)
        .n_seed(42 as u32)
        .n_threads(8 as u32)
        .build()
        .unwrap(),
    };
    let service = Arc::new(
      AppServiceStubBuilder::default()
        .with_data_service()
        .with_hub_service()
        .build()?,
    );
    let repo_alias = service
      .data_service()
      .find_alias("tinyllama:instruct")
      .unwrap();
    let result = update_alias.execute(service.clone());
    assert!(result.is_ok());
    let updated_alias = service
      .data_service()
      .find_alias("tinyllama:instruct")
      .unwrap();
    assert_ne!(repo_alias, updated_alias);
    let expected = Alias {
      alias: "tinyllama:instruct".to_string(),
      family: Some("tinyllama".to_string()),
      repo: Repo::try_from("TheBloke/TinyLlama-1.1B-Chat-v0.3-GGUF".to_string())?,
      filename: "tinyllama-1.1b-chat-v0.3.Q2_K.gguf".to_string(),
      snapshot: "b32046744d93031a26c8e925de2c8932c305f7b9".to_string(),
      features: vec!["chat".to_string()],
      chat_template: ChatTemplate::Repo(Repo::try_from("TinyLlama/TinyLlama-1.1B-Chat-v1.0")?),
      request_params: OAIRequestParamsBuilder::default()
        .frequency_penalty(1.0)
        .max_tokens(2048 as u16)
        .build()
        .unwrap(),
      context_params: GptContextParamsBuilder::default()
        .n_ctx(2048)
        .n_keep(2048)
        .n_parallel(2)
        .n_seed(42 as u32)
        .n_threads(8 as u32)
        .build()
        .unwrap(),
    };
    assert_eq!(expected, updated_alias);
    Ok(())
  }

  #[rstest]
  fn test_create_execute_downloads_model_saves_alias() -> anyhow::Result<()> {
    let create = CreateCommand::testalias();
    let mut mock_data_service = MockDataService::default();
    mock_data_service
      .expect_find_alias()
      .with(eq(create.alias.clone()))
      .return_once(|_| None);
    let mut mock_hub_service = MockHubService::default();
    mock_hub_service
      .expect_find_local_file()
      .with(
        eq(create.repo.clone()),
        eq(create.filename.clone()),
        eq(REFS_MAIN),
      )
      .return_once(|_, _, _| Ok(None));
    mock_hub_service
      .expect_download()
      .with(
        eq(create.repo.clone()),
        eq(create.filename.clone()),
        eq(None)
      )
      .return_once(|_, _, _| Ok(HubFile::testalias()));
    mock_hub_service
      .expect_find_local_file()
      .with(eq(Repo::llama3()), eq(TOKENIZER_CONFIG_JSON), eq(REFS_MAIN))
      .return_once(|_, _, _| Ok(Some(HubFile::llama3_tokenizer())));
    let alias = Alias::testalias();
    mock_data_service
      .expect_save_alias()
      .with(eq(alias))
      .return_once(|_| Ok(PathBuf::from(".")));
    let service = AppServiceStubMockBuilder::default()
      .hub_service(mock_hub_service)
      .data_service(mock_data_service)
      .build()?;
    create.execute(Arc::new(service))?;
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
    let mut mock_data_service = MockDataService::default();
    mock_data_service
      .expect_find_alias()
      .with(eq(create.alias.clone()))
      .return_once(|_| None);
    let mut mock_hub_service = MockHubService::new();
    mock_hub_service
      .expect_find_local_file()
      .with(
        eq(create.repo.clone()),
        eq(create.filename.clone()),
        eq(REFS_MAIN),
      )
      .return_once(|_, _, _| Ok(None));
    mock_hub_service
      .expect_download()
      .with(
        eq(create.repo.clone()),
        eq(create.filename.clone()),
        eq(None)
      )
      .return_once(|_, _, _| Ok(HubFile::testalias()));
    mock_hub_service
      .expect_find_local_file()
      .with(
        eq(tokenizer_repo.clone()),
        eq(TOKENIZER_CONFIG_JSON),
        eq(REFS_MAIN),
      )
      .return_once(|_, _, _| Ok(None));
    mock_hub_service
      .expect_download()
      .with(
        eq(tokenizer_repo.clone()),
        eq(TOKENIZER_CONFIG_JSON),
        eq(None)
      )
      .return_once(|_, _, _| Ok(HubFile::testalias_tokenizer()));
    let alias = Alias::test_alias_instruct_builder()
      .chat_template(chat_template.clone())
      .build()
      .unwrap();
    mock_data_service
      .expect_save_alias()
      .with(eq(alias))
      .return_once(|_| Ok(PathBuf::from("ignored")));
    let service = AppServiceStubMock::builder()
      .hub_service(mock_hub_service)
      .data_service(mock_data_service)
      .build()?;
    create.execute(Arc::new(service))?;
    Ok(())
  }
}
