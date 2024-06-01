use crate::{
  objs::{Alias, LocalModelFile, RemoteModel, Repo, REFS, REFS_MAIN},
  server::BODHI_HOME,
};
use derive_new::new;
use hf_hub::{api::sync::ApiError, Cache};
#[cfg(test)]
use mockall::automock;
use std::{
  fmt::{Debug, Formatter},
  fs, io,
  path::PathBuf,
  sync::Arc,
};
use thiserror::Error;
use validator::ValidationErrors;
use walkdir::WalkDir;

static MODELS_YAML: &str = "models.yaml";

#[derive(Debug, Error)]
pub enum DataServiceError {
  #[error("mutex error: {0}")]
  MutexPoison(String),
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
  #[error("{source}\npath: {path}")]
  IoWithDetail {
    #[source]
    source: io::Error,
    path: PathBuf,
  },
  #[error("{source}\nerror while serializing from file: '{filename}'")]
  SerdeYamlSerialize {
    #[source]
    source: serde_yaml::Error,
    filename: String,
  },
  #[error(transparent)]
  SerdeYamlDeserialize(#[from] serde_yaml::Error),
  #[error(transparent)]
  SerdeJsonDeserialize(#[from] serde_json::Error),
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
  #[error(transparent)]
  Validation(#[from] ValidationErrors),
  #[error("{0}")]
  BadRequest(String),
  #[error(transparent)]
  Anyhow(#[from] anyhow::Error),
  #[error("only files from refs/main supported")]
  OnlyRefsMainSupported,
}

pub type Result<T> = std::result::Result<T, DataServiceError>;

#[cfg_attr(test, automock)]
pub trait HubService: Debug {
  fn download(&self, repo: &Repo, filename: &str, force: bool) -> Result<LocalModelFile>;

  fn list_local_models(&self) -> Vec<LocalModelFile>;

  fn find_local_file(
    &self,
    repo: &Repo,
    filename: &str,
    snapshot: &str,
  ) -> Result<Option<LocalModelFile>>;

  fn hf_home(&self) -> PathBuf;

  fn model_file_path(&self, repo: &Repo, filename: &str, snapshot: &str) -> PathBuf;
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
    let models = self.list_remote_models()?;
    Ok(models.into_iter().find(|model| model.alias.eq(alias)))
  }

  fn save_alias(&self, alias: Alias) -> Result<PathBuf> {
    let contents = serde_yaml::to_string(&alias)?;
    let filename = self
      .bodhi_home
      .join("configs")
      .join(alias.config_filename());
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

  fn list_remote_models(&self) -> Result<Vec<RemoteModel>> {
    let models_file = self.bodhi_home.join(MODELS_YAML);
    if !models_file.exists() {
      return Err(DataServiceError::FileMissing {
        filename: String::from(MODELS_YAML),
        dirname: "".to_string(),
      });
    }
    let content = fs::read_to_string(models_file.clone())?;
    let models = serde_yaml::from_str::<Vec<RemoteModel>>(&content).map_err(|err| {
      DataServiceError::SerdeYamlSerialize {
        source: err,
        filename: models_file.display().to_string(),
      }
    })?;
    Ok(models)
  }
}

#[cfg_attr(test, automock)]
pub trait DataService: Debug {
  fn list_aliases(&self) -> Result<Vec<Alias>>;

  fn save_alias(&self, alias: Alias) -> Result<PathBuf>;

  fn find_alias(&self, alias: &str) -> Option<Alias>;

  fn list_remote_models(&self) -> Result<Vec<RemoteModel>>;

  fn find_remote_model(&self, alias: &str) -> Result<Option<RemoteModel>>;
}

#[derive(Clone)]
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
  fn download(&self, repo: &Repo, filename: &str, force: bool) -> Result<LocalModelFile> {
    let hf_repo = self.cache.repo(hf_hub::Repo::model(repo.to_string()));
    let from_cache = hf_repo.get(filename);
    let path = match from_cache {
      Some(path) if !force => path,
      Some(_) | None => self.download_sync(repo, filename)?,
    };
    path.try_into()
  }

  fn list_local_models(&self) -> Vec<LocalModelFile> {
    let cache = self.cache.path().as_path();
    WalkDir::new(cache)
      .follow_links(true)
      .into_iter()
      .filter_map(|e| e.ok())
      .filter(|entry| entry.path().is_file())
      .filter_map(|entry| {
        let path = entry.path().to_path_buf();
        let local_model_file = match LocalModelFile::try_from(path.clone()) {
          Ok(local_model_file) => local_model_file,
          Err(err) => {
            tracing::info!(?err, ?path, "error converting Path to LocalModelFile");
            return None;
          }
        };
        if local_model_file.filename.ends_with(".gguf") {
          Some(local_model_file)
        } else {
          None
        }
      })
      .collect::<Vec<_>>()
  }

  fn find_local_file(
    &self,
    repo: &Repo,
    filename: &str,
    snapshot: &str,
  ) -> Result<Option<LocalModelFile>> {
    let snapshot = if snapshot.starts_with(REFS) {
      if !snapshot.eq(REFS_MAIN) {
        return Err(DataServiceError::OnlyRefsMainSupported);
      }
      let refs_file = self
        .cache
        .path()
        .to_path_buf()
        .join(repo.path())
        .join(snapshot);
      std::fs::read_to_string(refs_file.clone()).map_err(|err| {
        let dirname = refs_file
          .parent()
          .map(|f| f.display().to_string())
          .unwrap_or(String::from("<unknown>"));
        let filename = refs_file
          .file_name()
          .map(|f| f.to_string_lossy().into_owned())
          .unwrap_or(String::from("<unknown>"));
        // TODO FileMissing instead of IoErr, not using err field
        DataServiceError::FileMissing { filename, dirname }
      })?
    } else {
      snapshot.to_owned()
    };
    let filepath = self
      .hf_home()
      .join(repo.path())
      .join("snapshots")
      .join(snapshot.clone())
      .join(filename);
    if filepath.exists() {
      let size = match fs::metadata(&filepath) {
        Ok(metadata) => Some(metadata.len()),
        Err(_) => None,
      };
      let local_model_file = LocalModelFile::new(
        self.hf_home(),
        repo.clone(),
        filename.to_string(),
        snapshot.to_string(),
        size,
      );
      Ok(Some(local_model_file))
    } else {
      Ok(None)
    }
  }

  fn hf_home(&self) -> PathBuf {
    self.cache.path().to_path_buf()
  }

  fn model_file_path(&self, repo: &Repo, filename: &str, snapshot: &str) -> PathBuf {
    let model_repo = hf_hub::Repo::model(repo.to_string());
    self
      .hf_home()
      .join(model_repo.folder_name())
      .join("snapshots")
      .join(snapshot)
      .join(filename)
  }
}

pub trait AppServiceFn: HubService + DataService + Send + Sync {}

#[derive(Debug, Clone)]
pub struct AppService {
  pub(super) hub_service: Arc<dyn HubService + Send + Sync>,
  pub(super) data_service: Arc<dyn DataService + Send + Sync>,
}

impl Default for AppService {
  fn default() -> Self {
    Self {
      hub_service: Arc::new(HfHubService::default()),
      data_service: Arc::new(LocalDataService::default()),
    }
  }
}

impl AppService {
  pub fn new(hub_service: HfHubService, data_service: LocalDataService) -> Self {
    Self {
      hub_service: Arc::new(hub_service),
      data_service: Arc::new(data_service),
    }
  }
}

impl HubService for AppService {
  fn download(&self, repo: &Repo, filename: &str, force: bool) -> Result<LocalModelFile> {
    self.hub_service.download(repo, filename, force)
  }

  fn list_local_models(&self) -> Vec<LocalModelFile> {
    self.hub_service.list_local_models()
  }

  fn find_local_file(
    &self,
    repo: &Repo,
    filename: &str,
    snapshot: &str,
  ) -> Result<Option<LocalModelFile>> {
    self.hub_service.find_local_file(repo, filename, snapshot)
  }

  fn hf_home(&self) -> PathBuf {
    self.hub_service.hf_home()
  }

  fn model_file_path(&self, repo: &Repo, filename: &str, snapshot: &str) -> PathBuf {
    self.hub_service.model_file_path(repo, filename, snapshot)
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

  fn list_remote_models(&self) -> Result<Vec<RemoteModel>> {
    self.data_service.list_remote_models()
  }
}

impl AppServiceFn for AppService {}

#[cfg(test)]
mod test {
  use super::HfHubService;
  use crate::objs::{Alias, LocalModelFile, RemoteModel, Repo};
  use crate::server::BODHI_HOME;
  use crate::service::{DataService, HubService, LocalDataService};
  use crate::test_utils::{
    data_service, hf_test_token_allowed, hf_test_token_public, hub_service, temp_hf_home,
    DataServiceTuple, HubServiceTuple,
  };
  use anyhow_trace::anyhow_trace;
  use rstest::rstest;
  use std::fs;
  use tempfile::{tempdir, TempDir};

  #[rstest]
  #[case(None)]
  #[case(hf_test_token_public())]
  fn test_hf_hub_service_download_public_file(
    temp_hf_home: TempDir,
    #[case] token: Option<String>,
  ) -> anyhow::Result<()> {
    let hf_cache = temp_hf_home.path().join("huggingface/hub");
    let service = HfHubService::new(hf_cache.clone(), false, token);
    let local_model_file = service.download(
      &Repo::try_new("amir36/test-model-repo".to_string())?,
      "tokenizer_config.json",
      false,
    )?;
    assert!(local_model_file.path().exists());
    let expected = LocalModelFile::new(
      hf_cache,
      Repo::try_new("amir36/test-model-repo".to_string())?,
      "tokenizer_config.json".to_string(),
      "f7d5db77208ab98318b45cba4a48fc33a47fe4f6".to_string(),
      Some(22),
    );
    assert_eq!(expected, local_model_file);
    let expected = r#"{
  "hello": "world"
}"#;
    assert_eq!(expected, fs::read_to_string(local_model_file.path())?);
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
    let local_model_file = service.download(
      &Repo::try_new("amir36/test-gated-repo".to_string())?,
      "tokenizer_config.json",
      false,
    );
    assert!(local_model_file.is_err());
    assert_eq!(expected, local_model_file.unwrap_err().to_string());
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
    let local_model_file = service.download(
      &Repo::try_new("amir36/test-gated-repo".to_string())?,
      "tokenizer_config.json",
      false,
    )?;
    let path = local_model_file.path();
    assert!(path.exists());
    let expected = temp_hf_home.path().join("huggingface/hub/models--amir36--test-gated-repo/snapshots/6ac8c08e39d0f68114b63ea98900632abcfb6758/tokenizer_config.json").display().to_string();
    assert_eq!(expected, path.display().to_string());
    let expected = r#"{
  "hello": "world"
}"#;
    assert_eq!(expected, fs::read_to_string(path)?);
    Ok(())
  }

  #[rstest]
  #[case("9ff8b00464fc439a64bb374769dec3dd627be1c2", "this is version 1\n")]
  #[case("e9149a12809580e8602995856f8098ce973d1080", "this is version 2\n")]
  #[case("refs/main", "this is version 2\n")]
  fn test_hub_service_find_local_file(
    hub_service: HubServiceTuple,
    #[case] snapshot: String,
    #[case] expected: String,
  ) -> anyhow::Result<()> {
    let HubServiceTuple(_temp, _, service) = hub_service;
    let repo = Repo::try_new("meta-llama/Llama-2-70b-chat-hf".to_string())?;
    let filename = "tokenizer_config.json";
    let local_model_file = service
      .find_local_file(&repo, filename, &snapshot)?
      .unwrap();
    let content = fs::read_to_string(local_model_file.path())?;
    assert_eq!(expected, content);
    Ok(())
  }

  #[rstest]
  fn test_hub_service_find_local_model_not_present(
    hub_service: HubServiceTuple,
  ) -> anyhow::Result<()> {
    let HubServiceTuple(_temp, _, service) = hub_service;
    let repo = Repo::try_new("meta-llama/Llama-2-70b-chat-hf".to_string())?;
    let filename = "tokenizer_config.json";
    let local_model_file =
      service.find_local_file(&repo, filename, "cfe96d938c52db7c6d936f99370c0801b24233c4")?;
    assert!(local_model_file.is_none());
    Ok(())
  }

  #[rstest]
  fn test_hub_service_find_local_model_err_on_non_main_refs(
    hub_service: HubServiceTuple,
  ) -> anyhow::Result<()> {
    let HubServiceTuple(_temp, _, service) = hub_service;
    let repo = Repo::try_new("meta-llama/Llama-2-70b-chat-hf".to_string())?;
    let filename = "tokenizer_config.json";
    let result = service.find_local_file(&repo, filename, "refs/custom");
    assert!(result.is_err());
    assert_eq!(
      "only files from refs/main supported",
      result.unwrap_err().to_string()
    );
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
    let local_model_file = service.download(
      &Repo::try_new("amir36/not-exists".to_string())?,
      "tokenizer_config.json",
      false,
    );
    assert!(local_model_file.is_err());
    assert_eq!(error, local_model_file.unwrap_err().to_string());
    Ok(())
  }

  #[rstest]
  fn test_local_data_service_models_file_missing(
    data_service: DataServiceTuple,
  ) -> anyhow::Result<()> {
    let DataServiceTuple(_temp_bodhi_home, bodhi_home, service) = data_service;
    fs::remove_file(bodhi_home.join("models.yaml"))?;
    let result = service.find_remote_model("testalias:instruct");
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
repo: MyFactory/testalias-gguf
filename: testalias.Q8_0.gguf
features:
  - chat
chat_template: llama3
"#,
    )?;
    let result = service.find_remote_model("testalias:instruct");
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
  #[case("testalias:instruct", true)]
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
  fn test_data_service_list_remote_models(data_service: DataServiceTuple) -> anyhow::Result<()> {
    let DataServiceTuple(_temp_dir, _, service) = data_service;
    let models = service.list_remote_models()?;
    let expected_1 = RemoteModel::llama3();
    let expected_2 = RemoteModel::test_alias();
    assert_eq!(6, models.len());
    assert!(models.contains(&expected_1));
    assert!(models.contains(&expected_2));
    Ok(())
  }

  #[rstest]
  fn test_local_data_service_find_alias(data_service: DataServiceTuple) -> anyhow::Result<()> {
    let DataServiceTuple(_temp, _, service) = data_service;
    let alias = service.find_alias("testalias-exists:instruct");
    let expected = Alias::test_alias_exists();
    assert_eq!(Some(expected), alias);
    Ok(())
  }

  #[rstest]
  fn test_local_data_service_list_aliases(data_service: DataServiceTuple) -> anyhow::Result<()> {
    let DataServiceTuple(_temp, _, service) = data_service;
    let result = service.list_aliases()?;
    assert_eq!(3, result.len());
    assert!(result.contains(&Alias::llama3()));
    assert!(result.contains(&Alias::test_alias_exists()));
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

  #[rstest]
  fn test_hf_hub_service_list_local_models(hub_service: HubServiceTuple) -> anyhow::Result<()> {
    let HubServiceTuple(_temp_hf_home, hf_cache, service) = hub_service;
    let models = service.list_local_models();
    let expected_1 = LocalModelFile::new(
      hf_cache,
      Repo::try_new("google/gemma-1.1-2b-it-GGUF".to_string())?,
      "2b_it_v1p1.gguf".to_string(),
      "5007652f7a641fe7170e0bad4f63839419bd9213".to_string(),
      Some(21),
    );
    assert_eq!(3, models.len());
    assert_eq!(&expected_1, models.first().unwrap());
    Ok(())
  }
}
