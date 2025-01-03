use objs::{AliasBuilder, AliasSource, AppError, BuilderError, ObjValidationError, Repo};
use services::{
  AliasExistsError, AppService, DataServiceError, HubDownloadable, HubServiceError, ObjExtsError,
  RemoteModelNotFoundError,
};
use std::sync::Arc;

#[derive(Debug, PartialEq)]
pub enum PullCommand {
  ByAlias {
    alias: String,
  },
  ByRepoFile {
    repo: Repo,
    filename: String,
    snapshot: Option<String>,
  },
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum PullCommandError {
  #[error(transparent)]
  Builder(#[from] BuilderError),
  #[error(transparent)]
  ObjExts(#[from] ObjExtsError),
  #[error(transparent)]
  HubServiceError(#[from] HubServiceError),
  #[error(transparent)]
  AliasExists(#[from] AliasExistsError),
  #[error(transparent)]
  RemoteModelNotFound(#[from] RemoteModelNotFoundError),
  #[error(transparent)]
  DataServiceError(#[from] DataServiceError),
  #[error(transparent)]
  ObjValidationError(#[from] ObjValidationError),
}

type Result<T> = std::result::Result<T, PullCommandError>;

impl PullCommand {
  #[allow(clippy::result_large_err)]
  pub fn execute(self, service: Arc<dyn AppService>) -> Result<()> {
    match &self {
      PullCommand::ByAlias { alias } => {
        if service.data_service().find_alias(alias).is_some() {
          return Err(AliasExistsError(alias.clone()).into());
        }
        let Some(model) = service.data_service().find_remote_model(alias)? else {
          return Err(RemoteModelNotFoundError::new(alias.clone()))?;
        };
        let local_model_file =
          service
            .hub_service()
            .download(&model.repo, &model.filename, None)?;
        let _ = model.chat_template.download(service.hub_service())?;
        let alias = AliasBuilder::default()
          .alias(model.alias)
          .repo(model.repo)
          .filename(model.filename)
          .snapshot(local_model_file.snapshot)
          .source(AliasSource::User)
          .chat_template(model.chat_template)
          .request_params(model.request_params)
          .context_params(model.context_params)
          .build()?;
        service.data_service().save_alias(&alias)?;
        println!(
          "model alias: '{}' saved to $BODHI_HOME/aliases",
          alias.alias
        );
        Ok(())
      }
      PullCommand::ByRepoFile {
        repo,
        filename,
        snapshot,
      } => {
        let model_file_exists =
          service
            .hub_service()
            .local_file_exists(repo, filename, snapshot.clone())?;
        if model_file_exists {
          println!("repo: '{repo}', filename: '{filename}' already exists in $HF_HOME");
          return Ok(());
        } else {
          service
            .hub_service()
            .download(repo, filename, snapshot.clone())?;
          println!("repo: '{repo}', filename: '{filename}' downloaded into $HF_HOME");
        }
        Ok(())
      }
    }
  }
}

#[cfg(test)]
mod test {
  use crate::{PullCommand, PullCommandError};
  use mockall::predicate::eq;
  use objs::{Alias, HubFile, RemoteModel, Repo, TOKENIZER_CONFIG_JSON};
  use pretty_assertions::assert_eq;
  use rstest::rstest;
  use services::{
    test_utils::{test_hf_service, AppServiceStubBuilder, TestHfService},
    AliasExistsError, AppService, ALIASES_DIR,
  };
  use std::{fs, sync::Arc};

  #[rstest]
  fn test_pull_by_alias_fails_if_alias_exists() -> anyhow::Result<()> {
    let service = AppServiceStubBuilder::default()
      .with_data_service()
      .build()?;
    let alias = "testalias-exists:instruct";
    let pull = PullCommand::ByAlias {
      alias: alias.to_string(),
    };
    let result = pull.execute(Arc::new(service));
    assert!(result.is_err());
    assert!(matches!(
      result.unwrap_err(),
      PullCommandError::AliasExists(arg) if arg == AliasExistsError(alias.to_string())
    ));
    Ok(())
  }

  #[rstest]
  fn test_pull_by_alias_creates_new_alias(
    mut test_hf_service: TestHfService,
  ) -> anyhow::Result<()> {
    let remote_model = RemoteModel::testalias();
    test_hf_service
      .expect_download()
      .with(
        eq(Repo::testalias()),
        eq(Repo::TESTALIAS_FILENAME),
        eq(None),
      )
      .return_once(|_, _, _| Ok(HubFile::testalias()));
    test_hf_service
      .expect_download()
      .with(
        eq(Repo::llama3_tokenizer()),
        eq(TOKENIZER_CONFIG_JSON),
        eq(None),
      )
      .return_once(|_, _, _| Ok(HubFile::llama3_tokenizer()));
    let service = AppServiceStubBuilder::default()
      .hub_service(Arc::new(test_hf_service))
      .with_data_service()
      .build()?;
    let pull = PullCommand::ByAlias {
      alias: remote_model.alias,
    };
    let service = Arc::new(service);
    pull.execute(service.clone())?;
    let created_alias = service
      .data_service()
      .find_alias("testalias:instruct")
      .ok_or(anyhow::anyhow!("alias not found"))?;
    assert_eq!(Alias::testalias(), created_alias);
    Ok(())
  }

  #[rstest]
  #[case(None)]
  #[case(Some("main".to_string()))]
  #[case(Some("b32046744d93031a26c8e925de2c8932c305f7b9".to_string()))]
  #[anyhow_trace::anyhow_trace]
  fn test_pull_by_repo_file_only_pulls_the_model(
    #[case] snapshot: Option<String>,
    mut test_hf_service: TestHfService,
  ) -> anyhow::Result<()> {
    let repo = Repo::testalias();
    let filename = Repo::testalias_filename();
    let pull = PullCommand::ByRepoFile {
      repo: repo.clone(),
      filename: filename.to_string(),
      snapshot: snapshot.clone(),
    };
    test_hf_service
      .expect_download()
      .with(eq(repo), eq(filename), eq(snapshot))
      .return_once(|_, _, _| Ok(HubFile::testalias()));
    let service = AppServiceStubBuilder::default()
      .hub_service(Arc::new(test_hf_service))
      .build()?;
    pull.execute(Arc::new(service))?;
    Ok(())
  }

  #[rstest]
  fn test_pull_by_alias_downloaded_model_using_stubs_create_alias_file() -> anyhow::Result<()> {
    let service = AppServiceStubBuilder::default()
      .with_hub_service()
      .with_data_service()
      .build()?;
    let service = Arc::new(service);
    let command = PullCommand::ByAlias {
      alias: "testalias:instruct".to_string(),
    };
    command.execute(service.clone())?;
    let alias = service
      .bodhi_home()
      .join(ALIASES_DIR)
      .join("testalias--instruct.yaml");
    assert!(alias.exists());
    let content = fs::read_to_string(alias)?;
    assert_eq!(
      r#"alias: testalias:instruct
repo: MyFactory/testalias-gguf
filename: testalias.Q8_0.gguf
snapshot: 5007652f7a641fe7170e0bad4f63839419bd9213
chat_template: llama3
"#,
      content
    );
    Ok(())
  }
}
