use hf_hub::{api::sync::ApiError, Cache, Repo};
use std::{
  fmt::{Debug, Formatter},
  path::PathBuf,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HubServiceError {
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
}

type Result<T> = std::result::Result<T, HubServiceError>;

#[derive(Default)]
pub(crate) struct HubService {
  cache: Cache,
  progress_bar: bool,
  token: Option<String>,
}

impl Debug for HubService {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("HubService")
      .field("cache", &self.cache.path())
      .finish()
  }
}

impl HubService {
  pub(crate) fn new(path: PathBuf, progress_bar: bool, token: Option<String>) -> Self {
    Self {
      cache: Cache::new(path),
      progress_bar,
      token,
    }
  }

  pub(crate) fn download(&self, repo: &str, filename: &str) -> Result<PathBuf> {
    let hf_repo = self.cache.repo(Repo::model(repo.to_string()));
    let from_cache = hf_repo.get(filename);
    match from_cache {
      Some(path) => Ok(path),
      None => {
        let path = self.download_sync(repo, filename)?;
        Ok(path)
      }
    }
  }

  fn download_sync(&self, repo: &str, file: &str) -> Result<PathBuf> {
    use hf_hub::api::sync::{ApiBuilder, ApiError};

    let api = ApiBuilder::from_cache(self.cache.clone())
      .with_progress(self.progress_bar)
      .with_token(self.token.clone())
      .build()?;
    tracing::info!("Downloading from repo {repo}, file {file}:");
    let path = match api.model(repo.to_string()).download(file) {
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
  use crate::test_utils::{hf_test_token_allowed, hf_test_token_public};

  use super::HubService;
  use dircpy::CopyBuilder;
  use rstest::{fixture, rstest};
  use std::{fs, path::Path};
  use tempfile::{tempdir, TempDir};

  #[fixture]
  fn test_dir() -> TempDir {
    let temp_dir = tempdir().expect("Failed to create a temporary directory");
    let src_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/huggingface");
    let dst_path = temp_dir.path().join("huggingface");

    CopyBuilder::new(src_path, dst_path)
      .overwrite(true)
      .run()
      .unwrap();
    temp_dir
  }

  #[rstest]
  #[case(None)]
  #[case(hf_test_token_public())]
  fn test_hub_service_download_public_file(
    test_dir: TempDir,
    #[case] token: Option<String>,
  ) -> anyhow::Result<()> {
    let path = test_dir.path().join("huggingface/hub");
    let service = HubService::new(path, false, token);
    let dest_file = service.download("amir36/test-model-repo", "tokenizer_config.json")?;
    assert!(dest_file.exists());
    let expected = test_dir.path().join("huggingface/hub/models--amir36--test-model-repo/snapshots/f7d5db77208ab98318b45cba4a48fc33a47fe4f6/tokenizer_config.json").display().to_string();
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
  fn test_hub_service_download_gated_file_not_allowed(
    test_dir: TempDir,
    #[case] token: Option<String>,
    #[case] expected: String,
  ) -> anyhow::Result<()> {
    let path = test_dir.path().join("huggingface/hub");
    let service = HubService::new(path, false, token);
    let dest_file = service.download("amir36/test-gated-repo", "tokenizer_config.json");
    assert!(dest_file.is_err());
    assert_eq!(expected, dest_file.unwrap_err().to_string());
    Ok(())
  }

  #[rstest]
  #[case(hf_test_token_allowed())]
  fn test_hub_service_download_gated_file_allowed(
    test_dir: TempDir,
    #[case] token: Option<String>,
  ) -> anyhow::Result<()> {
    let path = test_dir.path().join("huggingface/hub");
    let service = HubService::new(path, false, token);
    let dest_file = service.download("amir36/test-gated-repo", "tokenizer_config.json")?;
    assert!(dest_file.exists());
    let expected = test_dir.path().join("huggingface/hub/models--amir36--test-gated-repo/snapshots/6ac8c08e39d0f68114b63ea98900632abcfb6758/tokenizer_config.json").display().to_string();
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
  fn test_hub_service_download_request_error_not_found(
    test_dir: TempDir,
    #[case] token: Option<String>,
    #[case] error: String,
  ) -> anyhow::Result<()> {
    let path = test_dir.path().join("huggingface/hub");
    let service = HubService::new(path, false, token);
    let dest_file = service.download("amir36/not-exists", "tokenizer_config.json");
    assert!(dest_file.is_err());
    assert_eq!(error, dest_file.unwrap_err().to_string());
    Ok(())
  }
}
