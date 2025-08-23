use objs::{
  AliasBuilder, AliasSource, AppError, BuilderError, GptContextParams, OAIRequestParams,
  ObjValidationError, Repo,
};
use services::{
  AliasExistsError, AppService, DataServiceError, HubFileNotFoundError, HubServiceError,
  SNAPSHOT_MAIN,
};
use std::sync::Arc;
use tracing::debug;

#[derive(Debug, Clone, PartialEq, derive_new::new, derive_builder::Builder)]
#[allow(clippy::too_many_arguments)]
pub struct CreateCommand {
  #[new(into)]
  pub alias: String,
  pub repo: Repo,
  #[new(into)]
  pub filename: String,
  pub snapshot: Option<String>,
  // chat_template field removed since llama.cpp now handles chat templates
  #[builder(default = "true")]
  // #[deprecated(since = "0.1.0", note = "chat templates are now handled by llama.cpp")]
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
  Builder(#[from] BuilderError),
  // ObjExts error removed since chat templates are no longer used
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
  pub async fn execute(self, service: Arc<dyn AppService>) -> Result<()> {
    if service.data_service().find_alias(&self.alias).is_some() {
      if !self.update {
        return Err(AliasExistsError(self.alias.clone()).into());
      }
      debug!("Updating existing alias: '{}'", self.alias);
    } else {
      debug!("Creating new alias: '{}'", self.alias);
    }
    let file_exists =
      service
        .hub_service()
        .local_file_exists(&self.repo, &self.filename, self.snapshot.clone())?;
    let local_model_file = match file_exists {
      true => {
        debug!(
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
            .download(&self.repo, &self.filename, self.snapshot, None)
            .await?
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
    // Chat template download removed since llama.cpp now handles chat templates
    let alias = AliasBuilder::default()
      .alias(self.alias)
      .repo(self.repo)
      .filename(self.filename)
      .snapshot(local_model_file.snapshot)
      .source(AliasSource::User)
      .request_params(self.oai_request_params)
      .context_params(self.context_params)
      .build()?;
    service.data_service().save_alias(&alias)?;
    debug!(
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
  use objs::{Alias, AliasBuilder, HubFile, OAIRequestParamsBuilder, Repo};
  use pretty_assertions::assert_eq;
  use rstest::rstest;
  use services::{
    test_utils::{test_hf_service, AppServiceStubBuilder, TestHfService},
    AppService,
  };
  use std::sync::Arc;

  #[rstest]
  #[tokio::test]
  async fn test_create_execute_updates_if_exists() -> anyhow::Result<()> {
    let create_cmd = CreateCommand {
      alias: "tinyllama:instruct".to_string(),
      repo: Repo::tinyllama(),
      filename: Repo::TINYLLAMA_FILENAME.to_string(),
      snapshot: Some("main".to_string()),
      // chat_template field removed since llama.cpp now handles chat templates
      auto_download: false,
      update: true,
      oai_request_params: OAIRequestParamsBuilder::default()
        .frequency_penalty(1.0)
        .max_tokens(2048_u16)
        .build()
        .unwrap(),
      context_params: vec![
        "--ctx-size 2048".to_string(),
        "--n-keep 2048".to_string(),
        "--parallel 2".to_string(),
        "--seed 42".to_string(),
        "--threads 8".to_string(),
      ],
    };
    let service = Arc::new(
      AppServiceStubBuilder::default()
        .with_hub_service()
        .with_data_service()
        .build()?,
    );
    let repo_alias = service
      .data_service()
      .find_alias("tinyllama:instruct")
      .unwrap();
    let result = create_cmd.execute(service.clone()).await;
    assert!(result.is_ok());
    let updated_alias = service
      .data_service()
      .find_alias("tinyllama:instruct")
      .unwrap();
    assert_ne!(repo_alias, updated_alias);
    let expected = AliasBuilder::tinyllama()
      .request_params(
        OAIRequestParamsBuilder::default()
          .frequency_penalty(1.0)
          .max_tokens(2048_u16)
          .build()
          .unwrap(),
      )
      .context_params(vec![
        "--ctx-size 2048".to_string(),
        "--n-keep 2048".to_string(),
        "--parallel 2".to_string(),
        "--seed 42".to_string(),
        "--threads 8".to_string(),
      ])
      .build()
      .unwrap();
    assert_eq!(expected, updated_alias);
    Ok(())
  }

  #[rstest]
  #[case(None)]
  #[case(Some("main".to_string()))]
  #[case(Some("7de0799b8c9c12eff96e5c9612e39b041b3f4f5b".to_string()))]
  #[tokio::test]
  async fn test_cmd_create_downloads_model_saves_alias(
    #[case] snapshot: Option<String>,
    mut test_hf_service: TestHfService,
  ) -> anyhow::Result<()> {
    let create = CreateCommandBuilder::testalias()
      .snapshot(snapshot.clone())
      .build()
      .unwrap();
    test_hf_service
      .inner_mock
      .expect_download()
      .with(
        eq(create.repo.clone()),
        eq(create.filename.clone()),
        eq(snapshot.clone()),
        always(),
      )
      .return_once(|_, _, _, _| Ok(HubFile::testalias()));
    // Tokenizer download removed since llama.cpp now handles chat templates
    let service = Arc::new(
      AppServiceStubBuilder::default()
        .hub_service(Arc::new(test_hf_service))
        .with_data_service()
        .build()?,
    );
    create.execute(service.clone()).await?;
    let created = service
      .data_service()
      .find_alias("testalias:instruct")
      .unwrap();
    assert_eq!(Alias::testalias(), created);
    Ok(())
  }

  // Test for tokenizer config removed since llama.cpp now handles chat templates
}
