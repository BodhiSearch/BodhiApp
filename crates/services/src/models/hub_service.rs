use hf_hub::Cache;
use objs::{
  impl_error_from, AppError, ErrorType, HubFile, IoError, ModelAlias, ModelAliasBuilder,
  ObjValidationError, Repo,
};
use std::{
  collections::HashSet,
  fmt::{Debug, Formatter},
  fs,
  path::PathBuf,
  str::FromStr,
};
use walkdir::WalkDir;

use crate::Progress;

pub static SNAPSHOT_MAIN: &str = "main";

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum HubServiceError {
  #[error(
    "Access to '{repo}' requires approval. Visit https://huggingface.co/{repo} to request access."
  )]
  #[error_meta(error_type = ErrorType::Forbidden)]
  GatedAccess { repo: String, error: String },

  #[error("Repository '{repo}' not found or requires authentication. Run 'huggingface-cli login' to authenticate.")]
  #[error_meta(error_type = ErrorType::NotFound)]
  MayNotExist { repo: String, error: String },

  #[error("Repository '{repo}' is disabled or has been removed.")]
  #[error_meta(error_type = ErrorType::NotFound)]
  RepoDisabled { repo: String, error: String },

  #[error("Network error accessing Hugging Face. Check your internet connection and try again.")]
  #[error_meta(error_type = ErrorType::ServiceUnavailable)]
  Transport { repo: String, error: String },

  #[error("Hugging Face API error: {error}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  Unknown { repo: String, error: String },

  #[error("Failed to build API client: {error}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  Request { repo: String, error: String },

  #[error("File '{filename}' not found in repository '{repo}'.")]
  #[error_meta(error_type = ErrorType::NotFound)]
  FileNotFound {
    filename: String,
    repo: String,
    snapshot: String,
  },

  #[error(transparent)]
  ObjValidationError(#[from] ObjValidationError),
  #[error(transparent)]
  IoError(#[from] IoError),
}

impl_error_from!(std::io::Error, HubServiceError::IoError, ::objs::IoError);

type Result<T> = std::result::Result<T, HubServiceError>;

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait HubService: std::fmt::Debug + Send + Sync {
  #[allow(clippy::needless_lifetimes)]
  async fn download(
    &self,
    repo: &Repo,
    filename: &str,
    snapshot: Option<String>,
    progress: Option<Progress>,
  ) -> Result<HubFile>;

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

  fn list_local_tokenizer_configs(&self) -> Vec<Repo>;

  fn list_model_aliases(&self) -> Result<Vec<ModelAlias>>;
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

#[async_trait::async_trait]
impl HubService for HfHubService {
  async fn download(
    &self,
    repo: &Repo,
    filename: &str,
    snapshot: Option<String>,
    progress: Option<Progress>,
  ) -> Result<HubFile> {
    if self.local_file_exists(repo, filename, snapshot.clone())? {
      return self.find_local_file(repo, filename, snapshot.clone());
    }
    let snapshot = snapshot.unwrap_or(SNAPSHOT_MAIN.to_string());
    let model_repo =
      hf_hub::Repo::with_revision(repo.to_string(), hf_hub::RepoType::Model, snapshot);
    let hf_repo = self.cache.repo(model_repo.clone());
    let from_cache = hf_repo.get(filename);
    let path = match from_cache {
      Some(path) => path,
      None => {
        self
          .download_async_with_progress(model_repo, filename, progress)
          .await?
      }
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
        return Err(HubServiceError::FileNotFound {
          filename: filename.to_string(),
          repo: repo.to_string(),
          snapshot,
        });
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
      Err(HubServiceError::FileNotFound {
        filename: filename.to_string(),
        repo: repo.to_string(),
        snapshot,
      })
    }
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

  // model_chat_template method removed since llama.cpp now handles chat templates

  fn list_model_aliases(&self) -> Result<Vec<ModelAlias>> {
    let cache = self.hf_cache();
    let mut aliases = WalkDir::new(&cache)
      .follow_links(true)
      .into_iter()
      .filter_map(|e| e.ok())
      .filter(|entry| entry.file_type().is_file())
      .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "gguf"))
      .filter(|entry| {
        entry
          .path()
          .parent()
          .and_then(|p| p.parent())
          .is_some_and(|p| p.ends_with("snapshots"))
      })
      .filter_map(|entry| {
        let path = entry.path();
        let models_dir = path.ancestors().find(|p| {
          p.file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|name| name.starts_with("models--"))
        })?;

        let dir_name = models_dir.file_name()?.to_str()?;
        let repo_path = dir_name.strip_prefix("models--")?;
        let (owner, repo_name) = repo_path.split_once("--")?;
        let repo = Repo::from_str(&format!("{}/{}", owner, repo_name)).ok()?;

        let filename = path.file_name()?.to_str()?.to_string();
        let snapshot = path.parent()?.file_name()?.to_str()?.to_string();

        Some(HubFile::new(cache.clone(), repo, filename, snapshot, None))
      })
      .filter_map(|hub_file| {
        // Since llama.cpp now handles chat templates, we include all GGUF files
        let qualifier = hub_file
          .filename
          .split('.')
          .nth_back(1)
          .and_then(|s| s.split('-').nth_back(0))
          .unwrap_or_else(|| &hub_file.filename);
        let alias = ModelAliasBuilder::default()
          .alias(format!("{}:{}", hub_file.repo, qualifier))
          .repo(hub_file.repo)
          .filename(hub_file.filename)
          .snapshot(hub_file.snapshot)
          .build()
          .ok()?;
        Some(alias)
      })
      .collect::<Vec<_>>();

    // Sort by alias name and then by snapshot, remove duplicates keeping latest snapshot
    aliases.sort_by(|a, b| (&a.alias, &b.snapshot).cmp(&(&b.alias, &a.snapshot)));
    aliases.dedup_by(|a, b| a.alias == b.alias);

    Ok(aliases)
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

  pub fn new_from_hf_cache(
    hf_cache: PathBuf,
    hf_env_token: Option<String>,
    progress_bar: bool,
  ) -> std::result::Result<Self, std::io::Error> {
    fs::create_dir_all(&hf_cache)?;
    let cache = Cache::new(hf_cache);
    let token = hf_env_token.or_else(|| cache.token());
    Ok(Self {
      cache,
      progress_bar,
      token,
    })
  }

  pub fn progress_bar(&mut self, progress_bar: bool) {
    self.progress_bar = progress_bar;
  }

  async fn download_async_with_progress(
    &self,
    model_repo: hf_hub::Repo,
    filename: &str,
    progress: Option<Progress>,
  ) -> Result<PathBuf> {
    use hf_hub::api::tokio::{ApiBuilder, ApiError};

    let api = ApiBuilder::from_cache(self.cache.clone())
      .high()
      .with_progress(self.progress_bar)
      .with_token(self.token.clone())
      .build()
      .map_err(|err| HubServiceError::Request {
        repo: model_repo.url(),
        error: err.to_string(),
      })?;
    tracing::info!("Downloading from url {}", model_repo.api_url());
    let repo = model_repo.url();

    // Use download_with_progress to integrate with our progress tracking system
    let path = match match progress {
      Some(progress) => {
        api
          .repo(model_repo)
          .download_with_progress(filename, progress)
          .await
      }
      None => api.repo(model_repo).download(filename).await,
    } {
      Ok(path) => path,
      Err(err) => {
        let error_msg = err.to_string();
        let err = match err {
          ApiError::RequestError(reqwest_err) => {
            let status = reqwest_err.status().map(|s| s.as_u16()).unwrap_or(500);
            match status {
              403 => HubServiceError::GatedAccess {
                repo: repo.to_string(),
                error: error_msg,
              },
              401 if self.token.is_none() => HubServiceError::MayNotExist {
                repo: repo.to_string(),
                error: error_msg,
              },
              404 if self.token.is_some() => HubServiceError::RepoDisabled {
                repo: repo.to_string(),
                error: error_msg,
              },
              _ if reqwest_err.is_connect() || reqwest_err.is_timeout() => {
                HubServiceError::Transport {
                  repo: repo.to_string(),
                  error: error_msg,
                }
              }
              _ => HubServiceError::Unknown {
                repo: repo.to_string(),
                error: error_msg,
              },
            }
          }
          _ => HubServiceError::Request {
            repo: repo.to_string(),
            error: error_msg,
          },
        };
        return Err(err);
      }
    };

    Ok(path)
  }
}

#[cfg(test)]
#[path = "test_hub_service.rs"]
mod test_hub_service;
