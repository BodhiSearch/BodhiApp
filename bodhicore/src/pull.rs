use crate::{
  service::{AppService, AppServiceFn, DataService, DataServiceError, HubService},
  hf::{download_file, download_url},
  hf_tokenizer::{HubTokenizerConfig, TOKENIZER_CONFIG_FILENAME},
  list::find_remote_model,
  objs::Alias,
};
use anyhow::bail;
use regex::Regex;
use thiserror::Error;

#[derive(Debug, PartialEq)]
pub enum Pull {
  ByAlias {
    alias: String,
    force: bool,
  },
  ByRepoFile {
    repo: String,
    filename: String,
    force: bool,
  },
}

#[derive(Debug, Error)]
pub enum PullError {
  #[error(
    r#"alias '{0}' not found in pre-configured model aliases.
Run `bodhi list -r` to see list of pre-configured model aliases
"#
  )]
  AliasNotFound(String),
  #[error("alias '{0}' already exists. Use --force to overwrite the alias config")]
  AliasExists(String),
  #[error(transparent)]
  DataService(#[from] DataServiceError),
}

type Result<T> = std::result::Result<T, PullError>;

impl Pull {
  pub fn new(
    alias: Option<String>,
    repo: Option<String>,
    filename: Option<String>,
    force: bool,
  ) -> Self {
    match alias {
      Some(alias) => Pull::ByAlias { alias, force },
      None => match (repo, filename) {
        (Some(repo), Some(filename)) => Pull::ByRepoFile {
          repo,
          filename,
          force,
        },
        _ => todo!(),
      },
    }
  }

  pub fn execute(self, service: &dyn AppServiceFn) -> Result<()> {
    match self {
      Pull::ByAlias { alias, force } => {
        if !force && service.find_alias(&alias).is_some() {
          return Err(PullError::AliasExists(alias));
        }
        let Some(model) = service.find_remote_model(&alias)? else {
          return Err(PullError::AliasNotFound(alias));
        };
        service.download(&model.repo, &model.filename, force)?;
        let new_alias = Alias::new(
          alias,
          Some(model.family),
          Some(model.repo),
          Some(model.filename),
          model.features,
        );
        service.save_alias(new_alias)?;
        Ok(())
      }
      Pull::ByRepoFile {
        repo,
        filename,
        force,
      } => {
        service.download(&repo, &filename, force)?;
        Ok(())
      }
    }
  }
}

#[cfg(test)]
mod test {
  use crate::{
    service::{AppService, MockDataService, MockHubService},
    list::RemoteModel,
    objs::Alias,
    test_utils::{app_service_stub, AppServiceTuple},
    Pull,
  };
  use mockall::predicate::eq;
  use rstest::rstest;
  use std::path::PathBuf;

  #[rstest]
  fn test_pull_by_alias_fails_if_alias_exists_no_force(
    app_service_stub: AppServiceTuple,
  ) -> anyhow::Result<()> {
    let AppServiceTuple(_bodhi_home, _hf_home, _, _, service) = app_service_stub;
    let alias = String::from("testalias-exists:instruct");
    let pull = Pull::new(Some(alias.clone()), None, None, false);
    let result = pull.execute(&service);
    assert!(result.is_err());
    assert_eq!(
      "alias 'testalias-exists:instruct' already exists. Use --force to overwrite the alias config",
      result.unwrap_err().to_string()
    );
    Ok(())
  }

  #[rstest]
  fn test_pull_by_alias() -> anyhow::Result<()> {
    let mut mock_data_service = MockDataService::new();
    mock_data_service
      .expect_find_alias()
      .with(eq("test_pull_by_alias:instruct"))
      .times(1)
      .returning(|_| None);
    let remote_model = RemoteModel::new(
      String::from("test_pull_by_alias:instruct"),
      String::from("testalias"),
      String::from("MyFactory/testalias-neverdownload-gguf"),
      String::from("testalias-neverdownload.Q8_0.gguf"),
      vec![String::from("chat")],
      String::from("llama3"),
    );
    let remote_clone = remote_model.clone();
    mock_data_service
      .expect_find_remote_model()
      .with(eq("test_pull_by_alias:instruct"))
      .times(1)
      .returning(move |_| Ok(Some(remote_clone.clone())));
    let alias: Alias = remote_model.into();
    mock_data_service
      .expect_save_alias()
      .with(eq(alias))
      .times(1)
      .returning(|_| Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR"))));
    let mut mock_hub_service = MockHubService::new();
    mock_hub_service
      .expect_download()
      .with(
        eq("MyFactory/testalias-neverdownload-gguf"),
        eq("testalias-neverdownload.Q8_0.gguf"),
        eq(false),
      )
      .times(1)
      .returning(|_, _, _| Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR"))));
    let service = AppService::new(Box::new(mock_hub_service), Box::new(mock_data_service));
    let pull = Pull::new(
      Some(String::from("test_pull_by_alias:instruct")),
      None,
      None,
      false,
    );
    pull.execute(&service)?;
    Ok(())
  }

  #[rstest]
  fn test_pull_by_repo_file() -> anyhow::Result<()> {
    let pull = Pull::new(
      None,
      Some(String::from("google/gemma-7b-it-GGUF")),
      Some(String::from("gemma-7b-it.gguf")),
      false,
    );
    let mut mock_hub_service = MockHubService::new();
    mock_hub_service
      .expect_download()
      .with(
        eq("google/gemma-7b-it-GGUF"),
        eq("gemma-7b-it.gguf"),
        eq(false),
      )
      .times(1)
      .returning(|_, _, _| Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR"))));
    let service = AppService::new(Box::new(mock_hub_service), Box::new(MockDataService::new()));
    pull.execute(&service)?;
    Ok(())
  }
}
