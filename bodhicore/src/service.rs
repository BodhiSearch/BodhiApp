use crate::{objs::Alias, objs::RemoteModel, server::BODHI_HOME};
use derive_new::new;
use hf_hub::{api::sync::ApiError, Cache, Repo};
#[cfg(test)]
use mockall::automock;
use std::{
  fmt::{Debug, Formatter},
  fs, io,
  path::PathBuf,
};
use thiserror::Error;

static MODELS_YAML: &str = "models.yaml";

#[derive(Debug, Error)]
pub enum DataServiceError {
  #[error(transparent)]
  ApiError(#[from] ApiError),
  #[error(
    r#"{source}
huggingface repo '{repo}' is requires requesting for access from website.
Go to https://huggingface.co/{repo} to request access to the model and try again.
"#
  )]
  GatedAccess {
    #[source]
    source: ApiError,
    repo: String,
  },
  #[error(
    r#"{source}
You are not logged in to huggingface using CLI `huggingface-cli login`.
So either the huggingface repo '{repo}' does not exists, or is private, or requires request access.
Go to https://huggingface.co/{repo} to request access, login via CLI, and then try again.
"#
  )]
  MayBeNotExists {
    #[source]
    source: ApiError,
    repo: String,
  },
  #[error(transparent)]
  Io(#[from] io::Error),
  #[error("{source}\nerror while serializing from file: '{filename}'")]
  SerdeYamlSerialize {
    #[source]
    source: serde_yaml::Error,
    filename: String,
  },
  #[error(transparent)]
  SerdeYamlDeserialize(#[from] serde_yaml::Error),
  #[error(
    r#"directory '{dirname}' not found in $BODHI_HOME.
$BODHI_HOME might not have been initialized. Run `bodhi init` to setup $BODHI_HOME."#
  )]
  DirMissing { dirname: String },
  #[error(
    r#"file '{filename}' not found in $BODHI_HOME/{dirname}.
$BODHI_HOME might not have been initialized. Run `bodhi init` to setup $BODHI_HOME."#
  )]
  FileMissing { filename: String, dirname: String },
}

pub type Result<T> = std::result::Result<T, DataServiceError>;

#[cfg_attr(test, automock)]
pub trait HubService: Debug {
  fn download(&self, repo: &str, filename: &str, force: bool) -> Result<PathBuf>;
}

#[derive(Debug, Clone, PartialEq, new)]
pub struct LocalDataService {
  bodhi_home: PathBuf,
}

impl Default for LocalDataService {
  fn default() -> Self {
    let bodhi_home = match std::env::var(BODHI_HOME) {
      Ok(home) => home.into(),
      Err(_) => {
        let mut home = dirs::home_dir().expect("$HOME directory cannot be found");
        home.push(".cache");
        home.push("bodhi");
        home
      }
    };
    Self { bodhi_home }
  }
}

impl DataService for LocalDataService {
  fn find_remote_model(&self, alias: &str) -> Result<Option<RemoteModel>> {
    let models_file = self.bodhi_home.join(MODELS_YAML);
    if !models_file.exists() {
      return Err(DataServiceError::FileMissing {
        filename: String::from(MODELS_YAML),
        dirname: String::from(""),
      });
    }
    let content = fs::read_to_string(models_file.clone())?;
    let models = serde_yaml::from_str::<Vec<RemoteModel>>(&content).map_err(|err| {
      DataServiceError::SerdeYamlSerialize {
        source: err,
        filename: models_file.display().to_string(),
      }
    })?;
    Ok(models.into_iter().find(|model| model.alias.eq(alias)))
  }

  fn save_alias(&self, alias: Alias) -> Result<PathBuf> {
    let contents = serde_yaml::to_string(&alias)?;
    let alias_name = &alias.alias;
    let filename = self
      .bodhi_home
      .join("configs")
      .join(format!("{alias_name}.yaml"));
    fs::write(filename.clone(), contents)?;
    Ok(filename)
  }

  fn list_aliases(&self) -> Result<Vec<Alias>> {
    let config = self.bodhi_home.join("configs");
    if !config.exists() {
      return Err(DataServiceError::DirMissing {
        dirname: String::from("configs"),
      });
    }
    let yaml_files = fs::read_dir(config)?
      .filter_map(|entry| {
        let file_path = entry.ok()?.path();
        if let Some(extension) = file_path.extension() {
          if extension == "yaml" || extension == "yml" {
            Some(file_path)
          } else {
            None
          }
        } else {
          None
        }
      })
      .collect::<Vec<_>>();
    let mut aliases = yaml_files
      .into_iter()
      .filter_map(|yaml_file| {
        let filename = yaml_file.clone().display().to_string();
        match fs::read_to_string(yaml_file) {
          Ok(content) => match serde_yaml::from_str::<Alias>(&content) {
            Ok(alias) => Some(alias),
            Err(err) => {
              let err = DataServiceError::SerdeYamlDeserialize(err);
              tracing::warn!(filename, ?err, "Error deserializing model alias YAML file",);
              None
            }
          },
          Err(err) => {
            let err = DataServiceError::Io(err);
            tracing::warn!(filename, ?err, "Error reading model alias YAML file");
            None
          }
        }
      })
      .collect::<Vec<_>>();
    aliases.sort_by(|a, b| a.alias.cmp(&b.alias));
    Ok(aliases)
  }

  fn find_alias(&self, alias: &str) -> Option<Alias> {
    self
      .list_aliases()
      .unwrap_or_default()
      .into_iter()
      .find(|obj| obj.alias.eq(&alias))
  }
}

#[cfg_attr(test, automock)]
pub trait DataService: Debug {
  fn list_aliases(&self) -> Result<Vec<Alias>>;

  fn find_remote_model(&self, alias: &str) -> Result<Option<RemoteModel>>;

  fn save_alias(&self, alias: Alias) -> Result<PathBuf>;

  fn find_alias(&self, alias: &str) -> Option<Alias>;
}

pub struct HfHubService {
  cache: Cache,
  progress_bar: bool,
  token: Option<String>,
}

impl Default for HfHubService {
  fn default() -> Self {
    let cache = Cache::default();
    let token = cache.token();
    Self {
      cache,
      progress_bar: Default::default(),
      token,
    }
  }
}

impl Debug for HfHubService {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    let token_display = self
      .token
      .clone()
      .map(|_| String::from("****"))
      .unwrap_or(String::from("None"));
    f.debug_struct("HubService")
      .field("cache", &self.cache.path())
      .field("progress_bar", &self.progress_bar)
      .field("token", &token_display)
      .finish()
  }
}

impl HfHubService {
  pub fn new(hf_cache: PathBuf, progress_bar: bool, token: Option<String>) -> Self {
    Self {
      cache: Cache::new(hf_cache),
      progress_bar,
      token,
    }
  }

  fn download_sync(&self, repo: &str, filename: &str) -> Result<PathBuf> {
    use hf_hub::api::sync::{ApiBuilder, ApiError};

    let api = ApiBuilder::from_cache(self.cache.clone())
      .with_progress(self.progress_bar)
      .with_token(self.token.clone())
      .build()?;
    tracing::info!("Downloading from repo {repo}, file {filename}:");
    let path = match api.model(repo.to_string()).download(filename) {
      Ok(path) => path,
      Err(err) => {
        let err = match err {
          ApiError::RequestError(ureq_err) => match *ureq_err {
            ureq::Error::Status(status, response) if status == 403 => {
              DataServiceError::GatedAccess {
                source: ApiError::RequestError(Box::new(ureq::Error::Status(status, response))),
                repo: repo.to_string(),
              }
            }
            ureq::Error::Status(status, response) if self.token.is_none() && status == 401 => {
              DataServiceError::MayBeNotExists {
                source: ApiError::RequestError(Box::new(ureq::Error::Status(status, response))),
                repo: repo.to_string(),
              }
            }
            ureq_err => ApiError::RequestError(Box::new(ureq_err)).into(),
          },
          _ => err.into(),
        };
        return Err(err);
      }
    };
    Ok(path)
  }
}

impl HubService for HfHubService {
  fn download(&self, repo: &str, filename: &str, force: bool) -> Result<PathBuf> {
    let hf_repo = self.cache.repo(Repo::model(repo.to_string()));
    let from_cache = hf_repo.get(filename);
    match from_cache {
      Some(path) if !force => Ok(path),
      Some(_) | None => {
        let path = self.download_sync(repo, filename)?;
        Ok(path)
      }
    }
  }
}

pub trait AppServiceFn: HubService + DataService {}

#[derive(Debug, new)]
pub struct AppService {
  pub(super) hub_service: Box<dyn HubService>,
  pub(super) data_service: Box<dyn DataService>,
}

impl Default for AppService {
  fn default() -> Self {
    Self {
      hub_service: Box::<HfHubService>::default(),
      data_service: Box::<LocalDataService>::default(),
    }
  }
}

impl HubService for AppService {
  fn download(&self, repo: &str, filename: &str, force: bool) -> Result<PathBuf> {
    self.hub_service.download(repo, filename, force)
  }
}

impl DataService for AppService {
  fn find_remote_model(&self, alias: &str) -> Result<Option<RemoteModel>> {
    self.data_service.find_remote_model(alias)
  }

  fn save_alias(&self, alias: Alias) -> Result<PathBuf> {
    self.data_service.save_alias(alias)
  }

  fn list_aliases(&self) -> Result<Vec<Alias>> {
    self.data_service.list_aliases()
  }

  fn find_alias(&self, alias: &str) -> Option<Alias> {
    self.data_service.find_alias(alias)
  }
}

impl AppServiceFn for AppService {}

#[cfg(test)]
mod test {
  use super::HfHubService;
  use crate::cli::ChatTemplateId;
  use crate::objs::{Alias, ChatTemplate, Repo};
  use crate::server::BODHI_HOME;
  use crate::service::{DataService, HubService, LocalDataService};
  use crate::test_utils::{
    bodhi_home, data_service, hf_test_token_allowed, hf_test_token_public, temp_bodhi_home,
    temp_hf_home, DataServiceTuple,
  };
  use anyhow_trace::anyhow_trace;
  use rstest::{fixture, rstest};
  use serde_yaml;
  use std::fs;
  use std::path::Path;
  use std::path::PathBuf;
  use tempfile::{tempdir, TempDir};
  use tracing::{event, Level};

  #[rstest]
  #[case(None)]
  #[case(hf_test_token_public())]
  fn test_hf_hub_service_download_public_file(
    temp_hf_home: TempDir,
    #[case] token: Option<String>,
  ) -> anyhow::Result<()> {
    let hf_cache = temp_hf_home.path().join("huggingface/hub");
    let service = HfHubService::new(hf_cache, false, token);
    let dest_file = service.download("amir36/test-model-repo", "tokenizer_config.json", false)?;
    assert!(dest_file.exists());
    let expected = temp_hf_home.path().join("huggingface/hub/models--amir36--test-model-repo/snapshots/f7d5db77208ab98318b45cba4a48fc33a47fe4f6/tokenizer_config.json").display().to_string();
    assert_eq!(expected, dest_file.display().to_string());
    let expected = r#"{
  "hello": "world"
}"#;
    assert_eq!(expected, fs::read_to_string(dest_file)?);
    Ok(())
  }

  #[rstest]
  #[case(None, r#"request error: https://huggingface.co/amir36/test-gated-repo/resolve/main/tokenizer_config.json: status code 401
You are not logged in to huggingface using CLI `huggingface-cli login`.
So either the huggingface repo 'amir36/test-gated-repo' does not exists, or is private, or requires request access.
Go to https://huggingface.co/amir36/test-gated-repo to request access, login via CLI, and then try again.
"#)]
  #[case(hf_test_token_public(), r#"request error: https://huggingface.co/amir36/test-gated-repo/resolve/main/tokenizer_config.json: status code 403
huggingface repo 'amir36/test-gated-repo' is requires requesting for access from website.
Go to https://huggingface.co/amir36/test-gated-repo to request access to the model and try again.
"#)]
  fn test_hf_hub_service_download_gated_file_not_allowed(
    temp_hf_home: TempDir,
    #[case] token: Option<String>,
    #[case] expected: String,
  ) -> anyhow::Result<()> {
    let hf_cache = temp_hf_home.path().join("huggingface/hub");
    let service = HfHubService::new(hf_cache, false, token);
    let dest_file = service.download("amir36/test-gated-repo", "tokenizer_config.json", false);
    assert!(dest_file.is_err());
    assert_eq!(expected, dest_file.unwrap_err().to_string());
    Ok(())
  }

  #[rstest]
  #[case(hf_test_token_allowed())]
  fn test_hf_hub_service_download_gated_file_allowed(
    temp_hf_home: TempDir,
    #[case] token: Option<String>,
  ) -> anyhow::Result<()> {
    let hf_cache = temp_hf_home.path().join("huggingface/hub");
    let service = HfHubService::new(hf_cache, false, token);
    let dest_file = service.download("amir36/test-gated-repo", "tokenizer_config.json", false)?;
    assert!(dest_file.exists());
    let expected = temp_hf_home.path().join("huggingface/hub/models--amir36--test-gated-repo/snapshots/6ac8c08e39d0f68114b63ea98900632abcfb6758/tokenizer_config.json").display().to_string();
    assert_eq!(expected, dest_file.display().to_string());
    let expected = r#"{
  "hello": "world"
}"#;
    assert_eq!(expected, fs::read_to_string(dest_file)?);
    Ok(())
  }

  #[rstest]
  #[case(None, r#"request error: https://huggingface.co/amir36/not-exists/resolve/main/tokenizer_config.json: status code 401
You are not logged in to huggingface using CLI `huggingface-cli login`.
So either the huggingface repo 'amir36/not-exists' does not exists, or is private, or requires request access.
Go to https://huggingface.co/amir36/not-exists to request access, login via CLI, and then try again.
"#)]
  #[case(hf_test_token_public(), "request error: https://huggingface.co/amir36/not-exists/resolve/main/tokenizer_config.json: status code 404")]
  #[case(hf_test_token_allowed(), "request error: https://huggingface.co/amir36/not-exists/resolve/main/tokenizer_config.json: status code 404")]
  fn test_hf_hub_service_download_request_error_not_found(
    temp_hf_home: TempDir,
    #[case] token: Option<String>,
    #[case] error: String,
  ) -> anyhow::Result<()> {
    let hf_cache = temp_hf_home.path().join("huggingface/hub");
    let service = HfHubService::new(hf_cache, false, token);
    let dest_file = service.download("amir36/not-exists", "tokenizer_config.json", false);
    assert!(dest_file.is_err());
    assert_eq!(error, dest_file.unwrap_err().to_string());
    Ok(())
  }

  #[rstest]
  fn test_local_data_service_models_file_missing(
    data_service: DataServiceTuple,
  ) -> anyhow::Result<()> {
    let DataServiceTuple(_temp_bodhi_home, bodhi_home, service) = data_service;
    fs::remove_file(bodhi_home.join("models.yaml"))?;
    let result = service.find_remote_model("testalias-neverdownload:instruct");
    assert!(result.is_err());
    let expected = r#"file 'models.yaml' not found in $BODHI_HOME/.
$BODHI_HOME might not have been initialized. Run `bodhi init` to setup $BODHI_HOME."#;
    assert_eq!(expected, result.unwrap_err().to_string());
    Ok(())
  }

  #[rstest]
  #[anyhow_trace]
  fn test_local_data_service_models_file_corrupt(
    data_service: DataServiceTuple,
  ) -> anyhow::Result<()> {
    let DataServiceTuple(_temp, bodhi_home, service) = data_service;
    let models_file = bodhi_home.join("models.yaml");
    fs::write(
      &models_file,
      r#"
# alias is missing
- family: testalias
repo: MyFactory/testalias-neverdownload-gguf
filename: testalias-neverdownload.Q8_0.gguf
features:
  - chat
chat_template: llama3
"#,
    )?;
    let result = service.find_remote_model("testalias-neverdownload:instruct");
    assert!(result.is_err());
    let models_file = models_file.display().to_string();
    let expected = format!(
      r#".[0]: missing field `alias` at line 3 column 3
error while serializing from file: '{models_file}'"#
    );
    assert_eq!(expected, result.unwrap_err().to_string());
    Ok(())
  }

  #[rstest]
  #[case("testalias-neverdownload:instruct", true)]
  #[case("testalias-notexists", false)]
  fn test_local_data_service_find_remote_model(
    data_service: DataServiceTuple,
    #[case] alias: String,
    #[case] found: bool,
  ) -> anyhow::Result<()> {
    let DataServiceTuple(_temp, _, service) = data_service;
    let remote_model = service.find_remote_model(&alias)?;
    assert_eq!(found, remote_model.is_some());
    Ok(())
  }

  #[rstest]
  fn test_local_data_service_find_alias(data_service: DataServiceTuple) -> anyhow::Result<()> {
    let DataServiceTuple(_temp, _, service) = data_service;
    let alias = service.find_alias("testalias-exists:instruct");
    let expected = Alias::new(
      String::from("testalias-exists:instruct"),
      Some(String::from("testalias")),
      Some(Repo::new(String::from(
        "MyFactory/testalias-exists-instruct-gguf",
      ))),
      Some(String::from("testalias-exists-instruct.Q8_0.gguf")),
      vec![String::from("chat")],
      ChatTemplate::Id(ChatTemplateId::Llama3),
    );
    assert_eq!(Some(expected), alias);
    Ok(())
  }

  #[rstest]
  fn test_local_data_service_list_aliases(data_service: DataServiceTuple) -> anyhow::Result<()> {
    let DataServiceTuple(_temp, _, service) = data_service;
    let result = service.list_aliases()?;
    let expected = vec![
      Alias::new(
        String::from("llama3:instruct"),
        Some(String::from("llama3")),
        Some(Repo::new(String::from(
          "QuantFactory/Meta-Llama-3-8B-Instruct-GGUF",
        ))),
        Some(String::from("Meta-Llama-3-8B-Instruct.Q8_0.gguf")),
        vec![String::from("chat")],
        ChatTemplate::Id(ChatTemplateId::Llama3),
      ),
      Alias::new(
        String::from("testalias-exists:instruct"),
        Some(String::from("testalias")),
        Some(Repo::new(String::from(
          "MyFactory/testalias-exists-instruct-gguf",
        ))),
        Some(String::from("testalias-exists-instruct.Q8_0.gguf")),
        vec![String::from("chat")],
        ChatTemplate::Id(ChatTemplateId::Llama3),
      ),
      Alias::new(
        String::from("testalias-neverdownload:instruct"),
        Some(String::from("testalias")),
        Some(Repo::new(String::from(
          "MyFactory/testalias-neverdownload-gguf",
        ))),
        Some(String::from("testalias-neverdownload.Q8_0.gguf")),
        vec![String::from("chat")],
        ChatTemplate::Id(ChatTemplateId::Llama3),
      ),
    ];
    assert_eq!(expected, result);
    Ok(())
  }

  #[rstest]
  fn test_local_data_service_list_alias_dir_missing_error(
    data_service: DataServiceTuple,
  ) -> anyhow::Result<()> {
    let DataServiceTuple(_temp, bodhi_home, service) = data_service;
    fs::remove_dir_all(bodhi_home.join("configs"))?;
    let result = service.list_aliases();
    assert!(result.is_err());
    let expected = r#"directory 'configs' not found in $BODHI_HOME.
$BODHI_HOME might not have been initialized. Run `bodhi init` to setup $BODHI_HOME."#;
    assert_eq!(expected, result.unwrap_err().to_string());
    Ok(())
  }

  #[rstest]
  fn test_local_data_service_default_from_bodhi_home() -> anyhow::Result<()> {
    let bodhi_home = tempdir()?;
    std::env::set_var(BODHI_HOME, bodhi_home.path());
    let service = LocalDataService::default();
    let expected = LocalDataService::new(bodhi_home.path().to_path_buf());
    assert_eq!(expected, service);
    Ok(())
  }

  #[rstest]
  fn test_local_data_service_default_from_home_dir() -> anyhow::Result<()> {
    let home_dir = tempdir()?;
    std::env::remove_var(BODHI_HOME);
    std::env::set_var("HOME", home_dir.path());
    let service = LocalDataService::default();
    let expected =
      LocalDataService::new(home_dir.path().join(".cache").join("bodhi").to_path_buf());
    assert_eq!(expected, service);
    Ok(())
  }
}
