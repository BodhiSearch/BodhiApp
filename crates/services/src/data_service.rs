use crate::{HubService, HubServiceError, ALIASES_DIR, MODELS_YAML};
use objs::{
  impl_error_from, Alias, AppError, ErrorType, IoDirCreateError, IoError, IoFileDeleteError,
  IoFileReadError, IoFileWriteError, RemoteModel, SerdeYamlError, SerdeYamlWithPathError,
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
}

impl_error_from!(
  ::serde_yaml::Error,
  DataServiceError::SerdeYamlError,
  ::objs::SerdeYamlError
);
impl_error_from!(::std::io::Error, DataServiceError::Io, ::objs::IoError);

type Result<T> = std::result::Result<T, DataServiceError>;

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait DataService: Send + Sync + std::fmt::Debug {
  fn list_aliases(&self) -> Result<Vec<Alias>>;

  fn save_alias(&self, alias: &Alias) -> Result<PathBuf>;

  fn find_alias(&self, alias: &str) -> Option<Alias>;

  fn list_remote_models(&self) -> Result<Vec<RemoteModel>>;

  fn find_remote_model(&self, alias: &str) -> Result<Option<RemoteModel>>;

  fn copy_alias(&self, alias: &str, new_alias: &str) -> Result<()>;

  fn delete_alias(&self, alias: &str) -> Result<()>;

  fn alias_filename(&self, alias: &str) -> Result<PathBuf>;

  fn read_file(&self, folder: Option<String>, filename: &str) -> Result<Vec<u8>>;

  fn write_file(&self, folder: Option<String>, filename: &str, contents: &[u8]) -> Result<()>;

  fn find_file(&self, folder: Option<String>, filename: &str) -> Result<PathBuf>;
}

#[derive(Debug, Clone)]
pub struct LocalDataService {
  bodhi_home: PathBuf,
  hub_service: Arc<dyn HubService>,
}

impl LocalDataService {
  pub fn new(bodhi_home: PathBuf, hub_service: Arc<dyn HubService>) -> Self {
    Self {
      bodhi_home,
      hub_service,
    }
  }

  fn aliases_dir(&self) -> PathBuf {
    self.bodhi_home.join(ALIASES_DIR)
  }

  fn models_yaml(&self) -> PathBuf {
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

impl DataService for LocalDataService {
  fn find_remote_model(&self, alias: &str) -> Result<Option<RemoteModel>> {
    let models = self.list_remote_models()?;
    Ok(models.into_iter().find(|model| model.alias.eq(alias)))
  }

  fn save_alias(&self, alias: &Alias) -> Result<PathBuf> {
    let contents = serde_yaml::to_string(alias)?;
    let filename = self.aliases_dir().join(alias.config_filename());
    fs::write(filename.clone(), contents)
      .map_err(|err| IoFileWriteError::new(err, alias.config_filename().clone()))?;
    Ok(filename)
  }

  fn list_aliases(&self) -> Result<Vec<Alias>> {
    let user_aliases = self._list_aliases()?;
    let mut result = user_aliases.into_values().collect::<Vec<_>>();
    result.sort_by(|a, b| a.alias.cmp(&b.alias));
    let model_aliases = self.hub_service.list_model_aliases()?;
    result.extend(model_aliases);
    Ok(result)
  }

  fn find_alias(&self, alias: &str) -> Option<Alias> {
    let aliases = self.list_aliases();
    let aliases = aliases.unwrap_or_default();
    aliases.into_iter().find(|obj| obj.alias.eq(&alias))
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

  fn copy_alias(&self, alias: &str, new_alias: &str) -> Result<()> {
    let mut alias = self
      .find_alias(alias)
      .ok_or_else(|| AliasNotFoundError(alias.to_string()))?;
    match self.find_alias(new_alias) {
      Some(_) => Err(AliasExistsError(new_alias.to_string()))?,
      None => {
        alias.alias = new_alias.to_string();
        self.save_alias(&alias)?;
        Ok(())
      }
    }
  }

  fn delete_alias(&self, alias: &str) -> Result<()> {
    let (filename, _) = self
      ._list_aliases()?
      .into_iter()
      .find(|(_, item)| item.alias.eq(alias))
      .ok_or_else(|| AliasNotFoundError(alias.to_string()))?;
    fs::remove_file(&filename).map_err(|err| IoFileDeleteError::new(err, filename))?;
    Ok(())
  }

  fn alias_filename(&self, alias: &str) -> Result<PathBuf> {
    let (filename, _) = self
      ._list_aliases()?
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
  fn _list_aliases(&self) -> Result<HashMap<String, Alias>> {
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
            Ok(content) => match serde_yaml::from_str::<Alias>(&content) {
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
    test_utils::{test_data_service, TestDataService},
    AliasExistsError, AliasNotFoundError, DataFileNotFoundError, DataService, DataServiceError,
  };
  use anyhow_trace::anyhow_trace;
  use objs::{
    test_utils::{assert_error_message, setup_l10n},
    Alias, AppError, FluentLocalizationService, RemoteModel,
  };
  use rstest::rstest;
  use std::fs;
  use std::sync::Arc;

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
  fn test_local_data_service_models_file_missing(
    #[from(test_data_service)] service: TestDataService,
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
  #[anyhow_trace]
  fn test_local_data_service_models_file_corrupt(
    #[from(test_data_service)] service: TestDataService,
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
  fn test_local_data_service_find_remote_model(
    #[from(test_data_service)] service: TestDataService,
    #[case] alias: String,
    #[case] found: bool,
  ) -> anyhow::Result<()> {
    let remote_model = service.find_remote_model(&alias)?;
    assert_eq!(found, remote_model.is_some());
    Ok(())
  }

  #[rstest]
  fn test_local_data_service_list_remote_models(
    #[from(test_data_service)] service: TestDataService,
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
  fn test_local_data_service_find_alias(
    #[from(test_data_service)] service: TestDataService,
  ) -> anyhow::Result<()> {
    let alias = service.find_alias("testalias-exists:instruct");
    let expected = Alias::testalias_exists();
    assert_eq!(Some(expected), alias);
    Ok(())
  }

  #[rstest]
  fn test_local_data_service_list_aliases(
    #[from(test_data_service)] service: TestDataService,
  ) -> anyhow::Result<()> {
    let result = service.list_aliases()?;
    assert_eq!(6, result.len());
    assert!(result.contains(&Alias::llama3()));
    assert!(result.contains(&Alias::testalias_exists()));
    Ok(())
  }

  #[rstest]
  fn test_local_data_service_delete_alias(
    #[from(test_data_service)] service: TestDataService,
  ) -> anyhow::Result<()> {
    let exists = service
      .bodhi_home()
      .join("aliases")
      .join("tinyllama--instruct.yaml")
      .exists();
    assert!(exists);
    service.delete_alias("tinyllama:instruct")?;
    let exists = service
      .bodhi_home()
      .join("aliases")
      .join("tinyllama--instruct.yaml")
      .exists();
    assert!(!exists);
    Ok(())
  }

  #[rstest]
  fn test_local_data_service_delete_alias_not_found(
    #[from(test_data_service)] service: TestDataService,
  ) -> anyhow::Result<()> {
    let result = service.delete_alias("notexists--instruct.yaml");
    assert!(result.is_err());
    assert!(
      matches!(result.unwrap_err(), DataServiceError::AliasNotExists(alias) if alias == AliasNotFoundError("notexists--instruct.yaml".to_string()))
    );
    Ok(())
  }

  #[rstest]
  fn test_local_data_service_copy_alias(
    #[from(test_data_service)] service: TestDataService,
  ) -> anyhow::Result<()> {
    service.copy_alias("tinyllama:instruct", "tinyllama:mymodel")?;
    let new_alias = service
      .find_alias("tinyllama:mymodel")
      .expect("should have created new_alias");
    let mut expected = Alias::tinyllama();
    expected.alias = "tinyllama:mymodel".to_string();
    assert_eq!(expected, new_alias);
    Ok(())
  }

  #[rstest]
  fn test_local_data_service_read_file(
    #[from(test_data_service)] service: TestDataService,
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
  fn test_local_data_service_write_file(
    #[from(test_data_service)] service: TestDataService,
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
  fn test_local_data_service_write_file_create_folder(
    #[from(test_data_service)] service: TestDataService,
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
  fn test_local_data_service_read_file_not_found(
    #[from(test_data_service)] service: TestDataService,
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
}
