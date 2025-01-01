use serde::de::DeserializeOwned;

use crate::{AppRegInfo, AppStatus, SecretService, SecretServiceError};

const KEY_APP_STATUS: &str = "app_status";
const KEY_APP_AUTHZ: &str = "app_authz";
const KEY_APP_REG_INFO: &str = "app_reg_info";

type Result<T> = std::result::Result<T, SecretServiceError>;

pub fn set_secret<S, T>(slf: S, key: &str, value: T) -> Result<()>
where
  T: serde::Serialize,
  S: AsRef<dyn SecretService>,
{
  let value_str = serde_yaml::to_string(&value)?;
  slf.as_ref().set_secret_string(key, &value_str)
}

pub fn get_secret<S, T>(slf: S, key: &str) -> Result<Option<T>>
where
  T: DeserializeOwned,
  S: AsRef<dyn SecretService>,
{
  match slf.as_ref().get_secret_string(key)? {
    Some(value) => {
      let result = serde_yaml::from_str::<T>(&value)?;
      Ok(Some(result))
    }
    None => Ok(None),
  }
}

pub trait SecretServiceExt {
  fn authz(&self) -> Result<bool>;

  fn set_authz(&self, authz: bool) -> Result<()>;

  fn app_reg_info(&self) -> Result<Option<AppRegInfo>>;

  fn set_app_reg_info(&self, app_reg_info: &AppRegInfo) -> Result<()>;

  fn app_status(&self) -> Result<AppStatus>;

  fn set_app_status(&self, app_status: &AppStatus) -> Result<()>;
}

impl<T: AsRef<dyn SecretService>> SecretServiceExt for T {
  fn authz(&self) -> Result<bool> {
    let value = self
      .as_ref()
      .get_secret_string(KEY_APP_AUTHZ)?
      .map(|value| value.parse::<bool>().unwrap_or(true))
      .unwrap_or(true);
    Ok(value)
  }

  fn set_authz(&self, authz: bool) -> Result<()> {
    self
      .as_ref()
      .set_secret_string(KEY_APP_AUTHZ, authz.to_string().as_str())
  }

  fn app_reg_info(&self) -> Result<Option<AppRegInfo>> {
    get_secret::<_, AppRegInfo>(self, KEY_APP_REG_INFO)
  }

  fn set_app_reg_info(&self, app_reg_info: &AppRegInfo) -> Result<()> {
    set_secret(self, KEY_APP_REG_INFO, app_reg_info)
  }

  fn app_status(&self) -> Result<AppStatus> {
    get_secret::<_, AppStatus>(self, KEY_APP_STATUS).map(|value| value.unwrap_or_default())
  }

  fn set_app_status(&self, app_status: &AppStatus) -> Result<()> {
    set_secret(self, KEY_APP_STATUS, app_status)
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    generate_random_key, test_utils::app_reg_info, AppRegInfo, AppStatus, DefaultSecretService,
    SecretServiceExt,
  };
  use anyhow_trace::anyhow_trace;
  use objs::test_utils::temp_dir;
  use rstest::rstest;
  use tempfile::TempDir;

  #[rstest]
  #[anyhow_trace]
  fn test_secret_service_ext(temp_dir: TempDir, app_reg_info: AppRegInfo) -> anyhow::Result<()> {
    let secrets_path = temp_dir.path().join("secrets.yaml");
    let service = DefaultSecretService::new(generate_random_key(), &secrets_path)?;

    assert!(service.authz()?);
    service.set_authz(false)?;
    assert!(!service.authz()?);
    service.set_authz(true)?;
    assert!(service.authz()?);

    assert!(service.app_reg_info()?.is_none());

    service.set_app_reg_info(&app_reg_info)?;
    let retrieved_info = service.app_reg_info()?.expect("Should have app reg info");
    assert_eq!(app_reg_info, retrieved_info);

    let initial_status = service.app_status()?;
    assert_eq!(AppStatus::default(), initial_status);

    let new_status = AppStatus::Setup;
    service.set_app_status(&new_status)?;
    assert_eq!(new_status, service.app_status()?);

    Ok(())
  }
}
