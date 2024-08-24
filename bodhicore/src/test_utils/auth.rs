use crate::service::AppRegInfoBuilder;

impl AppRegInfoBuilder {
  pub fn test_default() -> Self {
    Self::default()
      .public_key("public_key".to_string())
      .issuer("https://id.mydomain.com/realms/myapp".to_string())
      .client_id("test-client".to_string())
      .client_secret("test-client-secret".to_string())
      .clone()
  }
}
