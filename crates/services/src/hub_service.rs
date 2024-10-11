use hf_hub::Cache;
use objs::{impl_error_from, AppError, ErrorType, HubFile, IoError, ObjValidationError, Repo};
use std::{
  collections::HashSet,
  fmt::{Debug, Formatter},
  fs,
  path::PathBuf,
};
use walkdir::WalkDir;

pub static SNAPSHOT_MAIN: &str = "main";

#[derive(Debug, PartialEq, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("hub_file_missing")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::NotFound, status = 404)]
pub struct HubFileNotFoundError {
  filename: String,
  repo: String,
  snapshot: String,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("remote_model_not_found")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::NotFound, status = 404)]
pub struct RemoteModelNotFoundError {
  alias: String,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("hub_api_error")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::InternalServer, status = self.error_status, code = self.error_code())]
pub struct HubApiError {
  error: String,
  error_status: u16,
  repo: String,
  kind: HubApiErrorKind,
}

impl HubApiError {
  pub fn error_code(&self) -> &str {
    match self.kind {
      HubApiErrorKind::GatedAccess => "hub_api_error-gated_access",
      HubApiErrorKind::NotExists => "hub_api_error-not_exists",
      HubApiErrorKind::MayBeNotExists => "hub_api_error-may_be_not_exists",
      HubApiErrorKind::Unknown => "hub_api_error-unknown",
      HubApiErrorKind::Transport => "hub_api_error-transport",
      HubApiErrorKind::Request => "hub_api_error-request",
    }
  }
}

#[derive(Debug, PartialEq, strum::Display)]
#[strum(serialize_all = "snake_case")]
pub enum HubApiErrorKind {
  GatedAccess,
  NotExists,
  MayBeNotExists,
  Unknown,
  Transport,
  Request,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum HubServiceError {
  #[error(transparent)]
  HubApiError(#[from] HubApiError),
  #[error(transparent)]
  HubFileNotFound(#[from] HubFileNotFoundError),
  #[error(transparent)]
  ObjValidationError(#[from] ObjValidationError),
  #[error(transparent)]
  IoError(#[from] IoError),
}

impl_error_from!(std::io::Error, HubServiceError::IoError, ::objs::IoError);

type Result<T> = std::result::Result<T, HubServiceError>;

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait HubService: std::fmt::Debug + Send + Sync {
  #[allow(clippy::needless_lifetimes)]
  fn download(&self, repo: &Repo, filename: &str, snapshot: Option<String>) -> Result<HubFile>;

  fn list_local_models(&self) -> Vec<HubFile>;

  fn find_local_file(
    &self,
    repo: &Repo,
    filename: &str,
    snapshot: Option<String>,
  ) -> Result<HubFile>;

  fn local_file_exists(
    &self,
    repo: &Repo,
    filename: &str,
    snapshot: Option<String>,
  ) -> Result<bool>;

  fn model_file_path(&self, repo: &Repo, filename: &str, snapshot: &str) -> PathBuf;

  fn list_local_tokenizer_configs(&self) -> Vec<Repo>;
}

impl HfHubService {
  fn hf_cache(&self) -> PathBuf {
    self.cache.path().to_path_buf()
  }

  #[allow(unused)]
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
  fn download(&self, repo: &Repo, filename: &str, snapshot: Option<String>) -> Result<HubFile> {
    let snapshot = snapshot.unwrap_or(SNAPSHOT_MAIN.to_string());
    let model_repo =
      hf_hub::Repo::with_revision(repo.to_string(), hf_hub::RepoType::Model, snapshot);
    let hf_repo = self.cache.repo(model_repo.clone());
    let from_cache = hf_repo.get(filename);
    let path = match from_cache {
      Some(path) => path,
      None => self.download_sync(model_repo, filename)?,
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

  fn local_file_exists(
    &self,
    repo: &Repo,
    filename: &str,
    snapshot: Option<String>,
  ) -> Result<bool> {
    let snapshot = snapshot.unwrap_or(SNAPSHOT_MAIN.to_string());
    let refs_file = self
      .hf_cache()
      .join(repo.path())
      .join(format!("refs/{}", snapshot));
    let snapshot = if refs_file.exists() {
      std::fs::read_to_string(refs_file)?
    } else {
      let snapshot_dir = self
        .hf_cache()
        .join(repo.path())
        .join("snapshots")
        .join(&snapshot);
      if snapshot_dir.exists() {
        snapshot
      } else {
        return Ok(false);
      }
    };
    let filepath = self
      .hf_cache()
      .join(repo.path())
      .join("snapshots")
      .join(&snapshot)
      .join(filename);
    Ok(filepath.exists())
  }

  fn find_local_file(
    &self,
    repo: &Repo,
    filename: &str,
    snapshot: Option<String>,
  ) -> Result<HubFile> {
    let snapshot = snapshot.unwrap_or(SNAPSHOT_MAIN.to_string());
    let refs_file = self
      .hf_cache()
      .join(repo.path())
      .join(format!("refs/{}", snapshot));
    let snapshot = if refs_file.exists() {
      std::fs::read_to_string(refs_file.clone())?
    } else {
      let snapshot_dir = self
        .hf_cache()
        .join(repo.path())
        .join("snapshots")
        .join(&snapshot);
      if snapshot_dir.exists() {
        snapshot
      } else {
        return Err(
          HubFileNotFoundError::new(filename.to_string(), repo.to_string(), snapshot).into(),
        );
      }
    };
    let filepath = self
      .hf_cache()
      .join(repo.path())
      .join("snapshots")
      .join(&snapshot)
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
      Ok(local_model_file)
    } else {
      Err(HubFileNotFoundError::new(filename.to_string(), repo.to_string(), snapshot).into())
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
              // TODO(snapshot support): list non-main tokenizer_config.json files
              let main_ref_path = path.join("refs").join(SNAPSHOT_MAIN);
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

  fn download_sync(&self, model_repo: hf_hub::Repo, filename: &str) -> Result<PathBuf> {
    use hf_hub::api::sync::{ApiBuilder, ApiError};

    let api = ApiBuilder::from_cache(self.cache.clone())
      .with_progress(self.progress_bar)
      .with_token(self.token.clone())
      .build()
      .map_err(|err| {
        HubApiError::new(
          err.to_string(),
          500,
          model_repo.url(),
          HubApiErrorKind::Request,
        )
      })?;
    tracing::info!("Downloading from url {}", model_repo.api_url());
    let repo = model_repo.url();
    let path = match api.repo(model_repo).download(filename) {
      Ok(path) => path,
      Err(err) => {
        let error_msg = err.to_string();
        let err = match err {
          ApiError::RequestError(ureq_err) => match *ureq_err {
            ureq::Error::Status(status, _response) if status == 403 => {
              HubApiError::new(error_msg, status, repo, HubApiErrorKind::GatedAccess)
            }
            ureq::Error::Status(status, _response) if self.token.is_none() && status == 401 => {
              HubApiError::new(error_msg, status, repo, HubApiErrorKind::MayBeNotExists)
            }
            ureq::Error::Status(status, _response) if self.token.is_some() && status == 404 => {
              HubApiError::new(error_msg, status, repo, HubApiErrorKind::NotExists)
            }
            ureq::Error::Status(status, _response) => {
              HubApiError::new(error_msg, status, repo, HubApiErrorKind::Unknown)
            }
            ureq::Error::Transport(_transport) => {
              HubApiError::new(error_msg, 500, repo, HubApiErrorKind::Transport)
            }
          },
          _ => HubApiError::new(error_msg, 500, repo, HubApiErrorKind::Request),
        };
        return Err(HubServiceError::HubApiError(err));
      }
    };
    Ok(path)
  }
}

#[cfg(test)]
mod test {
  use crate::{
    test_utils::{
      build_hf_service, hf_test_token_allowed, hf_test_token_public, setup_l10n_services,
      test_hf_service, TestHfService,
    },
    HubApiError, HubApiErrorKind, HubFileNotFoundError, HubService, HubServiceError,
    RemoteModelNotFoundError, SNAPSHOT_MAIN,
  };
  use anyhow_trace::anyhow_trace;
  use objs::{
    test_utils::{assert_error_message, temp_hf_home},
    AppError, FluentLocalizationService, HubFile, Repo,
  };
  use rstest::rstest;
  use std::{collections::HashSet, fs, sync::Arc};
  use strfmt::strfmt;
  use tempfile::TempDir;

  #[rstest]
  #[case(
    HubApiErrorKind::GatedAccess,
    r#"request error: https://someurl.com/my/repo: status code 403.
huggingface repo 'my/repo' is requires requesting for access from website.
Go to https://huggingface.co/my/repo to request access to the model and try again."#
  )]
  #[case(
    HubApiErrorKind::NotExists,
    r#"request error: https://someurl.com/my/repo: status code 403.
The huggingface repo 'my/repo' does not exists."#
  )]
  #[case(
    HubApiErrorKind::MayBeNotExists,
    r#"request error: https://someurl.com/my/repo: status code 403.
You are not logged in to huggingface using CLI `huggingface-cli login`.
So either the huggingface repo 'my/repo' does not exists, or is private, or requires request access.
Go to https://huggingface.co/my/repo to request access, login via CLI, and then try again."#
  )]
  #[case(
    HubApiErrorKind::Unknown,
    r#"request error: https://someurl.com/my/repo: status code 403.
An unknown error occurred accessing huggingface repo 'my/repo'."#
  )]
  #[case(HubApiErrorKind::Transport, r#"request error: https://someurl.com/my/repo: status code 403.
An error occurred while connecting to huggingface.co. Check your internet connection and try again."#)]
  #[case(
    HubApiErrorKind::Request,
    r#"request error: https://someurl.com/my/repo: status code 403.
An error occurred while requesting access to huggingface repo 'my/repo'."#
  )]
  #[serial_test::serial(localization)]
  fn test_hub_service_api_error(
    #[from(setup_l10n_services)] localization_service: Arc<FluentLocalizationService>,
    #[case] kind: HubApiErrorKind,
    #[case] message: String,
  ) {
    let error = HubApiError::new(
      "request error: https://someurl.com/my/repo: status code 403".to_string(),
      403,
      "my/repo".to_string(),
      kind,
    );
    assert_error_message(localization_service, &error.code(), error.args(), &message);
  }

  #[rstest]
  #[case(&HubFileNotFoundError::new(
    "testalias.gguf".to_string(),
    "test/repo".to_string(),
    "main".to_string(),
  ), "file 'testalias.gguf' not found in huggingface repo 'test/repo', snapshot 'main'")]
  #[case(&RemoteModelNotFoundError::new(
    "llama3:instruct".to_string(),
  ), "remote model alias 'llama3:instruct' not found, check your alias and try again")]
  #[serial_test::serial(localization)]
  fn test_hub_service_alias_not_found_error(
    #[from(setup_l10n_services)] localization_service: Arc<FluentLocalizationService>,
    #[case] error: &dyn AppError,
    #[case] expected: &str,
  ) {
    assert_error_message(localization_service, &error.code(), error.args(), expected);
  }

  #[rstest]
  #[case::anon(None, None, "2")]
  #[case::anon(None, Some("main".to_string()), "2")]
  #[case::anon(None, Some("7de0799b8c9c12eff96e5c9612e39b041b3f4f5b".to_string()), "2")]
  #[case::anon(None, Some("b19ae5e0a40d142016ea898e0ae6a1eb3f847b3f".to_string()), "1")]
  #[case::auth_public(hf_test_token_public(), None, "2")]
  #[case::auth_public(hf_test_token_public(), Some("main".to_string()), "2")]
  #[case::auth_public(
    hf_test_token_public(),
    Some("7de0799b8c9c12eff96e5c9612e39b041b3f4f5b".to_string()),
    "2"
  )]
  #[case::auth_public(
    hf_test_token_public(),
    Some("b19ae5e0a40d142016ea898e0ae6a1eb3f847b3f".to_string()),
    "1"
  )]
  fn test_hf_hub_service_download_public_file_with_snapshot(
    temp_hf_home: TempDir,
    #[case] token: Option<String>,
    #[case] snapshot: Option<String>,
    #[case] version: &str,
  ) -> anyhow::Result<()> {
    let hf_cache = temp_hf_home.path().join("huggingface/hub");
    let service = build_hf_service(token, temp_hf_home);
    let local_model_file = service.download(
      &Repo::try_from("amir36/test-model-repo")?,
      "tokenizer_config.json",
      snapshot.clone(),
    )?;
    assert!(local_model_file.path().exists());
    let mut sha = snapshot.unwrap_or("7de0799b8c9c12eff96e5c9612e39b041b3f4f5b".to_string());
    if sha == "main" {
      sha = "7de0799b8c9c12eff96e5c9612e39b041b3f4f5b".to_string();
    }
    let expected = HubFile::new(
      hf_cache,
      Repo::try_from("amir36/test-model-repo")?,
      "tokenizer_config.json".to_string(),
      sha,
      Some(20),
    );
    assert_eq!(expected, local_model_file);
    let expected = format!(
      r#"{{
  "version": "{version}"
}}"#
    );
    assert_eq!(expected, fs::read_to_string(local_model_file.path())?);
    Ok(())
  }

  const UNAUTH_ERR: &str = "request error: https://huggingface.co/{repo}/resolve/{sha}/tokenizer_config.json: status code 401";

  #[rstest]
  #[case::anon_not_exists("amir36/not-exists", None)]
  #[case::anon_not_exists("amir36/not-exists", Some("main".to_string()))]
  #[case::anon_not_exists("amir36/not-exists", Some("57a2b0118ef1cb0ab5d9544e5d9600d189f66a72".to_string()))]
  #[case::anon("amir36/test-gated-repo", None)]
  #[case::anon_main("amir36/test-gated-repo", Some("main".to_string()))]
  #[case::anon_latest("amir36/test-gated-repo", Some("57a2b0118ef1cb0ab5d9544e5d9600d189f66a72".to_string()))]
  #[case::anon_older("amir36/test-gated-repo", Some("6bbcc8a332f15cf670db6ec9e70f68427ae2ce27".to_string()))]
  #[case::anon_not_exists("amir36/test-gated-repo", Some("7de0799b8c9c12eff96e5c9612e39b041b3f4f5b".to_string()))]
  fn test_hf_hub_service_download_gets_unauth_error_if_downloading_as_anon(
    temp_hf_home: TempDir,
    #[case] repo: String,
    #[case] snapshot: Option<String>,
  ) -> anyhow::Result<()> {
    let service = build_hf_service(None, temp_hf_home);
    let local_model_file = service.download(
      &Repo::try_from(repo.clone())?,
      "tokenizer_config.json",
      snapshot.clone(),
    );
    assert!(local_model_file.is_err());
    let sha = snapshot.unwrap_or("main".to_string());
    let error = strfmt!(UNAUTH_ERR, repo => repo.clone(), sha)?;
    let err = local_model_file.unwrap_err();
    match err {
      HubServiceError::HubApiError(HubApiError {
        error: actual_error,
        error_status,
        repo: actual_repo,
        kind,
      }) => {
        assert_eq!(actual_error.to_string(), error);
        assert_eq!(error_status, 401);
        assert_eq!(actual_repo, repo);
        assert_eq!(kind, HubApiErrorKind::MayBeNotExists);
      }
      _ => panic!("Expected HubServiceError::MayBeNotExists, got {}", err),
    }
    Ok(())
  }

  const GATED_ERR: &str = "request error: https://huggingface.co/amir36/test-gated-repo/resolve/{sha}/tokenizer_config.json: status code 403";

  #[rstest]
  #[case(hf_test_token_public(), None, GATED_ERR)]
  #[case(hf_test_token_public(), Some("main".to_string()), GATED_ERR)]
  #[case(
    hf_test_token_public(),
    Some("57a2b0118ef1cb0ab5d9544e5d9600d189f66a72".to_string()),
    GATED_ERR
  )]
  #[case(
    hf_test_token_public(),
    Some("6bbcc8a332f15cf670db6ec9e70f68427ae2ce27".to_string()),
    GATED_ERR
  )]
  fn test_hf_hub_service_download_gated_error_if_downloading_with_token_for_gated_repo(
    temp_hf_home: TempDir,
    #[case] token: Option<String>,
    #[case] snapshot: Option<String>,
    #[case] error: &str,
  ) -> anyhow::Result<()> {
    let service = build_hf_service(token, temp_hf_home);
    let local_model_file = service.download(
      &Repo::try_from("amir36/test-gated-repo")?,
      "tokenizer_config.json",
      snapshot.clone(),
    );
    assert!(local_model_file.is_err());
    let sha = snapshot.unwrap_or("main".to_string());
    let error = strfmt!(error, repo => "amir36/test-gated-repo", sha)?;
    let err = local_model_file.unwrap_err();
    match err {
      HubServiceError::HubApiError(HubApiError {
        error: actual_error,
        error_status,
        repo,
        kind,
      }) => {
        assert_eq!(actual_error.to_string(), error);
        assert_eq!(error_status, 403);
        assert_eq!(repo.to_string(), "amir36/test-gated-repo".to_string());
        assert_eq!(kind, HubApiErrorKind::GatedAccess);
      }
      _ => panic!("Expected HubServiceError::GatedAccess, got {}", err),
    }
    Ok(())
  }

  const MAYBE_NOT_EXISTS: &str = "request error: https://huggingface.co/amir36/not-exists/resolve/{sha}/tokenizer_config.json: status code 404";

  #[rstest]
  #[anyhow_trace]
  #[case(hf_test_token_public(), None)]
  #[case(hf_test_token_public(), Some("main".to_string()))]
  #[case(hf_test_token_public(), Some("7de0799b8c9c12eff96e5c9612e39b041b3f4f5b".to_string()))]
  #[case(hf_test_token_allowed(), None)]
  #[case(hf_test_token_allowed(), Some("main".to_string()))]
  #[case(hf_test_token_allowed(), Some("7de0799b8c9c12eff96e5c9612e39b041b3f4f5b".to_string())
  )]
  #[case(hf_test_token_public(), None)]
  #[case(hf_test_token_public(), Some("main".to_string()))]
  #[case(hf_test_token_public(), Some("7de0799b8c9c12eff96e5c9612e39b041b3f4f5b".to_string()))]
  #[case(hf_test_token_allowed(), None)]
  #[case(hf_test_token_allowed(), Some("main".to_string()))]
  #[case(hf_test_token_allowed(), Some("7de0799b8c9c12eff96e5c9612e39b041b3f4f5b".to_string()))]
  fn test_hf_hub_service_download_not_found_if_downloading_with_token_for_not_exists_repo(
    temp_hf_home: TempDir,
    #[case] token: Option<String>,
    #[case] snapshot: Option<String>,
  ) -> anyhow::Result<()> {
    let sha = snapshot.clone().unwrap_or("main".to_string());
    let error = strfmt!(MAYBE_NOT_EXISTS, sha)?;
    let service = build_hf_service(token, temp_hf_home);
    let repo = Repo::try_from("amir36/not-exists")?;
    let local_model_file = service.download(&repo, "tokenizer_config.json", snapshot);
    assert!(local_model_file.is_err());
    let err = local_model_file.unwrap_err();
    match err {
      HubServiceError::HubApiError(HubApiError {
        error: actual_error,
        error_status,
        repo: actual_repo,
        kind,
      }) => {
        assert_eq!(actual_error.to_string(), error);
        assert_eq!(error_status, 404);
        assert_eq!(actual_repo, "amir36/not-exists");
        assert_eq!(kind, HubApiErrorKind::NotExists);
      }
      err => panic!("Expected HubServiceError::MayBeNotExists, got {}", err),
    }
    Ok(())
  }

  #[rstest]
  #[case(None, "2")]
  #[case( Some("main".to_string()), "2")]
  #[case( Some("57a2b0118ef1cb0ab5d9544e5d9600d189f66a72".to_string()), "2" )]
  #[case( Some("6bbcc8a332f15cf670db6ec9e70f68427ae2ce27".to_string()), "1" )]
  fn test_hf_hub_service_download_gated_file_allowed(
    #[with(hf_test_token_allowed(), true)]
    #[from(test_hf_service)]
    hf_service: TestHfService,
    #[case] snapshot: Option<String>,
    #[case] version: &str,
  ) -> anyhow::Result<()> {
    let local_model_file = hf_service.download(
      &Repo::try_from("amir36/test-gated-repo")?,
      "tokenizer_config.json",
      snapshot.clone(),
    )?;
    let path = local_model_file.path();
    assert!(path.exists());
    let sha = if snapshot.is_none() || snapshot.clone().unwrap() == "main" {
      "57a2b0118ef1cb0ab5d9544e5d9600d189f66a72"
    } else {
      &snapshot.unwrap()
    };
    let expected = hf_service
      .hf_cache()
      .join(format!(
        "models--amir36--test-gated-repo/snapshots/{sha}/tokenizer_config.json"
      ))
      .display()
      .to_string();
    assert_eq!(expected, path.display().to_string());
    let expected = format!(
      r#"{{
  "version": "{version}"
}}"#
    );
    assert_eq!(expected, fs::read_to_string(path)?);
    Ok(())
  }

  #[rstest]
  #[case(Some("main".to_string()), "this is version 2\n")]
  #[case(None, "this is version 2\n")]
  #[case(Some("9ff8b00464fc439a64bb374769dec3dd627be1c2".to_string()), "this is version 1\n")]
  #[case(Some("e9149a12809580e8602995856f8098ce973d1080".to_string()), "this is version 2\n")]
  fn test_hf_hub_service_find_local_file(
    #[from(test_hf_service)] service: TestHfService,
    #[case] snapshot: Option<String>,
    #[case] expected: String,
  ) -> anyhow::Result<()> {
    let repo = Repo::try_from("meta-llama/Llama-2-70b-chat-hf")?;
    let filename = "tokenizer_config.json";
    let local_model_file = service.find_local_file(&repo, filename, snapshot)?;
    let content = fs::read_to_string(local_model_file.path())?;
    assert_eq!(expected, content);
    Ok(())
  }

  #[rstest]
  fn test_hf_hub_service_find_local_model_not_present(
    #[from(test_hf_service)] service: TestHfService,
  ) -> anyhow::Result<()> {
    let repo = Repo::try_from("meta-llama/Llama-2-70b-chat-hf")?;
    let filename = "tokenizer_config.json";
    let snapshot = "cfe96d938c52db7c6d936f99370c0801b24233c4";
    let local_model_file = service.find_local_file(&repo, filename, Some(snapshot.to_string()));
    assert!(local_model_file.is_err());
    assert!(matches!(
      local_model_file.unwrap_err(),
      HubServiceError::HubFileNotFound(hub_file_not_found_error)
      if hub_file_not_found_error == HubFileNotFoundError::new(filename.to_string(), repo.to_string(), snapshot.to_string())
    ));
    Ok(())
  }

  #[rstest]
  fn test_hf_hub_service_find_local_model_with_non_main_refs(
    #[from(test_hf_service)] service: TestHfService,
  ) -> anyhow::Result<()> {
    let repo = Repo::fakemodel();
    let filename = "fakemodel.Q4_0.gguf";
    let result = service.find_local_file(&repo, filename, Some("non-main".to_string()));
    assert!(result.is_ok());
    let hub_file = result.unwrap();
    let snapshot = "191239b3e26b2882fb562ffccdd1cf0f65402adb";
    assert_eq!(
      HubFile::new(
        service.hf_cache(),
        repo.clone(),
        filename.to_string(),
        snapshot.to_string(),
        Some(25),
      ),
      hub_file
    );
    assert_eq!(
      "this is a non-main model\n",
      fs::read_to_string(
        service
          .hf_cache()
          .join(repo.path())
          .join("snapshots")
          .join(snapshot)
          .join(filename)
      )?
    );
    Ok(())
  }

  #[rstest]
  fn test_hf_hub_service_find_local_file_returns_error_if_refs_main_not_present(
    #[from(test_hf_service)] service: TestHfService,
  ) -> anyhow::Result<()> {
    let filename = "some-model-file.gguf";
    let repo = Repo::try_from("TheBloke/NotDownloaded")?;
    let result = service.find_local_file(&repo, filename, Some(SNAPSHOT_MAIN.to_string()));
    assert!(result.is_err());
    assert!(matches!(
      result.unwrap_err(),
      HubServiceError::HubFileNotFound(error)
      if error == HubFileNotFoundError::new(filename.to_string(), repo.to_string(), SNAPSHOT_MAIN.to_string())));
    Ok(())
  }

  #[rstest]
  fn test_hf_hub_service_list_local_models(
    #[from(test_hf_service)] service: TestHfService,
  ) -> anyhow::Result<()> {
    let models = service.list_local_models();
    let expected_1 = HubFile::new(
      service.hf_cache(),
      Repo::try_from("google/gemma-1.1-2b-it-GGUF")?,
      "2b_it_v1p1.gguf".to_string(),
      "5007652f7a641fe7170e0bad4f63839419bd9213".to_string(),
      Some(21),
    );
    assert_eq!(6, models.len());
    assert_eq!(&expected_1, models.first().unwrap());
    Ok(())
  }

  #[rstest]
  fn test_hf_hub_service_list_local_tokenizer_configs(
    #[from(test_hf_service)] service: TestHfService,
  ) -> anyhow::Result<()> {
    let repos = service.list_local_tokenizer_configs();
    assert_eq!(5, repos.len(), "Expected 5 repos with tokenizer configs");
    let expected_repos: HashSet<Repo> = [
      "meta-llama/Llama-2-70b-chat-hf",
      "meta-llama/Meta-Llama-3-70B-Instruct",
      "meta-llama/Meta-Llama-3-8B-Instruct",
      "MyFactory/testalias-gguf",
      "TinyLlama/TinyLlama-1.1B-Chat-v1.0",
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

  #[rstest]
  #[case("9ff8b00464fc439a64bb374769dec3dd627be1c2", true)]
  #[case("e9149a12809580e8602995856f8098ce973d1080", true)]
  #[case("main", true)]
  #[case("nonexistent_snapshot", false)]
  fn test_hf_hub_service_local_file_exists(
    #[from(test_hf_service)] service: TestHfService,
    #[case] snapshot: String,
    #[case] expected: bool,
  ) -> anyhow::Result<()> {
    let repo = Repo::try_from("meta-llama/Llama-2-70b-chat-hf")?;
    let filename = "tokenizer_config.json";
    let exists = service.local_file_exists(&repo, filename, Some(snapshot))?;
    assert_eq!(expected, exists);
    Ok(())
  }

  #[rstest]
  fn test_hf_hub_service_local_file_exists_refs_main_not_present(
    #[from(test_hf_service)] service: TestHfService,
  ) -> anyhow::Result<()> {
    let result = service.local_file_exists(
      &Repo::try_from("TheBloke/NotDownloaded")?,
      "some-model-file.gguf",
      Some(SNAPSHOT_MAIN.to_string()),
    );
    assert!(result.is_ok());
    assert_eq!(false, result.unwrap());
    Ok(())
  }

  #[rstest]
  fn test_hf_hub_service_local_file_exists_repo_not_exists(
    #[from(test_hf_service)] service: TestHfService,
  ) -> anyhow::Result<()> {
    let repo = Repo::try_from("nonexistent/repo")?;
    let filename = "some_file.txt";
    let snapshot = "some_snapshot";

    let exists = service.local_file_exists(&repo, filename, Some(snapshot.to_string()))?;
    assert!(!exists);
    Ok(())
  }
}
