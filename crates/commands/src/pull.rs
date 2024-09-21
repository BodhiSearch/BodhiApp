use crate::{CliError, Command};
use objs::{Alias, HubFile, ObjError, Repo, REFS_MAIN, TOKENIZER_CONFIG_JSON};
use services::{AppServiceFn, DataServiceError, HubServiceError};
use std::sync::Arc;

#[derive(Debug, PartialEq)]
pub enum PullCommand {
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

impl TryFrom<Command> for PullCommand {
  type Error = CliError;

  fn try_from(value: Command) -> std::result::Result<Self, Self::Error> {
    match value {
      Command::Pull {
        alias,
        repo,
        filename,
        force,
      } => {
        let pull_command = match alias {
          Some(alias) => PullCommand::ByAlias { alias, force },
          None => match (repo, filename) {
            (Some(repo), Some(filename)) => PullCommand::ByRepoFile {
              repo: Repo::try_from(repo)?,
              filename,
              force,
            },
            (repo, filename) => return Err(CliError::BadRequest(format!(
              "cannot initialize pull command with invalid state: repo={repo:?}, filename={filename:?}"
            ))),
          },
        };
        Ok(pull_command)
      }
      cmd => Err(CliError::ConvertCommand(
        cmd.to_string(),
        "pull".to_string(),
      )),
    }
  }
}

#[derive(Debug, thiserror::Error)]
pub enum PullCommandError {
  #[error(transparent)]
  HubServiceError(#[from] HubServiceError),
  #[error("model alias '{0}' already exists. Use --force to overwrite the model alias config")]
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
  pub fn execute(self, service: Arc<dyn AppServiceFn>) -> Result<()> {
    match self {
      PullCommand::ByAlias { alias, force } => {
        if !force && service.data_service().find_alias(&alias).is_some() {
          return Err(PullCommandError::AliasExists(alias));
        }
        let Some(model) = service.data_service().find_remote_model(&alias)? else {
          return Err(PullCommandError::AliasNotFound(alias));
        };
        let local_model_file = PullCommand::download_file_if_missing(
          service.clone(),
          &model.repo,
          &model.filename,
          REFS_MAIN,
          force,
        )?;
        _ = PullCommand::download_file_if_missing(
          service.clone(),
          &Repo::try_from(model.chat_template.clone())?,
          TOKENIZER_CONFIG_JSON,
          REFS_MAIN,
          force,
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
        force,
      } => {
        let local_model_file = service
          .hub_service()
          .find_local_file(&repo, &filename, REFS_MAIN)?;
        match local_model_file {
          Some(_) if !force => {
            println!("repo: '{repo}', filename: '{filename}' already exists in $HF_HOME");
            return Ok(());
          }
          _ => {
            service.hub_service().download(&repo, &filename, force)?;
            println!("repo: '{repo}', filename: '{filename}' downloaded into $HF_HOME");
          }
        }
        Ok(())
      }
    }
  }

  fn download_file_if_missing(
    service: Arc<dyn AppServiceFn>,
    repo: &Repo,
    filename: &str,
    snapshot: &str,
    force: bool,
  ) -> Result<HubFile> {
    let local_model_file = service
      .hub_service()
      .find_local_file(repo, filename, snapshot)?;
    match local_model_file {
      Some(local_model_file) if !force => {
        println!(
          "repo: '{}', filename: '{}' already exists in $HF_HOME",
          &repo, &filename
        );
        Ok(local_model_file)
      }
      _ => {
        let local_model_file = service.hub_service().download(repo, filename, force)?;
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
  use crate::{Command, PullCommand};
  use mockall::predicate::eq;
  use objs::{Alias, HubFile, RemoteModel, Repo, REFS_MAIN, TOKENIZER_CONFIG_JSON};
  use rstest::rstest;
  use services::{
    test_utils::{AppServiceStubBuilder, AppServiceStubMock},
    MockDataService, MockHubService, ALIASES_DIR,
  };
  use std::{fs, path::PathBuf, sync::Arc};

  #[rstest]
  fn test_pull_by_alias_fails_if_alias_exists_no_force() -> anyhow::Result<()> {
    let service = AppServiceStubBuilder::default()
      .with_data_service()
      .build()?;
    let alias = String::from("testalias-exists:instruct");
    let pull = PullCommand::ByAlias {
      alias,
      force: false,
    };
    let result = pull.execute(Arc::new(service));
    assert!(result.is_err());
    assert_eq!(
      "model alias 'testalias-exists:instruct' already exists. Use --force to overwrite the model alias config",
      result.unwrap_err().to_string()
    );
    Ok(())
  }

  #[rstest]
  fn test_pull_by_alias_creates_new_alias() -> anyhow::Result<()> {
    let remote_model = RemoteModel::testalias();
    let mut mock_data_service = MockDataService::new();
    mock_data_service
      .expect_find_alias()
      .with(eq(remote_model.alias.clone()))
      .times(1)
      .returning(|_| None);
    let remote_clone = remote_model.clone();
    mock_data_service
      .expect_find_remote_model()
      .with(eq(remote_model.alias.clone()))
      .return_once(move |_| Ok(Some(remote_clone.clone())));
    let mut mock_hub_service = MockHubService::new();
    mock_hub_service
      .expect_find_local_file()
      .with(
        eq(remote_model.repo.clone()),
        eq(remote_model.filename.clone()),
        eq(REFS_MAIN),
      )
      .return_once(|_, _, _| Ok(None));
    mock_hub_service
      .expect_download()
      .with(
        eq(remote_model.repo),
        eq(remote_model.filename.clone()),
        eq(false),
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
      .return_once(|_| Ok(PathBuf::from("ignored")));
    let service = AppServiceStubMock::builder()
      .hub_service(mock_hub_service)
      .data_service(mock_data_service)
      .build()?;
    let pull = PullCommand::ByAlias {
      alias: remote_model.alias,
      force: false,
    };
    pull.execute(Arc::new(service))?;
    Ok(())
  }

  #[rstest]
  fn test_pull_by_repo_file_only_pulls_the_model() -> anyhow::Result<()> {
    let repo = Repo::try_from("google/gemma-7b-it-GGUF")?;
    let filename = "gemma-7b-it.gguf";
    let pull = PullCommand::ByRepoFile {
      repo: repo.clone(),
      filename: filename.to_string(),
      force: false,
    };
    let mut mock_hub_service = MockHubService::new();
    mock_hub_service
      .expect_find_local_file()
      .with(eq(repo.clone()), eq(filename), eq(REFS_MAIN))
      .return_once(|_, _, _| Ok(None));
    mock_hub_service
      .expect_download()
      .with(eq(repo), eq(filename), eq(false))
      .return_once(|_, _, _| Ok(HubFile::testalias()));
    let mock_data_service = MockDataService::new();
    let service = AppServiceStubMock::builder()
      .hub_service(mock_hub_service)
      .data_service(mock_data_service)
      .build()?;
    pull.execute(Arc::new(service))?;
    Ok(())
  }

  #[rstest]
  #[case(Command::Pull {
    alias: Some("llama3:instruct".to_string()),
    repo: None,
    filename: None,
    force: false,
  }, PullCommand::ByAlias {
    alias: "llama3:instruct".to_string(),
    force: false,
  })]
  #[case(Command::Pull {
    alias: None,
    repo: Some("QuantFactory/Meta-Llama-3-8B-Instruct-GGUF".to_string()),
    filename: Some("Meta-Llama-3-8B-Instruct.Q8_0.gguf".to_string()),
    force: false,
  },
  PullCommand::ByRepoFile {
    repo: Repo::try_from("QuantFactory/Meta-Llama-3-8B-Instruct-GGUF").unwrap(), filename: "Meta-Llama-3-8B-Instruct.Q8_0.gguf".to_string(), 
    force: false
  })]
  fn test_pull_command_try_from_command(
    #[case] input: Command,
    #[case] expected: PullCommand,
  ) -> anyhow::Result<()> {
    let pull_command: PullCommand = PullCommand::try_from(input)?;
    assert_eq!(expected, pull_command);
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
      force: false,
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
