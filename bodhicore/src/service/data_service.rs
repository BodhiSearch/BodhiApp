use super::{ALIASES_DIR, MODELS_YAML};
use crate::{
  error::Common,
  objs::{Alias, RemoteModel},
};
use derive_new::new;
use std::{collections::HashMap, fmt::Debug, fs, io, path::PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum DataServiceError {
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
  Common(#[from] Common),
  #[error("source: {source}\npath:{path}\nfailed to create file/directory")]
  DirCreate {
    #[source]
    source: io::Error,
    path: String,
  },
  #[error("bodhi_home_err: failed to automatically set BODHI_HOME. Set it through environment variable $BODHI_HOME and try again.")]
  BodhiHome,
  #[error("hf_home_err: failed to automatically set HF_HOME. Set it through environment variable $HF_HOME and try again.")]
  HfHome,
  #[error("alias '{0}' not found in $BODHI_HOME/aliases")]
  AliasNotExists(String),
  #[error("alias '{0}' already exists in $BODHI_HOME/aliases")]
  AliasExists(String),
}

type Result<T> = std::result::Result<T, DataServiceError>;

#[cfg_attr(test, mockall::automock)]
pub trait DataService: std::fmt::Debug {
  fn list_aliases(&self) -> Result<Vec<Alias>>;

  fn save_alias(&self, alias: &Alias) -> Result<PathBuf>;

  fn find_alias(&self, alias: &str) -> Option<Alias>;

  fn list_remote_models(&self) -> Result<Vec<RemoteModel>>;

  fn find_remote_model(&self, alias: &str) -> Result<Option<RemoteModel>>;

  fn copy_alias(&self, alias: &str, new_alias: &str) -> Result<()>;

  fn delete_alias(&self, alias: &str) -> Result<()>;

  fn alias_filename(&self, alias: &str) -> Result<PathBuf>;
}

#[derive(Debug, Clone, PartialEq, new)]
pub struct LocalDataService {
  bodhi_home: PathBuf,
}

impl LocalDataService {
  fn aliases_dir(&self) -> PathBuf {
    self.bodhi_home.join(ALIASES_DIR)
  }

  fn models_yaml(&self) -> PathBuf {
    self.bodhi_home.join(MODELS_YAML)
  }
}

impl DataService for LocalDataService {
  fn find_remote_model(&self, alias: &str) -> Result<Option<RemoteModel>> {
    let models = self.list_remote_models()?;
    Ok(models.into_iter().find(|model| model.alias.eq(alias)))
  }

  fn save_alias(&self, alias: &Alias) -> Result<PathBuf> {
    let contents = serde_yaml::to_string(alias).map_err(Common::SerdeYamlDeserialize)?;
    let filename = self.aliases_dir().join(alias.config_filename());
    fs::write(filename.clone(), contents).map_err(|err| Common::IoFile {
      source: err,
      path: alias.config_filename().clone(),
    })?;
    Ok(filename)
  }

  fn list_aliases(&self) -> Result<Vec<Alias>> {
    let hashamp = self._list_aliases()?;
    let mut result = hashamp.into_values().collect::<Vec<_>>();
    result.sort_by(|a, b| a.alias.cmp(&b.alias));
    Ok(result)
  }

  fn find_alias(&self, alias: &str) -> Option<Alias> {
    self
      .list_aliases()
      .unwrap_or_default()
      .into_iter()
      .find(|obj| obj.alias.eq(&alias))
  }

  fn list_remote_models(&self) -> Result<Vec<RemoteModel>> {
    let models_file = self.models_yaml();
    if !models_file.exists() {
      return Err(DataServiceError::FileMissing {
        filename: String::from(MODELS_YAML),
        dirname: "".to_string(),
      });
    }
    let content = fs::read_to_string(models_file.clone()).map_err(|err| Common::IoFile {
      source: err,
      path: models_file.display().to_string(),
    })?;
    let models = serde_yaml::from_str::<Vec<RemoteModel>>(&content).map_err(|err| {
      Common::SerdeYamlSerialize {
        source: err,
        filename: models_file.display().to_string(),
      }
    })?;
    Ok(models)
  }

  fn copy_alias(&self, alias: &str, new_alias: &str) -> Result<()> {
    let mut alias = self
      .find_alias(alias)
      .ok_or_else(|| DataServiceError::AliasNotExists(alias.to_string()))?;
    if self.find_alias(new_alias).is_some() {
      return Err(DataServiceError::AliasExists(new_alias.to_string()));
    }
    alias.alias = new_alias.to_string();
    self.save_alias(&alias)?;
    Ok(())
  }

  fn delete_alias(&self, alias: &str) -> Result<()> {
    let (filename, _) = self
      ._list_aliases()?
      .into_iter()
      .find(|(_, item)| item.alias.eq(alias))
      .ok_or_else(|| DataServiceError::AliasNotExists(alias.to_string()))?;
    fs::remove_file(filename).map_err(Common::from)?;
    Ok(())
  }

  fn alias_filename(&self, alias: &str) -> Result<PathBuf> {
    let (filename, _) = self
      ._list_aliases()?
      .into_iter()
      .find(|(_, item)| item.alias.eq(alias))
      .ok_or_else(|| DataServiceError::AliasNotExists(alias.to_string()))?;
    let result = PathBuf::from(filename);
    assert!(
      result.exists(),
      "file should exists at path {}",
      result.display()
    );
    Ok(result)
  }
}

impl LocalDataService {
  fn _list_aliases(&self) -> Result<HashMap<String, Alias>> {
    {
      let aliases_dir = self.aliases_dir();
      let yaml_files = fs::read_dir(&aliases_dir).map_err(|err| Common::IoFile {
        source: err,
        path: aliases_dir.display().to_string(),
      })?;
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
                let err = Common::SerdeYamlDeserialize(err);
                tracing::warn!(filename, ?err, "Error deserializing model alias YAML file",);
                None
              }
            },
            Err(err) => {
              let err = Common::IoFile {
                source: err,
                path: filename,
              };
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
  use super::DataService;
  use crate::{
    objs::{Alias, RemoteModel},
    test_utils::{data_service, DataServiceTuple},
  };
  use anyhow_trace::anyhow_trace;
  use rstest::rstest;
  use std::fs;

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
      r#"serde_yaml_serialize: .[0]: missing field `alias` at line 3 column 3
filename='{models_file}'"#
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
  fn test_local_data_service_list_remote_models(
    data_service: DataServiceTuple,
  ) -> anyhow::Result<()> {
    let DataServiceTuple(_temp_dir, _, service) = data_service;
    let models = service.list_remote_models()?;
    let expected_1 = RemoteModel::llama3();
    let expected_2 = RemoteModel::testalias();
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
  fn test_local_data_service_delete_alias(data_service: DataServiceTuple) -> anyhow::Result<()> {
    let DataServiceTuple(_temp, bodhi_home, service) = data_service;
    let exists = bodhi_home
      .join("aliases")
      .join("tinyllama--instruct.yaml")
      .exists();
    assert!(exists);
    service.delete_alias("tinyllama:instruct")?;
    let exists = bodhi_home
      .join("aliases")
      .join("tinyllama--instruct.yaml")
      .exists();
    assert!(!exists);
    Ok(())
  }

  #[rstest]
  fn test_local_data_service_delete_alias_not_found(
    data_service: DataServiceTuple,
  ) -> anyhow::Result<()> {
    let DataServiceTuple(_temp, _, service) = data_service;
    let result = service.delete_alias("notexists--instruct.yaml");
    assert!(result.is_err());
    assert_eq!(
      "alias 'notexists--instruct.yaml' not found in $BODHI_HOME/aliases",
      result.unwrap_err().to_string()
    );
    Ok(())
  }

  #[rstest]
  fn test_local_data_service_copy_alias(data_service: DataServiceTuple) -> anyhow::Result<()> {
    let DataServiceTuple(_temp, _, service) = data_service;
    service.copy_alias("tinyllama:instruct", "tinyllama:mymodel")?;
    let new_alias = service
      .find_alias("tinyllama:mymodel")
      .expect("should have created new_alias");
    let mut expected = Alias::tinyllama();
    expected.alias = "tinyllama:mymodel".to_string();
    assert_eq!(expected, new_alias);
    Ok(())
  }
}
