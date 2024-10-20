use objs::{
  default_features, Alias, AppError, ChatTemplate, GptContextParams, OAIRequestParams,
  ObjValidationError, Repo, TOKENIZER_CONFIG_JSON,
};
use services::{
  AliasExistsError, AppService, DataServiceError, HubFileNotFoundError, HubServiceError,
  SNAPSHOT_MAIN,
};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, derive_builder::Builder)]
#[allow(clippy::too_many_arguments)]
pub struct CreateCommand {
  pub alias: String,
  pub repo: Repo,
  pub filename: String,
  pub snapshot: Option<String>,
  pub chat_template: ChatTemplate,
  pub family: Option<String>,
  #[builder(default = "true")]
  pub auto_download: bool,
  #[builder(default = "false")]
  pub update: bool,
  pub oai_request_params: OAIRequestParams,
  pub context_params: GptContextParams,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum CreateCommandError {
  #[error(transparent)]
  AliasExists(#[from] AliasExistsError),
  #[error(transparent)]
  ObjValidationError(#[from] ObjValidationError),
  #[error(transparent)]
  HubServiceError(#[from] HubServiceError),
  #[error(transparent)]
  DataServiceError(#[from] DataServiceError),
}

type Result<T> = std::result::Result<T, CreateCommandError>;

impl CreateCommand {
  #[allow(clippy::result_large_err)]
  pub fn execute(self, service: Arc<dyn AppService>) -> Result<()> {
    if service.data_service().find_alias(&self.alias).is_some() {
      if !self.update {
        return Err(AliasExistsError(self.alias.clone()).into());
      }
      println!("Updating existing alias: '{}'", self.alias);
    } else {
      println!("Creating new alias: '{}'", self.alias);
    }
    let file_exists =
      service
        .hub_service()
        .local_file_exists(&self.repo, &self.filename, self.snapshot.clone())?;
    let local_model_file = match file_exists {
      true => {
        println!(
          "repo: '{}', filename: '{}', already exists in $HF_HOME",
          &self.repo, &self.filename
        );
        service
          .hub_service()
          .find_local_file(&self.repo, &self.filename, self.snapshot.clone())?
      }
      false => {
        if self.auto_download {
          service
            .hub_service()
            .download(&self.repo, &self.filename, self.snapshot)?
        } else {
          return Err(CreateCommandError::HubServiceError(
            HubFileNotFoundError::new(
              self.filename.clone(),
              self.repo.to_string(),
              self
                .snapshot
                .clone()
                .unwrap_or_else(|| SNAPSHOT_MAIN.to_string()),
            )
            .into(),
          ));
        }
      }
    };
    let chat_template_repo = Repo::try_from(self.chat_template.clone())?;
    let file_exists =
      service
        .hub_service()
        .local_file_exists(&chat_template_repo, TOKENIZER_CONFIG_JSON, None)?;
    match file_exists {
      true => {
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
  use crate::{CreateCommand, CreateCommandBuilder};
  use mockall::predicate::*;
  use objs::{
    test_utils::SNAPSHOT, Alias, ChatTemplate, GptContextParams, GptContextParamsBuilder, HubFile,
    OAIRequestParams, OAIRequestParamsBuilder, Repo, TOKENIZER_CONFIG_JSON,
  };
  use rstest::rstest;
  use services::{
    test_utils::{test_hf_service, AppServiceStubBuilder, TestHfService},
    AppService,
  };
  use std::sync::Arc;

  #[rstest]
  fn test_create_execute_updates_if_exists() -> anyhow::Result<()> {
    let create_cmd = CreateCommand {
      alias: "tinyllama:instruct".to_string(),
      repo: Repo::try_from("TheBloke/TinyLlama-1.1B-Chat-v0.3-GGUF".to_string())?,
      filename: "tinyllama-1.1b-chat-v0.3.Q2_K.gguf".to_string(),
      snapshot: Some("main".to_string()),
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
    let result = create_cmd.execute(service.clone());
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
  #[case(None)]
  #[case(Some("main".to_string()))]
  #[case(Some("7de0799b8c9c12eff96e5c9612e39b041b3f4f5b".to_string()))]
  fn test_cmd_create_downloads_model_saves_alias(
    #[case] snapshot: Option<String>,
    mut test_hf_service: TestHfService,
  ) -> anyhow::Result<()> {
    let create = CreateCommandBuilder::testalias()
      .snapshot(snapshot.clone())
      .build()
      .unwrap();
    test_hf_service
      .expect_download()
      .with(
        eq(create.repo.clone()),
        eq(create.filename.clone()),
        eq(snapshot.clone()),
      )
      .return_once(|_, _, _| Ok(HubFile::testalias()));
    let service = Arc::new(
      AppServiceStubBuilder::default()
        .hub_service(Arc::new(test_hf_service))
        .with_data_service()
        .build()?,
    );
    create.execute(service.clone())?;
    let created = service
      .data_service()
      .find_alias("testalias:instruct")
      .unwrap();
    assert_eq!(
      Alias {
        alias: "testalias:instruct".to_string(),
        family: Some("testalias".to_string()),
        repo: Repo::testalias(),
        filename: Repo::testalias_filename(),
        snapshot: SNAPSHOT.to_string(),
        features: vec!["chat".to_string()],
        chat_template: ChatTemplate::Id(objs::ChatTemplateId::Llama3),
        request_params: OAIRequestParams::default(),
        context_params: GptContextParams::default()
      },
      created
    );
    Ok(())
  }

  #[rstest]
  fn test_cmd_create_with_tokenizer_config_downloads_tokenizer_saves_alias(
    mut test_hf_service: TestHfService,
  ) -> anyhow::Result<()> {
    let tokenizer_repo = Repo::try_from("MyFactory/testalias")?;
    let chat_template = ChatTemplate::Repo(tokenizer_repo.clone());
    let create = CreateCommandBuilder::testalias()
      .chat_template(chat_template.clone())
      .build()
      .unwrap();
    test_hf_service
      .expect_download()
      .with(
        eq(create.repo.clone()),
        eq(create.filename.clone()),
        eq(None),
      )
      .return_once(|_, _, _| Ok(HubFile::testalias()));
    test_hf_service
      .expect_download()
      .with(
        eq(tokenizer_repo.clone()),
        eq(TOKENIZER_CONFIG_JSON),
        eq(None),
      )
      .return_once(|_, _, _| Ok(HubFile::testalias_tokenizer()));
    let service = Arc::new(
      AppServiceStubBuilder::default()
        .hub_service(Arc::new(test_hf_service))
        .with_data_service()
        .build()?,
    );
    create.execute(service.clone())?;
    let created = service
      .data_service()
      .find_alias("testalias:instruct")
      .unwrap();
    assert_eq!(
      Alias {
        alias: "testalias:instruct".to_string(),
        family: Some("testalias".to_string()),
        repo: Repo::try_from("MyFactory/testalias-gguf").unwrap(),
        filename: "testalias.Q8_0.gguf".to_string(),
        snapshot: SNAPSHOT.to_string(),
        features: vec!["chat".to_string()],
        chat_template: ChatTemplate::Repo(Repo::try_from("MyFactory/testalias")?),
        request_params: OAIRequestParams::default(),
        context_params: GptContextParams::default()
      },
      created
    );
    Ok(())
  }
}
