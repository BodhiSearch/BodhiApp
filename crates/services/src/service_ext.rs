use serde::de::DeserializeOwned;

use crate::{AppRegInfo, AppStatus, SecretService, SecretServiceError};

const KEY_APP_STATUS: &str = "app_status";
const KEY_APP_AUTHZ: &str = "app_authz";
const KEY_APP_REG_INFO: &str = "app_reg_info";

pub const KEY_RESOURCE_TOKEN: &str = "X-Resource-Token";

type Result<T> = std::result::Result<T, SecretServiceError>;

pub fn set_secret<S, T>(slf: S, key: &str, value: &T) -> Result<()>
where
  T: serde::Serialize,
  S: AsRef<dyn SecretService>,
{
  let value_str = serde_yaml::to_string(value)?;
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

  fn authz_or_default(&self) -> bool {
    self.authz().unwrap_or(false)
  }

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
      .map(|value| value.parse::<bool>().unwrap_or(false))
      .unwrap_or(false);
    Ok(value)
  }

  fn set_authz(&self, authz: bool) -> Result<()> {
    self
      .as_ref()
      .set_secret_string(KEY_APP_AUTHZ, authz.to_string().as_str())
  }

  fn app_reg_info(&self) -> Result<Option<AppRegInfo>> {
    get_secret(self, KEY_APP_REG_INFO)
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
