use super::error::DataServiceError;
use crate::{
  objs::{Alias, RemoteModel},
  server::BODHI_HOME,
};
use derive_new::new;
#[cfg(test)]
use mockall::automock;
use std::{fmt::Debug, fs, path::PathBuf};

static MODELS_YAML: &str = "models.yaml";

#[cfg_attr(test, automock)]
pub trait DataService: Debug {
  fn list_aliases(&self) -> super::error::Result<Vec<Alias>>;

  fn save_alias(&self, alias: Alias) -> super::error::Result<PathBuf>;

  fn find_alias(&self, alias: &str) -> Option<Alias>;

  fn list_remote_models(&self) -> super::error::Result<Vec<RemoteModel>>;

  fn find_remote_model(&self, alias: &str) -> super::error::Result<Option<RemoteModel>>;
}

#[derive(Debug, Clone, PartialEq, new)]
pub struct LocalDataService {
  bodhi_home: PathBuf,
}

impl Default for LocalDataService {
  fn default() -> Self {
    let bodhi_home = match std::env::var(BODHI_HOME) {
      Ok(home) => home.into(),
      Err(_) => {
        let mut home = dirs::home_dir().expect("$HOME directory cannot be found");
        home.push(".cache");
        home.push("bodhi");
        home
      }
    };
    Self { bodhi_home }
  }
}

impl DataService for LocalDataService {
  fn find_remote_model(&self, alias: &str) -> super::error::Result<Option<RemoteModel>> {
    let models = self.list_remote_models()?;
    Ok(models.into_iter().find(|model| model.alias.eq(alias)))
  }

  fn save_alias(&self, alias: Alias) -> super::error::Result<PathBuf> {
    let contents = serde_yaml::to_string(&alias)?;
    let filename = self
      .bodhi_home
      .join("configs")
      .join(alias.config_filename());
    fs::write(filename.clone(), contents)?;
    Ok(filename)
  }

  fn list_aliases(&self) -> super::error::Result<Vec<Alias>> {
    let config = self.bodhi_home.join("configs");
    if !config.exists() {
      return Err(DataServiceError::DirMissing {
        dirname: String::from("configs"),
      });
    }
    let yaml_files = fs::read_dir(config)?
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
    let mut aliases = yaml_files
      .into_iter()
      .filter_map(|yaml_file| {
        let filename = yaml_file.clone().display().to_string();
        match fs::read_to_string(yaml_file) {
          Ok(content) => match serde_yaml::from_str::<Alias>(&content) {
            Ok(alias) => Some(alias),
            Err(err) => {
              let err = DataServiceError::SerdeYamlDeserialize(err);
              tracing::warn!(filename, ?err, "Error deserializing model alias YAML file",);
              None
            }
          },
          Err(err) => {
            let err = DataServiceError::Io(err);
            tracing::warn!(filename, ?err, "Error reading model alias YAML file");
            None
          }
        }
      })
      .collect::<Vec<_>>();
    aliases.sort_by(|a, b| a.alias.cmp(&b.alias));
    Ok(aliases)
  }

  fn find_alias(&self, alias: &str) -> Option<Alias> {
    self
      .list_aliases()
      .unwrap_or_default()
      .into_iter()
      .find(|obj| obj.alias.eq(&alias))
  }

  fn list_remote_models(&self) -> super::error::Result<Vec<RemoteModel>> {
    let models_file = self.bodhi_home.join(MODELS_YAML);
    if !models_file.exists() {
      return Err(DataServiceError::FileMissing {
        filename: String::from(MODELS_YAML),
        dirname: "".to_string(),
      });
    }
    let content = fs::read_to_string(models_file.clone())?;
    let models = serde_yaml::from_str::<Vec<RemoteModel>>(&content).map_err(|err| {
      DataServiceError::SerdeYamlSerialize {
        source: err,
        filename: models_file.display().to_string(),
      }
    })?;
    Ok(models)
  }
}

#[cfg(test)]
mod test {
  use super::{DataService, LocalDataService};
  use crate::{
    objs::{Alias, RemoteModel},
    server::BODHI_HOME,
    test_utils::{data_service, DataServiceTuple},
  };
  use anyhow_trace::anyhow_trace;
  use rstest::rstest;
  use std::fs;
  use tempfile::tempdir;

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
      r#".[0]: missing field `alias` at line 3 column 3
error while serializing from file: '{models_file}'"#
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
  fn test_local_data_service_list_alias_dir_missing_error(
    data_service: DataServiceTuple,
  ) -> anyhow::Result<()> {
    let DataServiceTuple(_temp, bodhi_home, service) = data_service;
    fs::remove_dir_all(bodhi_home.join("configs"))?;
    let result = service.list_aliases();
    assert!(result.is_err());
    let expected = r#"directory 'configs' not found in $BODHI_HOME.
$BODHI_HOME might not have been initialized. Run `bodhi init` to setup $BODHI_HOME."#;
    assert_eq!(expected, result.unwrap_err().to_string());
    Ok(())
  }

  #[rstest]
  fn test_local_data_service_default_from_bodhi_home() -> anyhow::Result<()> {
    let bodhi_home = tempdir()?;
    std::env::set_var(BODHI_HOME, bodhi_home.path());
    let service = LocalDataService::default();
    let expected = LocalDataService::new(bodhi_home.path().to_path_buf());
    assert_eq!(expected, service);
    Ok(())
  }

  #[rstest]
  fn test_local_data_service_default_from_home_dir() -> anyhow::Result<()> {
    let home_dir = tempdir()?;
    std::env::remove_var(BODHI_HOME);
    std::env::set_var("HOME", home_dir.path());
    let service = LocalDataService::default();
    let expected =
      LocalDataService::new(home_dir.path().join(".cache").join("bodhi").to_path_buf());
    assert_eq!(expected, service);
    Ok(())
  }
}
