use crate::{HfHubService, HubService, HubServiceError};
use objs::{test_utils::temp_hf_home, HubFile, Repo};
use rstest::fixture;
use std::path::PathBuf;
use tempfile::TempDir;

#[fixture]
pub fn hf_service(#[default(None)] token: Option<String>, temp_hf_home: TempDir) -> HfHubService {
  build_hf_service(token, temp_hf_home)
}

pub fn build_hf_service(token: Option<String>, temp_hf_home: TempDir) -> HfHubService {
  HfHubService::new(temp_hf_home.path().join("huggingface/hub"), false, token)
}

#[fixture]
pub fn test_hf_service(
  #[default(None)] token: Option<String>,
  temp_hf_home: TempDir,
) -> TestHfService {
  let inner = HfHubService::new(temp_hf_home.path().join("huggingface/hub"), false, token);
  TestHfService {
    _temp_dir: temp_hf_home,
    inner,
  }
}

#[derive(Debug)]
pub struct TestHfService {
  _temp_dir: TempDir,
  inner: HfHubService,
}

impl TestHfService {
  pub fn hf_cache(&self) -> PathBuf {
    self._temp_dir.path().join("huggingface/hub")
  }
}

type Result<T> = std::result::Result<T, HubServiceError>;

impl HubService for TestHfService {
  #[allow(clippy::needless_lifetimes)]
  fn download(&self, repo: &Repo, filename: &str, snapshot: Option<String>) -> Result<HubFile> {
    self.inner.download(repo, filename, snapshot)
  }

  fn list_local_models(&self) -> Vec<HubFile> {
    self.inner.list_local_models()
  }

  fn find_local_file(
    &self,
    repo: &Repo,
    filename: &str,
    snapshot: Option<String>,
  ) -> Result<Option<HubFile>> {
    self.inner.find_local_file(repo, filename, snapshot)
  }

  fn local_file_exists(
    &self,
    repo: &Repo,
    filename: &str,
    snapshot: Option<String>,
  ) -> Result<bool> {
    self.inner.local_file_exists(repo, filename, snapshot)
  }

  fn model_file_path(&self, repo: &Repo, filename: &str, snapshot: &str) -> PathBuf {
    self.inner.model_file_path(repo, filename, snapshot)
  }

  fn list_local_tokenizer_configs(&self) -> Vec<Repo> {
    self.inner.list_local_tokenizer_configs()
  }
}