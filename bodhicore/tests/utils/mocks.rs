use bodhicore::service::{ISecretService, SecretServiceError};

mockall::mock! {
  #[derive(Debug, Send, Sync)]
  pub SecretService {}

  impl ISecretService for SecretService {
    fn set_secret_string(&self, key: &str, value: &str) -> Result<(), SecretServiceError>;

    fn get_secret_string(&self, key: &str) -> Result<Option<String>, SecretServiceError>;

    fn delete_secret(&self, key: &str) -> Result<(), SecretServiceError>;
  }
}
