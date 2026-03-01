use crate::models::{HubFile, Repo};
use crate::{
  test_utils::{
    build_hf_service, generate_test_data_gguf_files, hf_test_token_allowed, hf_test_token_public,
    temp_hf_home, test_hf_service, TestHfService, SNAPSHOT,
  },
  HubService, HubServiceError, SNAPSHOT_MAIN,
};
use anyhow_trace::anyhow_trace;
use errmeta::AppError;
use pretty_assertions::assert_eq;
use rstest::rstest;
use std::{collections::HashSet, fs};
use strfmt::strfmt;
use tempfile::TempDir;

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
#[tokio::test]
#[anyhow_trace]
async fn test_hf_hub_service_download_public_file_with_snapshot(
  temp_hf_home: TempDir,
  #[case] token: Option<String>,
  #[case] snapshot: Option<String>,
  #[case] version: &str,
) -> anyhow::Result<()> {
  let hf_cache = temp_hf_home.path().join("huggingface").join("hub");
  let service = build_hf_service(token, temp_hf_home);
  let local_model_file = service
    .download(
      &Repo::try_from("amir36/test-model-repo")?,
      "tokenizer_config.json",
      snapshot.clone(),
      None,
    )
    .await?;
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

const UNAUTH_ERR: &str = "request error: HTTP status client error (401 Unauthorized) for url (https://huggingface.co/{repo}/resolve/{sha}/tokenizer_config.json)";

#[rstest]
#[case::anon_not_exists("amir36/not-exists", None)]
#[case::anon_not_exists("amir36/not-exists", Some("main".to_string()))]
#[case::anon_not_exists("amir36/not-exists", Some("57a2b0118ef1cb0ab5d9544e5d9600d189f66a72".to_string()))]
#[case::anon("amir36/test-gated-repo", None)]
#[case::anon_main("amir36/test-gated-repo", Some("main".to_string()))]
#[case::anon_latest("amir36/test-gated-repo", Some("57a2b0118ef1cb0ab5d9544e5d9600d189f66a72".to_string()))]
#[case::anon_older("amir36/test-gated-repo", Some("6bbcc8a332f15cf670db6ec9e70f68427ae2ce27".to_string()))]
#[case::anon_not_exists("amir36/test-gated-repo", Some("7de0799b8c9c12eff96e5c9612e39b041b3f4f5b".to_string()))]
#[tokio::test]
#[anyhow_trace]
async fn test_hf_hub_service_download_gets_unauth_error_if_downloading_as_anon(
  temp_hf_home: TempDir,
  #[case] repo: String,
  #[case] snapshot: Option<String>,
) -> anyhow::Result<()> {
  let service = build_hf_service(None, temp_hf_home);
  let local_model_file = service
    .download(
      &Repo::try_from(repo.clone())?,
      "tokenizer_config.json",
      snapshot.clone(),
      None,
    )
    .await;
  assert!(local_model_file.is_err());
  let sha = snapshot.unwrap_or("main".to_string());
  let error = strfmt!(UNAUTH_ERR, repo => repo.clone(), sha)?;
  let err = local_model_file.unwrap_err();
  match err {
    HubServiceError::MayNotExist {
      repo: actual_repo,
      error: actual_error,
    } => {
      assert_eq!(error, actual_error);
      assert_eq!(repo, actual_repo);
    }
    _ => panic!("Expected HubServiceError::MayNotExist, got {}", err),
  }
  Ok(())
}

const GATED_ERR: &str = "request error: HTTP status client error (403 Forbidden) for url (https://huggingface.co/amir36/test-gated-repo/resolve/{sha}/tokenizer_config.json)";

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
#[tokio::test]
#[anyhow_trace]
async fn test_hf_hub_service_download_gated_error_if_downloading_with_token_for_gated_repo(
  temp_hf_home: TempDir,
  #[case] token: Option<String>,
  #[case] snapshot: Option<String>,
  #[case] error: &str,
) -> anyhow::Result<()> {
  let service = build_hf_service(token, temp_hf_home);
  let local_model_file = service
    .download(
      &Repo::try_from("amir36/test-gated-repo")?,
      "tokenizer_config.json",
      snapshot.clone(),
      None,
    )
    .await;
  assert!(local_model_file.is_err());
  let sha = snapshot.unwrap_or("main".to_string());
  let error = strfmt!(error, repo => "amir36/test-gated-repo", sha)?;
  let err = local_model_file.unwrap_err();
  match err {
    HubServiceError::GatedAccess {
      repo,
      error: actual_error,
    } => {
      assert_eq!(error, actual_error);
      assert_eq!("amir36/test-gated-repo", repo);
    }
    _ => panic!("Expected HubServiceError::GatedAccess, got {}", err),
  }
  Ok(())
}

const MAYBE_NOT_EXISTS: &str = "request error: HTTP status client error (404 Not Found) for url (https://huggingface.co/amir36/not-exists/resolve/{sha}/tokenizer_config.json)";

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
#[tokio::test]
#[anyhow_trace]
async fn test_hf_hub_service_download_not_found_if_downloading_with_token_for_not_exists_repo(
  temp_hf_home: TempDir,
  #[case] token: Option<String>,
  #[case] snapshot: Option<String>,
) -> anyhow::Result<()> {
  let sha = snapshot.clone().unwrap_or("main".to_string());
  let error = strfmt!(MAYBE_NOT_EXISTS, sha)?;
  let service = build_hf_service(token, temp_hf_home);
  let repo = Repo::try_from("amir36/not-exists")?;
  let local_model_file = service
    .download(&repo, "tokenizer_config.json", snapshot, None)
    .await;
  assert!(local_model_file.is_err());
  let err = local_model_file.unwrap_err();
  match err {
    HubServiceError::RepoDisabled {
      repo: actual_repo,
      error: actual_error,
    } => {
      assert_eq!(error, actual_error);
      assert_eq!("amir36/not-exists", actual_repo);
    }
    err => panic!("Expected HubServiceError::RepoDisabled, got {}", err),
  }
  Ok(())
}

#[rstest]
#[case(None, "2")]
#[case( Some("main".to_string()), "2")]
#[case( Some("57a2b0118ef1cb0ab5d9544e5d9600d189f66a72".to_string()), "2" )]
#[case( Some("6bbcc8a332f15cf670db6ec9e70f68427ae2ce27".to_string()), "1" )]
#[tokio::test]
#[anyhow_trace]
async fn test_hf_hub_service_download_gated_file_allowed(
  #[with(hf_test_token_allowed(), true)]
  #[from(test_hf_service)]
  hf_service: TestHfService,
  #[case] snapshot: Option<String>,
  #[case] version: &str,
) -> anyhow::Result<()> {
  let local_model_file = hf_service
    .download(
      &Repo::try_from("amir36/test-gated-repo")?,
      "tokenizer_config.json",
      snapshot.clone(),
      None,
    )
    .await?;
  let path = local_model_file.path();
  assert!(path.exists());
  let sha = if snapshot.is_none() || snapshot.clone().unwrap() == "main" {
    "57a2b0118ef1cb0ab5d9544e5d9600d189f66a72"
  } else {
    &snapshot.unwrap()
  };
  let expected = hf_service
    .hf_cache()
    .join("models--amir36--test-gated-repo")
    .join("snapshots")
    .join(sha)
    .join("tokenizer_config.json")
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
fn test_hf_hub_service_does_not_download_if_file_exists(
  #[from(test_hf_service)] service: TestHfService,
) -> anyhow::Result<()> {
  let repo = Repo::fakemodel();
  let filename = "fakemodel.Q4_0.gguf";
  let local_model_file = service.find_local_file(&repo, filename, Some(SNAPSHOT.to_string()));
  assert!(local_model_file.is_ok());
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
  let err = local_model_file.unwrap_err();
  assert_eq!("hub_service_error-file_not_found", err.code());
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
  let snapshot = "9ca625120374ddaae21f067cb006517d14dc91a6";
  assert_eq!(
    HubFile::new(
      service.hf_cache(),
      repo.clone(),
      filename.to_string(),
      snapshot.to_string(),
      Some(704),
    ),
    hub_file
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
  let err = result.unwrap_err();
  assert_eq!("hub_service_error-file_not_found", err.code());
  Ok(())
}

#[rstest]
fn test_hf_hub_service_list_local_models(
  #[from(test_hf_service)] service: TestHfService,
) -> anyhow::Result<()> {
  let mut models = service.list_local_models();
  let expected_1 = HubFile::new(
    service.hf_cache(),
    Repo::try_from("FakeFactory/fakemodel-gguf")?,
    "fakemodel.Q4_0.gguf".to_string(),
    "5007652f7a641fe7170e0bad4f63839419bd9213".to_string(),
    Some(704),
  );
  assert_eq!(6, models.len());
  models.sort();
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
  assert!(!result.unwrap());
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

#[rstest]
fn test_list_model_aliases(
  #[from(generate_test_data_gguf_files)] _setup: &(),
  #[from(test_hf_service)] service: TestHfService,
) -> anyhow::Result<()> {
  let aliases = service.list_model_aliases()?;

  // Since llama.cpp now handles chat templates, we include all GGUF files
  // The exact count may vary based on test data, but we should have at least the core models
  assert!(aliases.len() >= 3);

  // Check that we have the expected core aliases
  let alias_names: Vec<String> = aliases.iter().map(|a| a.alias.clone()).collect();
  assert!(alias_names.contains(&"FakeFactory/fakemodel-gguf:Q4_0".to_string()));
  assert!(alias_names.contains(&"TheBloke/Llama-2-7B-Chat-GGUF:Q8_0".to_string()));
  assert!(alias_names.contains(&"TheBloke/TinyLlama-1.1B-Chat-v0.3-GGUF:Q2_K".to_string()));

  Ok(())
}
