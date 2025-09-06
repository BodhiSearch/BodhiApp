use crate::{
  db::{DbError, DbService},
  HubService, HubServiceError, ALIASES_DIR, MODELS_YAML,
};
use async_trait::async_trait;
use objs::{
  impl_error_from, Alias, AppError, ErrorType, IoDirCreateError, IoError, IoFileDeleteError,
  IoFileReadError, IoFileWriteError, RemoteModel, SerdeYamlError, SerdeYamlWithPathError,
  UserAlias,
};
use std::{collections::HashMap, fmt::Debug, fs, path::PathBuf, sync::Arc};

#[derive(Debug, PartialEq, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error("alias_exists")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::BadRequest)]
pub struct AliasExistsError(pub String);

#[derive(Debug, PartialEq, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error("alias_not_found")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::NotFound)]
pub struct AliasNotFoundError(pub String);

#[derive(Debug, PartialEq, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("data_file_missing")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::BadRequest)]
pub struct DataFileNotFoundError {
  filename: String,
  dirname: String,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum DataServiceError {
  #[error("dir_missing")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  DirMissing { dirname: String },
  #[error(transparent)]
  DataFileNotFound(#[from] DataFileNotFoundError),
  #[error(transparent)]
  DirCreate(#[from] IoDirCreateError),
  #[error(transparent)]
  Io(#[from] IoError),
  #[error(transparent)]
  IoFileRead(#[from] IoFileReadError),
  #[error(transparent)]
  IoFileDelete(#[from] IoFileDeleteError),
  #[error(transparent)]
  IoFileWrite(#[from] IoFileWriteError),
  #[error(transparent)]
  AliasNotExists(#[from] AliasNotFoundError),
  #[error(transparent)]
  AliasExists(#[from] AliasExistsError),
  #[error(transparent)]
  SerdeYamlErrorWithPath(#[from] SerdeYamlWithPathError),
  #[error(transparent)]
  SerdeYamlError(#[from] SerdeYamlError),
  #[error(transparent)]
  HubService(#[from] HubServiceError),
  #[error(transparent)]
  Db(#[from] DbError),
}

impl_error_from!(
  ::serde_yaml::Error,
  DataServiceError::SerdeYamlError,
  ::objs::SerdeYamlError
);
impl_error_from!(::std::io::Error, DataServiceError::Io, ::objs::IoError);

type Result<T> = std::result::Result<T, DataServiceError>;

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait DataService: Send + Sync + std::fmt::Debug {
  async fn list_aliases(&self) -> Result<Vec<Alias>>;

  fn save_alias(&self, alias: &UserAlias) -> Result<PathBuf>;

  async fn find_alias(&self, alias: &str) -> Option<Alias>;

  fn find_user_alias(&self, alias: &str) -> Option<UserAlias>;

  fn list_remote_models(&self) -> Result<Vec<RemoteModel>>;

  fn find_remote_model(&self, alias: &str) -> Result<Option<RemoteModel>>;

  async fn copy_alias(&self, alias: &str, new_alias: &str) -> Result<()>;

  async fn delete_alias(&self, alias: &str) -> Result<()>;

  fn alias_filename(&self, alias: &str) -> Result<PathBuf>;

  fn read_file(&self, folder: Option<String>, filename: &str) -> Result<Vec<u8>>;

  fn write_file(&self, folder: Option<String>, filename: &str, contents: &[u8]) -> Result<()>;

  fn find_file(&self, folder: Option<String>, filename: &str) -> Result<PathBuf>;
}

#[derive(Debug, Clone)]
pub struct LocalDataService {
  bodhi_home: PathBuf,
  hub_service: Arc<dyn HubService>,
  db_service: Arc<dyn DbService>,
}

impl LocalDataService {
  pub fn new(
    bodhi_home: PathBuf,
    hub_service: Arc<dyn HubService>,
    db_service: Arc<dyn DbService>,
  ) -> Self {
    Self {
      bodhi_home,
      hub_service,
      db_service,
    }
  }

  fn aliases_dir(&self) -> PathBuf {
    // TODO: take from setting service
    self.bodhi_home.join(ALIASES_DIR)
  }

  fn models_yaml(&self) -> PathBuf {
    // TODO: take from setting service
    self.bodhi_home.join(MODELS_YAML)
  }

  fn construct_path(&self, folder: &Option<String>, filename: &str) -> PathBuf {
    let mut path = self.bodhi_home.clone();
    if let Some(folder) = folder {
      path = path.join(folder);
    }
    path.join(filename)
  }
}

#[async_trait]
impl DataService for LocalDataService {
  fn find_remote_model(&self, alias: &str) -> Result<Option<RemoteModel>> {
    let models = self.list_remote_models()?;
    Ok(models.into_iter().find(|model| model.alias.eq(alias)))
  }

  fn save_alias(&self, alias: &UserAlias) -> Result<PathBuf> {
    let contents = serde_yaml::to_string(alias)?;
    let filename = self.aliases_dir().join(alias.config_filename());
    fs::write(filename.clone(), contents)
      .map_err(|err| IoFileWriteError::new(err, alias.config_filename().clone()))?;
    Ok(filename)
  }

  async fn list_aliases(&self) -> Result<Vec<Alias>> {
    let user_aliases = self.list_user_aliases()?;
    let mut result: Vec<Alias> = user_aliases
      .into_values()
      .map(|alias| Alias::User(alias))
      .collect();

    let model_aliases = self.hub_service.list_model_aliases()?;
    let model_alias_variants: Vec<Alias> = model_aliases
      .into_iter()
      .map(|alias| Alias::Model(alias))
      .collect();

    result.extend(model_alias_variants);

    // Add API aliases from database
    match self.db_service.list_api_model_aliases().await {
      Ok(api_aliases) => {
        let api_alias_variants: Vec<Alias> = api_aliases
          .into_iter()
          .map(|alias| Alias::Api(alias))
          .collect();
        result.extend(api_alias_variants);
      }
      Err(_) => {
        // Continue without API aliases if database is not available
        // This provides graceful degradation
      }
    }

    result.sort_by(|a, b| {
      let alias_a = match a {
        Alias::User(user) => &user.alias,
        Alias::Model(model) => &model.alias,
        Alias::Api(api) => &api.id,
      };
      let alias_b = match b {
        Alias::User(user) => &user.alias,
        Alias::Model(model) => &model.alias,
        Alias::Api(api) => &api.id,
      };
      alias_a.cmp(alias_b)
    });
    Ok(result)
  }

  async fn find_alias(&self, alias: &str) -> Option<Alias> {
    // Priority 1: Check user aliases (from YAML files)
    if let Some(user_alias) = self.find_user_alias(alias) {
      return Some(Alias::User(user_alias));
    }

    // Priority 2: Check model aliases (auto-discovered GGUF files)
    if let Ok(model_aliases) = self.hub_service.list_model_aliases() {
      if let Some(model) = model_aliases.into_iter().find(|m| m.alias == alias) {
        return Some(Alias::Model(model));
      }
    }

    // Priority 3: Check API aliases (from database) - with prefix-aware routing
    if let Ok(api_aliases) = self.db_service.list_api_model_aliases().await {
      // Use matchable_models() to check if the incoming alias matches any API alias
      // This automatically handles both prefixed (e.g., "azure/gpt-4") and direct (e.g., "gpt-4") matches
      if let Some(api) = api_aliases
        .into_iter()
        .find(|api| api.matchable_models().contains(&alias.to_string()))
      {
        return Some(Alias::Api(api));
      }
    }
    None
  }

  fn find_user_alias(&self, alias: &str) -> Option<UserAlias> {
    if let Ok(user_aliases) = self.list_user_aliases() {
      user_aliases.into_values().find(|user| user.alias == alias)
    } else {
      None
    }
  }

  fn list_remote_models(&self) -> Result<Vec<RemoteModel>> {
    let models_file = self.models_yaml();
    if !models_file.exists() {
      return Err(DataFileNotFoundError::new(String::from(MODELS_YAML), "".to_string()).into());
    }
    let content = fs::read_to_string(models_file.clone())
      .map_err(|err| IoFileReadError::new(err, models_file.display().to_string()))?;
    let models = serde_yaml::from_str::<Vec<RemoteModel>>(&content)
      .map_err(|err| SerdeYamlWithPathError::new(err, models_file.display().to_string()))?;
    Ok(models)
  }

  async fn copy_alias(&self, alias: &str, new_alias: &str) -> Result<()> {
    let mut user_alias = self
      .find_user_alias(alias)
      .ok_or_else(|| AliasNotFoundError(alias.to_string()))?;

    match self.find_user_alias(new_alias) {
      Some(_) => Err(AliasExistsError(new_alias.to_string()))?,
      None => {
        user_alias.alias = new_alias.to_string();
        self.save_alias(&user_alias)?;
        Ok(())
      }
    }
  }

  async fn delete_alias(&self, alias: &str) -> Result<()> {
    let (filename, _) = self
      .list_user_aliases()?
      .into_iter()
      .find(|(_, item)| item.alias.eq(alias))
      .ok_or_else(|| AliasNotFoundError(alias.to_string()))?;
    fs::remove_file(&filename).map_err(|err| IoFileDeleteError::new(err, filename))?;
    Ok(())
  }

  fn alias_filename(&self, alias: &str) -> Result<PathBuf> {
    let (filename, _) = self
      .list_user_aliases()?
      .into_iter()
      .find(|(_, item)| item.alias.eq(alias))
      .ok_or_else(|| AliasNotFoundError(alias.to_string()))?;
    Ok(PathBuf::from(filename))
  }

  fn find_file(&self, folder: Option<String>, filename: &str) -> Result<PathBuf> {
    let path = self.construct_path(&folder, filename);
    if !path.exists() {
      return Err(
        DataFileNotFoundError::new(filename.to_string(), folder.unwrap_or_default()).into(),
      );
    }
    Ok(path)
  }

  fn read_file(&self, folder: Option<String>, filename: &str) -> Result<Vec<u8>> {
    let path = self.find_file(folder, filename)?;
    let result =
      fs::read(&path).map_err(|err| IoFileReadError::new(err, path.display().to_string()))?;
    Ok(result)
  }

  fn write_file(&self, folder: Option<String>, filename: &str, contents: &[u8]) -> Result<()> {
    let path = self.construct_path(&folder, filename);
    if let Some(parent) = path.parent() {
      if !parent.exists() {
        fs::create_dir_all(parent)
          .map_err(|err| IoDirCreateError::new(err, parent.display().to_string()))?;
      }
    }
    fs::write(&path, contents)
      .map_err(|err| IoFileWriteError::new(err, path.display().to_string()))?;
    Ok(())
  }
}

impl LocalDataService {
  fn list_user_aliases(&self) -> Result<HashMap<String, UserAlias>> {
    {
      let aliases_dir = self.aliases_dir();
      let yaml_files = fs::read_dir(&aliases_dir)?;
      let yaml_files = yaml_files
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
      let aliases = yaml_files
        .into_iter()
        .filter_map(|yaml_file| {
          let filename = yaml_file.clone().display().to_string();
          match fs::read_to_string(yaml_file) {
            Ok(content) => match serde_yaml::from_str::<UserAlias>(&content) {
              Ok(alias) => Some((filename, alias)),
              Err(err) => {
                let err = SerdeYamlWithPathError::new(err, filename);
                tracing::warn!(?err, "Error deserializing model alias YAML file",);
                None
              }
            },
            Err(err) => {
              let err = IoFileReadError::new(err, filename);
              tracing::warn!(?err, "Error reading model alias YAML file");
              None
            }
          }
        })
        .collect::<HashMap<_, _>>();
      Ok(aliases)
    }
  }
}

#[cfg(test)]
mod test {
  use crate::{
    db::DbService,
    test_utils::{
      test_data_service, test_db_service, test_hf_service, TestDataService, TestDbService,
      TestHfService,
    },
    AliasExistsError, AliasNotFoundError, DataFileNotFoundError, DataService, DataServiceError,
    LocalDataService,
  };
  use anyhow_trace::anyhow_trace;
  use objs::{
    test_utils::{assert_error_message, setup_l10n, temp_bodhi_home},
    Alias, ApiFormat, AppError, FluentLocalizationService, RemoteModel, UserAlias,
  };
  use rstest::rstest;
  use std::{fs, sync::Arc};
  use tempfile::TempDir;

  #[rstest]
  #[case::dir_missing(&DataServiceError::DirMissing { dirname: "test".to_string() },
  r#"directory 'test' not found in $BODHI_HOME.
$BODHI_HOME might not have been initialized. Run `bodhi init` to setup $BODHI_HOME."#)]
  #[case::not_found(&DataServiceError::DataFileNotFound(DataFileNotFoundError::new("test.txt".to_string(), "test".to_string())),
  r#"file 'test.txt' not found in $BODHI_HOME/test.
$BODHI_HOME might not have been initialized. Run `bodhi init` to setup $BODHI_HOME."#)]
  #[case(&AliasNotFoundError("testalias".to_string()), "alias 'testalias' not found in $BODHI_HOME/aliases")]
  #[case(&AliasExistsError("testalias".to_string()), "alias 'testalias' already exists in $BODHI_HOME/aliases")]
  fn test_data_service_error(
    #[from(setup_l10n)] localization_service: &Arc<FluentLocalizationService>,
    #[case] error: &dyn AppError,
    #[case] message: String,
  ) {
    assert_error_message(localization_service, &error.code(), error.args(), &message);
  }

  #[rstest]
  #[tokio::test]
  #[awt]
  async fn test_local_data_service_models_file_missing(
    #[future]
    #[from(test_data_service)]
    service: TestDataService,
  ) -> anyhow::Result<()> {
    fs::remove_file(service.bodhi_home().join("models.yaml"))?;
    let result = service.find_remote_model("testalias:instruct");
    assert!(result.is_err());
    assert!(matches!(
      result.unwrap_err(),
      DataServiceError::DataFileNotFound(error) if error == DataFileNotFoundError::new("models.yaml".to_string(), "".to_string())
    ));
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  #[awt]
  #[anyhow_trace]
  async fn test_local_data_service_models_file_corrupt(
    #[future]
    #[from(test_data_service)]
    service: TestDataService,
  ) -> anyhow::Result<()> {
    let models_file = service.bodhi_home().join("models.yaml");
    fs::write(
      &models_file,
      r#"
# alias is missing
repo: MyFactory/testalias-gguf
filename: testalias.Q8_0.gguf
chat_template: llama3
"#,
    )?;
    let result = service.find_remote_model("testalias:instruct");
    assert!(result.is_err());
    assert!(matches!(
      result.unwrap_err(),
      DataServiceError::SerdeYamlErrorWithPath(_)
    ));
    Ok(())
  }

  #[rstest]
  #[case("testalias:instruct", true)]
  #[case("testalias-notexists", false)]
  #[tokio::test]
  #[awt]
  async fn test_local_data_service_find_remote_model(
    #[future]
    #[from(test_data_service)]
    service: TestDataService,
    #[case] alias: String,
    #[case] found: bool,
  ) -> anyhow::Result<()> {
    let remote_model = service.find_remote_model(&alias)?;
    assert_eq!(found, remote_model.is_some());
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  #[awt]
  async fn test_local_data_service_list_remote_models(
    #[future]
    #[from(test_data_service)]
    service: TestDataService,
  ) -> anyhow::Result<()> {
    let models = service.list_remote_models()?;
    let expected_1 = RemoteModel::llama3();
    let expected_2 = RemoteModel::testalias();
    assert_eq!(7, models.len());
    assert!(models.contains(&expected_1));
    assert!(models.contains(&expected_2));
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  #[awt]
  async fn test_local_data_service_find_alias(
    #[future]
    #[from(test_data_service)]
    service: TestDataService,
  ) -> anyhow::Result<()> {
    let alias = service.find_alias("testalias-exists:instruct").await;
    let expected = Alias::User(UserAlias::testalias_exists());
    assert_eq!(Some(expected), alias);
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  #[awt]
  async fn test_find_alias_not_found(
    #[future]
    #[from(test_data_service)]
    service: TestDataService,
  ) -> anyhow::Result<()> {
    let alias = service.find_alias("nonexistent-alias").await;
    assert_eq!(None, alias);
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  #[awt]
  async fn test_find_alias_api_by_model_name(
    temp_bodhi_home: TempDir,
    test_hf_service: TestHfService,
    #[future] test_db_service: TestDbService,
  ) -> anyhow::Result<()> {
    let db_service = Arc::new(test_db_service);
    let data_service = LocalDataService::new(
      temp_bodhi_home.path().join("bodhi"),
      Arc::new(test_hf_service),
      db_service.clone(),
    );

    // Insert API alias with multiple models
    let api_alias = objs::ApiAlias::new(
      "openai-api",
      ApiFormat::OpenAI,
      "https://api.openai.com/v1",
      vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string()],
      None,
      db_service.now(),
    );
    db_service
      .create_api_model_alias(&api_alias, "test-key")
      .await?;

    // Test finding by model name
    let found = data_service.find_alias("gpt-4").await;
    assert!(matches!(found, Some(Alias::Api(api)) if api.id == "openai-api"));

    let found = data_service.find_alias("gpt-3.5-turbo").await;
    assert!(matches!(found, Some(Alias::Api(api)) if api.id == "openai-api"));

    Ok(())
  }

  #[rstest]
  #[case("testalias-exists:instruct", true, "user")] // User alias exists
  #[case("gpt-4", true, "api")] // API model will be inserted
  #[case("nonexistent-model", false, "none")] // Should not exist
  #[tokio::test]
  #[awt]
  async fn test_find_alias_priority_cases(
    temp_bodhi_home: TempDir,
    test_hf_service: TestHfService,
    #[future] test_db_service: TestDbService,
    #[case] search_alias: &str,
    #[case] should_find: bool,
    #[case] expected_type: &str,
  ) -> anyhow::Result<()> {
    let db_service = Arc::new(test_db_service);
    let data_service = LocalDataService::new(
      temp_bodhi_home.path().join("bodhi"),
      Arc::new(test_hf_service),
      db_service.clone(),
    );

    // Insert API alias with gpt-4 model
    let api_alias = objs::ApiAlias::new(
      "test-api",
      ApiFormat::OpenAI,
      "https://api.openai.com/v1",
      vec!["gpt-4".to_string()],
      None,
      db_service.now(),
    );
    db_service
      .create_api_model_alias(&api_alias, "test-key")
      .await?;

    let found = data_service.find_alias(search_alias).await;

    if should_find {
      let alias = found.expect("Expected to find alias");
      match expected_type {
        "user" => assert!(matches!(alias, Alias::User(_))),
        "model" => assert!(matches!(alias, Alias::Model(_))),
        "api" => assert!(matches!(alias, Alias::Api(_))),
        _ => panic!("Invalid expected_type: {}", expected_type),
      }
    } else {
      assert!(found.is_none());
    }

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  #[awt]
  async fn test_find_alias_user_priority_over_api(
    temp_bodhi_home: TempDir,
    test_hf_service: TestHfService,
    #[future] test_db_service: TestDbService,
  ) -> anyhow::Result<()> {
    let db_service = Arc::new(test_db_service);
    let data_service = LocalDataService::new(
      temp_bodhi_home.path().join("bodhi"),
      Arc::new(test_hf_service),
      db_service.clone(),
    );

    // Insert API alias with model name that matches existing user alias
    let api_alias = objs::ApiAlias::new(
      "conflicting-api",
      ApiFormat::OpenAI,
      "https://api.openai.com/v1",
      vec!["testalias-exists:instruct".to_string()], // Same name as user alias
      None,
      db_service.now(),
    );
    db_service
      .create_api_model_alias(&api_alias, "test-key")
      .await?;

    // Should find user alias, not API alias (user has priority)
    let found = data_service.find_alias("testalias-exists:instruct").await;
    assert!(matches!(found, Some(Alias::User(_))));

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  #[awt]
  async fn test_local_data_service_list_aliases(
    #[future]
    #[from(test_data_service)]
    service: TestDataService,
  ) -> anyhow::Result<()> {
    let result = service.list_aliases().await?;
    assert!(result.len() >= 6);
    assert!(result.contains(&Alias::User(UserAlias::llama3())));
    assert!(result.contains(&Alias::User(UserAlias::testalias_exists())));
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  #[awt]
  async fn test_local_data_service_delete_alias(
    #[future]
    #[from(test_data_service)]
    service: TestDataService,
  ) -> anyhow::Result<()> {
    let exists = service
      .bodhi_home()
      .join("aliases")
      .join("tinyllama--instruct.yaml")
      .exists();
    assert!(exists);
    service.delete_alias("tinyllama:instruct").await?;
    let exists = service
      .bodhi_home()
      .join("aliases")
      .join("tinyllama--instruct.yaml")
      .exists();
    assert!(!exists);
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  #[awt]
  async fn test_local_data_service_delete_alias_not_found(
    #[future]
    #[from(test_data_service)]
    service: TestDataService,
  ) -> anyhow::Result<()> {
    let result = service.delete_alias("notexists--instruct.yaml").await;
    assert!(result.is_err());
    assert!(
      matches!(result.unwrap_err(), DataServiceError::AliasNotExists(alias) if alias == AliasNotFoundError("notexists--instruct.yaml".to_string()))
    );
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  #[awt]
  async fn test_local_data_service_copy_alias(
    #[future]
    #[from(test_data_service)]
    service: TestDataService,
  ) -> anyhow::Result<()> {
    service
      .copy_alias("tinyllama:instruct", "tinyllama:mymodel")
      .await?;
    let new_alias = service
      .find_user_alias("tinyllama:mymodel")
      .expect("should have created new_alias");
    let mut expected = UserAlias::tinyllama();
    expected.alias = "tinyllama:mymodel".to_string();
    assert_eq!(expected, new_alias);
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  #[awt]
  async fn test_local_data_service_read_file(
    #[future]
    #[from(test_data_service)]
    service: TestDataService,
  ) -> anyhow::Result<()> {
    let folder = Some("test_folder".to_string());
    let filename = "test_file.txt";
    let file_path = service.bodhi_home().join("test_folder").join(filename);
    fs::create_dir_all(file_path.parent().unwrap())?;
    fs::write(&file_path, b"test content")?;

    let content = service.read_file(folder.clone(), filename)?;
    assert_eq!(b"test content".to_vec(), content);

    let content = service.read_file(None, filename);
    assert!(content.is_err());
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  #[awt]
  async fn test_local_data_service_write_file(
    #[future]
    #[from(test_data_service)]
    service: TestDataService,
  ) -> anyhow::Result<()> {
    let folder = Some("test_folder".to_string());
    let filename = "test_file.txt";
    let file_path = service.bodhi_home().join("test_folder").join(filename);

    service.write_file(folder.clone(), filename, b"test content")?;
    assert!(file_path.exists());
    let content = fs::read(&file_path)?;
    assert_eq!(b"test content".to_vec(), content);

    service.write_file(None, filename, b"test content in root")?;
    let root_file_path = service.bodhi_home().join(filename);
    assert!(root_file_path.exists());
    let content = fs::read(&root_file_path)?;
    assert_eq!(b"test content in root".to_vec(), content);

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  #[awt]
  async fn test_local_data_service_write_file_create_folder(
    #[future]
    #[from(test_data_service)]
    service: TestDataService,
  ) -> anyhow::Result<()> {
    let folder = Some("new_folder".to_string());
    let filename = "new_file.txt";
    let file_path = service.bodhi_home().join("new_folder").join(filename);

    service.write_file(folder.clone(), filename, b"new content")?;
    assert!(file_path.exists());
    let content = fs::read(&file_path)?;
    assert_eq!(b"new content".to_vec(), content);

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  #[awt]
  async fn test_local_data_service_read_file_not_found(
    #[future]
    #[from(test_data_service)]
    service: TestDataService,
  ) -> anyhow::Result<()> {
    let folder = Some("non_existent_folder".to_string());
    let filename = "non_existent_file.txt";

    let result = service.read_file(folder, filename);
    assert!(result.is_err());
    assert!(matches!(
      result.unwrap_err(),
      DataServiceError::DataFileNotFound(error) if error == DataFileNotFoundError::new("non_existent_file.txt".to_string(), "non_existent_folder".to_string())
    ));
    Ok(())
  }

  #[rstest]
  #[case("azure/gpt-4", Some("azure/".to_string()), vec!["gpt-4".to_string()], "azure-openai")]
  #[case("gpt-4", None, vec!["gpt-4".to_string()], "legacy-api")]
  #[tokio::test]
  #[awt]
  async fn test_find_alias_with_prefix_matches(
    temp_bodhi_home: TempDir,
    test_hf_service: TestHfService,
    #[future] test_db_service: TestDbService,
    #[case] search_term: &str,
    #[case] api_prefix: Option<String>,
    #[case] api_models: Vec<String>,
    #[case] expected_id: &str,
  ) -> anyhow::Result<()> {
    let db_service = Arc::new(test_db_service);
    let data_service = LocalDataService::new(
      temp_bodhi_home.path().join("bodhi"),
      Arc::new(test_hf_service),
      db_service.clone(),
    );

    let test_alias = objs::ApiAlias::new(
      expected_id,
      ApiFormat::OpenAI,
      "https://api.openai.com/v1",
      api_models,
      api_prefix,
      db_service.now(),
    );
    db_service
      .create_api_model_alias(&test_alias, "test-key")
      .await?;

    let found = data_service.find_alias(search_term).await;
    let Some(Alias::Api(api)) = found else {
      panic!("Expected to find Api alias, but found none");
    };
    assert_eq!(expected_id, api.id);

    Ok(())
  }

  #[rstest]
  #[case("non-matching-term")]
  #[tokio::test]
  #[awt]
  async fn test_find_alias_with_non_matching_prefix_returns_none(
    #[case] search_term: &str,
    temp_bodhi_home: TempDir,
    test_hf_service: TestHfService,
    #[future] test_db_service: TestDbService,
  ) -> anyhow::Result<()> {
    let db_service = Arc::new(test_db_service);
    let data_service = LocalDataService::new(
      temp_bodhi_home.path().join("bodhi"),
      Arc::new(test_hf_service),
      db_service.clone(),
    );

    let test_alias = objs::ApiAlias::new(
      "azure-openai",
      ApiFormat::OpenAI,
      "https://api.openai.com/v1",
      vec!["gpt-4".to_string()],
      Some("azure/".to_string()),
      db_service.now(),
    );
    db_service
      .create_api_model_alias(&test_alias, "test-key")
      .await?;

    let found = data_service.find_alias(search_term).await;
    assert!(found.is_none());

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  #[awt]
  async fn test_find_alias_without_prefix_does_not_match_prefixed_api(
    temp_bodhi_home: TempDir,
    test_hf_service: TestHfService,
    #[future] test_db_service: TestDbService,
  ) -> anyhow::Result<()> {
    let db_service = Arc::new(test_db_service);
    let data_service = LocalDataService::new(
      temp_bodhi_home.path().join("bodhi"),
      Arc::new(test_hf_service),
      db_service.clone(),
    );

    // Create API alias with prefix
    let prefixed_alias = objs::ApiAlias::new(
      "azure-openai",
      ApiFormat::OpenAI,
      "https://api.azure.com/v1",
      vec!["gpt-4".to_string()],
      Some("azure/".to_string()),
      db_service.now(),
    );
    db_service
      .create_api_model_alias(&prefixed_alias, "test-key")
      .await?;

    // Searching for "gpt-4" should NOT match the prefixed API
    let found = data_service.find_alias("gpt-4").await;
    assert!(
      found.is_none(),
      "Should not match 'gpt-4' when API has prefix 'azure/'"
    );

    // Searching for "azure/gpt-4" SHOULD match
    let found = data_service.find_alias("azure/gpt-4").await;
    assert!(matches!(found, Some(Alias::Api(api)) if api.id == "azure-openai"));

    Ok(())
  }
}
