use hf_hub::{api::sync::ApiError, Cache};
use objs::{HubFile, ObjError, Repo, REFS, REFS_MAIN};
use std::{
  collections::HashSet,
  fmt::{Debug, Formatter},
  fs,
  path::PathBuf,
};
use walkdir::WalkDir;

#[derive(Debug, thiserror::Error)]
pub enum HubServiceError {
  #[error("file '{filename}' not found in $HF_HOME repo '{repo}'")]
  ModelFileMissing { filename: String, repo: String },
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
  #[error("only files from refs/main supported")]
  OnlyRefsMainSupported,
  #[error(transparent)]
  ObjError(#[from] ObjError),
  #[error(
    r#"file '{filename}' not found in $HF_HOME{dirname}.
Check Huggingface Home is set correctly using environment variable $HF_HOME or using command-line or settings file."#
  )]
  FileMissing { filename: String, dirname: String },

  #[error("chat_template not found in tokenizer_config.json")]
  ChatTemplate,
}

type Result<T> = std::result::Result<T, HubServiceError>;

#[cfg_attr(test, mockall::automock)]
pub trait HubService: std::fmt::Debug {
  fn download(&self, repo: &Repo, filename: &str, force: bool) -> Result<HubFile>;

  fn list_local_models(&self) -> Vec<HubFile>;

  fn find_local_file(&self, repo: &Repo, filename: &str, snapshot: &str)
    -> Result<Option<HubFile>>;

  fn model_file_path(&self, repo: &Repo, filename: &str, snapshot: &str) -> PathBuf;

  fn list_local_tokenizer_configs(&self) -> Vec<Repo>;
}

impl HfHubService {
  fn hf_cache(&self) -> PathBuf {
    self.cache.path().to_path_buf()
  }

  fn hf_home(&self) -> PathBuf {
    self
      .cache
      .path()
      .join("..")
      .canonicalize()
      .unwrap_or_else(|_| self.hf_cache().join(".."))
  }
}

impl HubService for HfHubService {
  fn download(&self, repo: &Repo, filename: &str, force: bool) -> Result<HubFile> {
    let hf_repo = self.cache.repo(hf_hub::Repo::model(repo.to_string()));
    let from_cache = hf_repo.get(filename);
    let path = match from_cache {
      Some(path) if !force => path,
      Some(_) | None => self.download_sync(repo, filename)?,
    };
    let result = HubFile::try_from(path)?;
    Ok(result)
  }

  fn list_local_models(&self) -> Vec<HubFile> {
    let cache = self.hf_cache();
    WalkDir::new(cache)
      .follow_links(true)
      .into_iter()
      .filter_map(|e| e.ok())
      .filter(|entry| entry.path().is_file())
      .filter_map(|entry| {
        let path = entry.path().to_path_buf();
        let local_model_file = match HubFile::try_from(path.clone()) {
          Ok(local_model_file) => local_model_file,
          Err(_) => {
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
  ) -> Result<Option<HubFile>> {
    let snapshot = if snapshot.starts_with(REFS) {
      if !snapshot.eq(REFS_MAIN) {
        return Err(HubServiceError::OnlyRefsMainSupported);
      }
      let refs_file = self.hf_cache().join(repo.path()).join(snapshot);
      if !refs_file.exists() {
        return Ok(None);
      }
      std::fs::read_to_string(refs_file.clone()).map_err(|_err| {
        let dirname = refs_file
          .parent()
          .map(|f| f.display().to_string())
          .unwrap_or(String::from("<unknown>"));
        let filename = refs_file
          .file_name()
          .map(|f| f.to_string_lossy().into_owned())
          .unwrap_or(String::from("<unknown>"));
        let hf_home = self.hf_home().display().to_string();
        let relative = dirname
          .strip_prefix(&hf_home)
          .unwrap_or_else(|| &dirname)
          .to_string();
        HubServiceError::FileMissing {
          filename,
          dirname: relative,
        }
      })?
    } else {
      snapshot.to_owned()
    };
    let filepath = self
      .hf_cache()
      .join(repo.path())
      .join("snapshots")
      .join(snapshot.clone())
      .join(filename);
    if filepath.exists() {
      let size = match fs::metadata(&filepath) {
        Ok(metadata) => Some(metadata.len()),
        Err(_) => None,
      };
      let local_model_file = HubFile::new(
        self.hf_cache(),
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

  fn model_file_path(&self, repo: &Repo, filename: &str, snapshot: &str) -> PathBuf {
    let model_repo = hf_hub::Repo::model(repo.to_string());
    self
      .hf_cache()
      .join(model_repo.folder_name())
      .join("snapshots")
      .join(snapshot)
      .join(filename)
  }

  fn list_local_tokenizer_configs(&self) -> Vec<Repo> {
    // TODO: support non-main snapshots
    let cache = self.hf_cache();
    let mut unique_repos = HashSet::new();

    if let Ok(entries) = fs::read_dir(&cache) {
      for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_dir() {
          if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
            if dir_name.starts_with("models--") {
              let main_ref_path = path.join("refs").join("main");
              if let Ok(snapshot) = fs::read_to_string(main_ref_path) {
                let snapshot = snapshot.trim();
                let tokenizer_config_path = path
                  .join("snapshots")
                  .join(snapshot)
                  .join("tokenizer_config.json");

                if tokenizer_config_path.exists() {
                  if let Some(repo_path) = dir_name.strip_prefix("models--") {
                    let repo_parts: Vec<&str> = repo_path.split("--").collect();
                    if repo_parts.len() >= 2 {
                      let owner = repo_parts[0];
                      let repo_name = repo_parts[1..].join("/");
                      let repo_string = format!("{}/{}", owner, repo_name);
                      if let Ok(repo) = Repo::try_from(repo_string) {
                        unique_repos.insert(repo);
                      }
                    }
                  }
                }
              }
            }
          }
        }
      }
    }
    unique_repos.into_iter().collect()
  }
}

#[derive(Clone)]
pub struct HfHubService {
  cache: Cache,
  progress_bar: bool,
  token: Option<String>,
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

  pub fn new_from_cache(cache: Cache, progress_bar: bool) -> Self {
    let token = cache.token();
    Self {
      cache,
      progress_bar,
      token,
    }
  }

  pub fn new_from_hf_cache(hf_cache: PathBuf, progress_bar: bool) -> Self {
    let cache = Cache::new(hf_cache);
    let token = cache.token();
    Self {
      cache,
      progress_bar,
      token,
    }
  }

  pub fn progress_bar(&mut self, progress_bar: bool) {
    self.progress_bar = progress_bar;
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
              HubServiceError::GatedAccess {
                source: ApiError::RequestError(Box::new(ureq::Error::Status(status, response))),
                repo: repo.to_string(),
              }
            }
            ureq::Error::Status(status, response) if self.token.is_none() && status == 401 => {
              HubServiceError::MayBeNotExists {
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

#[cfg(test)]
mod test {
  use super::{HfHubService, HubService};
  use crate::test_utils::{
    hf_test_token_allowed, hf_test_token_public, hub_service, HubServiceTuple,
  };
  use objs::test_utils::temp_hf_home;
  use objs::{HubFile, Repo, REFS_MAIN};
  use rstest::rstest;
  use std::{collections::HashSet, fs};
  use tempfile::TempDir;

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
      &Repo::try_from("amir36/test-model-repo")?,
      "tokenizer_config.json",
      false,
    )?;
    assert!(local_model_file.path().exists());
    let expected = HubFile::new(
      hf_cache,
      Repo::try_from("amir36/test-model-repo")?,
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
      &Repo::try_from("amir36/test-gated-repo")?,
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
      &Repo::try_from("amir36/test-gated-repo")?,
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
  fn test_hf_hub_service_find_local_file(
    hub_service: HubServiceTuple,
    #[case] snapshot: String,
    #[case] expected: String,
  ) -> anyhow::Result<()> {
    let HubServiceTuple(_temp, _, service) = hub_service;
    let repo = Repo::try_from("meta-llama/Llama-2-70b-chat-hf")?;
    let filename = "tokenizer_config.json";
    let local_model_file = service
      .find_local_file(&repo, filename, &snapshot)?
      .unwrap();
    let content = fs::read_to_string(local_model_file.path())?;
    assert_eq!(expected, content);
    Ok(())
  }

  #[rstest]
  fn test_hf_hub_service_find_local_model_not_present(
    hub_service: HubServiceTuple,
  ) -> anyhow::Result<()> {
    let HubServiceTuple(_temp, _, service) = hub_service;
    let repo = Repo::try_from("meta-llama/Llama-2-70b-chat-hf")?;
    let filename = "tokenizer_config.json";
    let local_model_file =
      service.find_local_file(&repo, filename, "cfe96d938c52db7c6d936f99370c0801b24233c4")?;
    assert!(local_model_file.is_none());
    Ok(())
  }

  #[rstest]
  fn test_hf_hub_service_find_local_model_err_on_non_main_refs(
    hub_service: HubServiceTuple,
  ) -> anyhow::Result<()> {
    let HubServiceTuple(_temp, _, service) = hub_service;
    let repo = Repo::try_from("meta-llama/Llama-2-70b-chat-hf")?;
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
      &Repo::try_from("amir36/not-exists")?,
      "tokenizer_config.json",
      false,
    );
    assert!(local_model_file.is_err());
    assert_eq!(error, local_model_file.unwrap_err().to_string());
    Ok(())
  }

  #[rstest]
  fn test_hf_hub_service_find_local_file_returns_not_found_if_refs_main_not_present(
    hub_service: HubServiceTuple,
  ) -> anyhow::Result<()> {
    let HubServiceTuple(_temp_hf_home, _hf_cache, service) = hub_service;
    let result = service.find_local_file(
      &Repo::try_from("TheBloke/NotDownloaded")?,
      "some-model-file.gguf",
      REFS_MAIN,
    )?;
    assert!(result.is_none());
    Ok(())
  }

  #[rstest]
  fn test_hf_hub_service_list_local_models(hub_service: HubServiceTuple) -> anyhow::Result<()> {
    let HubServiceTuple(_temp_hf_home, hf_cache, service) = hub_service;
    let models = service.list_local_models();
    let expected_1 = HubFile::new(
      hf_cache,
      Repo::try_from("google/gemma-1.1-2b-it-GGUF")?,
      "2b_it_v1p1.gguf".to_string(),
      "5007652f7a641fe7170e0bad4f63839419bd9213".to_string(),
      Some(21),
    );
    assert_eq!(4, models.len());
    assert_eq!(&expected_1, models.first().unwrap());
    Ok(())
  }

  #[rstest]
  fn test_hf_hub_service_list_local_tokenizer_configs(
    hub_service: HubServiceTuple,
  ) -> anyhow::Result<()> {
    let HubServiceTuple(_temp_hf_home, _hf_cache, service) = hub_service;
    let repos = service.list_local_tokenizer_configs();
    assert_eq!(4, repos.len(), "Expected 4 repos with tokenizer configs");
    let expected_repos: HashSet<Repo> = [
      "meta-llama/Llama-2-70b-chat-hf",
      "meta-llama/Meta-Llama-3-70B-Instruct",
      "meta-llama/Meta-Llama-3-8B-Instruct",
      "MyFactory/testalias-gguf",
    ]
    .iter()
    .map(|&s| Repo::try_from(s).unwrap())
    .collect();
    let result_set: HashSet<Repo> = repos.into_iter().collect();
    assert_eq!(
      expected_repos, result_set,
      "Mismatch in expected and actual repos"
    );
    Ok(())
  }
}
