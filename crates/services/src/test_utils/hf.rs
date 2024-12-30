use crate::{HfHubService, HubService, HubServiceError, MockHubService};
use derive_new::new;
use objs::{test_utils::temp_hf_home, HubFile, Repo};
use rstest::fixture;
use std::path::PathBuf;
use tempfile::TempDir;

#[fixture]
pub fn hf_service(#[default(None)] token: Option<String>, temp_hf_home: TempDir) -> HfHubService {
  build_hf_service(token, temp_hf_home)
}

pub fn build_hf_service(token: Option<String>, temp_hf_home: TempDir) -> HfHubService {
  HfHubService::new(
    temp_hf_home.path().join("huggingface").join("hub"),
    false,
    token,
  )
}

#[fixture]
pub fn test_hf_service(
  #[default(None)] token: Option<String>,
  #[default(false)] allow_downloads: bool,
  temp_hf_home: TempDir,
) -> TestHfService {
  let inner = HfHubService::new(
    temp_hf_home.path().join("huggingface").join("hub"),
    false,
    token,
  );
  TestHfService {
    _temp_dir: temp_hf_home,
    inner,
    inner_mock: MockHubService::new(),
    allow_downloads,
  }
}

#[derive(Debug)]
pub struct TestHfService {
  _temp_dir: TempDir,
  inner: HfHubService,
  inner_mock: MockHubService,
  allow_downloads: bool,
}

impl TestHfService {
  pub fn hf_cache(&self) -> PathBuf {
    self._temp_dir.path().join("huggingface").join("hub")
  }

  pub fn expect_download(
    &mut self,
  ) -> &mut crate::__mock_MockHubService_HubService::__download::Expectation {
    self.inner_mock.expect_download()
  }
}

type Result<T> = std::result::Result<T, HubServiceError>;

impl HubService for TestHfService {
  #[allow(clippy::needless_lifetimes)]
  fn download(&self, repo: &Repo, filename: &str, snapshot: Option<String>) -> Result<HubFile> {
    if self.allow_downloads {
      self.inner.download(repo, filename, snapshot)
    } else {
      self.inner_mock.download(repo, filename, snapshot)
    }
  }

  fn list_local_models(&self) -> Vec<HubFile> {
    self.inner.list_local_models()
  }

  fn find_local_file(
    &self,
    repo: &Repo,
    filename: &str,
    snapshot: Option<String>,
  ) -> Result<HubFile> {
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

  fn list_local_tokenizer_configs(&self) -> Vec<Repo> {
    self.inner.list_local_tokenizer_configs()
  }
}

#[derive(Debug, new)]
pub struct OfflineHubService {
  inner: HfHubService,
}

impl HubService for OfflineHubService {
  fn download(&self, repo: &Repo, filename: &str, snapshot: Option<String>) -> Result<HubFile> {
    if !self.inner.local_file_exists(repo, filename, snapshot.clone())? {
      assert!(false, "tried to download file in test");
    }
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
  ) -> Result<HubFile> {
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

  fn list_local_tokenizer_configs(&self) -> Vec<Repo> {
    self.inner.list_local_tokenizer_configs()
  }
}
