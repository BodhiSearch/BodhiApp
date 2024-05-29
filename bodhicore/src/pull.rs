use crate::{error::AppError, objs::Alias, service::AppServiceFn, Command, Repo};

#[derive(Debug, PartialEq)]
pub enum Pull {
  ByAlias {
    alias: String,
    force: bool,
  },
  ByRepoFile {
    repo: Repo,
    filename: String,
    force: bool,
  },
}

impl TryFrom<Command> for Pull {
  type Error = AppError;

  fn try_from(value: Command) -> Result<Self, Self::Error> {
    match value {
      Command::Pull {
        alias,
        repo,
        filename,
        force,
      } => {
        let pull_command = match alias {
          Some(alias) => Pull::ByAlias { alias, force },
          None => match (repo, filename) {
            (Some(repo), Some(filename)) => Pull::ByRepoFile {
              repo: Repo::try_new(repo)?,
              filename,
              force,
            },
            (repo, filename) => return Err(AppError::BadRequest(format!(
              "cannot initialize pull command with invalid state: repo={repo:?}, filename={filename:?}"
            ))),
          },
        };
        Ok(pull_command)
      }
      _ => Err(AppError::BadRequest(format!(
        "{value:?} cannot be converted into PullCommand, only `Command::Pull` variant supported."
      ))),
    }
  }
}

impl Pull {
  pub fn execute(self, service: &dyn AppServiceFn) -> crate::error::Result<()> {
    match self {
      Pull::ByAlias { alias, force } => {
        if !force && service.find_alias(&alias).is_some() {
          return Err(AppError::AliasExists(alias));
        }
        let Some(model) = service.find_remote_model(&alias)? else {
          return Err(AppError::AliasNotFound(alias));
        };
        service.download(&model.repo, &model.filename, force)?;
        let new_alias: Alias = model.into();
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
    objs::{Alias, ChatTemplate, ChatTemplateId, RemoteModel, Repo},
    service::{MockDataService, MockHubService},
    test_utils::{app_service_stub, AppServiceTuple, MockAppServiceFn},
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
    let pull = Pull::ByAlias {
      alias,
      force: false,
    };
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
      Repo::try_new(String::from("MyFactory/testalias-neverdownload-gguf"))?,
      String::from("testalias-neverdownload.Q8_0.gguf"),
      vec![String::from("chat")],
      ChatTemplate::Id(ChatTemplateId::Llama3),
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
    let service = MockAppServiceFn::new(mock_hub_service, mock_data_service);
    let pull = Pull::ByAlias {
      alias: "test_pull_by_alias:instruct".to_string(),
      force: false,
    };
    pull.execute(&service)?;
    Ok(())
  }

  #[rstest]
  fn test_pull_by_repo_file() -> anyhow::Result<()> {
    let pull = Pull::ByRepoFile {
      repo: Repo::try_new("google/gemma-7b-it-GGUF".to_string())?,
      filename: "gemma-7b-it.gguf".to_string(),
      force: false,
    };
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
    let service = MockAppServiceFn::new(mock_hub_service, MockDataService::new());
    pull.execute(&service)?;
    Ok(())
  }
}
