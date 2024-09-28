use objs::{Alias, HubFile, ObjError, Repo, TOKENIZER_CONFIG_JSON};
use services::{AppService, DataServiceError, HubServiceError};
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

#[derive(Debug, thiserror::Error)]
pub enum PullCommandError {
  #[error(transparent)]
  HubServiceError(#[from] HubServiceError),
  #[error("model alias '{0}' already exists")]
  AliasExists(String),
  #[error("alias '{0}' not found")]
  AliasNotFound(String),
  #[error(transparent)]
  DataServiceError(#[from] DataServiceError),
  #[error(transparent)]
  ObjError(#[from] ObjError),
}

type Result<T> = std::result::Result<T, PullCommandError>;

impl PullCommand {
  #[allow(clippy::result_large_err)]
  pub fn execute(self, service: Arc<dyn AppService>) -> Result<()> {
    match self {
      PullCommand::ByAlias { alias } => {
        if service.data_service().find_alias(&alias).is_some() {
          return Err(PullCommandError::AliasExists(alias));
        }
        let Some(model) = service.data_service().find_remote_model(&alias)? else {
          return Err(PullCommandError::AliasNotFound(alias));
        };
        let local_model_file = PullCommand::download_file_if_missing(
          service.clone(),
          &model.repo,
          &model.filename,
          None,
        )?;
        _ = PullCommand::download_file_if_missing(
          service.clone(),
          &Repo::try_from(model.chat_template.clone())?,
          TOKENIZER_CONFIG_JSON,
          None,
        )?;
        let alias = Alias::new(
          model.alias,
          Some(model.family),
          model.repo,
          model.filename,
          local_model_file.snapshot.clone(),
          model.features,
          model.chat_template,
          model.request_params,
          model.context_params,
        );
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
            .local_file_exists(&repo, &filename, snapshot.clone())?;
        if model_file_exists {
          println!("repo: '{repo}', filename: '{filename}' already exists in $HF_HOME");
          return Ok(());
        } else {
          service.hub_service().download(&repo, &filename, snapshot)?;
          println!("repo: '{repo}', filename: '{filename}' downloaded into $HF_HOME");
        }
        Ok(())
      }
    }
  }

  fn download_file_if_missing(
    service: Arc<dyn AppService>,
    repo: &Repo,
    filename: &str,
    snapshot: Option<String>,
  ) -> Result<HubFile> {
    let local_model_file =
      service
        .hub_service()
        .find_local_file(repo, filename, snapshot.clone())?;
    match local_model_file {
      Some(local_model_file) => {
        println!(
          "repo: '{}', filename: '{}' already exists in $HF_HOME",
          &repo, &filename
        );
        Ok(local_model_file)
      }
      _ => {
        let local_model_file = service.hub_service().download(repo, filename, snapshot)?;
        println!(
          "repo: '{}', filename: '{}' downloaded into $HF_HOME",
          repo, filename
        );
        Ok(local_model_file)
      }
    }
  }
}

#[cfg(test)]
mod test {
  use crate::PullCommand;
  use mockall::predicate::eq;
  use objs::{
    test_utils::SNAPSHOT, Alias, ChatTemplate, GptContextParams, HubFile, OAIRequestParams,
    RemoteModel, Repo,
  };
  use rstest::rstest;
  use services::{
    test_utils::{test_hf_service, AppServiceStubBuilder, TestHfService},
    AppService, ALIASES_DIR,
  };
  use std::{fs, sync::Arc};

  #[rstest]
  fn test_pull_by_alias_fails_if_alias_exists() -> anyhow::Result<()> {
    let service = AppServiceStubBuilder::default()
      .with_data_service()
      .build()?;
    let alias = String::from("testalias-exists:instruct");
    let pull = PullCommand::ByAlias { alias };
    let result = pull.execute(Arc::new(service));
    assert!(result.is_err());
    assert_eq!(
      "model alias 'testalias-exists:instruct' already exists",
      result.unwrap_err().to_string()
    );
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
        eq(Repo::try_from("MyFactory/testalias-gguf").unwrap()),
        eq("testalias.Q8_0.gguf"),
        eq(None),
      )
      .return_once(|_, _, _| Ok(HubFile::testalias()));
    let service = AppServiceStubBuilder::default()
      .with_data_service()
      .hub_service(Arc::new(test_hf_service))
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
    assert_eq!(
      Alias {
        alias: "testalias:instruct".to_string(),
        family: Some("testalias".to_string()),
        repo: Repo::try_from("MyFactory/testalias-gguf")?,
        filename: "testalias.Q8_0.gguf".to_string(),
        snapshot: SNAPSHOT.to_string(),
        features: vec!["chat".to_string()],
        chat_template: ChatTemplate::Id(objs::ChatTemplateId::Llama3),
        request_params: OAIRequestParams::default(),
        context_params: GptContextParams::default()
      },
      created_alias
    );
    Ok(())
  }

  #[rstest]
  #[case(None)]
  #[case(Some("main".to_string()))]
  #[case(Some("191239b3e26b2882fb562ffccdd1cf0f65402adb".to_string()))]
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
      .with_data_service()
      .with_hub_service()
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
family: testalias
repo: MyFactory/testalias-gguf
filename: testalias.Q8_0.gguf
snapshot: 5007652f7a641fe7170e0bad4f63839419bd9213
features:
- chat
chat_template: llama3
"#,
      content
    );
    Ok(())
  }
}
